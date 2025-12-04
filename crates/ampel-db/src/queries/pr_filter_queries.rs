use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities::pr_filter::{ActiveModel, Column, Entity, Model};

pub struct PrFilterQueries;

impl PrFilterQueries {
    /// Find PR filter settings by user ID
    pub async fn find_by_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
    }

    /// Create or update PR filter settings for a user
    pub async fn upsert(
        db: &DatabaseConnection,
        user_id: Uuid,
        allowed_actors: String,
        skip_labels: String,
        max_age_days: Option<i32>,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();
        let existing = Self::find_by_user(db, user_id).await?;

        if let Some(existing) = existing {
            let mut active: ActiveModel = existing.into();
            active.allowed_actors = Set(allowed_actors);
            active.skip_labels = Set(skip_labels);
            active.max_age_days = Set(max_age_days);
            active.updated_at = Set(now);
            active.update(db).await
        } else {
            let filter = ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                allowed_actors: Set(allowed_actors),
                skip_labels: Set(skip_labels),
                max_age_days: Set(max_age_days),
                created_at: Set(now),
                updated_at: Set(now),
            };
            filter.insert(db).await
        }
    }

    /// Delete PR filter settings for a user
    pub async fn delete(db: &DatabaseConnection, user_id: Uuid) -> Result<(), DbErr> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(())
    }
}
