use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::services::PrService;
use ampel_db::entities::{pr_metrics, pull_request, review};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCollectionJob;

impl From<DateTime<Utc>> for MetricsCollectionJob {
    fn from(_: DateTime<Utc>) -> Self {
        Self
    }
}

impl MetricsCollectionJob {
    pub async fn execute(&self, db: &DatabaseConnection) -> anyhow::Result<()> {
        // Find recently merged PRs without metrics
        let merged_prs = pull_request::Entity::find()
            .filter(pull_request::Column::State.eq("merged"))
            .filter(pull_request::Column::MergedAt.is_not_null())
            .all(db)
            .await?;

        tracing::info!("Checking {} merged PRs for metrics", merged_prs.len());

        for pr in merged_prs {
            // Check if metrics already exist
            let existing = pr_metrics::Entity::find()
                .filter(pr_metrics::Column::PullRequestId.eq(pr.id))
                .one(db)
                .await?;

            if existing.is_some() {
                continue;
            }

            if let Err(e) = self.collect_pr_metrics(db, &pr).await {
                tracing::error!("Failed to collect metrics for PR #{}: {}", pr.number, e);
            }
        }

        Ok(())
    }

    async fn collect_pr_metrics(
        &self,
        db: &DatabaseConnection,
        pr: &pull_request::Model,
    ) -> anyhow::Result<()> {
        let merged_at = pr
            .merged_at
            .ok_or_else(|| anyhow::anyhow!("PR not merged"))?;
        let created_at = pr.created_at;

        // Time to merge (total)
        let time_to_merge = (merged_at - created_at).num_seconds() as i32;

        // Get reviews to calculate time to first review
        let reviews = review::Entity::find()
            .filter(review::Column::PullRequestId.eq(pr.id))
            .all(db)
            .await?;

        let time_to_first_review = reviews
            .iter()
            .map(|r| r.submitted_at)
            .min()
            .map(|first_review| (first_review - created_at).num_seconds() as i32);

        // Time to approval
        let time_to_approval = reviews
            .iter()
            .filter(|r| r.state == "approved")
            .map(|r| r.submitted_at)
            .min()
            .map(|approval| (approval - created_at).num_seconds() as i32);

        // Count review rounds (unique reviewers who left changes_requested then approved)
        let review_rounds = reviews
            .iter()
            .filter(|r| r.state == "changes_requested")
            .count() as i32;

        // Check if bot author
        let is_bot = PrService::is_bot_author(&pr.author);

        let metrics = pr_metrics::ActiveModel {
            id: Set(Uuid::new_v4()),
            pull_request_id: Set(pr.id),
            repository_id: Set(pr.repository_id),
            time_to_first_review: Set(time_to_first_review),
            time_to_approval: Set(time_to_approval),
            time_to_merge: Set(Some(time_to_merge)),
            review_rounds: Set(Some(review_rounds)),
            comments_count: Set(Some(pr.comments_count)),
            is_bot: Set(is_bot),
            merged_at: Set(Some(merged_at)),
            recorded_at: Set(Utc::now()),
        };

        metrics.insert(db).await?;

        tracing::debug!(
            "Collected metrics for PR #{}: merge_time={}s, review_time={:?}s",
            pr.number,
            time_to_merge,
            time_to_first_review
        );

        Ok(())
    }
}
