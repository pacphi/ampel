//! Wiremock-backed tests for `GitLabProvider`'s `RemediationCapable` write primitives.
//!
//! ```bash
//! cargo test -p ampel-providers --test gitlab_remediation_tests
//! ```

use ampel_providers::gitlab::GitLabProvider;
use ampel_providers::remediation::RemediationCapable;
use ampel_providers::traits::ProviderCredentials;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn creds() -> ProviderCredentials {
    ProviderCredentials::Pat {
        token: "glpat_test".to_string(),
        username: None,
    }
}

// Project "acme/widget" URL-encodes to "acme%2Fwidget"; wiremock decodes the path, so
// matchers use the decoded "/projects/acme/widget/...".

#[test]
fn should_report_all_capabilities_supported() {
    let provider = GitLabProvider::new(None);
    assert_eq!(
        provider.capabilities(),
        ampel_providers::RemediationCaps::all()
    );
}

#[tokio::test]
async fn should_create_branch_via_query_params() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/acme%2Fwidget/repository/branches"))
        .and(query_param("branch", "consolidate"))
        .and(query_param("ref", "deadbeef"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    let provider = GitLabProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .create_branch(&creds(), "acme", "widget", "consolidate", "deadbeef")
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_create_merge_request() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/acme%2Fwidget/merge_requests"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 10, "iid": 5, "title": "Consolidated", "description": "rollup",
            "web_url": "https://gitlab.com/acme/widget/-/merge_requests/5",
            "state": "opened", "source_branch": "consolidate", "target_branch": "main",
            "author": { "username": "bot", "avatar_url": null },
            "draft": false, "merge_status": "can_be_merged", "has_conflicts": false,
            "created_at": "2026-06-24T00:00:00Z", "updated_at": "2026-06-24T00:00:00Z"
        })))
        .mount(&server)
        .await;
    let provider = GitLabProvider::new(Some(server.uri()));

    // Act
    let mr = provider
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
    assert_eq!(mr.number, 5);
    assert_eq!(mr.state, "open");
}

#[tokio::test]
async fn should_update_branch_via_mr_rebase_lookup() {
    // Arrange — lookup returns one open MR, then rebase succeeds.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/acme%2Fwidget/merge_requests"))
        .and(query_param("source_branch", "consolidate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "iid": 5 }
        ])))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(
            "/api/v4/projects/acme%2Fwidget/merge_requests/5/rebase",
        ))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    let provider = GitLabProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .update_branch_from_base(&creds(), "acme", "widget", "consolidate", "main")
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_error_when_no_mr_for_branch_update() {
    // Arrange — lookup returns no MRs.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/acme%2Fwidget/merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    let provider = GitLabProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .update_branch_from_base(&creds(), "acme", "widget", "consolidate", "main")
        .await;

    // Assert
    assert!(result.is_err());
}

#[tokio::test]
async fn should_close_merge_request_with_state_event() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/acme%2Fwidget/merge_requests/5"))
        .and(query_param("state_event", "close"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    let provider = GitLabProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .close_pull_request(&creds(), "acme", "widget", 5)
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_create_note_and_return_id() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v4/projects/acme%2Fwidget/merge_requests/5/notes",
        ))
        .and(query_param("body", "superseded by !10"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({ "id": 888 })))
        .mount(&server)
        .await;
    let provider = GitLabProvider::new(Some(server.uri()));

    // Act
    let id = provider
        .create_comment(&creds(), "acme", "widget", 5, "superseded by !10")
        .await
        .unwrap();

    // Assert
    assert_eq!(id, 888);
}

#[tokio::test]
async fn should_add_labels_comma_joined() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/acme%2Fwidget/merge_requests/5"))
        .and(query_param("add_labels", "remediation,bot"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    let provider = GitLabProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .add_labels(
            &creds(),
            "acme",
            "widget",
            5,
            &["remediation".to_string(), "bot".to_string()],
        )
        .await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_get_status_for_ref_from_commit_statuses() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v4/projects/acme%2Fwidget/repository/commits/consolidate/statuses",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "name": "test", "status": "success", "target_url": null,
              "started_at": null, "finished_at": null }
        ])))
        .mount(&server)
        .await;
    let provider = GitLabProvider::new(Some(server.uri()));

    // Act
    let checks = provider
        .get_status_for_ref(&creds(), "acme", "widget", "consolidate")
        .await
        .unwrap();

    // Assert
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].conclusion.as_deref(), Some("success"));
}

#[tokio::test]
async fn should_delete_branch() {
    // Arrange
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v4/projects/acme%2Fwidget/repository/branches/consolidate",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    let provider = GitLabProvider::new(Some(server.uri()));

    // Act
    let result = provider
        .delete_branch(&creds(), "acme", "widget", "consolidate")
        .await;

    // Assert
    assert!(result.is_ok());
}
