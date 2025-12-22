use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "provider_accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,

    // Provider identification
    pub provider: String,
    pub instance_url: Option<String>,

    // Account identification
    pub account_label: String,
    pub provider_user_id: String,
    pub provider_username: String,
    pub provider_email: Option<String>,
    pub avatar_url: Option<String>,

    // Authentication
    pub auth_type: String,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub access_token_encrypted: Vec<u8>,
    pub auth_username: Option<String>, // For Bitbucket Basic Auth

    // Token metadata
    pub scopes: Option<String>,
    pub token_expires_at: Option<DateTimeUtc>,
    pub last_validated_at: Option<DateTimeUtc>,
    pub validation_status: String,

    // Status
    pub is_active: bool,
    pub is_default: bool,

    // Timestamps
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Helper enums for type safety

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Pat,
    OAuth, // Legacy, for migration
}

impl AuthType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthType::Pat => "pat",
            AuthType::OAuth => "oauth",
        }
    }
}

impl std::fmt::Display for AuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AuthType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pat" => Ok(AuthType::Pat),
            "oauth" => Ok(AuthType::OAuth),
            _ => Err(format!("Invalid auth type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Pending,
    Valid,
    Invalid,
    Expired,
}

impl ValidationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationStatus::Pending => "pending",
            ValidationStatus::Valid => "valid",
            ValidationStatus::Invalid => "invalid",
            ValidationStatus::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ValidationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(ValidationStatus::Pending),
            "valid" => Ok(ValidationStatus::Valid),
            "invalid" => Ok(ValidationStatus::Invalid),
            "expired" => Ok(ValidationStatus::Expired),
            _ => Err(format!("Invalid validation status: {}", s)),
        }
    }
}
