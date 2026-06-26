//! Wiremock-backed tests for `GitHubProvider`'s `RemediationCapable` write primitives.
//!
//! Each test stands up a mock HTTP server, points the provider at it via the
//! `instance_url` constructor argument, exercises one write method, and asserts both
//! the request shape (method + path) and the mapped response.
//!
//! ```bash
//! cargo test -p ampel-providers --test github_remediation_tests
//! ```

use ampel_providers::github::GitHubProvider;
use ampel_providers::remediation::RemediationCapable;
use ampel_providers::traits::ProviderCredentials;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn creds() -> ProviderCredentials {
    ProviderCredentials::Pat {
        token: "ghp_test".to_string(),
        username: None,
    }
}

#[test]
fn should_report_all_capabilities_supported() {
    // Arrange
    let provider = GitHubProvider::new(None);

    // Act
    let caps = provider.capabilities();

    // Assert
    assert_eq!(caps, ampel_providers::RemediationCaps::all());
}

#[tokio::test]
async fn should_resolve_default_branch_sha() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/acme/widget"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "name": "widget", "full_name": "acme/widget",
            "html_url": "https://github.com/acme/widget", "default_branch": "main",
            "private": false, "archived": false, "owner": { "login": "acme" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/acme/widget/git/ref/heads/main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "abc123" }
        })))
        .mount(&server)
        .await;
    let provider = GitHubProvider::new(Some(server.uri()));

    // Act
    let sha = provider
        .get_default_branch_sha(&creds(), "acme", "widget")
        .await
        .unwrap();

    // Assert
    assert_eq!(sha, "abc123");
}

#[tokio::test]
async fn should_create_branch_from_sha() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repos/acme/widget/git/refs"))
        .and(body_partial_json(serde_json::json!({
            "ref": "refs/heads/consolidate", "sha": "deadbeef"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    let provider = GitHubProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .create_branch(&creds(), "acme", "widget", "consolidate", "deadbeef")
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_open_pull_request() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repos/acme/widget/pulls"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 99, "number": 42, "title": "Consolidated", "body": "rollup",
            "html_url": "https://github.com/acme/widget/pull/42", "state": "open",
            "head": { "ref": "consolidate" }, "base": { "ref": "main" },
            "user": { "login": "bot", "avatar_url": null },
            "created_at": "2026-06-24T00:00:00Z", "updated_at": "2026-06-24T00:00:00Z"
        })))
        .mount(&server)
        .await;
    let provider = GitHubProvider::new(Some(server.uri()));

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
    assert_eq!(pr.number, 42);
    assert_eq!(pr.source_branch, "consolidate");
    assert_eq!(pr.target_branch, "main");
}

#[tokio::test]
async fn should_close_pull_request_with_state_closed() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/repos/acme/widget/pulls/42"))
        .and(body_partial_json(serde_json::json!({ "state": "closed" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    let provider = GitHubProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .close_pull_request(&creds(), "acme", "widget", 42)
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_create_comment_and_return_id() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repos/acme/widget/issues/42/comments"))
        .and(body_partial_json(
            serde_json::json!({ "body": "superseded by #100" }),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({ "id": 7777 })))
        .mount(&server)
        .await;
    let provider = GitHubProvider::new(Some(server.uri()));

    // Act
    let id = provider
        .create_comment(&creds(), "acme", "widget", 42, "superseded by #100")
        .await
        .unwrap();

    // Assert
    assert_eq!(id, 7777);
}

#[tokio::test]
async fn should_add_labels() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repos/acme/widget/issues/42/labels"))
        .and(body_partial_json(
            serde_json::json!({ "labels": ["remediation"] }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    let provider = GitHubProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .add_labels(&creds(), "acme", "widget", 42, &["remediation".to_string()])
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_get_status_for_arbitrary_ref() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/acme/widget/commits/consolidate/check-runs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "check_runs": [
                { "name": "build", "status": "completed", "conclusion": "success",
                  "html_url": null, "started_at": null, "completed_at": null }
            ]
        })))
        .mount(&server)
        .await;
    let provider = GitHubProvider::new(Some(server.uri()));

    // Act
    let checks = provider
        .get_status_for_ref(&creds(), "acme", "widget", "consolidate")
        .await
        .unwrap();

    // Assert
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].name, "build");
    assert_eq!(checks[0].conclusion.as_deref(), Some("success"));
}

#[tokio::test]
async fn should_delete_branch() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/repos/acme/widget/git/refs/heads/consolidate"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    let provider = GitHubProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .delete_branch(&creds(), "acme", "widget", "consolidate")
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_surface_api_error_on_failed_branch_create() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repos/acme/widget/git/refs"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "message": "Reference already exists"
        })))
        .mount(&server)
        .await;
    let provider = GitHubProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .create_branch(&creds(), "acme", "widget", "consolidate", "deadbeef")
        .await;

    // Assert
    assert!(result.is_err());
}
