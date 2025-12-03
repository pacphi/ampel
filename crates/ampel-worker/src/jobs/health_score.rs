use chrono::{DateTime, Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_db::entities::{health_score, pr_metrics, pull_request, repository};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScoreJob;

impl From<DateTime<Utc>> for HealthScoreJob {
    fn from(_: DateTime<Utc>) -> Self {
        Self
    }
}

impl HealthScoreJob {
    pub async fn execute(&self, db: &DatabaseConnection) -> anyhow::Result<()> {
        let repos = repository::Entity::find().all(db).await?;

        tracing::info!("Calculating health scores for {} repositories", repos.len());

        for repo in repos {
            if let Err(e) = self.calculate_repo_health(db, &repo).await {
                tracing::error!(
                    "Failed to calculate health for repo {}: {}",
                    repo.full_name,
                    e
                );
            }
        }

        Ok(())
    }

    async fn calculate_repo_health(
        &self,
        db: &DatabaseConnection,
        repo: &repository::Model,
    ) -> anyhow::Result<()> {
        let now = Utc::now();
        let thirty_days_ago = now - Duration::days(30);
        let seven_days_ago = now - Duration::days(7);

        // Get metrics for last 30 days
        let metrics = pr_metrics::Entity::find()
            .filter(pr_metrics::Column::RepositoryId.eq(repo.id))
            .filter(pr_metrics::Column::MergedAt.gte(thirty_days_ago))
            .all(db)
            .await?;

        // Calculate average time to merge
        let avg_time_to_merge: Option<i32> = if !metrics.is_empty() {
            let total: i64 = metrics
                .iter()
                .filter_map(|m| m.time_to_merge)
                .map(|t| t as i64)
                .sum();
            let count = metrics.iter().filter(|m| m.time_to_merge.is_some()).count();
            if count > 0 {
                Some((total / count as i64) as i32)
            } else {
                None
            }
        } else {
            None
        };

        // Calculate average review time
        let avg_review_time: Option<i32> = if !metrics.is_empty() {
            let total: i64 = metrics
                .iter()
                .filter_map(|m| m.time_to_first_review)
                .map(|t| t as i64)
                .sum();
            let count = metrics
                .iter()
                .filter(|m| m.time_to_first_review.is_some())
                .count();
            if count > 0 {
                Some((total / count as i64) as i32)
            } else {
                None
            }
        } else {
            None
        };

        // Count stale PRs (open for > 7 days)
        let stale_prs = pull_request::Entity::find()
            .filter(pull_request::Column::RepositoryId.eq(repo.id))
            .filter(pull_request::Column::State.eq("open"))
            .filter(pull_request::Column::CreatedAt.lt(seven_days_ago))
            .count(db)
            .await? as i32;

        // Calculate PR throughput (merged in last 7 days)
        let recent_metrics = pr_metrics::Entity::find()
            .filter(pr_metrics::Column::RepositoryId.eq(repo.id))
            .filter(pr_metrics::Column::MergedAt.gte(seven_days_ago))
            .count(db)
            .await? as i32;

        // Calculate health score (0-100)
        let score = self.calculate_score(
            avg_time_to_merge,
            avg_review_time,
            stale_prs,
            recent_metrics,
        );

        // Save health score
        let health = health_score::ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(repo.id),
            score: Set(score),
            avg_time_to_merge: Set(avg_time_to_merge),
            avg_review_time: Set(avg_review_time),
            stale_pr_count: Set(Some(stale_prs)),
            failed_check_rate: Set(None), // Would need to track failed checks
            pr_throughput: Set(Some(recent_metrics)),
            calculated_at: Set(now),
        };

        health.insert(db).await?;

        tracing::debug!(
            "Health score for {}: {} (stale: {}, throughput: {})",
            repo.full_name,
            score,
            stale_prs,
            recent_metrics
        );

        Ok(())
    }

    fn calculate_score(
        &self,
        avg_time_to_merge: Option<i32>,
        avg_review_time: Option<i32>,
        stale_prs: i32,
        throughput: i32,
    ) -> i32 {
        let mut score = 100;

        // Penalize slow merge times (> 24h starts penalty, > 72h severe)
        if let Some(merge_time) = avg_time_to_merge {
            let hours = merge_time / 3600;
            if hours > 72 {
                score -= 30;
            } else if hours > 48 {
                score -= 20;
            } else if hours > 24 {
                score -= 10;
            }
        }

        // Penalize slow review times (> 4h starts penalty)
        if let Some(review_time) = avg_review_time {
            let hours = review_time / 3600;
            if hours > 24 {
                score -= 20;
            } else if hours > 8 {
                score -= 10;
            } else if hours > 4 {
                score -= 5;
            }
        }

        // Penalize stale PRs
        if stale_prs > 10 {
            score -= 25;
        } else if stale_prs > 5 {
            score -= 15;
        } else if stale_prs > 0 {
            score -= stale_prs * 2;
        }

        // Bonus for good throughput
        if throughput >= 10 {
            score += 10;
        } else if throughput >= 5 {
            score += 5;
        }

        score.clamp(0, 100)
    }
}
