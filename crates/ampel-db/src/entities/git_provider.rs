use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "git_providers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String, // github, gitlab, bitbucket
    pub provider_user_id: String,
    pub provider_username: String,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub access_token_encrypted: Vec<u8>,
    #[sea_orm(column_type = "VarBinary(StringLen::None)", nullable)]
    pub refresh_token_encrypted: Option<Vec<u8>>,
    pub token_expires_at: Option<DateTimeUtc>,
    pub scopes: String,               // JSON array stored as string
    pub instance_url: Option<String>, // For self-hosted GitLab/Bitbucket
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
