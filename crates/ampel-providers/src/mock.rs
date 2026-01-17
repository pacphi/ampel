//! Mock Git provider for testing
//!
//! This module provides a mock implementation of the GitProvider trait that allows
//! configuring responses for each method. It's useful for testing code that depends
//! on Git providers without making actual API calls.
//!
//! # Examples
//!
//! ```
//! use ampel_providers::mock::MockProvider;
//! use ampel_providers::traits::{GitProvider, ProviderCredentials, TokenValidation};
//! use ampel_core::models::GitProvider as Provider;
//!
//! #[tokio::test]
//! async fn test_with_mock_provider() {
//!     let credentials = ProviderCredentials::Pat {
//!         token: "test_token".to_string(),
//!         username: None,
//!     };
//!
//!     let mock = MockProvider::new()
//!         .with_validation_result(TokenValidation {
//!             is_valid: true,
//!             user_id: Some("123".to_string()),
//!             username: Some("testuser".to_string()),
//!             ..Default::default()
//!         });
//!
//!     let result = mock.validate_credentials(&credentials).await.unwrap();
//!     assert!(result.is_valid);
//!     assert_eq!(result.username, Some("testuser".to_string()));
//! }
//! ```

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{ProviderError, ProviderResult};
use crate::traits::{
    GitProvider, MergeResult, ProviderCICheck, ProviderCredentials, ProviderDiff,
    ProviderPullRequest, ProviderReview, ProviderUser, RateLimitInfo, TokenValidation,
};
use ampel_core::models::{DiscoveredRepository, GitProvider as Provider, MergeRequest};

/// Internal state for mock provider
#[derive(Debug, Clone, Default)]
struct MockState {
    validation_result: Option<TokenValidation>,
    user: Option<ProviderUser>,
    repositories: Vec<DiscoveredRepository>,
    pull_requests: HashMap<String, Vec<ProviderPullRequest>>,
    ci_checks: HashMap<String, Vec<ProviderCICheck>>,
    reviews: HashMap<String, Vec<ProviderReview>>,
    merge_results: HashMap<String, Result<MergeResult, ProviderError>>,
    rate_limit: Option<RateLimitInfo>,
    should_fail_validation: bool,
    should_fail_user: bool,
    should_fail_repositories: bool,
    should_fail_pull_requests: bool,
}

/// Mock Git provider for testing
///
/// Allows configuring responses for each trait method. All methods return
/// configured values or sensible defaults.
#[derive(Clone)]
pub struct MockProvider {
    provider_type: Provider,
    instance_url: Option<String>,
    state: Arc<Mutex<MockState>>,
}

impl MockProvider {
    /// Create a new mock provider with default GitHub provider type
    pub fn new() -> Self {
        Self {
            provider_type: Provider::GitHub,
            instance_url: None,
            state: Arc::new(Mutex::new(MockState::default())),
        }
    }

    /// Create a mock provider for a specific provider type
    pub fn new_with_provider(provider_type: Provider) -> Self {
        Self {
            provider_type,
            instance_url: None,
            state: Arc::new(Mutex::new(MockState::default())),
        }
    }

    /// Set the instance URL for the provider
    pub fn with_instance_url(mut self, url: String) -> Self {
        self.instance_url = Some(url);
        self
    }

    /// Configure the validation result to return
    pub fn with_validation_result(self, result: TokenValidation) -> Self {
        self.state.lock().unwrap().validation_result = Some(result);
        self
    }

    /// Configure validation to fail
    pub fn with_validation_failure(self) -> Self {
        self.state.lock().unwrap().should_fail_validation = true;
        self
    }

    /// Configure the user to return
    pub fn with_user(self, user: ProviderUser) -> Self {
        self.state.lock().unwrap().user = Some(user);
        self
    }

    /// Configure get_user to fail
    pub fn with_user_failure(self) -> Self {
        self.state.lock().unwrap().should_fail_user = true;
        self
    }

    /// Configure the list of repositories to return
    pub fn with_repositories(self, repos: Vec<DiscoveredRepository>) -> Self {
        self.state.lock().unwrap().repositories = repos;
        self
    }

    /// Configure list_repositories to fail
    pub fn with_repositories_failure(self) -> Self {
        self.state.lock().unwrap().should_fail_repositories = true;
        self
    }

    /// Configure pull requests for a specific repository
    pub fn with_pull_requests(
        self,
        owner: &str,
        repo: &str,
        prs: Vec<ProviderPullRequest>,
    ) -> Self {
        let key = format!("{}/{}", owner, repo);
        self.state.lock().unwrap().pull_requests.insert(key, prs);
        self
    }

    /// Configure list_pull_requests to fail
    pub fn with_pull_requests_failure(self) -> Self {
        self.state.lock().unwrap().should_fail_pull_requests = true;
        self
    }

    /// Configure CI checks for a specific PR
    pub fn with_ci_checks(
        self,
        owner: &str,
        repo: &str,
        pr_number: i32,
        checks: Vec<ProviderCICheck>,
    ) -> Self {
        let key = format!("{}/{}/{}", owner, repo, pr_number);
        self.state.lock().unwrap().ci_checks.insert(key, checks);
        self
    }

    /// Configure reviews for a specific PR
    pub fn with_reviews(
        self,
        owner: &str,
        repo: &str,
        pr_number: i32,
        reviews: Vec<ProviderReview>,
    ) -> Self {
        let key = format!("{}/{}/{}", owner, repo, pr_number);
        self.state.lock().unwrap().reviews.insert(key, reviews);
        self
    }

    /// Configure merge result for a specific PR
    pub fn with_merge_result(
        self,
        owner: &str,
        repo: &str,
        pr_number: i32,
        result: MergeResult,
    ) -> Self {
        let key = format!("{}/{}/{}", owner, repo, pr_number);
        self.state
            .lock()
            .unwrap()
            .merge_results
            .insert(key, Ok(result));
        self
    }

    /// Configure merge to fail with specific error
    pub fn with_merge_failure(
        self,
        owner: &str,
        repo: &str,
        pr_number: i32,
        error: ProviderError,
    ) -> Self {
        let key = format!("{}/{}/{}", owner, repo, pr_number);
        self.state
            .lock()
            .unwrap()
            .merge_results
            .insert(key, Err(error));
        self
    }

    /// Configure rate limit info
    pub fn with_rate_limit(self, rate_limit: RateLimitInfo) -> Self {
        self.state.lock().unwrap().rate_limit = Some(rate_limit);
        self
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GitProvider for MockProvider {
    fn provider_type(&self) -> Provider {
        self.provider_type
    }

    fn instance_url(&self) -> Option<&str> {
        self.instance_url.as_deref()
    }

    async fn validate_credentials(
        &self,
        _credentials: &ProviderCredentials,
    ) -> ProviderResult<TokenValidation> {
        let state = self.state.lock().unwrap();

        if state.should_fail_validation {
            return Err(ProviderError::AuthenticationFailed(
                "Mock validation failure".to_string(),
            ));
        }

        Ok(state
            .validation_result
            .clone()
            .unwrap_or_else(|| TokenValidation {
                is_valid: true,
                user_id: Some("mock_user_id".to_string()),
                username: Some("mockuser".to_string()),
                email: Some("mock@example.com".to_string()),
                avatar_url: Some("https://example.com/avatar.png".to_string()),
                scopes: vec!["repo".to_string(), "read:user".to_string()],
                expires_at: None,
                error_message: None,
            }))
    }

    async fn get_user(&self, _credentials: &ProviderCredentials) -> ProviderResult<ProviderUser> {
        let state = self.state.lock().unwrap();

        if state.should_fail_user {
            return Err(ProviderError::AuthenticationFailed(
                "Mock user failure".to_string(),
            ));
        }

        Ok(state.user.clone().unwrap_or_else(|| ProviderUser {
            id: "mock_user_id".to_string(),
            username: "mockuser".to_string(),
            email: Some("mock@example.com".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
        }))
    }

    async fn list_repositories(
        &self,
        _credentials: &ProviderCredentials,
        _page: i32,
        _per_page: i32,
    ) -> ProviderResult<Vec<DiscoveredRepository>> {
        let state = self.state.lock().unwrap();

        if state.should_fail_repositories {
            return Err(ProviderError::ApiError {
                status_code: 500,
                message: "Mock repository failure".to_string(),
            });
        }

        Ok(state.repositories.clone())
    }

    async fn get_repository(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
    ) -> ProviderResult<DiscoveredRepository> {
        let repos = self.list_repositories(credentials, 1, 100).await?;

        repos
            .into_iter()
            .find(|r| r.owner == owner && r.name == repo)
            .ok_or_else(|| {
                ProviderError::NotFound(format!("Repository {}/{} not found", owner, repo))
            })
    }

    async fn list_pull_requests(
        &self,
        _credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        _state: Option<&str>,
    ) -> ProviderResult<Vec<ProviderPullRequest>> {
        let state = self.state.lock().unwrap();

        if state.should_fail_pull_requests {
            return Err(ProviderError::ApiError {
                status_code: 500,
                message: "Mock pull request failure".to_string(),
            });
        }

        let key = format!("{}/{}", owner, repo);
        Ok(state.pull_requests.get(&key).cloned().unwrap_or_default())
    }

    async fn get_pull_request(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        number: i32,
    ) -> ProviderResult<ProviderPullRequest> {
        let prs = self
            .list_pull_requests(credentials, owner, repo, None)
            .await?;

        prs.into_iter()
            .find(|pr| pr.number == number)
            .ok_or_else(|| ProviderError::NotFound(format!("Pull request #{} not found", number)))
    }

    async fn get_ci_checks(
        &self,
        _credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderCICheck>> {
        let state = self.state.lock().unwrap();
        let key = format!("{}/{}/{}", owner, repo, pr_number);
        Ok(state.ci_checks.get(&key).cloned().unwrap_or_default())
    }

    async fn get_reviews(
        &self,
        _credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderReview>> {
        let state = self.state.lock().unwrap();
        let key = format!("{}/{}/{}", owner, repo, pr_number);
        Ok(state.reviews.get(&key).cloned().unwrap_or_default())
    }

    async fn merge_pull_request(
        &self,
        _credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
        _merge_request: &MergeRequest,
    ) -> ProviderResult<MergeResult> {
        let state = self.state.lock().unwrap();
        let key = format!("{}/{}/{}", owner, repo, pr_number);

        match state.merge_results.get(&key) {
            Some(Ok(result)) => Ok(result.clone()),
            Some(Err(err)) => Err(ProviderError::ApiError {
                status_code: 409,
                message: err.to_string(),
            }),
            None => Ok(MergeResult {
                merged: true,
                sha: Some("mock_sha_12345".to_string()),
                message: "Pull request successfully merged".to_string(),
            }),
        }
    }

    async fn get_rate_limit(
        &self,
        _credentials: &ProviderCredentials,
    ) -> ProviderResult<RateLimitInfo> {
        let state = self.state.lock().unwrap();

        Ok(state.rate_limit.clone().unwrap_or_else(|| RateLimitInfo {
            limit: 5000,
            remaining: 4999,
            reset_at: Utc::now() + chrono::Duration::hours(1),
        }))
    }

    async fn get_pull_request_diff(
        &self,
        _credentials: &ProviderCredentials,
        _owner: &str,
        _repo: &str,
        _pr_number: i32,
    ) -> ProviderResult<ProviderDiff> {
        // Mock implementation returns empty diff
        Ok(ProviderDiff {
            files: vec![],
            total_additions: 0,
            total_deletions: 0,
            total_changes: 0,
            total_files: 0,
            base_commit: "mock_base_commit".to_string(),
            head_commit: "mock_head_commit".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ampel_core::models::MergeStrategy;

    #[tokio::test]
    async fn test_mock_default_values() {
        let mock = MockProvider::new();
        let credentials = ProviderCredentials::Pat {
            token: "test".to_string(),
            username: None,
        };

        let validation = mock.validate_credentials(&credentials).await.unwrap();
        assert!(validation.is_valid);

        let user = mock.get_user(&credentials).await.unwrap();
        assert_eq!(user.username, "mockuser");

        let repos = mock.list_repositories(&credentials, 1, 10).await.unwrap();
        assert_eq!(repos.len(), 0);

        let rate_limit = mock.get_rate_limit(&credentials).await.unwrap();
        assert_eq!(rate_limit.limit, 5000);
    }

    #[tokio::test]
    async fn test_mock_configured_values() {
        let mock = MockProvider::new()
            .with_validation_result(TokenValidation {
                is_valid: true,
                username: Some("custom_user".to_string()),
                ..Default::default()
            })
            .with_user(ProviderUser {
                id: "123".to_string(),
                username: "custom_user".to_string(),
                email: None,
                avatar_url: None,
            });

        let credentials = ProviderCredentials::Pat {
            token: "test".to_string(),
            username: None,
        };

        let validation = mock.validate_credentials(&credentials).await.unwrap();
        assert_eq!(validation.username, Some("custom_user".to_string()));

        let user = mock.get_user(&credentials).await.unwrap();
        assert_eq!(user.username, "custom_user");
    }

    #[tokio::test]
    async fn test_mock_provider_type() {
        let github_mock = MockProvider::new();
        assert_eq!(github_mock.provider_type(), Provider::GitHub);

        let gitlab_mock = MockProvider::new_with_provider(Provider::GitLab);
        assert_eq!(gitlab_mock.provider_type(), Provider::GitLab);
    }

    #[tokio::test]
    async fn test_mock_instance_url() {
        let mock = MockProvider::new().with_instance_url("https://gitlab.company.com".to_string());
        assert_eq!(mock.instance_url(), Some("https://gitlab.company.com"));
    }

    #[tokio::test]
    async fn test_mock_merge_default() {
        let mock = MockProvider::new();
        let credentials = ProviderCredentials::Pat {
            token: "test".to_string(),
            username: None,
        };

        let merge_request = MergeRequest {
            strategy: MergeStrategy::Merge,
            commit_title: None,
            commit_message: None,
            delete_branch: false,
        };

        let result = mock
            .merge_pull_request(&credentials, "owner", "repo", 1, &merge_request)
            .await
            .unwrap();

        assert!(result.merged);
        assert!(result.sha.is_some());
    }
}
