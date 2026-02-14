use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use rust_i18n::t;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use ampel_core::models::{
    AddRepositoryRequest, AmpelStatus, DiscoveredRepository, GitProvider, Repository,
    RepositoryWithStatus,
};
use ampel_db::entities::provider_account;
use ampel_db::queries::{PrQueries, RepoQueries};
use ampel_worker::jobs::poll_repository::PollRepositoryJob;

use crate::extractors::{AuthUser, ValidatedJson};
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

// In-memory refresh status tracking
lazy_static::lazy_static! {
    static ref REFRESH_JOBS: Arc<RwLock<std::collections::HashMap<Uuid, RefreshJobStatus>>> =
        Arc::new(RwLock::new(std::collections::HashMap::new()));
}

#[derive(Debug, Deserialize)]
pub struct ListReposQuery {
    pub provider: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DiscoverReposQuery {
    pub provider: String,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// List user's watched repositories
pub async fn list_repositories(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(_query): Query<ListReposQuery>,
) -> Result<Json<ApiResponse<Vec<RepositoryWithStatus>>>, ApiError> {
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;

    let mut result = Vec::with_capacity(repos.len());

    for repo in repos {
        // Get PR statuses for this repo
        let prs = PrQueries::find_open_by_repository(&state.db, repo.id).await?;

        let pr_statuses: Vec<AmpelStatus> = prs
            .iter()
            .map(|_| AmpelStatus::Yellow) // Simplified - would need to fetch checks/reviews
            .collect();

        let status = AmpelStatus::for_repository(&pr_statuses);
        let repo_model: Repository = repo.into();

        result.push(RepositoryWithStatus {
            repository: repo_model,
            status,
            open_pr_count: pr_statuses.len() as i32,
        });
    }

    Ok(Json(ApiResponse::success(result)))
}

/// Get a single repository
pub async fn get_repository(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<RepositoryWithStatus>>, ApiError> {
    let repo = RepoQueries::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::not_found(t!("errors.repository.not_found")))?;

    // Verify ownership
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found(t!("errors.repository.not_found")));
    }

    let prs = PrQueries::find_open_by_repository(&state.db, repo.id).await?;
    let pr_statuses: Vec<AmpelStatus> = prs.iter().map(|_| AmpelStatus::Yellow).collect();
    let status = AmpelStatus::for_repository(&pr_statuses);
    let repo_model: Repository = repo.into();

    Ok(Json(ApiResponse::success(RepositoryWithStatus {
        repository: repo_model,
        status,
        open_pr_count: pr_statuses.len() as i32,
    })))
}

/// Discover available repositories from a provider
pub async fn discover_repositories(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<DiscoverReposQuery>,
) -> Result<Json<ApiResponse<Vec<DiscoveredRepository>>>, ApiError> {
    let provider_type: GitProvider = query
        .provider
        .parse()
        .map_err(|_| ApiError::bad_request(t!("errors.repository.invalid_provider")))?;

    // Get provider account
    let account = provider_account::Entity::find()
        .filter(provider_account::Column::UserId.eq(auth.user_id))
        .filter(provider_account::Column::Provider.eq(provider_type.to_string()))
        .filter(provider_account::Column::IsDefault.eq(true))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::bad_request(t!("errors.repository.provider_not_connected")))?;

    // Decrypt access token
    let access_token = state
        .encryption_service
        .decrypt(&account.access_token_encrypted)
        .map_err(|e| {
            ApiError::internal(t!("errors.provider.decrypt_failed", error = e.to_string()))
        })?;

    // Create credentials
    let credentials = ampel_providers::traits::ProviderCredentials::Pat {
        token: access_token,
        username: account.auth_username.clone(),
    };

    let provider = state
        .provider_factory
        .create(provider_type, account.instance_url.clone());

    // If no specific page requested, fetch all repositories by paginating through all pages
    let repos = if query.page.is_none() {
        let per_page = 100; // Use max allowed by GitHub
        let mut all_repos = Vec::new();
        let mut page = 1;

        loop {
            let page_repos = provider
                .list_repositories(&credentials, page, per_page)
                .await
                .map_err(|e| {
                    ApiError::internal(t!("errors.provider.error", error = e.to_string()))
                })?;

            let count = page_repos.len();
            all_repos.extend(page_repos);

            // If we got fewer than per_page, we've reached the last page
            if count < per_page as usize {
                break;
            }
            page += 1;

            // Safety limit to prevent infinite loops (1000 repos max)
            if page > 10 {
                break;
            }
        }
        all_repos
    } else {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(30);

        provider
            .list_repositories(&credentials, page, per_page)
            .await
            .map_err(|e| ApiError::internal(t!("errors.provider.error", error = e.to_string())))?
    };

    Ok(Json(ApiResponse::success(repos)))
}

/// Add a repository to watch list
pub async fn add_repository(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(req): ValidatedJson<AddRepositoryRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Repository>>), ApiError> {
    // Get provider account
    let account = provider_account::Entity::find()
        .filter(provider_account::Column::UserId.eq(auth.user_id))
        .filter(provider_account::Column::Provider.eq(req.provider.to_string()))
        .filter(provider_account::Column::IsDefault.eq(true))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::bad_request(t!("errors.repository.provider_not_connected")))?;

    // Decrypt access token
    let access_token = state
        .encryption_service
        .decrypt(&account.access_token_encrypted)
        .map_err(|e| {
            ApiError::internal(t!("errors.provider.decrypt_failed", error = e.to_string()))
        })?;

    // Create credentials
    let credentials = ampel_providers::traits::ProviderCredentials::Pat {
        token: access_token,
        username: account.auth_username.clone(),
    };

    let provider = state
        .provider_factory
        .create(req.provider, account.instance_url.clone());

    // Fetch repository details from provider
    let discovered = provider
        .get_repository(&credentials, &req.owner, &req.name)
        .await
        .map_err(|e| match e {
            ampel_providers::ProviderError::NotFound(_) => {
                ApiError::not_found(t!("errors.repository.not_found"))
            }
            _ => ApiError::internal(t!("errors.provider.error", error = e.to_string())),
        })?;

    // Check if already added
    if RepoQueries::find_by_provider_id(
        &state.db,
        auth.user_id,
        &req.provider.to_string(),
        &discovered.provider_id,
    )
    .await?
    .is_some()
    {
        return Err(ApiError::bad_request(t!("errors.repository.already_added")));
    }

    // Create repository
    let repo = RepoQueries::create(
        &state.db,
        auth.user_id,
        req.provider.to_string(),
        discovered.provider_id,
        discovered.owner,
        discovered.name,
        discovered.full_name,
        discovered.description,
        discovered.url,
        discovered.default_branch,
        discovered.is_private,
        discovered.is_archived,
        req.poll_interval_seconds.unwrap_or(300),
        Some(account.id),
    )
    .await?;

    let repo_model: Repository = repo.into();
    Ok((StatusCode::CREATED, Json(ApiResponse::success(repo_model))))
}

/// Remove a repository from watch list
pub async fn remove_repository(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let repo = RepoQueries::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::not_found(t!("errors.repository.not_found")))?;

    // Verify ownership
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found(t!("errors.repository.not_found")));
    }

    RepoQueries::delete(&state.db, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Update repository settings
pub async fn update_repository(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateRepositoryRequest>,
) -> Result<Json<ApiResponse<Repository>>, ApiError> {
    let repo = RepoQueries::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::not_found(t!("errors.repository.not_found")))?;

    // Verify ownership
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found(t!("errors.repository.not_found")));
    }

    if let Some(poll_interval) = body.poll_interval_seconds {
        RepoQueries::update_poll_interval(&state.db, id, poll_interval).await?;
    }

    let updated = RepoQueries::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::not_found(t!("errors.repository.not_found")))?;

    let repo_model: Repository = updated.into();
    Ok(Json(ApiResponse::success(repo_model)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateRepositoryRequest {
    pub poll_interval_seconds: Option<i32>,
}

/// Refresh job status tracking
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshJobStatus {
    pub job_id: Uuid,
    pub total_repositories: usize,
    pub completed: usize,
    pub current_repository: Option<String>,
    pub is_complete: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshJobResponse {
    pub job_id: Uuid,
}

/// Trigger refresh of all user repositories
pub async fn refresh_all_repositories(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<RefreshJobResponse>>, ApiError> {
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;
    let job_id = Uuid::new_v4();
    let total_repositories = repos.len();

    // Initialize job status
    let status = RefreshJobStatus {
        job_id,
        total_repositories,
        completed: 0,
        current_repository: None,
        is_complete: false,
        started_at: chrono::Utc::now(),
        completed_at: None,
    };

    REFRESH_JOBS.write().await.insert(job_id, status);

    // Spawn background task to refresh all repositories
    let db = state.db.clone();
    let encryption_service = state.encryption_service.clone();
    let provider_factory = state.provider_factory.clone();

    tokio::spawn(async move {
        let poll_job = PollRepositoryJob;

        for (idx, repo) in repos.iter().enumerate() {
            // Update status with current repository
            {
                let mut jobs = REFRESH_JOBS.write().await;
                if let Some(status) = jobs.get_mut(&job_id) {
                    status.current_repository = Some(format!("{}/{}", repo.owner, repo.name));
                }
            }

            // Poll the repository
            if let Err(e) = poll_job
                .poll_single_repo(&db, &encryption_service, &provider_factory, repo)
                .await
            {
                tracing::error!(
                    "Failed to refresh repository {}/{}: {}",
                    repo.owner,
                    repo.name,
                    e
                );
            }

            // Update progress
            {
                let mut jobs = REFRESH_JOBS.write().await;
                if let Some(status) = jobs.get_mut(&job_id) {
                    status.completed = idx + 1;
                }
            }
        }

        // Mark job as complete
        {
            let mut jobs = REFRESH_JOBS.write().await;
            if let Some(status) = jobs.get_mut(&job_id) {
                status.is_complete = true;
                status.completed_at = Some(chrono::Utc::now());
                status.current_repository = None;
            }
        }

        tracing::info!(
            "Completed refresh job {} for {} repositories",
            job_id,
            total_repositories
        );
    });

    Ok(Json(ApiResponse::success(RefreshJobResponse { job_id })))
}

/// Get refresh job status
pub async fn get_refresh_status(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Path(job_id): Path<Uuid>,
) -> Result<Json<ApiResponse<RefreshJobStatus>>, ApiError> {
    let jobs = REFRESH_JOBS.read().await;
    let status = jobs
        .get(&job_id)
        .ok_or_else(|| ApiError::not_found(t!("errors.refresh.job_not_found")))?;

    Ok(Json(ApiResponse::success(status.clone())))
}
