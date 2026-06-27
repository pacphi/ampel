use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "remediation_agent_session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub remediation_run_id: Uuid,
    pub model_provider_account_id: Option<Uuid>,
    pub playbook_ref: Option<String>,

    // Iteration accounting
    pub iterations: i32,
    pub max_iterations: Option<i32>,
    pub tokens_used: i64,
    /// Decimal cost stored as a string for cross-DB safety; parsed at the service layer.
    pub cost_usd: Option<String>,

    pub status: String,
    pub transcript_ref: Option<String>,

    // Timestamps
    pub started_at: DateTimeUtc,
    pub completed_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::remediation_run::Entity",
        from = "Column::RemediationRunId",
        to = "super::remediation_run::Column::Id",
        on_delete = "Cascade"
    )]
    RemediationRun,
}

impl Related<super::remediation_run::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RemediationRun.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
