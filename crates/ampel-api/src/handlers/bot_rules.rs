use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_db::entities::auto_merge_rule;
use ampel_db::queries::RepoQueries;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct AutoMergeRuleResponse {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub enabled: bool,
    pub bot_authors: Vec<String>,
    pub require_all_checks: bool,
    pub require_approval: bool,
    pub merge_strategy: String,
    pub delete_branch: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateAutoMergeRuleRequest {
    pub enabled: Option<bool>,
    pub bot_authors: Option<Vec<String>>,
    pub require_all_checks: Option<bool>,
    pub require_approval: Option<bool>,
    pub merge_strategy: Option<String>,
    pub delete_branch: Option<bool>,
}

/// Get auto-merge rule for a repository
pub async fn get_auto_merge_rule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(repo_id): Path<Uuid>,
) -> Result<Json<ApiResponse<AutoMergeRuleResponse>>, ApiError> {
    // Verify ownership
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    let rule = auto_merge_rule::Entity::find()
        .filter(auto_merge_rule::Column::RepositoryId.eq(repo_id))
        .one(&state.db)
        .await?;

    let response = if let Some(r) = rule {
        let bot_authors: Vec<String> = serde_json::from_str(&r.bot_authors).unwrap_or_default();
        AutoMergeRuleResponse {
            id: r.id,
            repository_id: r.repository_id,
            enabled: r.enabled,
            bot_authors,
            require_all_checks: r.require_all_checks,
            require_approval: r.require_approval,
            merge_strategy: r.merge_strategy,
            delete_branch: r.delete_branch,
        }
    } else {
        AutoMergeRuleResponse {
            id: Uuid::nil(),
            repository_id: repo_id,
            enabled: false,
            bot_authors: vec!["dependabot[bot]".to_string(), "renovate[bot]".to_string()],
            require_all_checks: true,
            require_approval: false,
            merge_strategy: "squash".to_string(),
            delete_branch: true,
        }
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Create or update auto-merge rule
pub async fn upsert_auto_merge_rule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(repo_id): Path<Uuid>,
    Json(req): Json<CreateAutoMergeRuleRequest>,
) -> Result<Json<ApiResponse<AutoMergeRuleResponse>>, ApiError> {
    // Verify ownership
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    let now = Utc::now();
    let existing = auto_merge_rule::Entity::find()
        .filter(auto_merge_rule::Column::RepositoryId.eq(repo_id))
        .one(&state.db)
        .await?;

    let default_bots = vec!["dependabot[bot]".to_string(), "renovate[bot]".to_string()];

    let updated = if let Some(existing) = existing {
        let mut active: auto_merge_rule::ActiveModel = existing.into();

        if let Some(v) = req.enabled {
            active.enabled = Set(v);
        }
        if let Some(v) = &req.bot_authors {
            active.bot_authors = Set(serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()));
        }
        if let Some(v) = req.require_all_checks {
            active.require_all_checks = Set(v);
        }
        if let Some(v) = req.require_approval {
            active.require_approval = Set(v);
        }
        if let Some(v) = &req.merge_strategy {
            active.merge_strategy = Set(v.clone());
        }
        if let Some(v) = req.delete_branch {
            active.delete_branch = Set(v);
        }
        active.updated_at = Set(now);

        active.update(&state.db).await?
    } else {
        let bot_authors = req.bot_authors.unwrap_or(default_bots.clone());
        let new_rule = auto_merge_rule::ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(repo_id),
            enabled: Set(req.enabled.unwrap_or(false)),
            bot_authors: Set(
                serde_json::to_string(&bot_authors).unwrap_or_else(|_| "[]".to_string())
            ),
            require_all_checks: Set(req.require_all_checks.unwrap_or(true)),
            require_approval: Set(req.require_approval.unwrap_or(false)),
            merge_strategy: Set(req.merge_strategy.unwrap_or_else(|| "squash".to_string())),
            delete_branch: Set(req.delete_branch.unwrap_or(true)),
            created_at: Set(now),
            updated_at: Set(now),
        };
        new_rule.insert(&state.db).await?
    };

    let bot_authors: Vec<String> = serde_json::from_str(&updated.bot_authors).unwrap_or_default();

    Ok(Json(ApiResponse::success(AutoMergeRuleResponse {
        id: updated.id,
        repository_id: updated.repository_id,
        enabled: updated.enabled,
        bot_authors,
        require_all_checks: updated.require_all_checks,
        require_approval: updated.require_approval,
        merge_strategy: updated.merge_strategy,
        delete_branch: updated.delete_branch,
    })))
}

/// Delete auto-merge rule
pub async fn delete_auto_merge_rule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(repo_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // Verify ownership
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    auto_merge_rule::Entity::delete_many()
        .filter(auto_merge_rule::Column::RepositoryId.eq(repo_id))
        .exec(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
