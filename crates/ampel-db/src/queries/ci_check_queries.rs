use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities::ci_check::{ActiveModel, Column, Entity, Model};

pub struct CICheckQueries;

impl CICheckQueries {
    /// Find all CI checks for a PR
    pub async fn find_by_pull_request(
        db: &DatabaseConnection,
        pull_request_id: Uuid,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::PullRequestId.eq(pull_request_id))
            .all(db)
            .await
    }

    /// Find CI check by PR and name
    pub async fn find_by_name(
        db: &DatabaseConnection,
        pull_request_id: Uuid,
        name: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::PullRequestId.eq(pull_request_id))
            .filter(Column::Name.eq(name))
            .one(db)
            .await
    }

    /// Create or update a CI check
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        db: &DatabaseConnection,
        pull_request_id: Uuid,
        name: String,
        status: String,
        conclusion: Option<String>,
        url: Option<String>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        duration_seconds: Option<i32>,
    ) -> Result<Model, DbErr> {
        // Check if check already exists
        if let Some(existing) = Self::find_by_name(db, pull_request_id, &name).await? {
            let mut active: ActiveModel = existing.into();
            active.status = Set(status);
            active.conclusion = Set(conclusion);
            active.url = Set(url);
            active.started_at = Set(started_at);
            active.completed_at = Set(completed_at);
            active.duration_seconds = Set(duration_seconds);
            return active.update(db).await;
        }

        // Create new check
        let check = ActiveModel {
            id: Set(Uuid::new_v4()),
            pull_request_id: Set(pull_request_id),
            name: Set(name),
            status: Set(status),
            conclusion: Set(conclusion),
            url: Set(url),
            started_at: Set(started_at),
            completed_at: Set(completed_at),
            duration_seconds: Set(duration_seconds),
        };

        check.insert(db).await
    }

    /// Delete all CI checks for a PR (for full refresh)
    pub async fn delete_by_pull_request(
        db: &DatabaseConnection,
        pull_request_id: Uuid,
    ) -> Result<u64, DbErr> {
        let result = Entity::delete_many()
            .filter(Column::PullRequestId.eq(pull_request_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    /// Find all CI checks for multiple PRs in one batch query
    /// Returns all CI checks for the given PR IDs
    pub async fn find_for_pull_requests(
        db: &DatabaseConnection,
        pull_request_ids: &[Uuid],
    ) -> Result<Vec<Model>, DbErr> {
        if pull_request_ids.is_empty() {
            return Ok(Vec::new());
        }

        Entity::find()
            .filter(Column::PullRequestId.is_in(pull_request_ids.iter().copied()))
            .all(db)
            .await
    }
}
