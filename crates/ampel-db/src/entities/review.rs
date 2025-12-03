use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "reviews")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub pull_request_id: Uuid,
    pub reviewer: String,
    pub reviewer_avatar_url: Option<String>,
    pub state: String, // approved, changes_requested, commented, pending, dismissed
    pub body: Option<String>,
    pub submitted_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pull_request::Entity",
        from = "Column::PullRequestId",
        to = "super::pull_request::Column::Id"
    )]
    PullRequest,
}

impl Related<super::pull_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PullRequest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for ampel_core::models::Review {
    fn from(model: Model) -> Self {
        let state = match model.state.as_str() {
            "approved" => ampel_core::models::ReviewState::Approved,
            "changes_requested" => ampel_core::models::ReviewState::ChangesRequested,
            "commented" => ampel_core::models::ReviewState::Commented,
            "pending" => ampel_core::models::ReviewState::Pending,
            "dismissed" => ampel_core::models::ReviewState::Dismissed,
            _ => ampel_core::models::ReviewState::Commented,
        };

        Self {
            id: model.id,
            pull_request_id: model.pull_request_id,
            reviewer: model.reviewer,
            reviewer_avatar_url: model.reviewer_avatar_url,
            state,
            body: model.body,
            submitted_at: model.submitted_at,
        }
    }
}
