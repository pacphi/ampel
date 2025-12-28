use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use rust_i18n::t;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::models::GitProvider;
use ampel_db::entities::{provider_account, repository};
use ampel_providers::traits::ProviderCredentials;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddAccountRequest {
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_url: Option<String>,
    pub account_label: String,
    pub access_token: String,
    /// Required for Bitbucket (username for Basic Auth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAccountResponse {
    pub id: Uuid,
    pub provider: String,
    pub instance_url: Option<String>,
    pub account_label: String,
    pub provider_username: String,
    pub provider_email: Option<String>,
    pub avatar_url: Option<String>,
    pub scopes: Vec<String>,
    pub token_expires_at: Option<String>,
    pub validation_status: String,
    pub last_validated_at: Option<String>,
    pub is_active: bool,
    pub is_default: bool,
    pub repository_count: i64,
    pub created_at: String,
}

impl ProviderAccountResponse {
    /// Convert from database model with repository count
    fn from_model_with_count(model: provider_account::Model, repo_count: u64) -> Self {
        let scopes: Vec<String> = model
            .scopes
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        Self {
            id: model.id,
            provider: model.provider,
            instance_url: model.instance_url,
            account_label: model.account_label,
            provider_username: model.provider_username,
            provider_email: model.provider_email,
            avatar_url: model.avatar_url,
            scopes,
            token_expires_at: model.token_expires_at.map(|dt| dt.to_rfc3339()),
            validation_status: model.validation_status,
            last_validated_at: model.last_validated_at.map(|dt| dt.to_rfc3339()),
            is_active: model.is_active,
            is_default: model.is_default,
            repository_count: repo_count as i64,
            created_at: model.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub is_valid: bool,
    pub validation_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub last_validated_at: String,
}

// ============================================================================
// Handler Functions
// ============================================================================

/// List all provider accounts for authenticated user
/// GET /api/accounts
pub async fn list_accounts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<ProviderAccountResponse>>>, ApiError> {
    // Find all accounts for the user
    let accounts = provider_account::Entity::find()
        .filter(provider_account::Column::UserId.eq(auth.user_id))
        .all(&state.db)
        .await?;

    let mut responses = Vec::new();

    // Get repository count for each account
    for account in accounts {
        let repo_count = repository::Entity::find()
            .filter(repository::Column::ProviderAccountId.eq(account.id))
            .count(&state.db)
            .await?;

        responses.push(ProviderAccountResponse::from_model_with_count(
            account, repo_count,
        ));
    }

    Ok(Json(ApiResponse::success(responses)))
}

/// Add a new PAT-based account
/// POST /api/accounts
pub async fn add_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AddAccountRequest>,
) -> Result<Json<ApiResponse<ProviderAccountResponse>>, ApiError> {
    // 1. Parse and validate provider type
    let provider_type: GitProvider = request
        .provider
        .parse()
        .map_err(|_| ApiError::bad_request(t!("errors.account.invalid_provider_type")))?;

    // 2. Create provider instance
    let provider = state
        .provider_factory
        .create(provider_type, request.instance_url.clone());

    // 3. Build credentials
    let credentials = ProviderCredentials::Pat {
        token: request.access_token.clone(),
        username: request.username.clone(),
    };

    // 4. Validate token with provider
    let validation = provider
        .validate_credentials(&credentials)
        .await
        .map_err(|e| ApiError::bad_request(t!("errors.account.validation_failed", error = e.to_string())))?;

    if !validation.is_valid {
        return Err(ApiError::bad_request(
            validation
                .error_message
                .unwrap_or_else(|| t!("errors.account.invalid_token")),
        ));
    }

    let provider_user_id = validation
        .user_id
        .ok_or_else(|| ApiError::bad_request(t!("errors.account.provider_no_user_id")))?;

    let provider_username = validation
        .username
        .ok_or_else(|| ApiError::bad_request(t!("errors.account.provider_no_username")))?;

    // 5. Check for duplicate account (same provider_user_id)
    let existing = provider_account::Entity::find()
        .filter(provider_account::Column::UserId.eq(auth.user_id))
        .filter(provider_account::Column::Provider.eq(&request.provider))
        .filter(provider_account::Column::InstanceUrl.eq(request.instance_url.as_ref().cloned()))
        .filter(provider_account::Column::ProviderUserId.eq(&provider_user_id))
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(ApiError::conflict(
            t!("errors.account.already_connected"),
        ));
    }

    // 6. Encrypt and store token
    let token_encrypted = state
        .encryption_service
        .encrypt(&request.access_token)
        .map_err(|e| ApiError::internal(t!("errors.encryption.failed", error = e.to_string())))?;

    // 7. Check if this should be the default account for this provider
    let is_first = provider_account::Entity::find()
        .filter(provider_account::Column::UserId.eq(auth.user_id))
        .filter(provider_account::Column::Provider.eq(&request.provider))
        .filter(provider_account::Column::InstanceUrl.eq(request.instance_url.as_ref().cloned()))
        .count(&state.db)
        .await?
        == 0;

    // 8. Create account record
    let now = Utc::now();
    let scopes_json = serde_json::to_string(&validation.scopes)
        .map_err(|e| ApiError::internal(t!("errors.general.serialization_failed", error = e.to_string())))?;

    let account = provider_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(auth.user_id),
        provider: Set(request.provider),
        instance_url: Set(request.instance_url),
        account_label: Set(request.account_label),
        provider_user_id: Set(provider_user_id),
        provider_username: Set(provider_username),
        provider_email: Set(validation.email),
        avatar_url: Set(validation.avatar_url),
        auth_type: Set("pat".to_string()),
        access_token_encrypted: Set(token_encrypted),
        auth_username: Set(request.username),
        scopes: Set(Some(scopes_json)),
        token_expires_at: Set(validation.expires_at),
        last_validated_at: Set(Some(now)),
        validation_status: Set("valid".to_string()),
        is_active: Set(true),
        is_default: Set(is_first), // First account is default
        created_at: Set(now),
        updated_at: Set(now),
    };

    let account = account.insert(&state.db).await?;

    tracing::info!(
        user_id = %auth.user_id,
        provider = %account.provider,
        account_label = %account.account_label,
        "Provider account created"
    );

    Ok(Json(ApiResponse::success(
        ProviderAccountResponse::from_model_with_count(account, 0),
    )))
}

/// Get single account details
/// GET /api/accounts/:id
pub async fn get_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProviderAccountResponse>>, ApiError> {
    // Find account and verify ownership
    let account = provider_account::Entity::find_by_id(account_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found(t!("errors.account.not_found")))?;

    // Verify ownership
    if account.user_id != auth.user_id {
        return Err(ApiError::not_found(t!("errors.account.not_found")));
    }

    // Get repository count
    let repo_count = repository::Entity::find()
        .filter(repository::Column::ProviderAccountId.eq(account_id))
        .count(&state.db)
        .await?;

    Ok(Json(ApiResponse::success(
        ProviderAccountResponse::from_model_with_count(account, repo_count),
    )))
}

/// Update account
/// PATCH /api/accounts/:id
pub async fn update_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<Json<ApiResponse<ProviderAccountResponse>>, ApiError> {
    // Find account and verify ownership
    let account = provider_account::Entity::find_by_id(account_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Account not found"))?;

    if account.user_id != auth.user_id {
        return Err(ApiError::not_found("Account not found"));
    }

    let mut active: provider_account::ActiveModel = account.clone().into();
    let now = Utc::now();

    // Update account_label if provided
    if let Some(label) = request.account_label {
        active.account_label = Set(label);
    }

    // Update is_active if provided
    if let Some(is_active) = request.is_active {
        active.is_active = Set(is_active);
    }

    // Update access_token if provided (re-validate and re-encrypt)
    if let Some(new_token) = request.access_token {
        // Create provider instance
        let provider_type: GitProvider = account
            .provider
            .parse()
            .map_err(|_| ApiError::internal(t!("errors.repository.invalid_provider_db")))?;
        let provider = state
            .provider_factory
            .create(provider_type, account.instance_url.clone());

        // Build credentials with new token
        let credentials = ProviderCredentials::Pat {
            token: new_token.clone(),
            username: account.auth_username.clone(),
        };

        // Validate new token
        let validation = provider
            .validate_credentials(&credentials)
            .await
            .map_err(|e| ApiError::bad_request(t!("errors.account.validation_failed", error = e.to_string())))?;

        if !validation.is_valid {
            return Err(ApiError::bad_request(
                validation
                    .error_message
                    .unwrap_or_else(|| t!("errors.account.invalid_token")),
            ));
        }

        // Encrypt new token
        let token_encrypted = state
            .encryption_service
            .encrypt(&new_token)
            .map_err(|e| ApiError::internal(t!("errors.encryption.failed", error = e.to_string())))?;

        active.access_token_encrypted = Set(token_encrypted);
        active.last_validated_at = Set(Some(now));
        active.validation_status = Set("valid".to_string());

        // Update scopes and expiration if available
        if !validation.scopes.is_empty() {
            let scopes_json = serde_json::to_string(&validation.scopes)
                .map_err(|e| ApiError::internal(t!("errors.general.serialization_failed", error = e.to_string())))?;
            active.scopes = Set(Some(scopes_json));
        }
        if validation.expires_at.is_some() {
            active.token_expires_at = Set(validation.expires_at);
        }
    }

    active.updated_at = Set(now);
    let updated = active.update(&state.db).await?;

    tracing::info!(
        user_id = %auth.user_id,
        account_id = %account_id,
        "Provider account updated"
    );

    // Get repository count
    let repo_count = repository::Entity::find()
        .filter(repository::Column::ProviderAccountId.eq(account_id))
        .count(&state.db)
        .await?;

    Ok(Json(ApiResponse::success(
        ProviderAccountResponse::from_model_with_count(updated, repo_count),
    )))
}

/// Delete account
/// DELETE /api/accounts/:id
pub async fn delete_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // Find account and verify ownership
    let account = provider_account::Entity::find_by_id(account_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Account not found"))?;

    if account.user_id != auth.user_id {
        return Err(ApiError::not_found("Account not found"));
    }

    // Use a transaction to handle repository updates and account deletion
    let txn = state.db.begin().await?;

    // Set provider_account_id to NULL for all repositories using this account
    repository::Entity::update_many()
        .col_expr(
            repository::Column::ProviderAccountId,
            sea_orm::sea_query::Expr::value(Option::<Uuid>::None),
        )
        .filter(repository::Column::ProviderAccountId.eq(account_id))
        .exec(&txn)
        .await?;

    // Delete the account
    provider_account::Entity::delete_by_id(account_id)
        .exec(&txn)
        .await?;

    txn.commit().await?;

    tracing::info!(
        user_id = %auth.user_id,
        account_id = %account_id,
        "Provider account deleted"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Validate account token
/// POST /api/accounts/:id/validate
pub async fn validate_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ValidationResult>>, ApiError> {
    // Find account and verify ownership
    let account = provider_account::Entity::find_by_id(account_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Account not found"))?;

    if account.user_id != auth.user_id {
        return Err(ApiError::not_found("Account not found"));
    }

    // Decrypt token
    let token = state
        .encryption_service
        .decrypt(&account.access_token_encrypted)
        .map_err(|e| ApiError::internal(t!("errors.encryption.decrypt_failed", error = e.to_string())))?;

    // Create provider instance
    let provider_type: GitProvider = account
        .provider
        .parse()
        .map_err(|_| ApiError::internal("Invalid provider type in database"))?;
    let provider = state
        .provider_factory
        .create(provider_type, account.instance_url.clone());

    // Build credentials
    let credentials = ProviderCredentials::Pat {
        token,
        username: account.auth_username.clone(),
    };

    // Validate with provider
    let validation = provider.validate_credentials(&credentials).await;

    let now = Utc::now();
    let mut active: provider_account::ActiveModel = account.into();

    let result = match validation {
        Ok(val) if val.is_valid => {
            active.validation_status = Set("valid".to_string());
            active.last_validated_at = Set(Some(now));
            active.updated_at = Set(now);

            ValidationResult {
                is_valid: true,
                validation_status: "valid".to_string(),
                error_message: None,
                last_validated_at: now.to_rfc3339(),
            }
        }
        Ok(val) => {
            active.validation_status = Set("invalid".to_string());
            active.last_validated_at = Set(Some(now));
            active.updated_at = Set(now);

            ValidationResult {
                is_valid: false,
                validation_status: "invalid".to_string(),
                error_message: val.error_message,
                last_validated_at: now.to_rfc3339(),
            }
        }
        Err(e) => {
            active.validation_status = Set("invalid".to_string());
            active.last_validated_at = Set(Some(now));
            active.updated_at = Set(now);

            ValidationResult {
                is_valid: false,
                validation_status: "invalid".to_string(),
                error_message: Some(e.to_string()),
                last_validated_at: now.to_rfc3339(),
            }
        }
    };

    active.update(&state.db).await?;

    tracing::info!(
        account_id = %account_id,
        is_valid = %result.is_valid,
        "Provider account validated"
    );

    Ok(Json(ApiResponse::success(result)))
}

/// Set account as default for its provider
/// POST /api/accounts/:id/set-default
pub async fn set_default_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProviderAccountResponse>>, ApiError> {
    // Find account and verify ownership
    let account = provider_account::Entity::find_by_id(account_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Account not found"))?;

    if account.user_id != auth.user_id {
        return Err(ApiError::not_found("Account not found"));
    }

    // Use a transaction to ensure atomicity
    let txn = state.db.begin().await?;

    // Unset previous default for this provider and instance
    provider_account::Entity::update_many()
        .col_expr(
            provider_account::Column::IsDefault,
            sea_orm::sea_query::Expr::value(false),
        )
        .filter(provider_account::Column::UserId.eq(auth.user_id))
        .filter(provider_account::Column::Provider.eq(&account.provider))
        .filter(provider_account::Column::InstanceUrl.eq(account.instance_url.as_ref().cloned()))
        .filter(provider_account::Column::IsDefault.eq(true))
        .exec(&txn)
        .await?;

    // Set this account as default
    let mut active: provider_account::ActiveModel = account.into();
    active.is_default = Set(true);
    active.updated_at = Set(Utc::now());

    let updated = active.update(&txn).await?;
    txn.commit().await?;

    tracing::info!(
        user_id = %auth.user_id,
        account_id = %account_id,
        "Provider account set as default"
    );

    // Get repository count
    let repo_count = repository::Entity::find()
        .filter(repository::Column::ProviderAccountId.eq(account_id))
        .count(&state.db)
        .await?;

    Ok(Json(ApiResponse::success(
        ProviderAccountResponse::from_model_with_count(updated, repo_count),
    )))
}
