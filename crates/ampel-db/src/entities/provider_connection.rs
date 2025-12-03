use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// PAT-based connections to Git providers (GitHub, GitLab, Bitbucket)
/// Users can have multiple connections per provider with different names
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "provider_connections")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,        // github, gitlab, bitbucket
    pub connection_name: String, // User-defined name: "work-github", "personal-gitlab"
    pub provider_user_id: String,
    pub provider_username: String,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub access_token_encrypted: Vec<u8>,
    pub scopes: Option<String>,     // JSON array stored as string (optional for PATs)
    pub base_url: Option<String>,   // For self-hosted: https://github.mycompany.com
    pub is_validated: bool,         // Whether the PAT has been validated
    pub validation_error: Option<String>, // Last validation error if any
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
    #[sea_orm(has_many = "super::repository::Entity")]
    Repositories,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::repository::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repositories.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
