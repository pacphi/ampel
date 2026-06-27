//! Remediation playbook CRUD + preview (Phase 4 — ADR-006).
//!
//! Playbooks are DB-stored overrides of the embedded default remediation
//! playbook. They drive the agentic remediation prompts, so they are **owned
//! resources**: every read and write is gated on the caller's access to the
//! playbook's `(scope_type, scope_id)`, mirroring `remediation_policy` /
//! `model_provider_account`.
//!
//! ## Authorization
//! - `scope_type=user`   → `scope_id == auth.user_id`.
//! - `scope_type=org`    → caller owns the organization.
//! - `scope_type=team`   → caller is an **admin** member (`team_member.role='admin'`).
//! - `scope_type=repository` → caller owns the repository.
//! - `scope_id IS NULL`  → built-in/global sentinel: readable by any authenticated
//!   caller, mutable by none (writes 404).
//!
//! Cross-scope reads return `404` (never leak existence); creating in a scope the
//! caller does not administer returns `403`. `list` returns only accessible rows.
//!
//! ## Preview (no model call)
//! `POST /api/remediation/playbooks/{id}/preview` resolves the stored YAML
//! through the worker's playbook resolver — which applies the ADR-006 tools-policy
//! CEILING (an override can only REMOVE tools) — and renders the trusted `system`
//! instruction with minijinja under STRICT undefined semantics, against TRUSTED
//! metadata only (repo name, branch, failure class). It NEVER calls a model and
//! NEVER interpolates untrusted content. The response also reports the resolved
//! output contract and clamped tools so an operator can lint a playbook safely.
//!
//! `ampel-api` already depends on `ampel-worker` (Cargo.toml), so the preview
//! reuses the worker's `playbook` + `playbook_resolver` modules directly — there
//! is no need to relocate playbook rendering into `ampel-core`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::remediation::{FailureClass, ScopeType};
use ampel_db::entities::{organization, remediation_playbook, repository, team_member};
use ampel_worker::services::playbook_resolver::{
    build_system_instruction, resolve, PlaybookContext, PlaybookScope,
};

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlaybookRequest {
    pub playbook_id: String,
    pub version: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    /// YAML playbook body (validated by parsing on write).
    pub content: String,
    pub source: Option<String>,
    pub enabled: Option<bool>,
    /// Ownership scope; defaults to `user` (owned by the caller).
    pub scope_type: Option<ScopeType>,
    /// Owning scope UUID. Required for non-`user` scopes; defaults to the caller
    /// for `user` scope.
    pub scope_id: Option<Uuid>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlaybookRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybookResponse {
    pub id: Uuid,
    pub playbook_id: String,
    pub version: i32,
    pub source: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub enabled: bool,
    pub scope_type: String,
    pub scope_id: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<remediation_playbook::Model> for PlaybookResponse {
    fn from(m: remediation_playbook::Model) -> Self {
        Self {
            id: m.id,
            playbook_id: m.playbook_id,
            version: m.version,
            source: m.source,
            name: m.name,
            description: m.description,
            content: m.content,
            enabled: m.enabled,
            scope_type: m.scope_type,
            scope_id: m.scope_id,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRequest {
    /// Failure class to select the task template (default `build_error`).
    pub failure_class: Option<String>,
    /// Trusted repo metadata for template rendering.
    pub repo_full_name: Option<String>,
    pub base_branch: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResponse {
    pub failure_class: String,
    pub role: String,
    /// The fully assembled, prompt-injection-safe trusted `system` instruction.
    pub system_instruction: String,
    pub output_contract: String,
    /// Tools after the ADR-006 ceiling clamp (override can only remove).
    pub allowed_tools: Vec<String>,
}

// ============================================================================
// Scope / ownership authorization
// ============================================================================

/// Whether `user_id` administers `(scope_type, scope_id)` — owner for
/// org/repository, `admin` member for team, self for user.
async fn scope_admin_access(
    state: &AppState,
    user_id: Uuid,
    scope_type: ScopeType,
    scope_id: Uuid,
) -> Result<bool, ApiError> {
    let allowed = match scope_type {
        ScopeType::User => scope_id == user_id,
        ScopeType::Repository => repository::Entity::find_by_id(scope_id)
            .one(&state.db)
            .await?
            .map(|r| r.user_id == user_id)
            .unwrap_or(false),
        ScopeType::Team => {
            team_member::Entity::find()
                .filter(team_member::Column::TeamId.eq(scope_id))
                .filter(team_member::Column::UserId.eq(user_id))
                .filter(team_member::Column::Role.eq("admin"))
                .count(&state.db)
                .await?
                > 0
        }
        ScopeType::Org => organization::Entity::find_by_id(scope_id)
            .one(&state.db)
            .await?
            .map(|o| o.owner_id == user_id)
            .unwrap_or(false),
    };
    Ok(allowed)
}

/// Read access: built-in/global rows (`scope_id IS NULL`) are readable by any
/// authenticated caller; scoped rows require administering their scope.
async fn can_read(
    state: &AppState,
    user_id: Uuid,
    row: &remediation_playbook::Model,
) -> Result<bool, ApiError> {
    match row.scope_id {
        None => Ok(true),
        Some(scope_id) => {
            let scope_type: ScopeType = row
                .scope_type
                .parse()
                .map_err(|_| ApiError::internal("invalid scope_type in database"))?;
            scope_admin_access(state, user_id, scope_type, scope_id).await
        }
    }
}

/// Write access: built-in/global rows are immutable; scoped rows require
/// administering their scope.
async fn can_write(
    state: &AppState,
    user_id: Uuid,
    row: &remediation_playbook::Model,
) -> Result<bool, ApiError> {
    match row.scope_id {
        None => Ok(false),
        Some(scope_id) => {
            let scope_type: ScopeType = row
                .scope_type
                .parse()
                .map_err(|_| ApiError::internal("invalid scope_type in database"))?;
            scope_admin_access(state, user_id, scope_type, scope_id).await
        }
    }
}

/// Load a playbook the caller may read, else `404` (no existence leak).
async fn load_readable(
    state: &AppState,
    user_id: Uuid,
    id: Uuid,
) -> Result<remediation_playbook::Model, ApiError> {
    let row = remediation_playbook::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Playbook not found"))?;
    if can_read(state, user_id, &row).await? {
        Ok(row)
    } else {
        Err(ApiError::not_found("Playbook not found"))
    }
}

/// Load a playbook the caller may modify, else `404` (no existence leak).
async fn load_writable(
    state: &AppState,
    user_id: Uuid,
    id: Uuid,
) -> Result<remediation_playbook::Model, ApiError> {
    let row = remediation_playbook::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Playbook not found"))?;
    if can_write(state, user_id, &row).await? {
        Ok(row)
    } else {
        Err(ApiError::not_found("Playbook not found"))
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/remediation/playbooks — only rows the caller can access (their scopes
/// plus built-in/global sentinels).
pub async fn list_playbooks(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<PlaybookResponse>>>, ApiError> {
    let user_id = auth.user_id;

    let owned_org_ids: Vec<Uuid> = organization::Entity::find()
        .filter(organization::Column::OwnerId.eq(user_id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|o| o.id)
        .collect();
    let admin_team_ids: Vec<Uuid> = team_member::Entity::find()
        .filter(team_member::Column::UserId.eq(user_id))
        .filter(team_member::Column::Role.eq("admin"))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|m| m.team_id)
        .collect();
    let owned_repo_ids: Vec<Uuid> = repository::Entity::find()
        .filter(repository::Column::UserId.eq(user_id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect();

    // Built-in/global sentinels (scope_id IS NULL) are visible to everyone.
    let mut condition = Condition::any().add(remediation_playbook::Column::ScopeId.is_null());
    condition = condition.add(
        Condition::all()
            .add(remediation_playbook::Column::ScopeType.eq(ScopeType::User.to_string()))
            .add(remediation_playbook::Column::ScopeId.eq(user_id)),
    );
    if !owned_org_ids.is_empty() {
        condition = condition.add(
            Condition::all()
                .add(remediation_playbook::Column::ScopeType.eq(ScopeType::Org.to_string()))
                .add(remediation_playbook::Column::ScopeId.is_in(owned_org_ids)),
        );
    }
    if !admin_team_ids.is_empty() {
        condition = condition.add(
            Condition::all()
                .add(remediation_playbook::Column::ScopeType.eq(ScopeType::Team.to_string()))
                .add(remediation_playbook::Column::ScopeId.is_in(admin_team_ids)),
        );
    }
    if !owned_repo_ids.is_empty() {
        condition = condition.add(
            Condition::all()
                .add(remediation_playbook::Column::ScopeType.eq(ScopeType::Repository.to_string()))
                .add(remediation_playbook::Column::ScopeId.is_in(owned_repo_ids)),
        );
    }

    let rows = remediation_playbook::Entity::find()
        .filter(condition)
        .all(&state.db)
        .await?;
    Ok(Json(ApiResponse::success(
        rows.into_iter().map(PlaybookResponse::from).collect(),
    )))
}

/// POST /api/remediation/playbooks — validates YAML before storing.
pub async fn create_playbook(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreatePlaybookRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PlaybookResponse>>), ApiError> {
    // Resolve and authorize the ownership scope before any work. User scope
    // defaults to the caller; other scopes require an explicit scope_id.
    let scope_type = req.scope_type.unwrap_or(ScopeType::User);
    let scope_id = match scope_type {
        ScopeType::User => req.scope_id.unwrap_or(auth.user_id),
        _ => req.scope_id.ok_or_else(|| {
            ApiError::bad_request("scope_id is required for non-user playbook scopes")
        })?,
    };
    if !scope_admin_access(&state, auth.user_id, scope_type, scope_id).await? {
        return Err(ApiError::forbidden(
            "you are not an administrator of the target playbook scope",
        ));
    }

    // Lint: the YAML must parse into a valid Playbook (ADR-006 schema).
    ampel_worker::services::playbook::Playbook::from_yaml(&req.content)
        .map_err(|e| ApiError::unprocessable_entity(format!("invalid playbook YAML: {e}")))?;

    let now = Utc::now();
    let model = remediation_playbook::ActiveModel {
        id: Set(Uuid::new_v4()),
        playbook_id: Set(req.playbook_id),
        version: Set(req.version.unwrap_or(1)),
        source: Set(req.source.unwrap_or_else(|| "db".to_string())),
        name: Set(req.name),
        description: Set(req.description),
        content: Set(req.content),
        enabled: Set(req.enabled.unwrap_or(true)),
        scope_type: Set(scope_type.to_string()),
        scope_id: Set(Some(scope_id)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let created = model.insert(&state.db).await?;
    tracing::info!(user_id = %auth.user_id, playbook_id = %created.playbook_id, "Remediation playbook created");
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(PlaybookResponse::from(created))),
    ))
}

/// GET /api/remediation/playbooks/{id}
pub async fn get_playbook(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PlaybookResponse>>, ApiError> {
    let row = load_readable(&state, auth.user_id, id).await?;
    Ok(Json(ApiResponse::success(PlaybookResponse::from(row))))
}

/// PATCH /api/remediation/playbooks/{id}
pub async fn update_playbook(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePlaybookRequest>,
) -> Result<Json<ApiResponse<PlaybookResponse>>, ApiError> {
    let row = load_writable(&state, auth.user_id, id).await?;

    if let Some(content) = &req.content {
        ampel_worker::services::playbook::Playbook::from_yaml(content)
            .map_err(|e| ApiError::unprocessable_entity(format!("invalid playbook YAML: {e}")))?;
    }

    let mut active: remediation_playbook::ActiveModel = row.into();
    if let Some(v) = req.name {
        active.name = Set(v);
    }
    if let Some(v) = req.description {
        active.description = Set(Some(v));
    }
    if let Some(v) = req.content {
        active.content = Set(v);
    }
    if let Some(v) = req.enabled {
        active.enabled = Set(v);
    }
    active.updated_at = Set(Utc::now());
    let updated = active.update(&state.db).await?;
    tracing::info!(user_id = %auth.user_id, playbook_id = %updated.playbook_id, "Remediation playbook updated");
    Ok(Json(ApiResponse::success(PlaybookResponse::from(updated))))
}

/// DELETE /api/remediation/playbooks/{id}
pub async fn delete_playbook(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let row = load_writable(&state, auth.user_id, id).await?;
    remediation_playbook::Entity::delete_by_id(row.id)
        .exec(&state.db)
        .await?;
    tracing::info!(user_id = %auth.user_id, playbook_id = %row.playbook_id, "Remediation playbook deleted");
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/remediation/playbooks/{id}/preview — render the prompt, no model call.
pub async fn preview_playbook(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<PreviewRequest>,
) -> Result<Json<ApiResponse<PreviewResponse>>, ApiError> {
    let row = load_readable(&state, auth.user_id, id).await?;

    let failure_class: FailureClass = req
        .failure_class
        .as_deref()
        .unwrap_or("build_error")
        .parse()
        .map_err(|_| ApiError::bad_request("invalid failure_class"))?;

    // Resolve as a DB override so the ADR-006 ceiling clamp is applied.
    let playbook = resolve(PlaybookScope::Global, None, Some(&row.content))
        .map_err(|e| ApiError::unprocessable_entity(format!("playbook resolve failed: {e}")))?;

    let task = playbook
        .select_task(failure_class)
        .map_err(|e| ApiError::unprocessable_entity(format!("playbook task select failed: {e}")))?;

    let ctx = PlaybookContext {
        repo_full_name: req
            .repo_full_name
            .unwrap_or_else(|| "owner/repo".to_string()),
        base_branch: req.base_branch.unwrap_or_else(|| "main".to_string()),
        failure_class: failure_class.to_string(),
    };

    let system_instruction = build_system_instruction(&playbook, task, &ctx)
        .map_err(|e| ApiError::unprocessable_entity(format!("playbook render failed: {e}")))?;

    Ok(Json(ApiResponse::success(PreviewResponse {
        failure_class: failure_class.to_string(),
        role: playbook.role.clone(),
        system_instruction,
        output_contract: playbook.output_contract.clone(),
        allowed_tools: playbook.tools_policy.allowed.clone(),
    })))
}
