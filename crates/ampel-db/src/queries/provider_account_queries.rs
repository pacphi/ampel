use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities::provider_account::{ActiveModel, Column, Entity, Model};

pub struct ProviderAccountQueries;

impl ProviderAccountQueries {
    /// Find all provider accounts for a user
    pub async fn find_by_user(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    /// Find provider account by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Find default account for a specific provider and user
    pub async fn find_default_for_provider(
        db: &DatabaseConnection,
        user_id: Uuid,
        provider: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Provider.eq(provider))
            .filter(Column::IsDefault.eq(true))
            .one(db)
            .await
    }

    /// Find default account for a specific provider and instance URL
    pub async fn find_default_for_provider_instance(
        db: &DatabaseConnection,
        user_id: Uuid,
        provider: &str,
        instance_url: Option<String>,
    ) -> Result<Option<Model>, DbErr> {
        let mut query = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Provider.eq(provider))
            .filter(Column::IsDefault.eq(true));

        query = match instance_url {
            Some(url) => query.filter(Column::InstanceUrl.eq(url)),
            None => query.filter(Column::InstanceUrl.is_null()),
        };

        query.one(db).await
    }

    /// Count provider accounts for a user and provider
    pub async fn count_by_user_and_provider(
        db: &DatabaseConnection,
        user_id: Uuid,
        provider: &str,
    ) -> Result<u64, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Provider.eq(provider))
            .count(db)
            .await
    }

    /// Set a provider account as default (unsets other defaults for same provider)
    pub async fn set_default(
        db: &DatabaseConnection,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Model, DbErr> {
        // First, find the account to get its provider and instance_url
        let account = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Provider account not found".to_string(),
            ))?;

        // Verify ownership
        if account.user_id != user_id {
            return Err(DbErr::Custom("Unauthorized".to_string()));
        }

        // Unset all other defaults for this user/provider/instance combination
        let mut unset_query = Entity::update_many()
            .col_expr(Column::IsDefault, sea_orm::sea_query::Expr::value(false))
            .col_expr(
                Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(Utc::now()),
            )
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Provider.eq(&account.provider))
            .filter(Column::IsDefault.eq(true));

        unset_query = match &account.instance_url {
            Some(url) => unset_query.filter(Column::InstanceUrl.eq(url)),
            None => unset_query.filter(Column::InstanceUrl.is_null()),
        };

        unset_query.exec(db).await?;

        // Set the target account as default
        let mut active: ActiveModel = account.into();
        active.is_default = Set(true);
        active.updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Find all active provider accounts for a user
    pub async fn find_active_by_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IsActive.eq(true))
            .all(db)
            .await
    }

    /// Find all provider accounts for a specific provider
    pub async fn find_by_user_and_provider(
        db: &DatabaseConnection,
        user_id: Uuid,
        provider: &str,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Provider.eq(provider))
            .all(db)
            .await
    }

    /// Update validation status and timestamp
    pub async fn update_validation_status(
        db: &DatabaseConnection,
        id: Uuid,
        validation_status: &str,
    ) -> Result<Model, DbErr> {
        let account = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Provider account not found".to_string(),
            ))?;

        let mut active: ActiveModel = account.into();
        active.validation_status = Set(validation_status.to_string());
        active.last_validated_at = Set(Some(Utc::now()));
        active.updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Update access token
    pub async fn update_access_token(
        db: &DatabaseConnection,
        id: Uuid,
        access_token_encrypted: Vec<u8>,
    ) -> Result<Model, DbErr> {
        let account = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Provider account not found".to_string(),
            ))?;

        let mut active: ActiveModel = account.into();
        active.access_token_encrypted = Set(access_token_encrypted);
        active.updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Toggle active status
    pub async fn set_active_status(
        db: &DatabaseConnection,
        id: Uuid,
        is_active: bool,
    ) -> Result<Model, DbErr> {
        let account = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Provider account not found".to_string(),
            ))?;

        let mut active: ActiveModel = account.into();
        active.is_active = Set(is_active);
        active.updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Delete a provider account
    pub async fn delete(db: &DatabaseConnection, id: Uuid, user_id: Uuid) -> Result<(), DbErr> {
        // Verify ownership before deletion
        let account = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Provider account not found".to_string(),
            ))?;

        if account.user_id != user_id {
            return Err(DbErr::Custom("Unauthorized".to_string()));
        }

        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }
}
