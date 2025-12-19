use axum::{
    extract::{Path, Query, State},
    Json,
};
use sea_orm::EntityTrait;
use uuid::Uuid;

use ampel_core::models::{
    AmpelStatus, CICheck, GitProvider, MergeRequest, PaginatedResponse, PullRequest,
    PullRequestFilter, PullRequestWithDetails, Review,
};
use ampel_db::entities::provider_account;
use ampel_db::queries::{CICheckQueries, PrQueries, RepoQueries, ReviewQueries};

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

/// List all open PRs for the user
pub async fn list_pull_requests(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(filter): Query<PullRequestFilter>,
) -> Result<Json<ApiResponse<PaginatedResponse<PullRequestWithDetails>>>, ApiError> {
    let page = filter.page.unwrap_or(1);
    let per_page = filter.per_page.unwrap_or(20);

    let (prs, total) =
        PrQueries::find_open_by_user(&state.db, auth.user_id, page as u64, per_page as u64).await?;

    let mut result = Vec::with_capacity(prs.len());

    for pr in prs {
        let repo = RepoQueries::find_by_id(&state.db, pr.repository_id)
            .await?
            .ok_or_else(|| ApiError::internal("Repository not found for PR"))?;

        let checks: Vec<CICheck> = CICheckQueries::find_by_pull_request(&state.db, pr.id)
            .await?
            .into_iter()
            .map(|c| c.into())
            .collect();

        let reviews: Vec<Review> = ReviewQueries::find_latest_by_pull_request(&state.db, pr.id)
            .await?
            .into_iter()
            .map(|r| r.into())
            .collect();

        let pr_model: PullRequest = pr.into();
        let status = AmpelStatus::for_pull_request(&pr_model, &checks, &reviews);

        result.push(PullRequestWithDetails {
            pull_request: pr_model,
            status,
            ci_checks: checks,
            reviews,
            repository_name: repo.name,
            repository_owner: repo.owner,
        });
    }

    Ok(Json(ApiResponse::success(PaginatedResponse::new(
        result,
        total as i64,
        page,
        per_page,
    ))))
}

/// Get PRs for a specific repository
pub async fn list_repository_prs(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(repo_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<PullRequestWithDetails>>>, ApiError> {
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    // Verify ownership
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    let prs = PrQueries::find_open_by_repository(&state.db, repo_id).await?;

    let mut result = Vec::with_capacity(prs.len());

    for pr in prs {
        let checks: Vec<CICheck> = CICheckQueries::find_by_pull_request(&state.db, pr.id)
            .await?
            .into_iter()
            .map(|c| c.into())
            .collect();

        let reviews: Vec<Review> = ReviewQueries::find_latest_by_pull_request(&state.db, pr.id)
            .await?
            .into_iter()
            .map(|r| r.into())
            .collect();

        let pr_model: PullRequest = pr.into();
        let status = AmpelStatus::for_pull_request(&pr_model, &checks, &reviews);

        result.push(PullRequestWithDetails {
            pull_request: pr_model,
            status,
            ci_checks: checks,
            reviews,
            repository_name: repo.name.clone(),
            repository_owner: repo.owner.clone(),
        });
    }

    Ok(Json(ApiResponse::success(result)))
}

/// Get a single PR with full details
pub async fn get_pull_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((repo_id, pr_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<PullRequestWithDetails>>, ApiError> {
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    // Verify ownership
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    let pr = PrQueries::find_by_id(&state.db, pr_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Pull request not found"))?;

    // Verify PR belongs to repo
    if pr.repository_id != repo_id {
        return Err(ApiError::not_found("Pull request not found"));
    }

    let checks: Vec<CICheck> = CICheckQueries::find_by_pull_request(&state.db, pr.id)
        .await?
        .into_iter()
        .map(|c| c.into())
        .collect();

    let reviews: Vec<Review> = ReviewQueries::find_latest_by_pull_request(&state.db, pr.id)
        .await?
        .into_iter()
        .map(|r| r.into())
        .collect();

    let pr_model: PullRequest = pr.into();
    let status = AmpelStatus::for_pull_request(&pr_model, &checks, &reviews);

    Ok(Json(ApiResponse::success(PullRequestWithDetails {
        pull_request: pr_model,
        status,
        ci_checks: checks,
        reviews,
        repository_name: repo.name,
        repository_owner: repo.owner,
    })))
}

/// Merge a pull request
pub async fn merge_pull_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((repo_id, pr_id)): Path<(Uuid, Uuid)>,
    Json(merge_req): Json<MergeRequest>,
) -> Result<Json<ApiResponse<MergeResultResponse>>, ApiError> {
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    // Verify ownership
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    let pr = PrQueries::find_by_id(&state.db, pr_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Pull request not found"))?;

    // Verify PR belongs to repo
    if pr.repository_id != repo_id {
        return Err(ApiError::not_found("Pull request not found"));
    }

    let provider_type: GitProvider = repo
        .provider
        .parse()
        .map_err(|_| ApiError::internal("Invalid provider"))?;

    // Get provider account
    let account = provider_account::Entity::find_by_id(
        repo.provider_account_id
            .ok_or(ApiError::bad_request("Repository not linked to account"))?,
    )
    .one(&state.db)
    .await?
    .ok_or(ApiError::not_found("Provider account not found"))?;

    // Decrypt access token
    let access_token = state
        .encryption_service
        .decrypt(&account.access_token_encrypted)
        .map_err(|e| ApiError::internal(format!("Failed to decrypt token: {}", e)))?;

    // Create credentials
    let credentials = ampel_providers::traits::ProviderCredentials::Pat {
        token: access_token,
        username: account.auth_username.clone(),
    };

    let provider = state
        .provider_factory
        .create(provider_type, account.instance_url.clone());

    let result = provider
        .merge_pull_request(&credentials, &repo.owner, &repo.name, pr.number, &merge_req)
        .await
        .map_err(|e| ApiError::bad_request(format!("Merge failed: {}", e)))?;

    // Update PR state in database
    if result.merged {
        PrQueries::update_state(
            &state.db,
            pr_id,
            "merged".to_string(),
            Some(chrono::Utc::now()),
            Some(chrono::Utc::now()),
        )
        .await?;
    }

    Ok(Json(ApiResponse::success(MergeResultResponse {
        merged: result.merged,
        sha: result.sha,
        message: result.message,
    })))
}

#[derive(Debug, serde::Serialize)]
pub struct MergeResultResponse {
    pub merged: bool,
    pub sha: Option<String>,
    pub message: String,
}

/// Refresh PR data from provider
pub async fn refresh_pull_request(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((repo_id, pr_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<PullRequestWithDetails>>, ApiError> {
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    // Verify ownership
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    let pr = PrQueries::find_by_id(&state.db, pr_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Pull request not found"))?;

    // Verify PR belongs to repo
    if pr.repository_id != repo_id {
        return Err(ApiError::not_found("Pull request not found"));
    }

    let provider_type: GitProvider = repo
        .provider
        .parse()
        .map_err(|_| ApiError::internal("Invalid provider"))?;

    // Get provider account
    let account = provider_account::Entity::find_by_id(
        repo.provider_account_id
            .ok_or(ApiError::bad_request("Repository not linked to account"))?,
    )
    .one(&state.db)
    .await?
    .ok_or(ApiError::not_found("Provider account not found"))?;

    // Decrypt access token
    let access_token = state
        .encryption_service
        .decrypt(&account.access_token_encrypted)
        .map_err(|e| ApiError::internal(format!("Failed to decrypt token: {}", e)))?;

    // Create credentials
    let credentials = ampel_providers::traits::ProviderCredentials::Pat {
        token: access_token.clone(),
        username: account.auth_username.clone(),
    };

    let provider = state
        .provider_factory
        .create(provider_type, account.instance_url.clone());

    // Fetch fresh PR data
    let fresh_pr = provider
        .get_pull_request(&credentials, &repo.owner, &repo.name, pr.number)
        .await
        .map_err(|e| ApiError::internal(format!("Provider error: {}", e)))?;

    // Update PR in database
    let state_str = fresh_pr.state.clone();
    let updated_pr = PrQueries::upsert(
        &state.db,
        repo_id,
        provider_type.to_string(),
        fresh_pr.provider_id,
        fresh_pr.number,
        fresh_pr.title,
        fresh_pr.description,
        fresh_pr.url,
        state_str,
        fresh_pr.source_branch,
        fresh_pr.target_branch,
        fresh_pr.author,
        fresh_pr.author_avatar_url,
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

    // Fetch and update CI checks
    CICheckQueries::delete_by_pull_request(&state.db, pr_id).await?;
    let fresh_checks = provider
        .get_ci_checks(&credentials, &repo.owner, &repo.name, pr.number)
        .await
        .unwrap_or_default();

    for check in &fresh_checks {
        CICheckQueries::upsert(
            &state.db,
            pr_id,
            check.name.clone(),
            check.status.clone(),
            check.conclusion.clone(),
            check.url.clone(),
            check.started_at,
            check.completed_at,
            check
                .completed_at
                .and_then(|c| check.started_at.map(|s| (c - s).num_seconds() as i32)),
        )
        .await?;
    }

    // Fetch and update reviews
    ReviewQueries::delete_by_pull_request(&state.db, pr_id).await?;
    let fresh_reviews = provider
        .get_reviews(&credentials, &repo.owner, &repo.name, pr.number)
        .await
        .unwrap_or_default();

    for review in &fresh_reviews {
        ReviewQueries::upsert(
            &state.db,
            Uuid::new_v4(), // Generate new ID for review
            pr_id,
            review.reviewer.clone(),
            review.reviewer_avatar_url.clone(),
            review.state.clone(),
            review.body.clone(),
            review.submitted_at,
        )
        .await?;
    }

    // Return updated PR with details
    let checks: Vec<CICheck> = CICheckQueries::find_by_pull_request(&state.db, pr_id)
        .await?
        .into_iter()
        .map(|c| c.into())
        .collect();

    let reviews: Vec<Review> = ReviewQueries::find_latest_by_pull_request(&state.db, pr_id)
        .await?
        .into_iter()
        .map(|r| r.into())
        .collect();

    let pr_model: PullRequest = updated_pr.into();
    let status = AmpelStatus::for_pull_request(&pr_model, &checks, &reviews);

    Ok(Json(ApiResponse::success(PullRequestWithDetails {
        pull_request: pr_model,
        status,
        ci_checks: checks,
        reviews,
        repository_name: repo.name,
        repository_owner: repo.owner,
    })))
}
