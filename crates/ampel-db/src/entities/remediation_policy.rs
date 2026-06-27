use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "remediation_policy")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    // Scope
    pub scope_type: String,
    pub scope_id: Uuid,

    // Activation / selection
    pub enabled: bool,
    pub min_open_prs: i32,
    /// JSON-encoded PR selection value object; (de)serialized at the service layer.
    pub pr_selection: String,

    // Autonomy / tiering
    pub autonomy_level: String,
    pub remediation_tier: String,
    pub max_prs_per_run: i32,
    /// JSON array of allowed targets; (de)serialized at the service layer.
    pub allowed_targets: String,

    // Behavior flags
    pub skip_draft: bool,
    pub require_green_before_merge: bool,
    pub air_gapped: bool,
    pub auto_merge_enabled: bool,
    pub auto_merge_rule: Option<String>,
    pub require_human_approval: bool,

    // Optional JSON config blobs
    pub agent_budget: Option<String>,
    pub notification_config: Option<String>,
    pub playbook_ref: Option<String>,

    // Timestamps
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
