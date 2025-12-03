use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities::user::{ActiveModel, Column, Entity, Model};

pub struct UserQueries;

impl UserQueries {
    /// Find user by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Find user by email
    pub async fn find_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find().filter(Column::Email.eq(email)).one(db).await
    }

    /// Create a new user (OAuth-based, no password)
    pub async fn create(
        db: &DatabaseConnection,
        email: String,
        display_name: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();
        let user = ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(email),
            display_name: Set(display_name),
            avatar_url: Set(avatar_url),
            created_at: Set(now),
            updated_at: Set(now),
        };

        user.insert(db).await
    }

    /// Update user display name
    pub async fn update_display_name(
        db: &DatabaseConnection,
        id: Uuid,
        display_name: Option<String>,
    ) -> Result<Model, DbErr> {
        let user = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

        let mut active: ActiveModel = user.into();
        active.display_name = Set(display_name);
        active.updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Update user avatar URL
    pub async fn update_avatar(
        db: &DatabaseConnection,
        id: Uuid,
        avatar_url: Option<String>,
    ) -> Result<Model, DbErr> {
        let user = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

        let mut active: ActiveModel = user.into();
        active.avatar_url = Set(avatar_url);
        active.updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Delete user
    pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<(), DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }
}
