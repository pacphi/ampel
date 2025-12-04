use std::collections::HashMap;
use std::time::Duration;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use uuid::Uuid;

use ampel_core::models::{GitProvider, MergeRequest, MergeStrategy};
use ampel_db::entities::git_provider;
use ampel_db::queries::{
    MergeOperationItemQueries, MergeOperationQueries, PrQueries, RepoQueries, UserSettingsQueries,
};

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkMergeRequest {
    pub pull_request_ids: Vec<Uuid>,
    pub strategy: Option<String>,
    pub delete_branch: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkMergeResponse {
    pub operation_id: Uuid,
    pub status: String,
    pub total: i32,
    pub success: i32,
    pub failed: i32,
    pub skipped: i32,
    pub results: Vec<MergeItemResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeItemResult {
    pub pull_request_id: Uuid,
    pub repository_name: String,
    pub pr_number: i32,
    pub pr_title: String,
    pub status: String,
    pub error_message: Option<String>,
    pub merge_sha: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

/// Bulk merge multiple PRs
pub async fn bulk_merge(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<BulkMergeRequest>,
) -> Result<Json<ApiResponse<BulkMergeResponse>>, ApiError> {
    if req.pull_request_ids.is_empty() {
        return Err(ApiError::bad_request("No pull requests specified"));
    }

    if req.pull_request_ids.len() > 50 {
        return Err(ApiError::bad_request(
            "Cannot merge more than 50 PRs at once",
        ));
    }

    // Get user settings for defaults
    let settings = UserSettingsQueries::get_or_create_default(&state.db, auth.user_id).await?;

    let merge_strategy = req
        .strategy
        .as_ref()
        .and_then(|s| match s.as_str() {
            "merge" => Some(MergeStrategy::Merge),
            "squash" => Some(MergeStrategy::Squash),
            "rebase" => Some(MergeStrategy::Rebase),
            _ => None,
        })
        .unwrap_or(match settings.default_merge_strategy.as_str() {
            "merge" => MergeStrategy::Merge,
            "rebase" => MergeStrategy::Rebase,
            _ => MergeStrategy::Squash,
        });

    let delete_branch = req
        .delete_branch
        .unwrap_or(settings.delete_branches_default);
    let merge_delay = Duration::from_secs(settings.merge_delay_seconds as u64);

    // Validate PRs and group by repository
    let mut prs_by_repo: HashMap<
        Uuid,
        Vec<(
            ampel_db::entities::pull_request::Model,
            ampel_db::entities::repository::Model,
        )>,
    > = HashMap::new();

    for pr_id in &req.pull_request_ids {
        let pr = PrQueries::find_by_id(&state.db, *pr_id)
            .await?
            .ok_or_else(|| ApiError::not_found(format!("Pull request {} not found", pr_id)))?;

        let repo = RepoQueries::find_by_id(&state.db, pr.repository_id)
            .await?
            .ok_or_else(|| ApiError::internal("Repository not found"))?;

        // Verify ownership
        if repo.user_id != auth.user_id {
            return Err(ApiError::not_found(format!(
                "Pull request {} not found",
                pr_id
            )));
        }

        prs_by_repo.entry(repo.id).or_default().push((pr, repo));
    }

    // Create merge operation record
    let operation =
        MergeOperationQueries::create(&state.db, auth.user_id, req.pull_request_ids.len() as i32)
            .await?;

    // Create items for each PR
    for pr_id in &req.pull_request_ids {
        let pr = PrQueries::find_by_id(&state.db, *pr_id).await?.unwrap();
        MergeOperationItemQueries::create(&state.db, operation.id, *pr_id, pr.repository_id)
            .await?;
    }

    // Process merges
    let mut results: Vec<MergeItemResult> = Vec::new();
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;

    for (_repo_id, repo_prs) in prs_by_repo {
        let mut is_first = true;

        for (pr, repo) in repo_prs {
            // Add delay between merges in same repo (except first)
            if !is_first && merge_delay.as_secs() > 0 {
                sleep(merge_delay).await;
            }
            is_first = false;

            let item = ampel_db::entities::merge_operation_item::Entity::find()
                .filter(
                    ampel_db::entities::merge_operation_item::Column::MergeOperationId
                        .eq(operation.id),
                )
                .filter(ampel_db::entities::merge_operation_item::Column::PullRequestId.eq(pr.id))
                .one(&state.db)
                .await?
                .unwrap();

            // Check if PR is still open
            if pr.state != "open" {
                let result = MergeItemResult {
                    pull_request_id: pr.id,
                    repository_name: repo.full_name.clone(),
                    pr_number: pr.number,
                    pr_title: pr.title.clone(),
                    status: "skipped".to_string(),
                    error_message: Some("PR is not open".to_string()),
                    merge_sha: None,
                };
                results.push(result);
                skipped_count += 1;
                MergeOperationItemQueries::update_status(
                    &state.db,
                    item.id,
                    "skipped",
                    Some("PR is not open".to_string()),
                    None,
                )
                .await?;
                continue;
            }

            // Get provider and token
            let provider_type: GitProvider = repo
                .provider
                .parse()
                .map_err(|_| ApiError::internal("Invalid provider"))?;

            let connection = git_provider::Entity::find()
                .filter(git_provider::Column::UserId.eq(auth.user_id))
                .filter(git_provider::Column::Provider.eq(provider_type.to_string()))
                .one(&state.db)
                .await?;

            let connection = match connection {
                Some(c) => c,
                None => {
                    let result = MergeItemResult {
                        pull_request_id: pr.id,
                        repository_name: repo.full_name.clone(),
                        pr_number: pr.number,
                        pr_title: pr.title.clone(),
                        status: "failed".to_string(),
                        error_message: Some("Provider not connected".to_string()),
                        merge_sha: None,
                    };
                    results.push(result);
                    failed_count += 1;
                    MergeOperationItemQueries::update_status(
                        &state.db,
                        item.id,
                        "failed",
                        Some("Provider not connected".to_string()),
                        None,
                    )
                    .await?;
                    continue;
                }
            };

            // Decrypt token
            let access_token = match state
                .encryption_service
                .decrypt(&connection.access_token_encrypted)
            {
                Ok(t) => t,
                Err(e) => {
                    let result = MergeItemResult {
                        pull_request_id: pr.id,
                        repository_name: repo.full_name.clone(),
                        pr_number: pr.number,
                        pr_title: pr.title.clone(),
                        status: "failed".to_string(),
                        error_message: Some(format!("Token error: {}", e)),
                        merge_sha: None,
                    };
                    results.push(result);
                    failed_count += 1;
                    MergeOperationItemQueries::update_status(
                        &state.db,
                        item.id,
                        "failed",
                        Some(format!("Token error: {}", e)),
                        None,
                    )
                    .await?;
                    continue;
                }
            };

            // Attempt merge
            let provider = state.provider_factory.create(provider_type);
            let merge_request = MergeRequest {
                strategy: merge_strategy,
                commit_title: None,
                commit_message: None,
                delete_branch,
            };

            match provider
                .merge_pull_request(
                    &access_token,
                    &repo.owner,
                    &repo.name,
                    pr.number,
                    &merge_request,
                )
                .await
            {
                Ok(merge_result) => {
                    if merge_result.merged {
                        // Update PR state
                        PrQueries::update_state(
                            &state.db,
                            pr.id,
                            "merged".to_string(),
                            Some(chrono::Utc::now()),
                            Some(chrono::Utc::now()),
                        )
                        .await?;

                        let result = MergeItemResult {
                            pull_request_id: pr.id,
                            repository_name: repo.full_name.clone(),
                            pr_number: pr.number,
                            pr_title: pr.title.clone(),
                            status: "success".to_string(),
                            error_message: None,
                            merge_sha: merge_result.sha.clone(),
                        };
                        results.push(result);
                        success_count += 1;
                        MergeOperationItemQueries::update_status(
                            &state.db,
                            item.id,
                            "success",
                            None,
                            merge_result.sha,
                        )
                        .await?;
                    } else {
                        let result = MergeItemResult {
                            pull_request_id: pr.id,
                            repository_name: repo.full_name.clone(),
                            pr_number: pr.number,
                            pr_title: pr.title.clone(),
                            status: "failed".to_string(),
                            error_message: Some(merge_result.message),
                            merge_sha: None,
                        };
                        results.push(result);
                        failed_count += 1;
                        MergeOperationItemQueries::update_status(
                            &state.db,
                            item.id,
                            "failed",
                            Some("Merge not completed".to_string()),
                            None,
                        )
                        .await?;
                    }
                }
                Err(e) => {
                    let result = MergeItemResult {
                        pull_request_id: pr.id,
                        repository_name: repo.full_name.clone(),
                        pr_number: pr.number,
                        pr_title: pr.title.clone(),
                        status: "failed".to_string(),
                        error_message: Some(e.to_string()),
                        merge_sha: None,
                    };
                    results.push(result);
                    failed_count += 1;
                    MergeOperationItemQueries::update_status(
                        &state.db,
                        item.id,
                        "failed",
                        Some(e.to_string()),
                        None,
                    )
                    .await?;
                }
            }
        }
    }

    // Update operation status
    let final_status = if failed_count == 0 && skipped_count == 0 {
        "completed"
    } else if success_count == 0 {
        "failed"
    } else {
        "completed"
    };

    MergeOperationQueries::update_counts(
        &state.db,
        operation.id,
        success_count,
        failed_count,
        skipped_count,
        final_status,
    )
    .await?;

    // TODO: Send notifications via notification service

    Ok(Json(ApiResponse::success(BulkMergeResponse {
        operation_id: operation.id,
        status: final_status.to_string(),
        total: req.pull_request_ids.len() as i32,
        success: success_count,
        failed: failed_count,
        skipped: skipped_count,
        results,
    })))
}

/// Get a single merge operation
pub async fn get_operation(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(operation_id): Path<Uuid>,
) -> Result<Json<ApiResponse<BulkMergeResponse>>, ApiError> {
    let operation =
        MergeOperationQueries::find_by_id_and_user(&state.db, operation_id, auth.user_id)
            .await?
            .ok_or_else(|| ApiError::not_found("Merge operation not found"))?;

    let items = MergeOperationItemQueries::find_by_operation(&state.db, operation_id).await?;

    let mut results = Vec::with_capacity(items.len());
    for item in items {
        let pr = PrQueries::find_by_id(&state.db, item.pull_request_id).await?;
        let repo = RepoQueries::find_by_id(&state.db, item.repository_id).await?;

        let (pr_number, pr_title, repo_name) = match (pr, repo) {
            (Some(p), Some(r)) => (p.number, p.title, r.full_name),
            _ => (0, "Unknown".to_string(), "Unknown".to_string()),
        };

        results.push(MergeItemResult {
            pull_request_id: item.pull_request_id,
            repository_name: repo_name,
            pr_number,
            pr_title,
            status: item.status,
            error_message: item.error_message,
            merge_sha: item.merge_sha,
        });
    }

    Ok(Json(ApiResponse::success(BulkMergeResponse {
        operation_id: operation.id,
        status: operation.status,
        total: operation.total_count,
        success: operation.success_count,
        failed: operation.failed_count,
        skipped: operation.skipped_count,
        results,
    })))
}

/// List merge operations for user
pub async fn list_operations(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<BulkMergeResponse>>>, ApiError> {
    let limit = pagination.per_page.unwrap_or(20);
    let operations = MergeOperationQueries::find_by_user(&state.db, auth.user_id, limit).await?;

    let mut result = Vec::with_capacity(operations.len());

    for operation in operations {
        let items = MergeOperationItemQueries::find_by_operation(&state.db, operation.id).await?;

        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let pr = PrQueries::find_by_id(&state.db, item.pull_request_id).await?;
            let repo = RepoQueries::find_by_id(&state.db, item.repository_id).await?;

            let (pr_number, pr_title, repo_name) = match (pr, repo) {
                (Some(p), Some(r)) => (p.number, p.title, r.full_name),
                _ => (0, "Unknown".to_string(), "Unknown".to_string()),
            };

            results.push(MergeItemResult {
                pull_request_id: item.pull_request_id,
                repository_name: repo_name,
                pr_number,
                pr_title,
                status: item.status,
                error_message: item.error_message,
                merge_sha: item.merge_sha,
            });
        }

        result.push(BulkMergeResponse {
            operation_id: operation.id,
            status: operation.status,
            total: operation.total_count,
            success: operation.success_count,
            failed: operation.failed_count,
            skipped: operation.skipped_count,
            results,
        });
    }

    Ok(Json(ApiResponse::success(result)))
}
