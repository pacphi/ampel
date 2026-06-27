//! Remediation playbook CRUD + preview (Phase 4 — ADR-006).
//!
//! Playbooks are DB-stored overrides of the embedded default remediation
//! playbook. They are global operator resources (keyed by `playbook_id` slug +
//! `version`), so every endpoint requires authentication.
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
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::remediation::FailureClass;
use ampel_db::entities::remediation_playbook;
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
// Handlers
// ============================================================================

/// GET /api/remediation/playbooks
pub async fn list_playbooks(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<PlaybookResponse>>>, ApiError> {
    let rows = remediation_playbook::Entity::find().all(&state.db).await?;
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
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PlaybookResponse>>, ApiError> {
    let row = remediation_playbook::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Playbook not found"))?;
    Ok(Json(ApiResponse::success(PlaybookResponse::from(row))))
}

/// PATCH /api/remediation/playbooks/{id}
pub async fn update_playbook(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePlaybookRequest>,
) -> Result<Json<ApiResponse<PlaybookResponse>>, ApiError> {
    let row = remediation_playbook::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Playbook not found"))?;

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
    let row = remediation_playbook::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Playbook not found"))?;
    remediation_playbook::Entity::delete_by_id(row.id)
        .exec(&state.db)
        .await?;
    tracing::info!(user_id = %auth.user_id, playbook_id = %row.playbook_id, "Remediation playbook deleted");
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/remediation/playbooks/{id}/preview — render the prompt, no model call.
pub async fn preview_playbook(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<PreviewRequest>,
) -> Result<Json<ApiResponse<PreviewResponse>>, ApiError> {
    let row = remediation_playbook::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Playbook not found"))?;

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
