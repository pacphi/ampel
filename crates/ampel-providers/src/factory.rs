use std::sync::Arc;

use ampel_core::models::GitProvider as Provider;

use crate::{BitbucketProvider, GitHubProvider, GitLabProvider, GitProvider};

/// Factory for creating Git provider instances
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn new() -> Self {
        Self
    }

    /// Create a provider instance for the given type and optional instance URL
    pub fn create(&self, provider: Provider, instance_url: Option<String>) -> Arc<dyn GitProvider> {
        match provider {
            Provider::GitHub => Arc::new(GitHubProvider::new(instance_url)),
            Provider::GitLab => Arc::new(GitLabProvider::new(instance_url)),
            Provider::Bitbucket => Arc::new(BitbucketProvider::new(instance_url)),
        }
    }
}

impl Default for ProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}
