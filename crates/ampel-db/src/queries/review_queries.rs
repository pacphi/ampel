use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    Set,
};
use uuid::Uuid;

use crate::entities::review::{ActiveModel, Column, Entity, Model};

pub struct ReviewQueries;

impl ReviewQueries {
    /// Find all reviews for a PR
    pub async fn find_by_pull_request(
        db: &DatabaseConnection,
        pull_request_id: Uuid,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::PullRequestId.eq(pull_request_id))
            .order_by_desc(Column::SubmittedAt)
            .all(db)
            .await
    }

    /// Find the latest review from each reviewer for a PR
    pub async fn find_latest_by_pull_request(
        db: &DatabaseConnection,
        pull_request_id: Uuid,
    ) -> Result<Vec<Model>, DbErr> {
        // For simplicity, we'll get all reviews and filter in memory
        // In production, you'd want a more efficient query
        let all_reviews = Self::find_by_pull_request(db, pull_request_id).await?;

        let mut latest_by_reviewer: std::collections::HashMap<String, Model> =
            std::collections::HashMap::new();

        for review in all_reviews {
            latest_by_reviewer
                .entry(review.reviewer.clone())
                .or_insert(review);
        }

        Ok(latest_by_reviewer.into_values().collect())
    }

    /// Create or update a review
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        db: &DatabaseConnection,
        id: Uuid,
        pull_request_id: Uuid,
        reviewer: String,
        reviewer_avatar_url: Option<String>,
        state: String,
        body: Option<String>,
        submitted_at: chrono::DateTime<Utc>,
    ) -> Result<Model, DbErr> {
        // Check if review already exists
        if let Some(existing) = Entity::find_by_id(id).one(db).await? {
            let mut active: ActiveModel = existing.into();
            active.state = Set(state);
            active.body = Set(body);
            active.submitted_at = Set(submitted_at);
            return active.update(db).await;
        }

        // Create new review
        let review = ActiveModel {
            id: Set(id),
            pull_request_id: Set(pull_request_id),
            reviewer: Set(reviewer),
            reviewer_avatar_url: Set(reviewer_avatar_url),
            state: Set(state),
            body: Set(body),
            submitted_at: Set(submitted_at),
        };

        review.insert(db).await
    }

    /// Delete all reviews for a PR (for full refresh)
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
}
