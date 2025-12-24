/// Integration tests for dashboard visibility breakdown tiles
///
/// Tests visibility breakdown calculations for repositories and PRs.
mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::Utc;
use common::{create_test_app, TestDb};
use sea_orm::Set;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

/// Helper to register a user and return access token and user_id
async fn register_and_login(app: &axum::Router) -> (String, Uuid) {
    // Register the user
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": format!("test-{}@example.com", Uuid::new_v4()),
                "password": "SecurePassword123!",
                "displayName": "Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    let register_body = axum::body::to_bytes(register_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let register_json: Value = serde_json::from_slice(&register_body).unwrap();

    let token = register_json["data"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Call /api/auth/me to get the user ID
    let me_request = Request::builder()
        .method("GET")
        .uri("/api/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let me_response = app.clone().oneshot(me_request).await.unwrap();
    let me_body = axum::body::to_bytes(me_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let me_json: Value = serde_json::from_slice(&me_body).unwrap();

    let user_id = Uuid::parse_str(me_json["data"]["id"].as_str().unwrap()).unwrap();
    (token, user_id)
}

/// Create a test repository directly in the database
async fn create_repository(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
    name: &str,
    is_private: bool,
    is_archived: bool,
) -> ampel_db::entities::repository::Model {
    use ampel_db::entities::repository::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let repo = ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        provider: Set("github".to_string()),
        provider_id: Set(format!("repo-{}", Uuid::new_v4())),
        owner: Set("test-owner".to_string()),
        name: Set(name.to_string()),
        full_name: Set(format!("owner/{}", name)),
        description: Set(Some("Test repository".to_string())),
        url: Set(format!("https://github.com/test-org/{}", name)),
        default_branch: Set("main".to_string()),
        is_private: Set(is_private),
        is_archived: Set(is_archived),
        poll_interval_seconds: Set(300),
        last_polled_at: Set(Some(Utc::now())),
        group_id: Set(None),
        provider_account_id: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    repo.insert(db).await.unwrap()
}

/// Create a test pull request directly in the database
async fn create_pull_request(
    db: &sea_orm::DatabaseConnection,
    repository_id: Uuid,
    pr_number: i32,
) -> ampel_db::entities::pull_request::Model {
    use ampel_db::entities::pull_request::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let pr = ActiveModel {
        id: Set(Uuid::new_v4()),
        repository_id: Set(repository_id),
        provider: Set("github".to_string()),
        provider_id: Set(format!("pr-{}", Uuid::new_v4())),
        number: Set(pr_number),
        title: Set(format!("Test PR #{}", pr_number)),
        description: Set(Some("Test pull request".to_string())),
        url: Set(format!("https://github.com/test/pr/{}", Uuid::new_v4())),
        state: Set("open".to_string()),
        source_branch: Set("feature".to_string()),
        target_branch: Set("main".to_string()),
        author: Set("testuser".to_string()),
        author_avatar_url: Set(Some("https://avatar.com/test".to_string())),
        is_draft: Set(false),
        is_mergeable: Set(Some(true)),
        has_conflicts: Set(false),
        additions: Set(10),
        deletions: Set(5),
        changed_files: Set(2),
        commits_count: Set(1),
        comments_count: Set(0),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        merged_at: Set(None),
        closed_at: Set(None),
        last_synced_at: Set(Utc::now()),
    };

    pr.insert(db).await.unwrap()
}

/// Create CI check for PR directly in the database
async fn create_ci_check(
    db: &sea_orm::DatabaseConnection,
    pr_id: Uuid,
    name: &str,
    status: &str,
) -> ampel_db::entities::ci_check::Model {
    use ampel_db::entities::ci_check::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let check = ActiveModel {
        id: Set(Uuid::new_v4()),
        pull_request_id: Set(pr_id),
        name: Set(name.to_string()),
        status: Set("completed".to_string()),
        conclusion: Set(Some(status.to_string())),
        url: Set(Some("https://ci.example.com/build".to_string())),
        started_at: Set(Some(Utc::now())),
        completed_at: Set(Some(Utc::now())),
        duration_seconds: Set(Some(60)),
    };

    check.insert(db).await.unwrap()
}

/// Create review for PR directly in the database
async fn create_review(
    db: &sea_orm::DatabaseConnection,
    pr_id: Uuid,
    reviewer: &str,
    state: &str,
) -> ampel_db::entities::review::Model {
    use ampel_db::entities::review::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let review = ActiveModel {
        id: Set(Uuid::new_v4()),
        pull_request_id: Set(pr_id),
        reviewer: Set(reviewer.to_string()),
        reviewer_avatar_url: Set(Some("https://avatar.com/reviewer".to_string())),
        state: Set(state.to_string()),
        body: Set(Some("LGTM".to_string())),
        submitted_at: Set(Utc::now()),
    };

    review.insert(db).await.unwrap()
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
    let (token, user_id) = register_and_login(&app).await;

    // Create 5 public repositories
    for i in 1..=5 {
        create_repository(
            test_db.connection(),
            user_id,
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
    let (token, user_id) = register_and_login(&app).await;

    // Create 3 private repositories
    for i in 1..=3 {
        create_repository(
            test_db.connection(),
            user_id,
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
    let (token, user_id) = register_and_login(&app).await;

    // Create 2 public, 3 private, 1 archived repository
    let public_repo1 =
        create_repository(test_db.connection(), user_id, "public-1", false, false).await;
    let _public_repo2 =
        create_repository(test_db.connection(), user_id, "public-2", false, false).await;
    let private_repo1 =
        create_repository(test_db.connection(), user_id, "private-1", true, false).await;
    let _private_repo2 =
        create_repository(test_db.connection(), user_id, "private-2", true, false).await;
    let _private_repo3 =
        create_repository(test_db.connection(), user_id, "private-3", true, false).await;
    let archived_repo =
        create_repository(test_db.connection(), user_id, "archived-1", false, true).await;

    // Create PRs with different statuses
    // Public repo 1: 1 green PR, 1 red PR
    let pr1 = create_pull_request(test_db.connection(), public_repo1.id, 1).await;
    create_ci_check(test_db.connection(), pr1.id, "ci-1", "success").await;
    create_review(test_db.connection(), pr1.id, "reviewer-1", "approved").await;

    let pr2 = create_pull_request(test_db.connection(), public_repo1.id, 2).await;
    create_ci_check(test_db.connection(), pr2.id, "ci-2", "failure").await;

    // Private repo 1: 1 green PR
    let pr3 = create_pull_request(test_db.connection(), private_repo1.id, 1).await;
    create_ci_check(test_db.connection(), pr3.id, "ci-3", "success").await;
    create_review(test_db.connection(), pr3.id, "reviewer-2", "approved").await;

    // Archived repo: 1 yellow PR
    let pr4 = create_pull_request(test_db.connection(), archived_repo.id, 1).await;
    create_ci_check(test_db.connection(), pr4.id, "ci-4", "pending").await;

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
    let (token, user_id) = register_and_login(&app).await;

    // Create archived repository with open PRs
    let archived_repo =
        create_repository(test_db.connection(), user_id, "archived-repo", true, true).await;

    // Create 2 PRs: 1 green, 1 red
    let pr1 = create_pull_request(test_db.connection(), archived_repo.id, 1).await;
    create_ci_check(test_db.connection(), pr1.id, "ci-1", "success").await;
    create_review(test_db.connection(), pr1.id, "reviewer-1", "approved").await;

    let pr2 = create_pull_request(test_db.connection(), archived_repo.id, 2).await;
    create_ci_check(test_db.connection(), pr2.id, "ci-2", "failure").await;

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
    let (token, user_id) = register_and_login(&app).await;

    // Create diverse set of repositories
    let public_repo =
        create_repository(test_db.connection(), user_id, "public", false, false).await;
    let private_repo =
        create_repository(test_db.connection(), user_id, "private", true, false).await;
    let archived_repo =
        create_repository(test_db.connection(), user_id, "archived", false, true).await;

    // Create PRs with various statuses
    let pr1 = create_pull_request(test_db.connection(), public_repo.id, 1).await;
    create_ci_check(test_db.connection(), pr1.id, "ci-1", "success").await;
    create_review(test_db.connection(), pr1.id, "reviewer-1", "approved").await;

    let pr2 = create_pull_request(test_db.connection(), private_repo.id, 1).await;
    create_ci_check(test_db.connection(), pr2.id, "ci-2", "failure").await;

    let pr3 = create_pull_request(test_db.connection(), archived_repo.id, 1).await;
    create_ci_check(test_db.connection(), pr3.id, "ci-3", "pending").await;

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
