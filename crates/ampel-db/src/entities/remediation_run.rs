use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "remediation_run")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub repository_id: Uuid,
    pub policy_id: Uuid,

    // Trigger metadata
    pub triggered_by: String,
    pub triggered_by_user_id: Option<Uuid>,

    // State machine
    pub state: String,
    /// Snapshot of the granted autonomy level for this run (snake_case string).
    /// Stored on the run so the orchestrator's autonomy gate does not need to
    /// re-resolve the policy mid-flight. Added in the Phase-2 columns migration.
    pub autonomy_level: String,
    /// The verified consolidated-ref HEAD SHA captured at `verify` time — the
    /// TOCTOU anchor re-checked immediately before merge. Added in Phase 2.
    pub head_sha: Option<String>,
    /// JSON snapshot of the selected PRs at run time; parsed at the service layer.
    pub pr_selection_snapshot: String,
    /// JSON consolidation plan; parsed at the service layer.
    pub consolidation_plan: Option<String>,
    pub consolidated_pr_number: Option<i64>,
    pub merged: bool,

    // Branch / CI
    pub branch_name: String,
    pub ci_status: String,
    pub ci_logs_url: Option<String>,
    pub merge_strategy: Option<String>,

    // Execution bookkeeping
    pub attempts: i32,
    pub error_message: Option<String>,
    pub error_class: Option<String>,

    // Timestamps
    pub started_at: DateTimeUtc,
    pub completed_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::remediation_run_pr::Entity")]
    RemediationRunPr,
    #[sea_orm(has_many = "super::remediation_agent_session::Entity")]
    RemediationAgentSession,
}

impl Related<super::remediation_run_pr::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RemediationRunPr.def()
    }
}

impl Related<super::remediation_agent_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RemediationAgentSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
