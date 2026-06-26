//! Wiremock-backed tests for `BitbucketProvider`'s `RemediationCapable` write primitives,
//! including the two operations Bitbucket does not support.
//!
//! ```bash
//! cargo test -p ampel-providers --test bitbucket_remediation_tests
//! ```

use ampel_providers::bitbucket::BitbucketProvider;
use ampel_providers::error::ProviderError;
use ampel_providers::remediation::RemediationCapable;
use ampel_providers::traits::ProviderCredentials;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn creds() -> ProviderCredentials {
    ProviderCredentials::Pat {
        token: "bb_app_password".to_string(),
        username: Some("acme-bot".to_string()),
    }
}

#[test]
fn should_report_partial_capabilities() {
    // Arrange
    let provider = BitbucketProvider::new(None);

    // Act
    let caps = provider.capabilities();

    // Assert — only update_branch_from_base and add_labels are unsupported.
    assert!(!caps.update_branch_from_base);
    assert!(!caps.add_labels);
    assert!(caps.create_branch);
    assert!(caps.create_pull_request);
    assert!(caps.close_pull_request);
    assert!(caps.create_comment);
    assert!(caps.get_status_for_ref);
    assert!(caps.delete_branch);
}

#[tokio::test]
async fn should_return_not_supported_for_update_branch_from_base() {
    // Arrange
    let provider = BitbucketProvider::new(None);

    // Act
    let result = provider
        .update_branch_from_base(&creds(), "acme", "widget", "consolidate", "main")
        .await;

    // Assert
    assert!(matches!(result, Err(ProviderError::NotSupported(_))));
}

#[tokio::test]
async fn should_return_not_supported_for_add_labels() {
    // Arrange
    let provider = BitbucketProvider::new(None);

    // Act
    let result = provider
        .add_labels(&creds(), "acme", "widget", 5, &["bot".to_string()])
        .await;

    // Assert
    assert!(matches!(result, Err(ProviderError::NotSupported(_))));
}

#[tokio::test]
async fn should_create_branch() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repositories/acme/widget/refs/branches"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    let provider = BitbucketProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .create_branch(&creds(), "acme", "widget", "consolidate", "deadbeef")
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_create_pull_request() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repositories/acme/widget/pullrequests"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 7, "title": "Consolidated", "description": "rollup",
            "links": { "html": { "href": "https://bitbucket.org/acme/widget/pull-requests/7" } },
            "state": "OPEN",
            "source": { "branch": { "name": "consolidate" } },
            "destination": { "branch": { "name": "main" } },
            "author": { "username": "bot", "display_name": "Bot", "links": null },
            "created_on": "2026-06-24T00:00:00Z", "updated_on": "2026-06-24T00:00:00Z"
        })))
        .mount(&server)
        .await;
    let provider = BitbucketProvider::new(Some(server.uri()));

    // Act
    let pr = provider
        .create_pull_request(
            &creds(),
            "acme",
            "widget",
            "Consolidated",
            "rollup",
            "consolidate",
            "main",
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(pr.number, 7);
    assert_eq!(pr.state, "open");
    assert_eq!(pr.source_branch, "consolidate");
}

#[tokio::test]
async fn should_decline_pull_request_on_close() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repositories/acme/widget/pullrequests/7/decline"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    let provider = BitbucketProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .close_pull_request(&creds(), "acme", "widget", 7)
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_create_comment_and_return_id() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repositories/acme/widget/pullrequests/7/comments"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({ "id": 555 })))
        .mount(&server)
        .await;
    let provider = BitbucketProvider::new(Some(server.uri()));

    // Act
    let id = provider
        .create_comment(&creds(), "acme", "widget", 7, "superseded")
        .await
        .unwrap();

    // Assert
    assert_eq!(id, 555);
}

#[tokio::test]
async fn should_get_status_for_ref() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/repositories/acme/widget/commit/consolidate/statuses",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "values": [
                { "key": "build", "name": "Build", "state": "SUCCESSFUL",
                  "url": null, "created_on": null, "updated_on": null }
            ],
            "next": null
        })))
        .mount(&server)
        .await;
    let provider = BitbucketProvider::new(Some(server.uri()));

    // Act
    let checks = provider
        .get_status_for_ref(&creds(), "acme", "widget", "consolidate")
        .await
        .unwrap();

    // Assert
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].name, "Build");
    assert_eq!(checks[0].conclusion.as_deref(), Some("success"));
}

#[tokio::test]
async fn should_delete_branch() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/repositories/acme/widget/refs/branches/consolidate"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    let provider = BitbucketProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .delete_branch(&creds(), "acme", "widget", "consolidate")
        .await;

    // Assert
    assert!(result.is_ok());
}
