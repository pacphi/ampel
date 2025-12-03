use axum::{extract::State, Json};
use serde::Serialize;

use ampel_core::models::{AmpelStatus, RepositoryWithStatus};
use ampel_db::queries::{PrQueries, RepoQueries};

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    pub total_repositories: i32,
    pub total_open_prs: i32,
    pub status_counts: StatusCounts,
    pub provider_counts: ProviderCounts,
}

#[derive(Debug, Serialize)]
pub struct StatusCounts {
    pub green: i32,
    pub yellow: i32,
    pub red: i32,
}

#[derive(Debug, Serialize)]
pub struct ProviderCounts {
    pub github: i32,
    pub gitlab: i32,
    pub bitbucket: i32,
}

/// Get dashboard summary statistics
pub async fn get_summary(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;

    let mut total_open_prs = 0;
    let mut green_count = 0;
    let mut yellow_count = 0;
    let red_count = 0;
    let mut github_count = 0;
    let mut gitlab_count = 0;
    let mut bitbucket_count = 0;

    for repo in &repos {
        let open_prs = PrQueries::count_open_by_repository(&state.db, repo.id).await? as i32;
        total_open_prs += open_prs;

        // Count by provider
        match repo.provider.as_str() {
            "github" => github_count += 1,
            "gitlab" => gitlab_count += 1,
            "bitbucket" => bitbucket_count += 1,
            _ => {}
        }

        // Simplified status counting (would need full PR data in production)
        if open_prs > 0 {
            yellow_count += 1;
        } else {
            green_count += 1;
        }
    }

    Ok(Json(ApiResponse::success(DashboardSummary {
        total_repositories: repos.len() as i32,
        total_open_prs,
        status_counts: StatusCounts {
            green: green_count,
            yellow: yellow_count,
            red: red_count,
        },
        provider_counts: ProviderCounts {
            github: github_count,
            gitlab: gitlab_count,
            bitbucket: bitbucket_count,
        },
    })))
}

/// Get repositories for grid/list view
pub async fn get_grid(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<RepositoryWithStatus>>>, ApiError> {
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;

    let mut result = Vec::with_capacity(repos.len());

    for repo in repos {
        let open_prs = PrQueries::count_open_by_repository(&state.db, repo.id).await? as i32;

        // Simplified status - would need to compute based on actual PR statuses
        let status = if open_prs == 0 {
            AmpelStatus::None
        } else {
            AmpelStatus::Yellow
        };

        result.push(RepositoryWithStatus {
            repository: repo.into(),
            status,
            open_pr_count: open_prs,
        });
    }

    Ok(Json(ApiResponse::success(result)))
}
