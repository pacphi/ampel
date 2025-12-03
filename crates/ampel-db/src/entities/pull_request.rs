use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "pull_requests")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub repository_id: Uuid,
    pub provider: String,
    pub provider_id: String,
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub state: String, // open, closed, merged
    pub source_branch: String,
    pub target_branch: String,
    pub author: String,
    pub author_avatar_url: Option<String>,
    pub is_draft: bool,
    pub is_mergeable: Option<bool>,
    pub has_conflicts: bool,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub commits_count: i32,
    pub comments_count: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub merged_at: Option<DateTimeUtc>,
    pub closed_at: Option<DateTimeUtc>,
    pub last_synced_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::repository::Entity",
        from = "Column::RepositoryId",
        to = "super::repository::Column::Id"
    )]
    Repository,
    #[sea_orm(has_many = "super::ci_check::Entity")]
    CIChecks,
    #[sea_orm(has_many = "super::review::Entity")]
    Reviews,
}

impl Related<super::repository::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repository.def()
    }
}

impl Related<super::ci_check::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CIChecks.def()
    }
}

impl Related<super::review::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Reviews.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for ampel_core::models::PullRequest {
    fn from(model: Model) -> Self {
        let state = match model.state.as_str() {
            "open" => ampel_core::models::PullRequestState::Open,
            "closed" => ampel_core::models::PullRequestState::Closed,
            "merged" => ampel_core::models::PullRequestState::Merged,
            _ => ampel_core::models::PullRequestState::Open,
        };

        Self {
            id: model.id,
            repository_id: model.repository_id,
            provider: model
                .provider
                .parse()
                .unwrap_or(ampel_core::models::GitProvider::GitHub),
            provider_id: model.provider_id,
            number: model.number,
            title: model.title,
            description: model.description,
            url: model.url,
            state,
            source_branch: model.source_branch,
            target_branch: model.target_branch,
            author: model.author,
            author_avatar_url: model.author_avatar_url,
            is_draft: model.is_draft,
            is_mergeable: model.is_mergeable,
            has_conflicts: model.has_conflicts,
            additions: model.additions,
            deletions: model.deletions,
            changed_files: model.changed_files,
            commits_count: model.commits_count,
            comments_count: model.comments_count,
            created_at: model.created_at,
            updated_at: model.updated_at,
            merged_at: model.merged_at,
            closed_at: model.closed_at,
            last_synced_at: model.last_synced_at,
        }
    }
}
