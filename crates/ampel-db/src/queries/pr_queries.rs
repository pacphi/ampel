use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

use crate::entities::pull_request::{ActiveModel, Column, Entity, Model};

pub struct PrQueries;

impl PrQueries {
    /// Find PR by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Find all open PRs for a repository
    pub async fn find_open_by_repository(
        db: &DatabaseConnection,
        repository_id: Uuid,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::RepositoryId.eq(repository_id))
            .filter(Column::State.eq("open"))
            .order_by_desc(Column::UpdatedAt)
            .all(db)
            .await
    }

    /// Find all PRs for a repository (any state)
    pub async fn find_by_repository(
        db: &DatabaseConnection,
        repository_id: Uuid,
        limit: u64,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::RepositoryId.eq(repository_id))
            .order_by_desc(Column::UpdatedAt)
            .limit(limit)
            .all(db)
            .await
    }

    /// Find PR by repository and number
    pub async fn find_by_number(
        db: &DatabaseConnection,
        repository_id: Uuid,
        number: i32,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::RepositoryId.eq(repository_id))
            .filter(Column::Number.eq(number))
            .one(db)
            .await
    }

    /// Find all open PRs for a user (across all their repositories)
    pub async fn find_open_by_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        use crate::entities::repository;

        let offset = (page - 1) * per_page;

        let prs = Entity::find()
            .inner_join(repository::Entity)
            .filter(repository::Column::UserId.eq(user_id))
            .filter(Column::State.eq("open"))
            .order_by_desc(Column::UpdatedAt)
            .offset(offset)
            .limit(per_page)
            .all(db)
            .await?;

        let total = Entity::find()
            .inner_join(repository::Entity)
            .filter(repository::Column::UserId.eq(user_id))
            .filter(Column::State.eq("open"))
            .count(db)
            .await?;

        Ok((prs, total))
    }

    /// Create or update a PR
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        db: &DatabaseConnection,
        repository_id: Uuid,
        provider: String,
        provider_id: String,
        number: i32,
        title: String,
        description: Option<String>,
        url: String,
        state: String,
        source_branch: String,
        target_branch: String,
        author: String,
        author_avatar_url: Option<String>,
        is_draft: bool,
        is_mergeable: Option<bool>,
        has_conflicts: bool,
        additions: i32,
        deletions: i32,
        changed_files: i32,
        commits_count: i32,
        comments_count: i32,
        created_at: chrono::DateTime<Utc>,
        updated_at: chrono::DateTime<Utc>,
        merged_at: Option<chrono::DateTime<Utc>>,
        closed_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<Model, DbErr> {
        // Check if PR already exists
        if let Some(existing) = Self::find_by_number(db, repository_id, number).await? {
            let mut active: ActiveModel = existing.into();
            active.title = Set(title);
            active.description = Set(description);
            active.state = Set(state);
            active.is_draft = Set(is_draft);
            active.is_mergeable = Set(is_mergeable);
            active.has_conflicts = Set(has_conflicts);
            active.additions = Set(additions);
            active.deletions = Set(deletions);
            active.changed_files = Set(changed_files);
            active.commits_count = Set(commits_count);
            active.comments_count = Set(comments_count);
            active.updated_at = Set(updated_at);
            active.merged_at = Set(merged_at);
            active.closed_at = Set(closed_at);
            active.last_synced_at = Set(Utc::now());
            return active.update(db).await;
        }

        // Create new PR
        let pr = ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(repository_id),
            provider: Set(provider),
            provider_id: Set(provider_id),
            number: Set(number),
            title: Set(title),
            description: Set(description),
            url: Set(url),
            state: Set(state),
            source_branch: Set(source_branch),
            target_branch: Set(target_branch),
            author: Set(author),
            author_avatar_url: Set(author_avatar_url),
            is_draft: Set(is_draft),
            is_mergeable: Set(is_mergeable),
            has_conflicts: Set(has_conflicts),
            additions: Set(additions),
            deletions: Set(deletions),
            changed_files: Set(changed_files),
            commits_count: Set(commits_count),
            comments_count: Set(comments_count),
            created_at: Set(created_at),
            updated_at: Set(updated_at),
            merged_at: Set(merged_at),
            closed_at: Set(closed_at),
            last_synced_at: Set(Utc::now()),
        };

        pr.insert(db).await
    }

    /// Update PR state
    pub async fn update_state(
        db: &DatabaseConnection,
        id: Uuid,
        state: String,
        merged_at: Option<chrono::DateTime<Utc>>,
        closed_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<Model, DbErr> {
        let pr = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("PR not found".to_string()))?;

        let mut active: ActiveModel = pr.into();
        active.state = Set(state);
        active.merged_at = Set(merged_at);
        active.closed_at = Set(closed_at);
        active.updated_at = Set(Utc::now());
        active.last_synced_at = Set(Utc::now());
        active.update(db).await
    }

    /// Delete PR
    pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<(), DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }

    /// Count open PRs by repository
    pub async fn count_open_by_repository(
        db: &DatabaseConnection,
        repository_id: Uuid,
    ) -> Result<u64, DbErr> {
        Entity::find()
            .filter(Column::RepositoryId.eq(repository_id))
            .filter(Column::State.eq("open"))
            .count(db)
            .await
    }
}
