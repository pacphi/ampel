use ampel_providers::bitbucket::BitbucketProvider;
use ampel_providers::traits::{GitProvider, ProviderCredentials};

#[test]
fn test_bitbucket_provider_type() {
    let provider = BitbucketProvider::new(None);
    assert_eq!(provider.provider_type(), ampel_core::models::GitProvider::Bitbucket);
}

#[test]
fn test_bitbucket_cloud_instance() {
    let provider = BitbucketProvider::new(None);
    assert_eq!(provider.instance_url(), None);
}

#[test]
fn test_bitbucket_self_hosted_instance() {
    let provider = BitbucketProvider::new(Some("https://bitbucket.company.com/api".to_string()));
    assert_eq!(provider.instance_url(), Some("https://bitbucket.company.com/api"));
}

#[test]
fn test_provider_credentials_pat_with_username() {
    let credentials = ProviderCredentials::Pat {
        token: "app_password_1234567890".to_string(),
        username: Some("testuser".to_string()),
    };

    let ProviderCredentials::Pat { token, username } = credentials;
    assert_eq!(token, "app_password_1234567890");
    assert_eq!(username, Some("testuser".to_string()));
}

#[tokio::test]
async fn test_bitbucket_validate_credentials_missing_username() {
    let provider = BitbucketProvider::new(None);

    let credentials = ProviderCredentials::Pat {
        token: "app_password".to_string(),
        username: None,
    };

    let result = provider.validate_credentials(&credentials).await;

    // Should return Ok with is_valid=false and appropriate error message
    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.is_valid);
    assert!(validation.error_message.is_some());
    assert!(validation
        .error_message
        .unwrap()
        .contains("Bitbucket App Passwords require a username"));
}

#[tokio::test]
async fn test_bitbucket_validate_credentials_empty_username() {
    let provider = BitbucketProvider::new(None);

    let credentials = ProviderCredentials::Pat {
        token: "app_password".to_string(),
        username: Some("".to_string()),
    };

    let result = provider.validate_credentials(&credentials).await;

    // Should return Ok with is_valid=false and appropriate error message
    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.is_valid);
    assert!(validation.error_message.is_some());
    assert!(validation
        .error_message
        .unwrap()
        .contains("Bitbucket App Passwords require a username"));
}

