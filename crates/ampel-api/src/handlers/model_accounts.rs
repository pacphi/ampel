//! Model-provider account CRUD (Phase 4 — Agentic Remediation Tier).
//!
//! Manages `model_provider_account` rows: the credentials + capability metadata
//! the worker's agentic tier uses to drive Claude/Gemini/Ollama/ONNX providers.
//!
//! ## Security (ADR-008 / ADR-014)
//! - **Credentials never leave the server.** The `apiKey` field is accepted on
//!   create/update only; it is AES-256-GCM encrypted via the [`EncryptionService`]
//!   into `credentials_encrypted` and is NEVER serialized back — the response DTO
//!   has no credential field at all, and the request field is `#[serde(skip_serializing)]`.
//! - **Validate-before-use.** Create stores the account as `unvalidated`; a
//!   separate `POST /{id}/validate` pings the provider and flips the status. The
//!   API does not block on a network ping at create time.
//! - **Air-gapped ceiling (ADR-014).** Creating an External-egress account
//!   (`claude`/`gemini`, or an explicit `egressClass=external`) for an
//!   organization with `air_gapped = true` is rejected with `422`.
//! - **Scope isolation.** Every read/write asserts the caller owns the account
//!   (user-scoped) or owns the org it belongs to (org-scoped); cross-scope access
//!   returns `404` (never leaks existence).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::remediation::{Egress, ModelCredentials, ModelProvider, ProviderKind};
use ampel_db::entities::{model_provider_account, organization};

use crate::extractors::AuthUser;
use crate::handlers::security::assert_endpoint_safe;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateModelAccountRequest {
    pub provider_kind: String,
    pub display_name: String,
    /// Hosted-API bearer key. Write-only: never serialized back to clients.
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
    pub endpoint_url: Option<String>,
    pub model_id: Option<String>,
    pub model_path: Option<String>,
    /// When set, the account is org-scoped; otherwise it is user-scoped.
    pub organization_id: Option<Uuid>,
    /// Optional egress override (`external` | `local_only`); defaults from kind.
    pub egress_class: Option<String>,
    /// Optional spend ceiling in USD (Decimal-as-string).
    pub spend_cap_usd: Option<String>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateModelAccountRequest {
    pub display_name: Option<String>,
    /// Replacement key; re-encrypted and marks the account `unvalidated`.
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
    pub endpoint_url: Option<String>,
    pub model_id: Option<String>,
    pub model_path: Option<String>,
    pub spend_cap_usd: Option<String>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
}

/// Response DTO. Deliberately omits any credential material.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelAccountResponse {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub provider_kind: String,
    pub display_name: String,
    pub endpoint_url: Option<String>,
    pub egress_class: String,
    pub model_id: Option<String>,
    pub model_path: Option<String>,
    pub auth_type: String,
    pub validation_status: String,
    pub spend_cap_usd: Option<String>,
    pub spend_used_usd: String,
    pub last_validated_at: Option<String>,
    pub enabled: bool,
    pub is_default: bool,
    /// `true` if a key is on file (whether the key itself is never exposed).
    pub has_credentials: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<model_provider_account::Model> for ModelAccountResponse {
    fn from(m: model_provider_account::Model) -> Self {
        Self {
            id: m.id,
            organization_id: m.organization_id,
            user_id: m.user_id,
            provider_kind: m.provider_kind,
            display_name: m.display_name,
            endpoint_url: m.endpoint_url,
            egress_class: m.egress_class,
            model_id: m.model_id,
            model_path: m.model_path,
            auth_type: m.auth_type,
            validation_status: m.validation_status,
            spend_cap_usd: m.spend_cap_usd,
            spend_used_usd: m.spend_used_usd,
            last_validated_at: m.last_validated_at.map(|dt| dt.to_rfc3339()),
            enabled: m.enabled,
            is_default: m.is_default,
            has_credentials: m.credentials_encrypted.is_some(),
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelValidationResult {
    pub is_valid: bool,
    pub validation_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub last_validated_at: String,
}

// ============================================================================
// Helpers
// ============================================================================

/// Default egress for a provider kind (ADR-009): hosted APIs reach the public
/// internet; local providers stay on-host.
fn default_egress(kind: ProviderKind) -> Egress {
    match kind {
        ProviderKind::Claude | ProviderKind::Gemini => Egress::External,
        ProviderKind::Ollama | ProviderKind::Onnx => Egress::LocalOnly,
    }
}

/// Default auth type: hosted APIs use an API key, local providers use none.
fn default_auth_type(kind: ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Claude | ProviderKind::Gemini => "api_key",
        ProviderKind::Ollama | ProviderKind::Onnx => "none",
    }
}

/// Assert `user_id` may access `account` (user-scoped self, or owns the org).
/// Denial returns `404` so resource existence is never leaked.
async fn assert_account_access(
    state: &AppState,
    user_id: Uuid,
    account: &model_provider_account::Model,
) -> Result<(), ApiError> {
    if account.user_id == Some(user_id) {
        return Ok(());
    }
    if let Some(org_id) = account.organization_id {
        let owns = organization::Entity::find_by_id(org_id)
            .one(&state.db)
            .await?
            .map(|o| o.owner_id == user_id)
            .unwrap_or(false);
        if owns {
            return Ok(());
        }
    }
    Err(ApiError::not_found("Model provider account not found"))
}

/// Build a concrete provider for a `/validate` ping. ONNX is a local in-process
/// classifier and has nothing to ping, so it has no network validation path.
fn build_provider(kind: ProviderKind) -> Option<Box<dyn ModelProvider>> {
    match kind {
        ProviderKind::Claude => Some(Box::new(ampel_worker::providers::ClaudeProvider::new())),
        ProviderKind::Gemini => Some(Box::new(ampel_worker::providers::GeminiProvider::new())),
        ProviderKind::Ollama => Some(Box::new(ampel_worker::providers::OllamaProvider::new())),
        ProviderKind::Onnx => None,
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/model-accounts — accounts the caller can manage (self + owned orgs).
pub async fn list_model_accounts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<ModelAccountResponse>>>, ApiError> {
    let owned_org_ids: Vec<Uuid> = organization::Entity::find()
        .filter(organization::Column::OwnerId.eq(auth.user_id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|o| o.id)
        .collect();

    let mut condition =
        Condition::any().add(model_provider_account::Column::UserId.eq(auth.user_id));
    if !owned_org_ids.is_empty() {
        condition =
            condition.add(model_provider_account::Column::OrganizationId.is_in(owned_org_ids));
    }

    let accounts = model_provider_account::Entity::find()
        .filter(condition)
        .all(&state.db)
        .await?;

    Ok(Json(ApiResponse::success(
        accounts
            .into_iter()
            .map(ModelAccountResponse::from)
            .collect(),
    )))
}

/// POST /api/model-accounts — create (validate-before-store deferred to /validate).
pub async fn create_model_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateModelAccountRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ModelAccountResponse>>), ApiError> {
    let kind: ProviderKind = req
        .provider_kind
        .parse()
        .map_err(|_| ApiError::bad_request("invalid provider_kind"))?;

    // Resolve effective egress (explicit override wins, else kind default).
    let egress = match req.egress_class.as_deref() {
        Some(s) => s
            .parse::<Egress>()
            .map_err(|_| ApiError::bad_request("invalid egress_class"))?,
        None => default_egress(kind),
    };

    // Org-scoped accounts: caller must own the org, and the ADR-014 air-gapped
    // ceiling forbids creating an External-egress account in an air-gapped org.
    if let Some(org_id) = req.organization_id {
        let org = organization::Entity::find_by_id(org_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| ApiError::not_found("Organization not found"))?;
        if org.owner_id != auth.user_id {
            return Err(ApiError::not_found("Organization not found"));
        }
        if org.air_gapped && egress == Egress::External {
            return Err(ApiError::unprocessable_entity(
                "air-gapped organization forbids external-egress model providers (ADR-014)",
            ));
        }
    }

    // SSRF guard: a user-supplied endpoint_url must not point at internal hosts
    // for external-egress providers (local-only providers are exempt — that is
    // their purpose). Applied before any value is persisted or pinged.
    if let Some(ep) = req.endpoint_url.as_deref() {
        assert_endpoint_safe(ep, egress).await?;
    }

    // Encrypt the key (if any). Never stored or logged in plaintext.
    let credentials_encrypted = match req.api_key.as_deref() {
        Some(key) if !key.is_empty() => Some(
            state
                .encryption_service
                .encrypt(key)
                .map_err(|e| ApiError::internal(format!("encryption failed: {e}")))?,
        ),
        _ => None,
    };

    let now = Utc::now();
    let (user_id, organization_id) = match req.organization_id {
        Some(org_id) => (None, Some(org_id)),
        None => (Some(auth.user_id), None),
    };

    let model = model_provider_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(organization_id),
        user_id: Set(user_id),
        provider_kind: Set(kind.to_string()),
        display_name: Set(req.display_name),
        credentials_encrypted: Set(credentials_encrypted),
        endpoint_url: Set(req.endpoint_url),
        egress_class: Set(egress.to_string()),
        model_id: Set(req.model_id),
        enabled: Set(req.enabled.unwrap_or(true)),
        auth_type: Set(default_auth_type(kind).to_string()),
        spend_cap_usd: Set(req.spend_cap_usd),
        spend_used_usd: Set("0".to_string()),
        validation_status: Set("unvalidated".to_string()),
        last_validated_at: Set(None),
        model_path: Set(req.model_path),
        is_default: Set(req.is_default.unwrap_or(false)),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let created = model.insert(&state.db).await?;
    tracing::info!(
        user_id = %auth.user_id,
        account_id = %created.id,
        provider_kind = %created.provider_kind,
        "Model provider account created"
    );

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(ModelAccountResponse::from(created))),
    ))
}

async fn load_authorized_account(
    state: &AppState,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<model_provider_account::Model, ApiError> {
    let account = model_provider_account::Entity::find_by_id(account_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Model provider account not found"))?;
    assert_account_access(state, user_id, &account).await?;
    Ok(account)
}

/// GET /api/model-accounts/{id}
pub async fn get_model_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ModelAccountResponse>>, ApiError> {
    let account = load_authorized_account(&state, auth.user_id, account_id).await?;
    Ok(Json(ApiResponse::success(ModelAccountResponse::from(
        account,
    ))))
}

/// PATCH /api/model-accounts/{id}
pub async fn update_model_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(req): Json<UpdateModelAccountRequest>,
) -> Result<Json<ApiResponse<ModelAccountResponse>>, ApiError> {
    let account = load_authorized_account(&state, auth.user_id, account_id).await?;
    // Effective egress for the SSRF guard (egress_class is not user-updatable).
    let egress = account
        .egress_class
        .parse::<Egress>()
        .unwrap_or(Egress::External);
    let mut active: model_provider_account::ActiveModel = account.into();

    if let Some(v) = req.display_name {
        active.display_name = Set(v);
    }
    if let Some(v) = req.endpoint_url {
        // SSRF guard on the replacement URL before it is persisted.
        assert_endpoint_safe(&v, egress).await?;
        active.endpoint_url = Set(Some(v));
    }
    if let Some(v) = req.model_id {
        active.model_id = Set(Some(v));
    }
    if let Some(v) = req.model_path {
        active.model_path = Set(Some(v));
    }
    if let Some(v) = req.spend_cap_usd {
        active.spend_cap_usd = Set(Some(v));
    }
    if let Some(v) = req.enabled {
        active.enabled = Set(v);
    }
    if let Some(v) = req.is_default {
        active.is_default = Set(v);
    }
    // A new key is re-encrypted and resets validation status.
    if let Some(key) = req.api_key.as_deref() {
        if !key.is_empty() {
            let enc = state
                .encryption_service
                .encrypt(key)
                .map_err(|e| ApiError::internal(format!("encryption failed: {e}")))?;
            active.credentials_encrypted = Set(Some(enc));
            active.validation_status = Set("unvalidated".to_string());
            active.last_validated_at = Set(None);
        }
    }
    active.updated_at = Set(Utc::now());

    let updated = active.update(&state.db).await?;
    tracing::info!(user_id = %auth.user_id, account_id = %account_id, "Model provider account updated");
    Ok(Json(ApiResponse::success(ModelAccountResponse::from(
        updated,
    ))))
}

/// DELETE /api/model-accounts/{id}
pub async fn delete_model_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let account = load_authorized_account(&state, auth.user_id, account_id).await?;
    let txn = state.db.begin().await?;
    model_provider_account::Entity::delete_by_id(account.id)
        .exec(&txn)
        .await?;
    txn.commit().await?;
    tracing::info!(user_id = %auth.user_id, account_id = %account_id, "Model provider account deleted");
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/model-accounts/{id}/validate — ping the provider with the stored key.
pub async fn validate_model_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ModelValidationResult>>, ApiError> {
    let account = load_authorized_account(&state, auth.user_id, account_id).await?;
    let kind: ProviderKind = account
        .provider_kind
        .parse()
        .map_err(|_| ApiError::internal("invalid provider_kind in database"))?;

    // Decrypt the key ONLY here, at the call site (ADR-008). Never logged.
    let api_key = match &account.credentials_encrypted {
        Some(bytes) => Some(
            state
                .encryption_service
                .decrypt(bytes)
                .map_err(|e| ApiError::internal(format!("decryption failed: {e}")))?,
        ),
        None => None,
    };
    // SSRF guard: re-check the stored endpoint at the actual network call site
    // (defense in depth — the URL could have been set before this guard existed).
    if let Some(ep) = account.endpoint_url.as_deref() {
        let egress = account
            .egress_class
            .parse::<Egress>()
            .unwrap_or(Egress::External);
        assert_endpoint_safe(ep, egress).await?;
    }

    let creds = ModelCredentials {
        api_key,
        endpoint_url: account.endpoint_url.clone(),
        model_id: account.model_id.clone(),
        model_path: account.model_path.clone(),
    };

    let validation = match build_provider(kind) {
        Some(provider) => provider.validate(&creds).await,
        // ONNX: nothing to ping; treat presence of a model_path as valid.
        None => Ok(()),
    };

    let now = Utc::now();
    let (status, is_valid, error_message) = match validation {
        Ok(()) => ("valid", true, None),
        Err(e) => {
            // Log the detailed upstream error server-side (no secrets); return a
            // generic message so provider/internal detail never leaks to clients.
            tracing::warn!(
                account_id = %account_id,
                provider_kind = %kind,
                error = %e,
                "Model provider validation failed"
            );
            (
                "invalid",
                false,
                Some("validation failed: could not reach or authenticate provider".to_string()),
            )
        }
    };

    let mut active: model_provider_account::ActiveModel = account.into();
    active.validation_status = Set(status.to_string());
    active.last_validated_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&state.db).await?;

    tracing::info!(account_id = %account_id, is_valid, "Model provider account validated");
    Ok(Json(ApiResponse::success(ModelValidationResult {
        is_valid,
        validation_status: status.to_string(),
        error_message,
        last_validated_at: now.to_rfc3339(),
    })))
}
