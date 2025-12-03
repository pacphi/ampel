use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::git_provider::Entity")]
    GitProviders,
    #[sea_orm(has_many = "super::repository::Entity")]
    Repositories,
    #[sea_orm(has_many = "super::organization::Entity")]
    Organizations,
}

impl Related<super::git_provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GitProviders.def()
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

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for ampel_core::models::User {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            email: model.email,
            password_hash: model.password_hash,
            display_name: model.display_name,
            avatar_url: model.avatar_url,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
