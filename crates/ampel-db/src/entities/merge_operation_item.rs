use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "merge_operation_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub merge_operation_id: Uuid,
    pub pull_request_id: Uuid,
    pub repository_id: Uuid,
    pub status: String, // pending, success, failed, skipped
    pub error_message: Option<String>,
    pub merge_sha: Option<String>,
    pub merged_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::merge_operation::Entity",
        from = "Column::MergeOperationId",
        to = "super::merge_operation::Column::Id"
    )]
    MergeOperation,
    #[sea_orm(
        belongs_to = "super::pull_request::Entity",
        from = "Column::PullRequestId",
        to = "super::pull_request::Column::Id"
    )]
    PullRequest,
}

impl Related<super::merge_operation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MergeOperation.def()
    }
}

impl Related<super::pull_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PullRequest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
