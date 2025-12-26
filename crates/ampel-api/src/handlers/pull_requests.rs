//! Pull request handlers for listing, viewing, merging, and refreshing PRs.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::models::{AmpelStatus, GitProvider, MergeRequest, MergeStrategy};
use ampel_db::entities::provider_account;
use ampel_db::queries::{CICheckQueries, PrQueries, RepoQueries, ReviewQueries};
use sea_orm::EntityTrait;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

/// Query parameters for listing pull requests
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPrsQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub status: Option<String>,
}

/// Pull request response with computed status
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestResponse {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub provider: String,
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub author: String,
    pub author_avatar_url: Option<String>,
    pub is_draft: bool,
    pub is_mergeable: Option<bool>,
    pub has_conflicts: bool,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub commits_count: i32,
    pub comments_count: i32,
    pub ampel_status: AmpelStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Paginated PR list response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedPrsResponse {
    pub items: Vec<PullRequestResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

/// Merge request body
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergePrRequest {
    pub strategy: Option<String>,
    pub delete_branch: Option<bool>,
}

/// Merge response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeResponse {
    pub merged: bool,
    pub message: String,
    pub sha: Option<String>,
}

/// List all open pull requests for the authenticated user
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id))]
pub async fn list_pull_requests(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListPrsQuery>,
) -> Result<Json<ApiResponse<PaginatedPrsResponse>>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    // Get PRs with pagination
    let (prs, total) =
        PrQueries::find_open_by_user(&state.db, auth.user_id, page, per_page).await?;

    if prs.is_empty() {
        return Ok(Json(ApiResponse::success(PaginatedPrsResponse {
            items: Vec::new(),
            total: 0,
            page,
            per_page,
        })));
    }

    // Batch load CI checks and reviews for all PRs
    let pr_ids: Vec<_> = prs.iter().map(|pr| pr.id).collect();
    let all_ci_checks = CICheckQueries::find_for_pull_requests(&state.db, &pr_ids).await?;
    let all_reviews = ReviewQueries::find_for_pull_requests(&state.db, &pr_ids).await?;

    // Build lookup maps
    let mut ci_checks_by_pr: std::collections::HashMap<Uuid, Vec<_>> =
        std::collections::HashMap::new();
    for ci_check in all_ci_checks {
        ci_checks_by_pr
            .entry(ci_check.pull_request_id)
            .or_default()
            .push(ci_check);
    }

    let mut reviews_by_pr: std::collections::HashMap<Uuid, Vec<_>> =
        std::collections::HashMap::new();
    for review in all_reviews {
        reviews_by_pr
            .entry(review.pull_request_id)
            .or_default()
            .push(review);
    }

    // Convert to response format with computed status
    let mut items = Vec::with_capacity(prs.len());
    for pr_model in prs {
        let ci_checks = ci_checks_by_pr
            .get(&pr_model.id)
            .cloned()
            .unwrap_or_default();
        let reviews = reviews_by_pr.get(&pr_model.id).cloned().unwrap_or_default();

        // Convert to core models for status calculation
        let pr: ampel_core::models::PullRequest = pr_model.clone().into();
        let ci_checks: Vec<ampel_core::models::CICheck> =
            ci_checks.into_iter().map(|c| c.into()).collect();
        let reviews: Vec<ampel_core::models::Review> =
            reviews.into_iter().map(|r| r.into()).collect();

        let ampel_status = AmpelStatus::for_pull_request(&pr, &ci_checks, &reviews);

        // Filter by status if requested
        if let Some(ref status_filter) = params.status {
            let matches = match status_filter.to_lowercase().as_str() {
                "green" => ampel_status == AmpelStatus::Green,
                "yellow" => ampel_status == AmpelStatus::Yellow,
                "red" => ampel_status == AmpelStatus::Red,
                _ => true,
            };
            if !matches {
                continue;
            }
        }

        items.push(PullRequestResponse {
            id: pr_model.id,
            repository_id: pr_model.repository_id,
            provider: pr_model.provider,
            number: pr_model.number,
            title: pr_model.title,
            description: pr_model.description,
            url: pr_model.url,
            state: pr_model.state,
            source_branch: pr_model.source_branch,
            target_branch: pr_model.target_branch,
            author: pr_model.author,
            author_avatar_url: pr_model.author_avatar_url,
            is_draft: pr_model.is_draft,
            is_mergeable: pr_model.is_mergeable,
            has_conflicts: pr_model.has_conflicts,
            additions: pr_model.additions,
            deletions: pr_model.deletions,
            changed_files: pr_model.changed_files,
            commits_count: pr_model.commits_count,
            comments_count: pr_model.comments_count,
            ampel_status,
            created_at: pr_model.created_at,
            updated_at: pr_model.updated_at,
        });
    }

    Ok(Json(ApiResponse::success(PaginatedPrsResponse {
        items,
        total,
        page,
        per_page,
    })))
}

/// List pull requests for a specific repository
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id, repo_id = %repo_id))]
pub async fn list_repository_prs(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(repo_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<PullRequestResponse>>>, ApiError> {
    // Verify repository ownership
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    // Get open PRs for repository
    let prs = PrQueries::find_open_by_repository(&state.db, repo_id).await?;

    if prs.is_empty() {
        return Ok(Json(ApiResponse::success(Vec::new())));
    }

    // Batch load CI checks and reviews
    let pr_ids: Vec<_> = prs.iter().map(|pr| pr.id).collect();
    let all_ci_checks = CICheckQueries::find_for_pull_requests(&state.db, &pr_ids).await?;
    let all_reviews = ReviewQueries::find_for_pull_requests(&state.db, &pr_ids).await?;

    // Build lookup maps
    let mut ci_checks_by_pr: std::collections::HashMap<Uuid, Vec<_>> =
        std::collections::HashMap::new();
    for ci_check in all_ci_checks {
        ci_checks_by_pr
            .entry(ci_check.pull_request_id)
            .or_default()
            .push(ci_check);
    }

    let mut reviews_by_pr: std::collections::HashMap<Uuid, Vec<_>> =
        std::collections::HashMap::new();
    for review in all_reviews {
        reviews_by_pr
            .entry(review.pull_request_id)
            .or_default()
            .push(review);
    }

    // Convert to response format
    let mut items = Vec::with_capacity(prs.len());
    for pr_model in prs {
        let ci_checks = ci_checks_by_pr
            .get(&pr_model.id)
            .cloned()
            .unwrap_or_default();
        let reviews = reviews_by_pr.get(&pr_model.id).cloned().unwrap_or_default();

        let pr: ampel_core::models::PullRequest = pr_model.clone().into();
        let ci_checks: Vec<ampel_core::models::CICheck> =
            ci_checks.into_iter().map(|c| c.into()).collect();
        let reviews: Vec<ampel_core::models::Review> =
            reviews.into_iter().map(|r| r.into()).collect();

        let ampel_status = AmpelStatus::for_pull_request(&pr, &ci_checks, &reviews);

        items.push(PullRequestResponse {
            id: pr_model.id,
            repository_id: pr_model.repository_id,
            provider: pr_model.provider,
            number: pr_model.number,
            title: pr_model.title,
            description: pr_model.description,
            url: pr_model.url,
            state: pr_model.state,
            source_branch: pr_model.source_branch,
            target_branch: pr_model.target_branch,
            author: pr_model.author,
            author_avatar_url: pr_model.author_avatar_url,
            is_draft: pr_model.is_draft,
            is_mergeable: pr_model.is_mergeable,
            has_conflicts: pr_model.has_conflicts,
            additions: pr_model.additions,
            deletions: pr_model.deletions,
            changed_files: pr_model.changed_files,
            commits_count: pr_model.commits_count,
            comments_count: pr_model.comments_count,
            ampel_status,
            created_at: pr_model.created_at,
            updated_at: pr_model.updated_at,
        });
    }

    Ok(Json(ApiResponse::success(items)))
}

/// Get a single pull request
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id))]
pub async fn get_pull_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((repo_id, pr_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<PullRequestResponse>>, ApiError> {
    // Verify repository ownership
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    // Get PR
    let pr_model = PrQueries::find_by_id(&state.db, pr_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Pull request not found"))?;

    // Verify PR belongs to repository
    if pr_model.repository_id != repo_id {
        return Err(ApiError::not_found("Pull request not found"));
    }

    // Get CI checks and reviews
    let ci_checks = CICheckQueries::find_for_pull_requests(&state.db, &[pr_id]).await?;
    let reviews = ReviewQueries::find_for_pull_requests(&state.db, &[pr_id]).await?;

    let pr: ampel_core::models::PullRequest = pr_model.clone().into();
    let ci_checks: Vec<ampel_core::models::CICheck> =
        ci_checks.into_iter().map(|c| c.into()).collect();
    let reviews: Vec<ampel_core::models::Review> = reviews.into_iter().map(|r| r.into()).collect();

    let ampel_status = AmpelStatus::for_pull_request(&pr, &ci_checks, &reviews);

    Ok(Json(ApiResponse::success(PullRequestResponse {
        id: pr_model.id,
        repository_id: pr_model.repository_id,
        provider: pr_model.provider,
        number: pr_model.number,
        title: pr_model.title,
        description: pr_model.description,
        url: pr_model.url,
        state: pr_model.state,
        source_branch: pr_model.source_branch,
        target_branch: pr_model.target_branch,
        author: pr_model.author,
        author_avatar_url: pr_model.author_avatar_url,
        is_draft: pr_model.is_draft,
        is_mergeable: pr_model.is_mergeable,
        has_conflicts: pr_model.has_conflicts,
        additions: pr_model.additions,
        deletions: pr_model.deletions,
        changed_files: pr_model.changed_files,
        commits_count: pr_model.commits_count,
        comments_count: pr_model.comments_count,
        ampel_status,
        created_at: pr_model.created_at,
        updated_at: pr_model.updated_at,
    })))
}

/// Merge a pull request
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id))]
pub async fn merge_pull_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((repo_id, pr_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<MergePrRequest>,
) -> Result<Json<ApiResponse<MergeResponse>>, ApiError> {
    // Verify repository ownership
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    // Get PR
    let pr = PrQueries::find_by_id(&state.db, pr_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Pull request not found"))?;

    if pr.repository_id != repo_id {
        return Err(ApiError::not_found("Pull request not found"));
    }

    if pr.state != "open" {
        return Err(ApiError::bad_request("Pull request is not open"));
    }

    // Get provider account
    let account = match repo.provider_account_id {
        Some(account_id) => provider_account::Entity::find_by_id(account_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| ApiError::not_found("Provider account not found"))?,
        None => return Err(ApiError::bad_request("Repository not linked to account")),
    };

    // Decrypt token
    let access_token = state
        .encryption_service
        .decrypt(&account.access_token_encrypted)
        .map_err(|e| ApiError::internal(format!("Token error: {}", e)))?;

    let credentials = ampel_providers::traits::ProviderCredentials::Pat {
        token: access_token,
        username: account.auth_username.clone(),
    };

    let provider_type: GitProvider = repo
        .provider
        .parse()
        .map_err(|_| ApiError::internal("Invalid provider"))?;

    let provider = state
        .provider_factory
        .create(provider_type, account.instance_url.clone());

    // Parse merge strategy
    let strategy = req
        .strategy
        .as_ref()
        .and_then(|s| match s.as_str() {
            "merge" => Some(MergeStrategy::Merge),
            "squash" => Some(MergeStrategy::Squash),
            "rebase" => Some(MergeStrategy::Rebase),
            _ => None,
        })
        .unwrap_or(MergeStrategy::Squash);

    let merge_request = MergeRequest {
        strategy,
        commit_title: None,
        commit_message: None,
        delete_branch: req.delete_branch.unwrap_or(false),
    };

    match provider
        .merge_pull_request(
            &credentials,
            &repo.owner,
            &repo.name,
            pr.number,
            &merge_request,
        )
        .await
    {
        Ok(result) => {
            if result.merged {
                // Update PR state
                PrQueries::update_state(
                    &state.db,
                    pr.id,
                    "merged".to_string(),
                    Some(chrono::Utc::now()),
                    Some(chrono::Utc::now()),
                )
                .await?;
            }

            Ok(Json(ApiResponse::success(MergeResponse {
                merged: result.merged,
                message: result.message,
                sha: result.sha,
            })))
        }
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

/// Refresh pull request data from provider
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id))]
pub async fn refresh_pull_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((repo_id, pr_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<PullRequestResponse>>, ApiError> {
    // Verify repository ownership
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    // Get existing PR
    let pr = PrQueries::find_by_id(&state.db, pr_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Pull request not found"))?;

    if pr.repository_id != repo_id {
        return Err(ApiError::not_found("Pull request not found"));
    }

    // Get provider account
    let account = match repo.provider_account_id {
        Some(account_id) => provider_account::Entity::find_by_id(account_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| ApiError::not_found("Provider account not found"))?,
        None => return Err(ApiError::bad_request("Repository not linked to account")),
    };

    // Decrypt token
    let access_token = state
        .encryption_service
        .decrypt(&account.access_token_encrypted)
        .map_err(|e| ApiError::internal(format!("Token error: {}", e)))?;

    let credentials = ampel_providers::traits::ProviderCredentials::Pat {
        token: access_token,
        username: account.auth_username.clone(),
    };

    let provider_type: GitProvider = repo
        .provider
        .parse()
        .map_err(|_| ApiError::internal("Invalid provider"))?;

    let provider = state
        .provider_factory
        .create(provider_type, account.instance_url.clone());

    // Fetch fresh PR data from provider
    let fresh_pr = provider
        .get_pull_request(&credentials, &repo.owner, &repo.name, pr.number)
        .await
        .map_err(|e| ApiError::bad_request(format!("Failed to refresh: {}", e)))?;

    // Update PR in database
    let updated_pr = PrQueries::upsert(
        &state.db,
        repo_id,
        repo.provider.clone(),
        fresh_pr.provider_id.clone(),
        fresh_pr.number,
        fresh_pr.title.clone(),
        fresh_pr.description.clone(),
        fresh_pr.url.clone(),
        fresh_pr.state.clone(),
        fresh_pr.source_branch.clone(),
        fresh_pr.target_branch.clone(),
        fresh_pr.author.clone(),
        fresh_pr.author_avatar_url.clone(),
        fresh_pr.is_draft,
        fresh_pr.is_mergeable,
        fresh_pr.has_conflicts,
        fresh_pr.additions,
        fresh_pr.deletions,
        fresh_pr.changed_files,
        fresh_pr.commits_count,
        fresh_pr.comments_count,
        fresh_pr.created_at,
        fresh_pr.updated_at,
        fresh_pr.merged_at,
        fresh_pr.closed_at,
    )
    .await?;

    // Get CI checks and reviews for status calculation
    let ci_checks = CICheckQueries::find_for_pull_requests(&state.db, &[updated_pr.id]).await?;
    let reviews = ReviewQueries::find_for_pull_requests(&state.db, &[updated_pr.id]).await?;

    let pr_core: ampel_core::models::PullRequest = updated_pr.clone().into();
    let ci_checks: Vec<ampel_core::models::CICheck> =
        ci_checks.into_iter().map(|c| c.into()).collect();
    let reviews: Vec<ampel_core::models::Review> = reviews.into_iter().map(|r| r.into()).collect();

    let ampel_status = AmpelStatus::for_pull_request(&pr_core, &ci_checks, &reviews);

    Ok(Json(ApiResponse::success(PullRequestResponse {
        id: updated_pr.id,
        repository_id: updated_pr.repository_id,
        provider: updated_pr.provider,
        number: updated_pr.number,
        title: updated_pr.title,
        description: updated_pr.description,
        url: updated_pr.url,
        state: updated_pr.state,
        source_branch: updated_pr.source_branch,
        target_branch: updated_pr.target_branch,
        author: updated_pr.author,
        author_avatar_url: updated_pr.author_avatar_url,
        is_draft: updated_pr.is_draft,
        is_mergeable: updated_pr.is_mergeable,
        has_conflicts: updated_pr.has_conflicts,
        additions: updated_pr.additions,
        deletions: updated_pr.deletions,
        changed_files: updated_pr.changed_files,
        commits_count: updated_pr.commits_count,
        comments_count: updated_pr.comments_count,
        ampel_status,
        created_at: updated_pr.created_at,
        updated_at: updated_pr.updated_at,
    })))
}
