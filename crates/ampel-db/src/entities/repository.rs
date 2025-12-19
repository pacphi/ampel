use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "repositories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,    // github, gitlab, bitbucket
    pub provider_id: String, // ID from the provider
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub url: String,
    pub default_branch: String,
    pub is_private: bool,
    pub is_archived: bool,
    pub poll_interval_seconds: i32,
    pub last_polled_at: Option<DateTimeUtc>,
    pub group_id: Option<Uuid>,
    pub provider_account_id: Option<Uuid>, // Link to provider_account for multi-account support
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
    #[sea_orm(has_many = "super::pull_request::Entity")]
    PullRequests,
    #[sea_orm(
        belongs_to = "super::provider_account::Entity",
        from = "Column::ProviderAccountId",
        to = "super::provider_account::Column::Id"
    )]
    ProviderAccount,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::pull_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PullRequests.def()
    }
}

impl Related<super::provider_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProviderAccount.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for ampel_core::models::Repository {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            provider: model
                .provider
                .parse()
                .unwrap_or(ampel_core::models::GitProvider::GitHub),
            provider_id: model.provider_id,
            owner: model.owner,
            name: model.name,
            full_name: model.full_name,
            description: model.description,
            url: model.url,
            default_branch: model.default_branch,
            is_private: model.is_private,
            is_archived: model.is_archived,
            poll_interval_seconds: model.poll_interval_seconds,
            last_polled_at: model.last_polled_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
