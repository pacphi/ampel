//! Deterministic, in-memory tests for `MockProvider`'s `RemediationCapable` impl.
//!
//! Unlike the wiremock-backed provider tests, these need no HTTP server: the mock records
//! every write into an inspectable call log and returns stable values, so worker/job tests
//! (Phases 1–4) can drive the full remediation write surface offline.
//!
//! ```bash
//! cargo test -p ampel-providers --all-features --test mock_remediation_tests
//! ```

use ampel_providers::mock::{MockProvider, RemediationCall};
use ampel_providers::remediation::RemediationCapable;
use ampel_providers::traits::ProviderCredentials;
use ampel_providers::RemediationCaps;

fn creds() -> ProviderCredentials {
    ProviderCredentials::Pat {
        token: "mock_token".to_string(),
        username: None,
    }
}

#[test]
fn should_report_all_capabilities_by_default() {
    // Arrange
    let mock = MockProvider::new();

    // Act
    let caps = mock.capabilities();

    // Assert
    assert_eq!(caps, RemediationCaps::all());
}

#[test]
fn should_report_configured_partial_capabilities() {
    // Arrange — simulate a partial-support provider (Bitbucket-like).
    let partial = RemediationCaps {
        create_branch: true,
        create_pull_request: true,
        ..Default::default()
    };
    let mock = MockProvider::new().with_remediation_caps(partial.clone());

    // Act
    let caps = mock.capabilities();

    // Assert
    assert_eq!(caps, partial);
    assert!(!caps.delete_branch);
}

#[tokio::test]
async fn should_return_default_branch_sha_when_unset() {
    // Arrange
    let mock = MockProvider::new();

    // Act
    let sha = mock
        .get_default_branch_sha(&creds(), "acme", "widget")
        .await
        .unwrap();

    // Assert
    assert_eq!(sha, "mockdefaultsha000000000000000000000000000");
}

#[tokio::test]
async fn should_return_configured_default_branch_sha() {
    // Arrange
    let mock = MockProvider::new().with_default_branch_sha("deadbeef");

    // Act
    let sha = mock
        .get_default_branch_sha(&creds(), "acme", "widget")
        .await
        .unwrap();

    // Assert
    assert_eq!(sha, "deadbeef");
}

#[tokio::test]
async fn should_record_create_branch_call() {
    // Arrange
    let mock = MockProvider::new();

    // Act
    mock.create_branch(&creds(), "acme", "widget", "remediation/foo", "abc123")
        .await
        .unwrap();

    // Assert
    assert_eq!(
        mock.remediation_calls(),
        vec![RemediationCall::CreateBranch {
            owner: "acme".to_string(),
            repo: "widget".to_string(),
            branch_name: "remediation/foo".to_string(),
            from_sha: "abc123".to_string(),
        }]
    );
}

#[tokio::test]
async fn should_record_update_branch_from_base_call() {
    // Arrange
    let mock = MockProvider::new();

    // Act
    mock.update_branch_from_base(&creds(), "acme", "widget", "feature", "main")
        .await
        .unwrap();

    // Assert
    assert_eq!(
        mock.remediation_calls(),
        vec![RemediationCall::UpdateBranchFromBase {
            owner: "acme".to_string(),
            repo: "widget".to_string(),
            branch_name: "feature".to_string(),
            base_branch: "main".to_string(),
        }]
    );
}

#[tokio::test]
async fn should_return_open_pull_request_on_create() {
    // Arrange
    let mock = MockProvider::new();

    // Act
    let pr = mock
        .create_pull_request(&creds(), "acme", "widget", "Title", "Body", "head", "main")
        .await
        .unwrap();

    // Assert
    assert_eq!(pr.number, 1);
    assert_eq!(pr.state, "open");
    assert_eq!(pr.source_branch, "head");
    assert_eq!(pr.target_branch, "main");
    assert_eq!(pr.title, "Title");
}

#[tokio::test]
async fn should_assign_monotonic_pr_numbers() {
    // Arrange
    let mock = MockProvider::new();

    // Act
    let first = mock
        .create_pull_request(&creds(), "acme", "widget", "A", "", "h1", "main")
        .await
        .unwrap();
    let second = mock
        .create_pull_request(&creds(), "acme", "widget", "B", "", "h2", "main")
        .await
        .unwrap();

    // Assert
    assert_eq!(first.number, 1);
    assert_eq!(second.number, 2);
}

#[tokio::test]
async fn should_return_monotonic_comment_ids() {
    // Arrange
    let mock = MockProvider::new();

    // Act
    let id1 = mock
        .create_comment(&creds(), "acme", "widget", 7, "first")
        .await
        .unwrap();
    let id2 = mock
        .create_comment(&creds(), "acme", "widget", 7, "second")
        .await
        .unwrap();

    // Assert
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[tokio::test]
async fn should_record_add_labels_call() {
    // Arrange
    let mock = MockProvider::new();
    let labels = vec!["bug".to_string(), "auto".to_string()];

    // Act
    mock.add_labels(&creds(), "acme", "widget", 42, &labels)
        .await
        .unwrap();

    // Assert
    assert_eq!(
        mock.remediation_calls(),
        vec![RemediationCall::AddLabels {
            owner: "acme".to_string(),
            repo: "widget".to_string(),
            pr_number: 42,
            labels,
        }]
    );
}

#[tokio::test]
async fn should_record_close_and_update_and_delete_calls_in_order() {
    // Arrange
    let mock = MockProvider::new();

    // Act
    mock.update_pull_request(&creds(), "acme", "widget", 1, Some("t"), None)
        .await
        .unwrap();
    mock.close_pull_request(&creds(), "acme", "widget", 1)
        .await
        .unwrap();
    mock.delete_branch(&creds(), "acme", "widget", "stale")
        .await
        .unwrap();

    // Assert
    assert_eq!(
        mock.remediation_calls(),
        vec![
            RemediationCall::UpdatePullRequest {
                owner: "acme".to_string(),
                repo: "widget".to_string(),
                pr_number: 1,
            },
            RemediationCall::ClosePullRequest {
                owner: "acme".to_string(),
                repo: "widget".to_string(),
                pr_number: 1,
            },
            RemediationCall::DeleteBranch {
                owner: "acme".to_string(),
                repo: "widget".to_string(),
                branch_name: "stale".to_string(),
            },
        ]
    );
}

#[tokio::test]
async fn should_return_empty_status_for_ref_by_default() {
    // Arrange
    let mock = MockProvider::new();

    // Act
    let checks = mock
        .get_status_for_ref(&creds(), "acme", "widget", "main")
        .await
        .unwrap();

    // Assert
    assert!(checks.is_empty());
}

#[tokio::test]
async fn should_reject_unsupported_write_with_not_supported_error() {
    // Arrange — a provider that does NOT support delete_branch.
    let caps = RemediationCaps {
        delete_branch: false,
        ..RemediationCaps::all()
    };
    let mock = MockProvider::new().with_remediation_caps(caps);

    // Act
    let result = mock
        .delete_branch(&creds(), "acme", "widget", "feature")
        .await;

    // Assert
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("delete_branch"), "unexpected error: {msg}");
}

#[tokio::test]
async fn should_not_record_call_when_capability_unsupported() {
    // Arrange
    let caps = RemediationCaps {
        create_pull_request: false,
        ..RemediationCaps::all()
    };
    let mock = MockProvider::new().with_remediation_caps(caps);

    // Act
    let _ = mock
        .create_pull_request(&creds(), "acme", "widget", "T", "B", "h", "main")
        .await;

    // Assert — the rejected write left no trace in the call log.
    assert!(mock.remediation_calls().is_empty());
}
