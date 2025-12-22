use axum::{extract::State, Json};
use serde::Serialize;

use ampel_core::models::{AmpelStatus, RepositoryWithStatus};
use ampel_db::queries::{CICheckQueries, PrQueries, RepoQueries, ReviewQueries};

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub total_repositories: i32,
    pub total_open_prs: i32,
    pub status_counts: StatusCounts,
    pub provider_counts: ProviderCounts,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCounts {
    pub green: i32,
    pub yellow: i32,
    pub red: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
    let mut red_count = 0;
    let mut github_count = 0;
    let mut gitlab_count = 0;
    let mut bitbucket_count = 0;

    for repo in &repos {
        // Get all open PRs for this repository
        let open_prs = PrQueries::find_open_by_repository(&state.db, repo.id).await?;
        total_open_prs += open_prs.len() as i32;

        // Count by provider
        match repo.provider.as_str() {
            "github" => github_count += 1,
            "gitlab" => gitlab_count += 1,
            "bitbucket" => bitbucket_count += 1,
            _ => {}
        }

        // Calculate actual PR statuses based on CI checks and reviews
        for pr_model in &open_prs {
            // Load CI checks and reviews for this PR
            let ci_checks = CICheckQueries::find_by_pull_request(&state.db, pr_model.id).await?;
            let reviews = ReviewQueries::find_by_pull_request(&state.db, pr_model.id).await?;

            // Convert database models to core models
            let pr: ampel_core::models::PullRequest = pr_model.clone().into();
            let ci_checks: Vec<ampel_core::models::CICheck> =
                ci_checks.into_iter().map(|c| c.into()).collect();
            let reviews: Vec<ampel_core::models::Review> =
                reviews.into_iter().map(|r| r.into()).collect();

            // Calculate status using the actual logic
            let status = AmpelStatus::for_pull_request(&pr, &ci_checks, &reviews);

            // Count by status
            match status {
                AmpelStatus::Green => green_count += 1,
                AmpelStatus::Yellow => yellow_count += 1,
                AmpelStatus::Red => red_count += 1,
                AmpelStatus::None => {} // Should not happen for open PRs
            }
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
        let open_prs = PrQueries::find_open_by_repository(&state.db, repo.id).await?;
        let open_pr_count = open_prs.len() as i32;

        // Calculate repository status based on actual PR statuses
        let mut pr_statuses = Vec::new();

        for pr_model in &open_prs {
            // Load CI checks and reviews for this PR
            let ci_checks = CICheckQueries::find_by_pull_request(&state.db, pr_model.id).await?;
            let reviews = ReviewQueries::find_by_pull_request(&state.db, pr_model.id).await?;

            // Convert database models to core models
            let pr: ampel_core::models::PullRequest = pr_model.clone().into();
            let ci_checks: Vec<ampel_core::models::CICheck> =
                ci_checks.into_iter().map(|c| c.into()).collect();
            let reviews: Vec<ampel_core::models::Review> =
                reviews.into_iter().map(|r| r.into()).collect();

            // Calculate status for this PR
            let pr_status = AmpelStatus::for_pull_request(&pr, &ci_checks, &reviews);
            pr_statuses.push(pr_status);
        }

        // Aggregate status for the repository (worst status wins)
        let status = AmpelStatus::for_repository(&pr_statuses);

        result.push(RepositoryWithStatus {
            repository: repo.into(),
            status,
            open_pr_count,
        });
    }

    Ok(Json(ApiResponse::success(result)))
}
