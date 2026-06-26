use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "model_provider_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    // Ownership (org- or user-scoped)
    pub organization_id: Option<Uuid>,
    pub user_id: Option<Uuid>,

    pub provider_kind: String,
    pub display_name: String,

    /// AES-256-GCM encrypted credentials; encryption happens at the service layer.
    #[sea_orm(column_type = "VarBinary(StringLen::None)", nullable)]
    pub credentials_encrypted: Option<Vec<u8>>,

    pub endpoint_url: Option<String>,
    pub egress_class: String,
    pub model_id: Option<String>,
    pub enabled: bool,

    // Timestamps
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
