use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    Set,
};
use uuid::Uuid;

use crate::entities::repository::{ActiveModel, Column, Entity, Model};

pub struct RepoQueries;

impl RepoQueries {
    /// Find repository by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Find all repositories for a user
    pub async fn find_by_user_id(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_asc(Column::FullName)
            .all(db)
            .await
    }

    /// Find repository by user, provider, and provider ID
    pub async fn find_by_provider_id(
        db: &DatabaseConnection,
        user_id: Uuid,
        provider: &str,
        provider_id: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Provider.eq(provider))
            .filter(Column::ProviderId.eq(provider_id))
            .one(db)
            .await
    }

    /// Create a new repository
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        db: &DatabaseConnection,
        user_id: Uuid,
        provider: String,
        provider_id: String,
        owner: String,
        name: String,
        full_name: String,
        description: Option<String>,
        url: String,
        default_branch: String,
        is_private: bool,
        is_archived: bool,
        poll_interval_seconds: i32,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();
        let repo = ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            provider: Set(provider),
            provider_id: Set(provider_id),
            owner: Set(owner),
            name: Set(name),
            full_name: Set(full_name),
            description: Set(description),
            url: Set(url),
            default_branch: Set(default_branch),
            is_private: Set(is_private),
            is_archived: Set(is_archived),
            poll_interval_seconds: Set(poll_interval_seconds),
            last_polled_at: Set(None),
            provider_account_id: Set(None),
            group_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        repo.insert(db).await
    }

    /// Update last polled timestamp
    pub async fn update_last_polled(db: &DatabaseConnection, id: Uuid) -> Result<Model, DbErr> {
        let repo = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Repository not found".to_string()))?;

        let mut active: ActiveModel = repo.into();
        active.last_polled_at = Set(Some(Utc::now()));
        active.updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Update poll interval
    pub async fn update_poll_interval(
        db: &DatabaseConnection,
        id: Uuid,
        poll_interval_seconds: i32,
    ) -> Result<Model, DbErr> {
        let repo = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Repository not found".to_string()))?;

        let mut active: ActiveModel = repo.into();
        active.poll_interval_seconds = Set(poll_interval_seconds);
        active.updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Find repositories due for polling
    pub async fn find_due_for_polling(
        db: &DatabaseConnection,
        limit: u64,
    ) -> Result<Vec<Model>, DbErr> {
        use sea_orm::{Condition, QuerySelect};

        let now = Utc::now();

        // Find repos where last_polled_at is null or older than poll_interval_seconds
        Entity::find()
            .filter(
                Condition::any()
                    .add(Column::LastPolledAt.is_null())
                    // This is a simplification - in production you'd use raw SQL or a more complex query
                    .add(Column::LastPolledAt.lt(now)),
            )
            .order_by_asc(Column::LastPolledAt)
            .limit(limit)
            .all(db)
            .await
    }

    /// Delete repository
    pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<(), DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }
}
