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

    // Phase 4 (ADR-008): auth + validation + spend accounting.
    /// How credentials authenticate (`api_key`, `none` for local providers).
    pub auth_type: String,
    /// Optional spend ceiling in USD; Decimal-as-string for cross-DB exactness.
    pub spend_cap_usd: Option<String>,
    /// Cumulative spend in USD; Decimal-as-string, defaults to "0".
    pub spend_used_usd: String,
    /// `unvalidated` | `valid` | `invalid` (set after a provider ping).
    pub validation_status: String,
    pub last_validated_at: Option<DateTimeUtc>,
    /// On-disk model path (ONNX local providers).
    pub model_path: Option<String>,
    /// Whether this is the default account for its scope.
    pub is_default: bool,

    // Timestamps
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
