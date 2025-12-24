use axum::{extract::State, Json};
use metrics::{counter, histogram};
use serde::{Deserialize, Serialize};

use ampel_core::models::{AmpelStatus, RepositoryWithStatus};
use ampel_db::queries::{CICheckQueries, PrQueries, RepoQueries, ReviewQueries};

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub total_repositories: i32,
    pub total_open_prs: i32,
    pub status_counts: StatusCounts,
    pub provider_counts: ProviderCounts,
    pub repository_breakdown: VisibilityBreakdown,
    pub open_prs_breakdown: VisibilityBreakdown,
    pub ready_to_merge_breakdown: VisibilityBreakdown,
    pub needs_attention_breakdown: VisibilityBreakdown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCounts {
    pub green: i32,
    pub yellow: i32,
    pub red: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCounts {
    pub github: i32,
    pub gitlab: i32,
    pub bitbucket: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityBreakdown {
    pub public: i32,
    pub private: i32,
    pub archived: i32,
}

/// Get dashboard summary statistics
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id))]
pub async fn get_summary(
    State(mut state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    use std::collections::HashMap;
    use std::time::Instant;
    let start = Instant::now();

    // Try to get from cache if Redis is available
    if let Some(redis) = &mut state.redis {
        if let Some(cached_summary) = crate::cache::get_dashboard_cache(redis, auth.user_id).await {
            let duration = start.elapsed();
            tracing::info!(
                duration_ms = duration.as_millis(),
                user_id = %auth.user_id,
                cache_hit = true,
                "Dashboard summary served from cache"
            );
            return Ok(Json(ApiResponse::success(cached_summary)));
        }
    }

    // Query 1: Get all repositories for user
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;
    tracing::debug!(repo_count = repos.len(), "Retrieved repositories for user");

    if repos.is_empty() {
        return Ok(Json(ApiResponse::success(DashboardSummary {
            total_repositories: 0,
            total_open_prs: 0,
            status_counts: StatusCounts {
                green: 0,
                yellow: 0,
                red: 0,
            },
            provider_counts: ProviderCounts {
                github: 0,
                gitlab: 0,
                bitbucket: 0,
            },
            repository_breakdown: VisibilityBreakdown::default(),
            open_prs_breakdown: VisibilityBreakdown::default(),
            ready_to_merge_breakdown: VisibilityBreakdown::default(),
            needs_attention_breakdown: VisibilityBreakdown::default(),
        })));
    }

    // Create repo lookup map for quick access
    let repo_ids: Vec<_> = repos.iter().map(|r| r.id).collect();
    let repo_map: HashMap<_, _> = repos.iter().map(|r| (r.id, r)).collect();

    // Query 2: Batch load all open PRs for all repositories
    let all_open_prs = PrQueries::find_open_for_repositories(&state.db, &repo_ids).await?;
    tracing::debug!(
        pr_count = all_open_prs.len(),
        "Loaded all open PRs in batch"
    );

    // Count repository breakdowns and providers
    let mut repo_breakdown = VisibilityBreakdown::default();
    let mut github_count = 0;
    let mut gitlab_count = 0;
    let mut bitbucket_count = 0;

    for repo in &repos {
        if repo.is_archived {
            repo_breakdown.archived += 1;
        } else if repo.is_private {
            repo_breakdown.private += 1;
        } else {
            repo_breakdown.public += 1;
        }

        match repo.provider.as_str() {
            "github" => github_count += 1,
            "gitlab" => gitlab_count += 1,
            "bitbucket" => bitbucket_count += 1,
            _ => {}
        }
    }

    if all_open_prs.is_empty() {
        return Ok(Json(ApiResponse::success(DashboardSummary {
            total_repositories: repos.len() as i32,
            total_open_prs: 0,
            status_counts: StatusCounts {
                green: 0,
                yellow: 0,
                red: 0,
            },
            provider_counts: ProviderCounts {
                github: github_count,
                gitlab: gitlab_count,
                bitbucket: bitbucket_count,
            },
            repository_breakdown: repo_breakdown,
            open_prs_breakdown: VisibilityBreakdown::default(),
            ready_to_merge_breakdown: VisibilityBreakdown::default(),
            needs_attention_breakdown: VisibilityBreakdown::default(),
        })));
    }

    let pr_ids: Vec<_> = all_open_prs.iter().map(|pr| pr.id).collect();

    // Query 3: Batch load all CI checks for all PRs
    let all_ci_checks = CICheckQueries::find_for_pull_requests(&state.db, &pr_ids).await?;
    tracing::debug!(
        ci_check_count = all_ci_checks.len(),
        "Loaded all CI checks in batch"
    );

    // Query 4: Batch load all reviews for all PRs
    let all_reviews = ReviewQueries::find_for_pull_requests(&state.db, &pr_ids).await?;
    tracing::debug!(
        review_count = all_reviews.len(),
        "Loaded all reviews in batch"
    );

    // Build lookup maps for O(1) access
    let mut ci_checks_by_pr: HashMap<uuid::Uuid, Vec<_>> = HashMap::new();
    for ci_check in all_ci_checks {
        ci_checks_by_pr
            .entry(ci_check.pull_request_id)
            .or_default()
            .push(ci_check);
    }

    let mut reviews_by_pr: HashMap<uuid::Uuid, Vec<_>> = HashMap::new();
    for review in all_reviews {
        reviews_by_pr
            .entry(review.pull_request_id)
            .or_default()
            .push(review);
    }

    // Initialize counters
    let total_open_prs = all_open_prs.len() as i32;
    let mut green_count = 0;
    let mut yellow_count = 0;
    let mut red_count = 0;
    let mut open_prs_breakdown = VisibilityBreakdown::default();
    let mut ready_breakdown = VisibilityBreakdown::default();
    let mut needs_attention_breakdown = VisibilityBreakdown::default();

    // Process all PRs with their CI checks and reviews
    for pr_model in &all_open_prs {
        let repo = repo_map
            .get(&pr_model.repository_id)
            .expect("PR repository must exist");

        // Count open PRs by repo visibility
        if repo.is_archived {
            open_prs_breakdown.archived += 1;
        } else if repo.is_private {
            open_prs_breakdown.private += 1;
        } else {
            open_prs_breakdown.public += 1;
        }

        // Get CI checks and reviews for this PR from lookup maps
        let ci_checks = ci_checks_by_pr
            .get(&pr_model.id)
            .cloned()
            .unwrap_or_default();
        let reviews = reviews_by_pr.get(&pr_model.id).cloned().unwrap_or_default();

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
            AmpelStatus::Green => {
                green_count += 1;
                if repo.is_archived {
                    ready_breakdown.archived += 1;
                } else if repo.is_private {
                    ready_breakdown.private += 1;
                } else {
                    ready_breakdown.public += 1;
                }
            }
            AmpelStatus::Yellow => yellow_count += 1,
            AmpelStatus::Red => {
                red_count += 1;
                if repo.is_archived {
                    needs_attention_breakdown.archived += 1;
                } else if repo.is_private {
                    needs_attention_breakdown.private += 1;
                } else {
                    needs_attention_breakdown.public += 1;
                }
            }
            AmpelStatus::None => {}
        }
    }

    let duration = start.elapsed();

    // Build summary
    let summary = DashboardSummary {
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
        repository_breakdown: repo_breakdown,
        open_prs_breakdown,
        ready_to_merge_breakdown: ready_breakdown,
        needs_attention_breakdown,
    };

    // Cache the summary if Redis is available
    if let Some(redis) = &mut state.redis {
        crate::cache::set_dashboard_cache(redis, auth.user_id, &summary).await;
    }

    // Log summary statistics with structured fields
    tracing::info!(
        duration_ms = duration.as_millis(),
        total_repos = repos.len(),
        total_open_prs,
        green_count,
        yellow_count,
        red_count,
        github_count,
        gitlab_count,
        bitbucket_count,
        public_repos = summary.repository_breakdown.public,
        private_repos = summary.repository_breakdown.private,
        archived_repos = summary.repository_breakdown.archived,
        cache_hit = false,
        "Dashboard summary generated"
    );

    // METRICS COLLECTION:
    // Record response duration as histogram
    histogram!("ampel_dashboard_summary_duration_seconds").record(duration.as_secs_f64());

    // Record breakdown counts by visibility status
    counter!("ampel_dashboard_breakdown_total", "visibility" => "green")
        .increment(green_count as u64);
    counter!("ampel_dashboard_breakdown_total", "visibility" => "yellow")
        .increment(yellow_count as u64);
    counter!("ampel_dashboard_breakdown_total", "visibility" => "red").increment(red_count as u64);

    Ok(Json(ApiResponse::success(summary)))
}

/// Get repositories for grid/list view
pub async fn get_grid(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<RepositoryWithStatus>>>, ApiError> {
    use std::collections::HashMap;

    // Query 1: Get all repositories for user
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;

    if repos.is_empty() {
        return Ok(Json(ApiResponse::success(Vec::new())));
    }

    // Query 2: Batch load all open PRs for all repositories
    let repo_ids: Vec<_> = repos.iter().map(|r| r.id).collect();
    let all_open_prs = PrQueries::find_open_for_repositories(&state.db, &repo_ids).await?;

    // Group PRs by repository for easy access
    let mut prs_by_repo: HashMap<uuid::Uuid, Vec<_>> = HashMap::new();
    for pr in &all_open_prs {
        prs_by_repo.entry(pr.repository_id).or_default().push(pr);
    }

    // If no PRs at all, return repos with None status
    if all_open_prs.is_empty() {
        let result: Vec<_> = repos
            .into_iter()
            .map(|repo| RepositoryWithStatus {
                repository: repo.into(),
                status: AmpelStatus::None,
                open_pr_count: 0,
            })
            .collect();
        return Ok(Json(ApiResponse::success(result)));
    }

    let pr_ids: Vec<_> = all_open_prs.iter().map(|pr| pr.id).collect();

    // Query 3: Batch load all CI checks for all PRs
    let all_ci_checks = CICheckQueries::find_for_pull_requests(&state.db, &pr_ids).await?;

    // Query 4: Batch load all reviews for all PRs
    let all_reviews = ReviewQueries::find_for_pull_requests(&state.db, &pr_ids).await?;

    // Build lookup maps for O(1) access
    let mut ci_checks_by_pr: HashMap<uuid::Uuid, Vec<_>> = HashMap::new();
    for ci_check in all_ci_checks {
        ci_checks_by_pr
            .entry(ci_check.pull_request_id)
            .or_default()
            .push(ci_check);
    }

    let mut reviews_by_pr: HashMap<uuid::Uuid, Vec<_>> = HashMap::new();
    for review in all_reviews {
        reviews_by_pr
            .entry(review.pull_request_id)
            .or_default()
            .push(review);
    }

    // Process each repository
    let mut result = Vec::with_capacity(repos.len());

    for repo in repos {
        let repo_prs = prs_by_repo.get(&repo.id);
        let open_pr_count = repo_prs.map(|prs| prs.len() as i32).unwrap_or(0);

        // Calculate repository status based on actual PR statuses
        let mut pr_statuses = Vec::new();

        if let Some(repo_prs) = repo_prs {
            for pr_model in repo_prs {
                // Get CI checks and reviews for this PR from lookup maps
                let ci_checks = ci_checks_by_pr
                    .get(&pr_model.id)
                    .cloned()
                    .unwrap_or_default();
                let reviews = reviews_by_pr.get(&pr_model.id).cloned().unwrap_or_default();

                // Convert database models to core models
                let pr: ampel_core::models::PullRequest = (*pr_model).clone().into();
                let ci_checks: Vec<ampel_core::models::CICheck> =
                    ci_checks.into_iter().map(|c| c.into()).collect();
                let reviews: Vec<ampel_core::models::Review> =
                    reviews.into_iter().map(|r| r.into()).collect();

                // Calculate status for this PR
                let pr_status = AmpelStatus::for_pull_request(&pr, &ci_checks, &reviews);
                pr_statuses.push(pr_status);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_breakdown_default() {
        let breakdown = VisibilityBreakdown::default();
        assert_eq!(breakdown.public, 0);
        assert_eq!(breakdown.private, 0);
        assert_eq!(breakdown.archived, 0);
    }

    #[test]
    fn test_visibility_breakdown_serialization() {
        let breakdown = VisibilityBreakdown {
            public: 10,
            private: 5,
            archived: 2,
        };

        let json = serde_json::to_string(&breakdown).unwrap();
        assert!(json.contains("\"public\":10"));
        assert!(json.contains("\"private\":5"));
        assert!(json.contains("\"archived\":2"));
    }

    #[test]
    fn test_visibility_breakdown_clone() {
        let breakdown = VisibilityBreakdown {
            public: 3,
            private: 7,
            archived: 1,
        };

        let cloned = breakdown.clone();
        assert_eq!(cloned.public, 3);
        assert_eq!(cloned.private, 7);
        assert_eq!(cloned.archived, 1);
    }

    #[test]
    fn test_dashboard_summary_has_all_fields() {
        let summary = DashboardSummary {
            total_repositories: 10,
            total_open_prs: 5,
            status_counts: StatusCounts {
                green: 2,
                yellow: 2,
                red: 1,
            },
            provider_counts: ProviderCounts {
                github: 8,
                gitlab: 2,
                bitbucket: 0,
            },
            repository_breakdown: VisibilityBreakdown {
                public: 6,
                private: 3,
                archived: 1,
            },
            open_prs_breakdown: VisibilityBreakdown {
                public: 3,
                private: 2,
                archived: 0,
            },
            ready_to_merge_breakdown: VisibilityBreakdown {
                public: 1,
                private: 1,
                archived: 0,
            },
            needs_attention_breakdown: VisibilityBreakdown {
                public: 1,
                private: 0,
                archived: 0,
            },
        };

        // Verify all fields are accessible
        assert_eq!(summary.total_repositories, 10);
        assert_eq!(summary.total_open_prs, 5);
        assert_eq!(summary.repository_breakdown.public, 6);
        assert_eq!(summary.open_prs_breakdown.private, 2);
        assert_eq!(summary.ready_to_merge_breakdown.public, 1);
        assert_eq!(summary.needs_attention_breakdown.public, 1);
    }
}
