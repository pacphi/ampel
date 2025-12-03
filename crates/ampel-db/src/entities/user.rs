use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// User account - authenticates via OAuth (GitHub/Google), no password
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::provider_connection::Entity")]
    ProviderConnections,
    #[sea_orm(has_many = "super::repository::Entity")]
    Repositories,
    #[sea_orm(has_many = "super::organization::Entity")]
    Organizations,
    #[sea_orm(has_many = "super::user_oauth_account::Entity")]
    OauthAccounts,
}

impl Related<super::provider_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProviderConnections.def()
    }
}

impl Related<super::repository::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repositories.def()
    }
}

impl Related<super::organization::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organizations.def()
    }
}

impl Related<super::user_oauth_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OauthAccounts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for ampel_core::models::User {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            email: model.email,
            display_name: model.display_name,
            avatar_url: model.avatar_url,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
