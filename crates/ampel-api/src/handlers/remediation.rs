//! Fleet PR Remediation — Phase 1 (Policy CRUD + Dry-Run) API layer.
//!
//! Policy **writes** (create/update/delete/toggle) are implemented here against
//! `ampel-db`'s canonical `remediation_policy` ActiveModel, because `ampel-core`
//! cannot depend on `ampel-db` (dependency cycle) and its `RemediationService`
//! is read-only. Read/planning endpoints (`/preview`, `/fleet`) delegate to
//! `ampel-core`'s `RemediationService` / `PolicyResolver`.
//!
//! Security: `/preview` and `/fleet` are READ-ONLY. No `RemediationCapable`
//! provider is constructed anywhere in this module, so no repository write
//! primitive (push/merge/comment) is reachable from these paths. No secrets are
//! logged.

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

use ampel_core::remediation::{
    AutonomyLevel, ConsolidationPlan, PrSelectionStrategy, RemediationTier, ScopeType,
};
use ampel_core::services::{PolicyResolver, RemediationService};
use ampel_db::entities::{organization, pull_request, remediation_policy, repository, team_member};

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

// ============================================================================
// DTOs
// ============================================================================

/// Create payload. Enums map to `ampel_core::remediation` value-object types and
/// (de)serialize as snake_case, round-tripping the DB string columns.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePolicyRequest {
    pub scope_type: ScopeType,
    pub scope_id: Uuid,
    pub enabled: Option<bool>,
    pub min_open_prs: i32,
    pub pr_selection: Option<PrSelectionStrategy>,
    pub autonomy_level: AutonomyLevel,
    /// Required when `autonomy_level` is `fully_autonomous` (DDD invariant).
    pub remediation_tier: Option<RemediationTier>,
    pub max_prs_per_run: i32,
    pub allowed_targets: Option<Vec<String>>,
    pub skip_draft: Option<bool>,
    pub require_green_before_merge: Option<bool>,
    pub air_gapped: Option<bool>,
    pub auto_merge_enabled: Option<bool>,
    pub auto_merge_rule: Option<String>,
    pub require_human_approval: Option<bool>,
    pub agent_budget: Option<serde_json::Value>,
    pub notification_config: Option<serde_json::Value>,
    pub playbook_ref: Option<String>,
}

/// Partial update payload. Only present fields are applied.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePolicyRequest {
    pub enabled: Option<bool>,
    pub min_open_prs: Option<i32>,
    pub pr_selection: Option<PrSelectionStrategy>,
    pub autonomy_level: Option<AutonomyLevel>,
    pub remediation_tier: Option<RemediationTier>,
    pub max_prs_per_run: Option<i32>,
    pub allowed_targets: Option<Vec<String>>,
    pub skip_draft: Option<bool>,
    pub require_green_before_merge: Option<bool>,
    pub air_gapped: Option<bool>,
    pub auto_merge_enabled: Option<bool>,
    pub auto_merge_rule: Option<String>,
    pub require_human_approval: Option<bool>,
    pub agent_budget: Option<serde_json::Value>,
    pub notification_config: Option<serde_json::Value>,
    pub playbook_ref: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyResponse {
    pub id: Uuid,
    pub scope_type: String,
    pub scope_id: Uuid,
    pub enabled: bool,
    pub min_open_prs: i32,
    pub pr_selection: PrSelectionStrategy,
    pub autonomy_level: String,
    pub remediation_tier: String,
    pub max_prs_per_run: i32,
    pub allowed_targets: Vec<String>,
    pub skip_draft: bool,
    pub require_green_before_merge: bool,
    pub air_gapped: bool,
    pub auto_merge_enabled: bool,
    pub auto_merge_rule: Option<String>,
    pub require_human_approval: bool,
    pub agent_budget: Option<serde_json::Value>,
    pub notification_config: Option<serde_json::Value>,
    pub playbook_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<remediation_policy::Model> for PolicyResponse {
    fn from(m: remediation_policy::Model) -> Self {
        Self {
            id: m.id,
            scope_type: m.scope_type,
            scope_id: m.scope_id,
            enabled: m.enabled,
            min_open_prs: m.min_open_prs,
            pr_selection: serde_json::from_str(&m.pr_selection).unwrap_or_default(),
            autonomy_level: m.autonomy_level,
            remediation_tier: m.remediation_tier,
            max_prs_per_run: m.max_prs_per_run,
            allowed_targets: serde_json::from_str(&m.allowed_targets).unwrap_or_default(),
            skip_draft: m.skip_draft,
            require_green_before_merge: m.require_green_before_merge,
            air_gapped: m.air_gapped,
            auto_merge_enabled: m.auto_merge_enabled,
            auto_merge_rule: m.auto_merge_rule,
            require_human_approval: m.require_human_approval,
            agent_budget: m
                .agent_budget
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok()),
            notification_config: m
                .notification_config
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok()),
            playbook_ref: m.playbook_ref,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FleetRow {
    pub repository_id: Uuid,
    pub name: String,
    pub open_pr_count: i64,
    pub eligible: bool,
    /// One of: `none`, `disabled`, `dry_run`, `suggest`, `auto_with_approval`, `auto_merge`.
    pub policy_state: String,
    pub air_gapped: bool,
}

// ============================================================================
// Validation
// ============================================================================

/// Validate the cross-field invariants shared by create and update.
fn validate_invariants(
    auto_merge_enabled: bool,
    require_human_approval: bool,
    min_open_prs: i32,
    max_prs_per_run: i32,
) -> Result<(), ApiError> {
    if auto_merge_enabled && require_human_approval {
        return Err(ApiError::unprocessable_entity(
            "auto_merge_enabled cannot be combined with require_human_approval",
        ));
    }
    if min_open_prs < 1 {
        return Err(ApiError::unprocessable_entity("min_open_prs must be >= 1"));
    }
    if max_prs_per_run < 1 {
        return Err(ApiError::unprocessable_entity(
            "max_prs_per_run must be >= 1",
        ));
    }
    Ok(())
}

// ============================================================================
// Scope / tenant authorization
// ============================================================================

/// Ensure `user_id` may manage policies for `(scope_type, scope_id)`. Returns a
/// 404 (rather than 403) on denial to avoid leaking the existence of resources.
async fn assert_scope_access(
    state: &AppState,
    user_id: Uuid,
    scope_type: ScopeType,
    scope_id: Uuid,
) -> Result<(), ApiError> {
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

    if allowed {
        Ok(())
    } else {
        Err(ApiError::not_found("Policy scope not found"))
    }
}

/// Collect the scope ids the caller can manage, grouped by scope type.
struct CallerScopes {
    user_id: Uuid,
    repo_ids: Vec<Uuid>,
    team_ids: Vec<Uuid>,
    org_ids: Vec<Uuid>,
}

async fn caller_scopes(state: &AppState, user_id: Uuid) -> Result<CallerScopes, ApiError> {
    let repo_ids = repository::Entity::find()
        .filter(repository::Column::UserId.eq(user_id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect();

    let team_ids: Vec<Uuid> = team_member::Entity::find()
        .filter(team_member::Column::UserId.eq(user_id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|m| m.team_id)
        .collect();

    let org_ids = organization::Entity::find()
        .filter(organization::Column::OwnerId.eq(user_id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|o| o.id)
        .collect();

    Ok(CallerScopes {
        user_id,
        repo_ids,
        team_ids,
        org_ids,
    })
}

// ============================================================================
// Policy CRUD
// ============================================================================

/// GET /api/remediation/policies
pub async fn list_policies(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<PolicyResponse>>>, ApiError> {
    let scopes = caller_scopes(&state, auth.user_id).await?;

    let mut condition = Condition::any().add(
        Condition::all()
            .add(remediation_policy::Column::ScopeType.eq(ScopeType::User.to_string()))
            .add(remediation_policy::Column::ScopeId.eq(scopes.user_id)),
    );
    if !scopes.repo_ids.is_empty() {
        condition = condition.add(
            Condition::all()
                .add(remediation_policy::Column::ScopeType.eq(ScopeType::Repository.to_string()))
                .add(remediation_policy::Column::ScopeId.is_in(scopes.repo_ids)),
        );
    }
    if !scopes.team_ids.is_empty() {
        condition = condition.add(
            Condition::all()
                .add(remediation_policy::Column::ScopeType.eq(ScopeType::Team.to_string()))
                .add(remediation_policy::Column::ScopeId.is_in(scopes.team_ids)),
        );
    }
    if !scopes.org_ids.is_empty() {
        condition = condition.add(
            Condition::all()
                .add(remediation_policy::Column::ScopeType.eq(ScopeType::Org.to_string()))
                .add(remediation_policy::Column::ScopeId.is_in(scopes.org_ids)),
        );
    }

    let policies = remediation_policy::Entity::find()
        .filter(condition)
        .all(&state.db)
        .await?;

    let responses = policies.into_iter().map(PolicyResponse::from).collect();
    Ok(Json(ApiResponse::success(responses)))
}

/// POST /api/remediation/policies
pub async fn create_policy(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreatePolicyRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PolicyResponse>>), ApiError> {
    assert_scope_access(&state, auth.user_id, req.scope_type, req.scope_id).await?;

    let auto_merge_enabled = req.auto_merge_enabled.unwrap_or(false);
    let require_human_approval = req.require_human_approval.unwrap_or(false);
    validate_invariants(
        auto_merge_enabled,
        require_human_approval,
        req.min_open_prs,
        req.max_prs_per_run,
    )?;

    // DDD invariant: fully_autonomous requires an explicit remediation_tier.
    if req.autonomy_level == AutonomyLevel::FullyAutonomous && req.remediation_tier.is_none() {
        return Err(ApiError::unprocessable_entity(
            "fully_autonomous requires an explicit remediation_tier",
        ));
    }
    let remediation_tier = req
        .remediation_tier
        .unwrap_or(RemediationTier::ConsolidateOnly);

    let pr_selection = req.pr_selection.unwrap_or_default();
    let pr_selection_json = serde_json::to_string(&pr_selection)
        .map_err(|e| ApiError::internal(format!("serialize pr_selection: {e}")))?;
    let allowed_targets = req.allowed_targets.unwrap_or_default();
    let allowed_targets_json = serde_json::to_string(&allowed_targets)
        .map_err(|e| ApiError::internal(format!("serialize allowed_targets: {e}")))?;

    let now = Utc::now();
    let model = remediation_policy::ActiveModel {
        id: Set(Uuid::new_v4()),
        scope_type: Set(req.scope_type.to_string()),
        scope_id: Set(req.scope_id),
        enabled: Set(req.enabled.unwrap_or(true)),
        min_open_prs: Set(req.min_open_prs),
        pr_selection: Set(pr_selection_json),
        autonomy_level: Set(req.autonomy_level.to_string()),
        remediation_tier: Set(remediation_tier.to_string()),
        max_prs_per_run: Set(req.max_prs_per_run),
        allowed_targets: Set(allowed_targets_json),
        skip_draft: Set(req.skip_draft.unwrap_or(true)),
        require_green_before_merge: Set(req.require_green_before_merge.unwrap_or(true)),
        air_gapped: Set(req.air_gapped.unwrap_or(false)),
        auto_merge_enabled: Set(auto_merge_enabled),
        auto_merge_rule: Set(req.auto_merge_rule),
        require_human_approval: Set(require_human_approval),
        agent_budget: Set(req.agent_budget.map(|v| v.to_string())),
        notification_config: Set(req.notification_config.map(|v| v.to_string())),
        playbook_ref: Set(req.playbook_ref),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let created = model.insert(&state.db).await?;
    tracing::info!(
        user_id = %auth.user_id,
        policy_id = %created.id,
        scope_type = %created.scope_type,
        "Remediation policy created"
    );

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(PolicyResponse::from(created))),
    ))
}

/// Load a policy and assert the caller may access its scope.
async fn load_authorized_policy(
    state: &AppState,
    user_id: Uuid,
    policy_id: Uuid,
) -> Result<remediation_policy::Model, ApiError> {
    let policy = remediation_policy::Entity::find_by_id(policy_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Policy not found"))?;

    let scope_type: ScopeType = policy
        .scope_type
        .parse()
        .map_err(|_| ApiError::internal("invalid scope_type in database"))?;
    assert_scope_access(state, user_id, scope_type, policy.scope_id).await?;
    Ok(policy)
}

/// GET /api/remediation/policies/:id
pub async fn get_policy(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(policy_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PolicyResponse>>, ApiError> {
    let policy = load_authorized_policy(&state, auth.user_id, policy_id).await?;
    Ok(Json(ApiResponse::success(PolicyResponse::from(policy))))
}

/// PATCH /api/remediation/policies/:id
pub async fn update_policy(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(policy_id): Path<Uuid>,
    Json(req): Json<UpdatePolicyRequest>,
) -> Result<Json<ApiResponse<PolicyResponse>>, ApiError> {
    let policy = load_authorized_policy(&state, auth.user_id, policy_id).await?;

    // Compute effective values for invariant validation.
    let effective_auto_merge = req.auto_merge_enabled.unwrap_or(policy.auto_merge_enabled);
    let effective_human_approval = req
        .require_human_approval
        .unwrap_or(policy.require_human_approval);
    let effective_min = req.min_open_prs.unwrap_or(policy.min_open_prs);
    let effective_max = req.max_prs_per_run.unwrap_or(policy.max_prs_per_run);
    validate_invariants(
        effective_auto_merge,
        effective_human_approval,
        effective_min,
        effective_max,
    )?;

    let mut active: remediation_policy::ActiveModel = policy.into();

    if let Some(v) = req.enabled {
        active.enabled = Set(v);
    }
    if let Some(v) = req.min_open_prs {
        active.min_open_prs = Set(v);
    }
    if let Some(v) = req.pr_selection {
        let json = serde_json::to_string(&v)
            .map_err(|e| ApiError::internal(format!("serialize pr_selection: {e}")))?;
        active.pr_selection = Set(json);
    }
    if let Some(v) = req.autonomy_level {
        active.autonomy_level = Set(v.to_string());
    }
    if let Some(v) = req.remediation_tier {
        active.remediation_tier = Set(v.to_string());
    }
    if let Some(v) = req.max_prs_per_run {
        active.max_prs_per_run = Set(v);
    }
    if let Some(v) = req.allowed_targets {
        let json = serde_json::to_string(&v)
            .map_err(|e| ApiError::internal(format!("serialize allowed_targets: {e}")))?;
        active.allowed_targets = Set(json);
    }
    if let Some(v) = req.skip_draft {
        active.skip_draft = Set(v);
    }
    if let Some(v) = req.require_green_before_merge {
        active.require_green_before_merge = Set(v);
    }
    if let Some(v) = req.air_gapped {
        active.air_gapped = Set(v);
    }
    if let Some(v) = req.auto_merge_enabled {
        active.auto_merge_enabled = Set(v);
    }
    if let Some(v) = req.auto_merge_rule {
        active.auto_merge_rule = Set(Some(v));
    }
    if let Some(v) = req.require_human_approval {
        active.require_human_approval = Set(v);
    }
    if let Some(v) = req.agent_budget {
        active.agent_budget = Set(Some(v.to_string()));
    }
    if let Some(v) = req.notification_config {
        active.notification_config = Set(Some(v.to_string()));
    }
    if let Some(v) = req.playbook_ref {
        active.playbook_ref = Set(Some(v));
    }
    active.updated_at = Set(Utc::now());

    let updated = active.update(&state.db).await?;
    tracing::info!(user_id = %auth.user_id, policy_id = %updated.id, "Remediation policy updated");
    Ok(Json(ApiResponse::success(PolicyResponse::from(updated))))
}

/// DELETE /api/remediation/policies/:id
pub async fn delete_policy(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(policy_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let policy = load_authorized_policy(&state, auth.user_id, policy_id).await?;
    remediation_policy::Entity::delete_by_id(policy.id)
        .exec(&state.db)
        .await?;
    tracing::info!(user_id = %auth.user_id, policy_id = %policy_id, "Remediation policy deleted");
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/remediation/policies/:id/toggle
pub async fn toggle_policy(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(policy_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PolicyResponse>>, ApiError> {
    let policy = load_authorized_policy(&state, auth.user_id, policy_id).await?;
    let next = !policy.enabled;
    let mut active: remediation_policy::ActiveModel = policy.into();
    active.enabled = Set(next);
    active.updated_at = Set(Utc::now());
    let updated = active.update(&state.db).await?;
    tracing::info!(
        user_id = %auth.user_id,
        policy_id = %updated.id,
        enabled = %next,
        "Remediation policy toggled"
    );
    Ok(Json(ApiResponse::success(PolicyResponse::from(updated))))
}

// ============================================================================
// Read-only planning: preview + fleet
// ============================================================================

/// POST /api/remediation/repositories/:repo_id/preview
///
/// Read-only dry run. Delegates to `ampel-core`'s read-only `RemediationService`;
/// performs ZERO repository writes.
pub async fn preview_repository(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(repo_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ConsolidationPlan>>, ApiError> {
    // Tenant scoping: caller must own the repository.
    let repo = repository::Entity::find_by_id(repo_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    let service = RemediationService::new(state.db.clone());
    let plan = service.preview(repo_id).await?;
    Ok(Json(ApiResponse::success(plan)))
}

/// GET /api/remediation/fleet
///
/// Per-repo eligibility + policy state for the caller's managed repos. Read-only.
pub async fn get_fleet(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<FleetRow>>>, ApiError> {
    let repos = repository::Entity::find()
        .filter(repository::Column::UserId.eq(auth.user_id))
        .all(&state.db)
        .await?;

    // Owned-org air-gapped ceiling (matches ADR-014 for the common case).
    let org_air_gapped = organization::Entity::find()
        .filter(organization::Column::OwnerId.eq(auth.user_id))
        .filter(organization::Column::AirGapped.eq(true))
        .count(&state.db)
        .await?
        > 0;

    let resolver = PolicyResolver::new(state.db.clone());
    let mut rows = Vec::with_capacity(repos.len());

    for repo in repos {
        let open_pr_count = pull_request::Entity::find()
            .filter(pull_request::Column::RepositoryId.eq(repo.id))
            .filter(pull_request::Column::State.eq("open"))
            .count(&state.db)
            .await? as i64;

        // `resolve` only returns enabled, fully-resolved policies.
        let resolved = resolver.resolve(repo.id).await?;

        let (eligible, policy_state, air_gapped) = match resolved {
            Some(criteria) => (
                open_pr_count >= criteria.min_open_prs as i64,
                policy_state_label(criteria.autonomy_level).to_string(),
                criteria.air_gapped,
            ),
            None => {
                // Distinguish a disabled repo-scoped policy from no policy at all.
                let disabled = remediation_policy::Entity::find()
                    .filter(
                        remediation_policy::Column::ScopeType.eq(ScopeType::Repository.to_string()),
                    )
                    .filter(remediation_policy::Column::ScopeId.eq(repo.id))
                    .filter(remediation_policy::Column::Enabled.eq(false))
                    .count(&state.db)
                    .await?
                    > 0;
                let state_label = if disabled { "disabled" } else { "none" };
                (false, state_label.to_string(), org_air_gapped)
            }
        };

        rows.push(FleetRow {
            repository_id: repo.id,
            name: repo.full_name,
            open_pr_count,
            eligible,
            policy_state,
            air_gapped,
        });
    }

    Ok(Json(ApiResponse::success(rows)))
}

/// Map an autonomy level to the fleet `policy_state` label.
fn policy_state_label(level: AutonomyLevel) -> &'static str {
    match level {
        AutonomyLevel::DryRunOnly => "dry_run",
        AutonomyLevel::SuggestOnly => "suggest",
        AutonomyLevel::AutoWithApproval => "auto_with_approval",
        AutonomyLevel::FullyAutonomous => "auto_merge",
    }
}
