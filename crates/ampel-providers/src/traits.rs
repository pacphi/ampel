use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::ProviderResult;
use ampel_core::models::{DiscoveredRepository, GitProvider as Provider, MergeRequest};

/// Rate limit information from a provider
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: i32,
    pub remaining: i32,
    pub reset_at: DateTime<Utc>,
}

/// OAuth token information
#[derive(Debug, Clone)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

/// User info from OAuth provider
#[derive(Debug, Clone)]
pub struct ProviderUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// PR data from provider (before conversion to our model)
#[derive(Debug, Clone)]
pub struct ProviderPullRequest {
    pub provider_id: String,
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub state: String,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// CI check data from provider
#[derive(Debug, Clone)]
pub struct ProviderCICheck {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub url: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Review data from provider
#[derive(Debug, Clone)]
pub struct ProviderReview {
    pub id: String,
    pub reviewer: String,
    pub reviewer_avatar_url: Option<String>,
    pub state: String,
    pub body: Option<String>,
    pub submitted_at: DateTime<Utc>,
}

/// Merge result from provider
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub merged: bool,
    pub sha: Option<String>,
    pub message: String,
}

/// Unified interface for Git providers (GitHub, GitLab, Bitbucket)
#[async_trait]
pub trait GitProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> Provider;

    /// Get OAuth authorization URL
    fn get_oauth_url(&self, state: &str) -> String;

    /// Exchange OAuth code for tokens
    async fn exchange_code(&self, code: &str) -> ProviderResult<OAuthToken>;

    /// Refresh OAuth token
    async fn refresh_token(&self, refresh_token: &str) -> ProviderResult<OAuthToken>;

    /// Get authenticated user info
    async fn get_user(&self, access_token: &str) -> ProviderResult<ProviderUser>;

    /// List repositories accessible to the user
    async fn list_repositories(
        &self,
        access_token: &str,
        page: i32,
        per_page: i32,
    ) -> ProviderResult<Vec<DiscoveredRepository>>;

    /// Get a specific repository
    async fn get_repository(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
    ) -> ProviderResult<DiscoveredRepository>;

    /// List pull requests for a repository
    async fn list_pull_requests(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> ProviderResult<Vec<ProviderPullRequest>>;

    /// Get a specific pull request
    async fn get_pull_request(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        number: i32,
    ) -> ProviderResult<ProviderPullRequest>;

    /// Get CI checks for a pull request
    async fn get_ci_checks(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderCICheck>>;

    /// Get reviews for a pull request
    async fn get_reviews(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderReview>>;

    /// Merge a pull request
    async fn merge_pull_request(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        pr_number: i32,
        merge_request: &MergeRequest,
    ) -> ProviderResult<MergeResult>;

    /// Get current rate limit status
    async fn get_rate_limit(&self, access_token: &str) -> ProviderResult<RateLimitInfo>;
}
