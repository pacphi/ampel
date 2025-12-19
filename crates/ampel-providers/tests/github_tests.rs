use ampel_providers::github::GitHubProvider;
use ampel_providers::traits::{GitProvider, ProviderCredentials, TokenValidation};

#[test]
fn test_github_provider_cloud() {
    let provider = GitHubProvider::new(None);

    // Test provider type and instance URL
    assert_eq!(provider.provider_type(), ampel_core::models::GitProvider::GitHub);
    assert_eq!(provider.instance_url(), None);
}

#[test]
fn test_token_validation_struct_default() {
    let validation = TokenValidation::default();

    assert!(!validation.is_valid);
    assert!(validation.user_id.is_none());
    assert!(validation.username.is_none());
    assert!(validation.email.is_none());
    assert!(validation.avatar_url.is_none());
    assert_eq!(validation.scopes.len(), 0);
    assert!(validation.expires_at.is_none());
    assert!(validation.error_message.is_none());
}

#[test]
fn test_provider_credentials_pat_variant() {
    let credentials = ProviderCredentials::Pat {
        token: "test_token".to_string(),
        username: None,
    };

    let ProviderCredentials::Pat { token, username } = credentials;
    assert_eq!(token, "test_token");
    assert!(username.is_none());
}

#[test]
fn test_provider_credentials_pat_with_username() {
    let credentials = ProviderCredentials::Pat {
        token: "test_token".to_string(),
        username: Some("testuser".to_string()),
    };

    let ProviderCredentials::Pat { token, username } = credentials;
    assert_eq!(token, "test_token");
    assert_eq!(username, Some("testuser".to_string()));
}

#[test]
fn test_github_cloud_instance() {
    let provider_cloud = GitHubProvider::new(None);
    assert_eq!(provider_cloud.instance_url(), None);
}

#[test]
fn test_github_enterprise_instance() {
    let provider_enterprise = GitHubProvider::new(Some("https://github.enterprise.com/api/v3".to_string()));
    assert_eq!(provider_enterprise.instance_url(), Some("https://github.enterprise.com/api/v3"));
}
