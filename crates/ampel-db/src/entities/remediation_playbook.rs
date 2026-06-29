use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "remediation_playbook")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Stable slug identifier; unique together with `version`.
    pub playbook_id: String,
    pub version: i32,
    pub source: String,
    pub name: String,
    pub description: Option<String>,
    /// YAML playbook content; parsed at the service layer.
    pub content: String,
    pub enabled: bool,

    /// Ownership scope: `org` | `team` | `user` | `repository`. Authorization is
    /// gated on the caller's access to `(scope_type, scope_id)`.
    pub scope_type: String,
    /// Owning scope UUID. `None` marks a built-in/global sentinel playbook
    /// (readable by any authenticated caller, mutable by none).
    pub scope_id: Option<Uuid>,

    // Timestamps
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
