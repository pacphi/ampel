use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities::user_settings::{ActiveModel, Column, Entity, Model};

pub struct UserSettingsQueries;

impl UserSettingsQueries {
    /// Find user settings by user ID
    pub async fn find_by_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
    }

    /// Create or update user settings
    pub async fn upsert(
        db: &DatabaseConnection,
        user_id: Uuid,
        merge_delay_seconds: i32,
        require_approval: bool,
        delete_branches_default: bool,
        default_merge_strategy: String,
        skip_review_requirement: bool,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();
        let existing = Self::find_by_user(db, user_id).await?;

        if let Some(existing) = existing {
            let mut active: ActiveModel = existing.into();
            active.merge_delay_seconds = Set(merge_delay_seconds);
            active.require_approval = Set(require_approval);
            active.delete_branches_default = Set(delete_branches_default);
            active.default_merge_strategy = Set(default_merge_strategy);
            active.skip_review_requirement = Set(skip_review_requirement);
            active.updated_at = Set(now);
            active.update(db).await
        } else {
            let settings = ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                merge_delay_seconds: Set(merge_delay_seconds),
                require_approval: Set(require_approval),
                delete_branches_default: Set(delete_branches_default),
                default_merge_strategy: Set(default_merge_strategy),
                skip_review_requirement: Set(skip_review_requirement),
                created_at: Set(now),
                updated_at: Set(now),
            };
            settings.insert(db).await
        }
    }

    /// Get or create default settings for a user
    pub async fn get_or_create_default(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Model, DbErr> {
        if let Some(settings) = Self::find_by_user(db, user_id).await? {
            Ok(settings)
        } else {
            Self::upsert(
                db,
                user_id,
                5,                    // merge_delay_seconds
                false,                // require_approval
                false,                // delete_branches_default
                "squash".to_string(), // default_merge_strategy
                false,                // skip_review_requirement
            )
            .await
        }
    }
}
