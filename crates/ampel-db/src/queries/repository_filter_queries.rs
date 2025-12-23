use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities::repository_filter::{ActiveModel, Column, Entity, Model};

pub struct RepositoryFilterQueries;

impl RepositoryFilterQueries {
    /// Find repository filters by user ID
    pub async fn find_by_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
    }

    /// Get repository filters or create default if not exists
    pub async fn get_or_create_default(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Model, DbErr> {
        if let Some(filters) = Self::find_by_user(db, user_id).await? {
            Ok(filters)
        } else {
            // Create default filters (all enabled)
            let now = Utc::now();
            let filters = ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                include_public: Set(true),
                include_private: Set(true),
                include_archived: Set(true),
                created_at: Set(now),
                updated_at: Set(now),
            };
            filters.insert(db).await
        }
    }

    /// Update repository filters
    pub async fn update(
        db: &DatabaseConnection,
        user_id: Uuid,
        include_public: Option<bool>,
        include_private: Option<bool>,
        include_archived: Option<bool>,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();
        let existing = Self::get_or_create_default(db, user_id).await?;

        let mut active: ActiveModel = existing.into();

        if let Some(val) = include_public {
            active.include_public = Set(val);
        }
        if let Some(val) = include_private {
            active.include_private = Set(val);
        }
        if let Some(val) = include_archived {
            active.include_archived = Set(val);
        }

        active.updated_at = Set(now);
        active.update(db).await
    }
}
