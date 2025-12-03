use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ci_checks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub pull_request_id: Uuid,
    pub name: String,
    pub status: String,             // queued, in_progress, completed
    pub conclusion: Option<String>, // success, failure, neutral, cancelled, skipped, timed_out, action_required
    pub url: Option<String>,
    pub started_at: Option<DateTimeUtc>,
    pub completed_at: Option<DateTimeUtc>,
    pub duration_seconds: Option<i32>,
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

impl From<Model> for ampel_core::models::CICheck {
    fn from(model: Model) -> Self {
        let status = match model.status.as_str() {
            "queued" => ampel_core::models::CICheckStatus::Queued,
            "in_progress" => ampel_core::models::CICheckStatus::InProgress,
            "completed" => ampel_core::models::CICheckStatus::Completed,
            _ => ampel_core::models::CICheckStatus::Queued,
        };

        let conclusion = model.conclusion.as_ref().map(|c| match c.as_str() {
            "success" => ampel_core::models::CICheckConclusion::Success,
            "failure" => ampel_core::models::CICheckConclusion::Failure,
            "neutral" => ampel_core::models::CICheckConclusion::Neutral,
            "cancelled" => ampel_core::models::CICheckConclusion::Cancelled,
            "skipped" => ampel_core::models::CICheckConclusion::Skipped,
            "timed_out" => ampel_core::models::CICheckConclusion::TimedOut,
            "action_required" => ampel_core::models::CICheckConclusion::ActionRequired,
            _ => ampel_core::models::CICheckConclusion::Neutral,
        });

        Self {
            id: model.id,
            pull_request_id: model.pull_request_id,
            name: model.name,
            status,
            conclusion,
            url: model.url,
            started_at: model.started_at,
            completed_at: model.completed_at,
            duration_seconds: model.duration_seconds,
        }
    }
}
