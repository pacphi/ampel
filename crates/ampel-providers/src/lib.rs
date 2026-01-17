pub mod bitbucket;
pub mod error;
pub mod factory;
pub mod github;
pub mod gitlab;
pub mod traits;
pub mod utils;

#[cfg(any(test, feature = "test-utils"))]
pub mod mock;

pub use bitbucket::BitbucketProvider;
pub use error::ProviderError;
pub use factory::ProviderFactory;
pub use github::GitHubProvider;
pub use gitlab::GitLabProvider;
pub use traits::GitProvider;

#[cfg(any(test, feature = "test-utils"))]
pub use mock::MockProvider;
