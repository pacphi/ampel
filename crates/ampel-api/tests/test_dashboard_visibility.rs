/// Integration tests for dashboard visibility breakdown tiles
///
/// Tests visibility breakdown calculations for repositories and PRs.
mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use common::{create_test_app, TestDb};
use serde_json::{json, Value};
use tower::ServiceExt;

/// Helper to register a user and return access token
async fn register_and_login(app: &axum::Router) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "test@example.com",
                "password": "SecurePassword123!",
                "displayName": "Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    json["data"]["accessToken"].as_str().unwrap().to_string()
}

/// Helper to create an organization for the user
async fn create_organization(app: &axum::Router, token: &str) -> i64 {
    let request = Request::builder()
        .method("POST")
        .uri("/api/organizations")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Test Organization",
                "slug": "test-org"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    json["data"]["id"].as_i64().unwrap()
}

/// Helper to create a repository with specific visibility
async fn create_repository(
    app: &axum::Router,
    token: &str,
    org_id: i64,
    name: &str,
    is_private: bool,
    is_archived: bool,
) -> i64 {
    let request = Request::builder()
        .method("POST")
        .uri("/api/repositories")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "organizationId": org_id,
                "provider": "github",
                "externalId": format!("ext-{}", name),
                "name": name,
                "fullName": format!("test-org/{}", name),
                "url": format!("https://github.com/test-org/{}", name),
                "isPrivate": is_private,
                "isArchived": is_archived
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    json["data"]["id"].as_i64().unwrap()
}

/// Helper to create a pull request
async fn create_pull_request(app: &axum::Router, token: &str, repo_id: i64, pr_number: i64) -> i64 {
    let request = Request::builder()
        .method("POST")
        .uri("/api/pull-requests")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "repositoryId": repo_id,
                "externalId": format!("pr-{}", pr_number),
                "number": pr_number,
                "title": format!("Test PR #{}", pr_number),
                "url": format!("https://github.com/test-org/repo/pull/{}", pr_number),
                "state": "open",
                "isDraft": false,
                "author": "test-author"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    json["data"]["id"].as_i64().unwrap()
}

/// Helper to create a CI check with specific status
async fn create_ci_check(app: &axum::Router, token: &str, pr_id: i64, name: &str, status: &str) {
    let request = Request::builder()
        .method("POST")
        .uri("/api/ci-checks")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "pullRequestId": pr_id,
                "externalId": format!("check-{}", name),
                "name": name,
                "status": status,
                "conclusion": status
            })
            .to_string(),
        ))
        .unwrap();

    let _ = app.clone().oneshot(request).await.unwrap();
}

/// Helper to create a review with specific state
async fn create_review(app: &axum::Router, token: &str, pr_id: i64, reviewer: &str, state: &str) {
    let request = Request::builder()
        .method("POST")
        .uri("/api/reviews")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "pullRequestId": pr_id,
                "externalId": format!("review-{}", reviewer),
                "reviewer": reviewer,
                "state": state
            })
            .to_string(),
        ))
        .unwrap();

    let _ = app.clone().oneshot(request).await.unwrap();
}

#[tokio::test]
async fn test_all_public_repositories() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;
    let org_id = create_organization(&app, &token).await;

    // Create 5 public repositories
    for i in 1..=5 {
        create_repository(
            &app,
            &token,
            org_id,
            &format!("public-repo-{}", i),
            false,
            false,
        )
        .await;
    }

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["totalRepositories"], 5);
    assert_eq!(json["data"]["repositoryBreakdown"]["public"], 5);
    assert_eq!(json["data"]["repositoryBreakdown"]["private"], 0);
    assert_eq!(json["data"]["repositoryBreakdown"]["archived"], 0);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_all_private_repositories() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;
    let org_id = create_organization(&app, &token).await;

    // Create 3 private repositories
    for i in 1..=3 {
        create_repository(
            &app,
            &token,
            org_id,
            &format!("private-repo-{}", i),
            true,
            false,
        )
        .await;
    }

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["totalRepositories"], 3);
    assert_eq!(json["data"]["repositoryBreakdown"]["public"], 0);
    assert_eq!(json["data"]["repositoryBreakdown"]["private"], 3);
    assert_eq!(json["data"]["repositoryBreakdown"]["archived"], 0);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_mixed_visibility_with_prs() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;
    let org_id = create_organization(&app, &token).await;

    // Create 2 public, 3 private, 1 archived repository
    let public_repo1 = create_repository(&app, &token, org_id, "public-1", false, false).await;
    let _public_repo2 = create_repository(&app, &token, org_id, "public-2", false, false).await;
    let private_repo1 = create_repository(&app, &token, org_id, "private-1", true, false).await;
    let _private_repo2 = create_repository(&app, &token, org_id, "private-2", true, false).await;
    let _private_repo3 = create_repository(&app, &token, org_id, "private-3", true, false).await;
    let archived_repo = create_repository(&app, &token, org_id, "archived-1", false, true).await;

    // Create PRs with different statuses
    // Public repo 1: 1 green PR, 1 red PR
    let pr1 = create_pull_request(&app, &token, public_repo1, 1).await;
    create_ci_check(&app, &token, pr1, "ci-1", "success").await;
    create_review(&app, &token, pr1, "reviewer-1", "approved").await;

    let pr2 = create_pull_request(&app, &token, public_repo1, 2).await;
    create_ci_check(&app, &token, pr2, "ci-2", "failure").await;

    // Private repo 1: 1 green PR
    let pr3 = create_pull_request(&app, &token, private_repo1, 1).await;
    create_ci_check(&app, &token, pr3, "ci-3", "success").await;
    create_review(&app, &token, pr3, "reviewer-2", "approved").await;

    // Archived repo: 1 yellow PR
    let pr4 = create_pull_request(&app, &token, archived_repo, 1).await;
    create_ci_check(&app, &token, pr4, "ci-4", "pending").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify repository breakdown
    assert_eq!(json["data"]["totalRepositories"], 6);
    assert_eq!(json["data"]["repositoryBreakdown"]["public"], 2);
    assert_eq!(json["data"]["repositoryBreakdown"]["private"], 3);
    assert_eq!(json["data"]["repositoryBreakdown"]["archived"], 1);

    // Verify open PRs breakdown
    assert_eq!(json["data"]["totalOpenPrs"], 4);
    assert_eq!(json["data"]["openPrsBreakdown"]["public"], 2);
    assert_eq!(json["data"]["openPrsBreakdown"]["private"], 1);
    assert_eq!(json["data"]["openPrsBreakdown"]["archived"], 1);

    // Verify ready to merge breakdown (green PRs)
    assert_eq!(json["data"]["statusCounts"]["green"], 2);
    assert_eq!(json["data"]["readyToMergeBreakdown"]["public"], 1);
    assert_eq!(json["data"]["readyToMergeBreakdown"]["private"], 1);
    assert_eq!(json["data"]["readyToMergeBreakdown"]["archived"], 0);

    // Verify needs attention breakdown (red PRs)
    assert_eq!(json["data"]["statusCounts"]["red"], 1);
    assert_eq!(json["data"]["needsAttentionBreakdown"]["public"], 1);
    assert_eq!(json["data"]["needsAttentionBreakdown"]["private"], 0);
    assert_eq!(json["data"]["needsAttentionBreakdown"]["archived"], 0);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_archived_repositories_with_open_prs() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;
    let org_id = create_organization(&app, &token).await;

    // Create archived repository with open PRs
    let archived_repo = create_repository(&app, &token, org_id, "archived-repo", true, true).await;

    // Create 2 PRs: 1 green, 1 red
    let pr1 = create_pull_request(&app, &token, archived_repo, 1).await;
    create_ci_check(&app, &token, pr1, "ci-1", "success").await;
    create_review(&app, &token, pr1, "reviewer-1", "approved").await;

    let pr2 = create_pull_request(&app, &token, archived_repo, 2).await;
    create_ci_check(&app, &token, pr2, "ci-2", "failure").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Repository is counted as archived
    assert_eq!(json["data"]["repositoryBreakdown"]["archived"], 1);
    assert_eq!(json["data"]["repositoryBreakdown"]["public"], 0);
    assert_eq!(json["data"]["repositoryBreakdown"]["private"], 0);

    // PRs should be counted in archived breakdown
    assert_eq!(json["data"]["openPrsBreakdown"]["archived"], 2);
    assert_eq!(json["data"]["readyToMergeBreakdown"]["archived"], 1);
    assert_eq!(json["data"]["needsAttentionBreakdown"]["archived"], 1);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_breakdown_totals_match_top_level_counts() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;
    let org_id = create_organization(&app, &token).await;

    // Create diverse set of repositories
    let public_repo = create_repository(&app, &token, org_id, "public", false, false).await;
    let private_repo = create_repository(&app, &token, org_id, "private", true, false).await;
    let archived_repo = create_repository(&app, &token, org_id, "archived", false, true).await;

    // Create PRs with various statuses
    let pr1 = create_pull_request(&app, &token, public_repo, 1).await;
    create_ci_check(&app, &token, pr1, "ci-1", "success").await;
    create_review(&app, &token, pr1, "reviewer-1", "approved").await;

    let pr2 = create_pull_request(&app, &token, private_repo, 1).await;
    create_ci_check(&app, &token, pr2, "ci-2", "failure").await;

    let pr3 = create_pull_request(&app, &token, archived_repo, 1).await;
    create_ci_check(&app, &token, pr3, "ci-3", "pending").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify repository breakdown totals match
    let repo_breakdown_total = json["data"]["repositoryBreakdown"]["public"]
        .as_i64()
        .unwrap()
        + json["data"]["repositoryBreakdown"]["private"]
            .as_i64()
            .unwrap()
        + json["data"]["repositoryBreakdown"]["archived"]
            .as_i64()
            .unwrap();
    assert_eq!(
        repo_breakdown_total,
        json["data"]["totalRepositories"].as_i64().unwrap()
    );

    // Verify open PRs breakdown totals match
    let prs_breakdown_total = json["data"]["openPrsBreakdown"]["public"].as_i64().unwrap()
        + json["data"]["openPrsBreakdown"]["private"]
            .as_i64()
            .unwrap()
        + json["data"]["openPrsBreakdown"]["archived"]
            .as_i64()
            .unwrap();
    assert_eq!(
        prs_breakdown_total,
        json["data"]["totalOpenPrs"].as_i64().unwrap()
    );

    // Verify ready to merge breakdown totals match
    let ready_breakdown_total = json["data"]["readyToMergeBreakdown"]["public"]
        .as_i64()
        .unwrap()
        + json["data"]["readyToMergeBreakdown"]["private"]
            .as_i64()
            .unwrap()
        + json["data"]["readyToMergeBreakdown"]["archived"]
            .as_i64()
            .unwrap();
    assert_eq!(
        ready_breakdown_total,
        json["data"]["statusCounts"]["green"].as_i64().unwrap()
    );

    // Verify needs attention breakdown totals match
    let needs_attention_total = json["data"]["needsAttentionBreakdown"]["public"]
        .as_i64()
        .unwrap()
        + json["data"]["needsAttentionBreakdown"]["private"]
            .as_i64()
            .unwrap()
        + json["data"]["needsAttentionBreakdown"]["archived"]
            .as_i64()
            .unwrap();
    assert_eq!(
        needs_attention_total,
        json["data"]["statusCounts"]["red"].as_i64().unwrap()
    );

    test_db.cleanup().await;
}
