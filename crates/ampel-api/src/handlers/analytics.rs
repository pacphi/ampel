use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_db::entities::{health_score, pr_metrics};
use ampel_db::queries::RepoQueries;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsSummary {
    pub total_prs_merged: i64,
    pub avg_time_to_merge_hours: f64,
    pub avg_review_time_hours: f64,
    pub bot_pr_percentage: f64,
    pub top_contributors: Vec<ContributorStats>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributorStats {
    pub author: String,
    pub pr_count: i64,
    pub avg_merge_time_hours: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryHealthResponse {
    pub repository_id: Uuid,
    pub repository_name: String,
    pub current_score: i32,
    pub trend: String, // up, down, stable
    pub metrics: HealthMetrics,
    pub history: Vec<HealthScorePoint>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthMetrics {
    pub avg_time_to_merge_hours: f64,
    pub avg_review_time_hours: f64,
    pub stale_pr_count: i32,
    pub failed_check_rate: f64,
    pub pr_throughput_per_week: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthScorePoint {
    pub date: chrono::DateTime<Utc>,
    pub score: i32,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub days: Option<i64>,
    pub repository_id: Option<Uuid>,
}

/// Get analytics summary
pub async fn get_analytics_summary(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<ApiResponse<AnalyticsSummary>>, ApiError> {
    let days = query.days.unwrap_or(30);
    let cutoff = Utc::now() - Duration::days(days);

    // Get user's repositories
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;
    let repo_ids: Vec<Uuid> = repos.iter().map(|r| r.id).collect();

    if repo_ids.is_empty() {
        return Ok(Json(ApiResponse::success(AnalyticsSummary {
            total_prs_merged: 0,
            avg_time_to_merge_hours: 0.0,
            avg_review_time_hours: 0.0,
            bot_pr_percentage: 0.0,
            top_contributors: vec![],
        })));
    }

    // Get metrics for these repositories
    let metrics = pr_metrics::Entity::find()
        .filter(pr_metrics::Column::RepositoryId.is_in(repo_ids.clone()))
        .filter(pr_metrics::Column::MergedAt.gte(cutoff))
        .all(&state.db)
        .await?;

    let total_prs = metrics.len() as i64;
    let bot_prs = metrics.iter().filter(|m| m.is_bot).count() as i64;

    let avg_time_to_merge: f64 = if total_prs > 0 {
        let total_seconds: i64 = metrics
            .iter()
            .filter_map(|m| m.time_to_merge)
            .map(|t| t as i64)
            .sum();
        (total_seconds as f64) / (total_prs as f64) / 3600.0
    } else {
        0.0
    };

    let avg_review_time: f64 = if total_prs > 0 {
        let total_seconds: i64 = metrics
            .iter()
            .filter_map(|m| m.time_to_first_review)
            .map(|t| t as i64)
            .sum();
        (total_seconds as f64) / (total_prs as f64) / 3600.0
    } else {
        0.0
    };

    let bot_percentage = if total_prs > 0 {
        (bot_prs as f64) / (total_prs as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(ApiResponse::success(AnalyticsSummary {
        total_prs_merged: total_prs,
        avg_time_to_merge_hours: avg_time_to_merge,
        avg_review_time_hours: avg_review_time,
        bot_pr_percentage: bot_percentage,
        top_contributors: vec![], // Would require joining with PR data
    })))
}

/// Get repository health score
pub async fn get_repository_health(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(repo_id): Path<Uuid>,
) -> Result<Json<ApiResponse<RepositoryHealthResponse>>, ApiError> {
    // Verify ownership
    let repo = RepoQueries::find_by_id(&state.db, repo_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    // Get latest health score
    let latest_score = health_score::Entity::find()
        .filter(health_score::Column::RepositoryId.eq(repo_id))
        .order_by_desc(health_score::Column::CalculatedAt)
        .one(&state.db)
        .await?;

    // Get history (last 30 days)
    let history = health_score::Entity::find()
        .filter(health_score::Column::RepositoryId.eq(repo_id))
        .filter(health_score::Column::CalculatedAt.gte(Utc::now() - Duration::days(30)))
        .order_by_asc(health_score::Column::CalculatedAt)
        .all(&state.db)
        .await?;

    let (current_score, metrics, trend) = if let Some(score) = latest_score {
        let prev_score = history.iter().rev().nth(1).map(|h| h.score);
        let trend = match prev_score {
            Some(prev) if score.score > prev => "up",
            Some(prev) if score.score < prev => "down",
            _ => "stable",
        };

        (
            score.score,
            HealthMetrics {
                avg_time_to_merge_hours: score.avg_time_to_merge.unwrap_or(0) as f64 / 3600.0,
                avg_review_time_hours: score.avg_review_time.unwrap_or(0) as f64 / 3600.0,
                stale_pr_count: score.stale_pr_count.unwrap_or(0),
                failed_check_rate: score.failed_check_rate.unwrap_or(0.0) as f64,
                pr_throughput_per_week: score.pr_throughput.unwrap_or(0),
            },
            trend.to_string(),
        )
    } else {
        (
            0,
            HealthMetrics {
                avg_time_to_merge_hours: 0.0,
                avg_review_time_hours: 0.0,
                stale_pr_count: 0,
                failed_check_rate: 0.0,
                pr_throughput_per_week: 0,
            },
            "stable".to_string(),
        )
    };

    let history_points: Vec<HealthScorePoint> = history
        .into_iter()
        .map(|h| HealthScorePoint {
            date: h.calculated_at,
            score: h.score,
        })
        .collect();

    Ok(Json(ApiResponse::success(RepositoryHealthResponse {
        repository_id: repo_id,
        repository_name: repo.full_name,
        current_score,
        trend,
        metrics,
        history: history_points,
    })))
}

/// Get all repositories health overview
pub async fn get_health_overview(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<RepositoryHealthResponse>>>, ApiError> {
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;

    let mut results = Vec::new();

    for repo in repos {
        let latest_score = health_score::Entity::find()
            .filter(health_score::Column::RepositoryId.eq(repo.id))
            .order_by_desc(health_score::Column::CalculatedAt)
            .one(&state.db)
            .await?;

        let (current_score, metrics) = if let Some(score) = latest_score {
            (
                score.score,
                HealthMetrics {
                    avg_time_to_merge_hours: score.avg_time_to_merge.unwrap_or(0) as f64 / 3600.0,
                    avg_review_time_hours: score.avg_review_time.unwrap_or(0) as f64 / 3600.0,
                    stale_pr_count: score.stale_pr_count.unwrap_or(0),
                    failed_check_rate: score.failed_check_rate.unwrap_or(0.0) as f64,
                    pr_throughput_per_week: score.pr_throughput.unwrap_or(0),
                },
            )
        } else {
            (
                0,
                HealthMetrics {
                    avg_time_to_merge_hours: 0.0,
                    avg_review_time_hours: 0.0,
                    stale_pr_count: 0,
                    failed_check_rate: 0.0,
                    pr_throughput_per_week: 0,
                },
            )
        };

        results.push(RepositoryHealthResponse {
            repository_id: repo.id,
            repository_name: repo.full_name,
            current_score,
            trend: "stable".to_string(),
            metrics,
            history: vec![],
        });
    }

    Ok(Json(ApiResponse::success(results)))
}
