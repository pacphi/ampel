use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::AmpelStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum GitProvider {
    GitHub,
    GitLab,
    Bitbucket,
}

impl std::fmt::Display for GitProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitProvider::GitHub => write!(f, "github"),
            GitProvider::GitLab => write!(f, "gitlab"),
            GitProvider::Bitbucket => write!(f, "bitbucket"),
        }
    }
}

impl std::str::FromStr for GitProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(GitProvider::GitHub),
            "gitlab" => Ok(GitProvider::GitLab),
            "bitbucket" => Ok(GitProvider::Bitbucket),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connection_id: Option<Uuid>, // Which PAT connection is used to access this repo
    pub provider: GitProvider,
    pub provider_id: String,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub url: String,
    pub default_branch: String,
    pub is_private: bool,
    pub is_archived: bool,
    pub poll_interval_seconds: i32,
    pub last_polled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryWithStatus {
    #[serde(flatten)]
    pub repository: Repository,
    pub status: AmpelStatus,
    pub open_pr_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddRepositoryRequest {
    pub provider: GitProvider,
    #[validate(length(min = 1, message = "Owner is required"))]
    pub owner: String,
    #[validate(length(min = 1, message = "Repository name is required"))]
    pub name: String,
    #[validate(range(
        min = 60,
        max = 3600,
        message = "Poll interval must be 60-3600 seconds"
    ))]
    pub poll_interval_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryGroup {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// PAT-based connection to a Git provider
/// Users can have multiple connections per provider with different names
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: GitProvider,
    pub connection_name: String, // User-defined name: "work-github", "personal-gitlab"
    pub provider_user_id: String,
    pub provider_username: String,
    pub access_token_encrypted: Vec<u8>,
    pub scopes: Option<Vec<String>>,
    pub base_url: Option<String>,        // For self-hosted: https://github.mycompany.com
    pub is_validated: bool,
    pub validation_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConnectionResponse {
    pub id: Uuid,
    pub provider: GitProvider,
    pub connection_name: String,
    pub provider_username: String,
    pub base_url: Option<String>,
    pub is_validated: bool,
    pub validation_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<ProviderConnection> for ProviderConnectionResponse {
    fn from(conn: ProviderConnection) -> Self {
        Self {
            id: conn.id,
            provider: conn.provider,
            connection_name: conn.connection_name,
            provider_username: conn.provider_username,
            base_url: conn.base_url,
            is_validated: conn.is_validated,
            validation_error: conn.validation_error,
            created_at: conn.created_at,
        }
    }
}

/// Request to add a new PAT connection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddConnectionRequest {
    pub provider: GitProvider,
    #[validate(length(min = 1, max = 100, message = "Connection name must be 1-100 characters"))]
    pub connection_name: String,
    #[validate(length(min = 1, message = "Access token is required"))]
    pub access_token: String,
    pub base_url: Option<String>, // For self-hosted instances
}

/// Request to update an existing connection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateConnectionRequest {
    #[validate(length(min = 1, max = 100, message = "Connection name must be 1-100 characters"))]
    pub connection_name: Option<String>,
    pub access_token: Option<String>, // For rotating the PAT
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredRepository {
    pub provider: GitProvider,
    pub provider_id: String,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub url: String,
    pub default_branch: String,
    pub is_private: bool,
    pub is_archived: bool,
}
