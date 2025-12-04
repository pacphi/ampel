use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use uuid::Uuid;

use crate::entities::merge_operation::{ActiveModel, Column, Entity, Model};
use crate::entities::merge_operation_item;

pub struct MergeOperationQueries;

impl MergeOperationQueries {
    /// Find merge operation by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Find merge operation by ID and user (for authorization)
    pub async fn find_by_id_and_user(
        db: &DatabaseConnection,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id)
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
    }

    /// List merge operations for a user, ordered by started_at desc
    pub async fn find_by_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        limit: u64,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::StartedAt)
            .limit(limit)
            .all(db)
            .await
    }

    /// Create a new merge operation
    pub async fn create(
        db: &DatabaseConnection,
        user_id: Uuid,
        total_count: i32,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();
        let operation = ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            started_at: Set(now),
            completed_at: Set(None),
            total_count: Set(total_count),
            success_count: Set(0),
            failed_count: Set(0),
            skipped_count: Set(0),
            status: Set("in_progress".to_string()),
            notification_sent: Set(false),
        };
        operation.insert(db).await
    }

    /// Update operation counts and status
    pub async fn update_counts(
        db: &DatabaseConnection,
        id: Uuid,
        success_count: i32,
        failed_count: i32,
        skipped_count: i32,
        status: &str,
    ) -> Result<Model, DbErr> {
        let operation = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Merge operation not found".to_string(),
            ))?;

        let mut active: ActiveModel = operation.into();
        active.success_count = Set(success_count);
        active.failed_count = Set(failed_count);
        active.skipped_count = Set(skipped_count);
        active.status = Set(status.to_string());
        if status == "completed" || status == "failed" {
            active.completed_at = Set(Some(Utc::now()));
        }
        active.update(db).await
    }

    /// Mark notification as sent
    pub async fn mark_notification_sent(db: &DatabaseConnection, id: Uuid) -> Result<(), DbErr> {
        let operation = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Merge operation not found".to_string(),
            ))?;

        let mut active: ActiveModel = operation.into();
        active.notification_sent = Set(true);
        active.update(db).await?;
        Ok(())
    }
}

pub struct MergeOperationItemQueries;

impl MergeOperationItemQueries {
    /// Find items by operation ID
    pub async fn find_by_operation(
        db: &DatabaseConnection,
        operation_id: Uuid,
    ) -> Result<Vec<merge_operation_item::Model>, DbErr> {
        merge_operation_item::Entity::find()
            .filter(merge_operation_item::Column::MergeOperationId.eq(operation_id))
            .all(db)
            .await
    }

    /// Create a new merge operation item
    pub async fn create(
        db: &DatabaseConnection,
        operation_id: Uuid,
        pull_request_id: Uuid,
        repository_id: Uuid,
    ) -> Result<merge_operation_item::Model, DbErr> {
        let item = merge_operation_item::ActiveModel {
            id: Set(Uuid::new_v4()),
            merge_operation_id: Set(operation_id),
            pull_request_id: Set(pull_request_id),
            repository_id: Set(repository_id),
            status: Set("pending".to_string()),
            error_message: Set(None),
            merge_sha: Set(None),
            merged_at: Set(None),
        };
        item.insert(db).await
    }

    /// Update item status after merge attempt
    pub async fn update_status(
        db: &DatabaseConnection,
        id: Uuid,
        status: &str,
        error_message: Option<String>,
        merge_sha: Option<String>,
    ) -> Result<merge_operation_item::Model, DbErr> {
        let item = merge_operation_item::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Merge operation item not found".to_string(),
            ))?;

        let mut active: merge_operation_item::ActiveModel = item.into();
        active.status = Set(status.to_string());
        active.error_message = Set(error_message);
        active.merge_sha = Set(merge_sha);
        if status == "success" {
            active.merged_at = Set(Some(Utc::now()));
        }
        active.update(db).await
    }
}
