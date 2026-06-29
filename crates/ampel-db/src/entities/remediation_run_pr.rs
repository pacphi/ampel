use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "remediation_run_pr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub remediation_run_id: Uuid,
    pub pr_number: i64,
    /// JSON-encoded disposition value object; parsed at the service layer.
    pub disposition: String,

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
