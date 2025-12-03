use chrono::{DateTime, Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use ampel_db::entities::pull_request;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupJob;

impl From<DateTime<Utc>> for CleanupJob {
    fn from(_: DateTime<Utc>) -> Self {
        Self
    }
}

impl CleanupJob {
    pub async fn execute(&self, db: &DatabaseConnection) -> anyhow::Result<()> {
        // Delete closed PRs older than 30 days
        let cutoff = Utc::now() - Duration::days(30);

        let deleted = pull_request::Entity::delete_many()
            .filter(pull_request::Column::State.ne("open"))
            .filter(pull_request::Column::ClosedAt.lt(cutoff))
            .exec(db)
            .await?;

        tracing::info!(
            "Cleaned up {} old closed pull requests",
            deleted.rows_affected
        );

        Ok(())
    }
}
