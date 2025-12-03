use std::sync::Arc;

use ampel_core::models::GitProvider as Provider;

use crate::{BitbucketProvider, GitHubProvider, GitLabProvider, GitProvider};

/// Factory for creating Git provider instances
pub struct ProviderFactory {
    github_client_id: String,
    github_client_secret: String,
    github_redirect_uri: String,
    gitlab_client_id: String,
    gitlab_client_secret: String,
    gitlab_redirect_uri: String,
    gitlab_base_url: Option<String>,
    bitbucket_client_id: String,
    bitbucket_client_secret: String,
    bitbucket_redirect_uri: String,
}

impl ProviderFactory {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        github_client_id: String,
        github_client_secret: String,
        github_redirect_uri: String,
        gitlab_client_id: String,
        gitlab_client_secret: String,
        gitlab_redirect_uri: String,
        gitlab_base_url: Option<String>,
        bitbucket_client_id: String,
        bitbucket_client_secret: String,
        bitbucket_redirect_uri: String,
    ) -> Self {
        Self {
            github_client_id,
            github_client_secret,
            github_redirect_uri,
            gitlab_client_id,
            gitlab_client_secret,
            gitlab_redirect_uri,
            gitlab_base_url,
            bitbucket_client_id,
            bitbucket_client_secret,
            bitbucket_redirect_uri,
        }
    }

    /// Create a provider instance for the given provider type
    pub fn create(&self, provider: Provider) -> Arc<dyn GitProvider> {
        match provider {
            Provider::GitHub => Arc::new(GitHubProvider::new(
                self.github_client_id.clone(),
                self.github_client_secret.clone(),
                self.github_redirect_uri.clone(),
            )),
            Provider::GitLab => Arc::new(GitLabProvider::new(
                self.gitlab_client_id.clone(),
                self.gitlab_client_secret.clone(),
                self.gitlab_redirect_uri.clone(),
                self.gitlab_base_url.clone(),
            )),
            Provider::Bitbucket => Arc::new(BitbucketProvider::new(
                self.bitbucket_client_id.clone(),
                self.bitbucket_client_secret.clone(),
                self.bitbucket_redirect_uri.clone(),
            )),
        }
    }

    /// Create all provider instances
    pub fn create_all(&self) -> Vec<Arc<dyn GitProvider>> {
        vec![
            self.create(Provider::GitHub),
            self.create(Provider::GitLab),
            self.create(Provider::Bitbucket),
        ]
    }

    /// Create a provider instance with a custom base URL (for self-hosted instances or PAT auth)
    /// OAuth credentials are not used when a base URL is provided - the provider only needs
    /// the access token for API calls.
    pub fn create_with_base_url(
        &self,
        provider: Provider,
        base_url: Option<String>,
    ) -> Arc<dyn GitProvider> {
        match provider {
            Provider::GitHub => Arc::new(GitHubProvider::new_with_base_url(base_url)),
            Provider::GitLab => Arc::new(GitLabProvider::new_with_base_url(base_url)),
            Provider::Bitbucket => Arc::new(BitbucketProvider::new_with_base_url(base_url)),
        }
    }
}
