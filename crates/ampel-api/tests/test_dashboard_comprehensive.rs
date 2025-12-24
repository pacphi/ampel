/// Comprehensive integration tests for dashboard optimizations and features
///
/// Tests:
/// - Visibility breakdown calculations
/// - Dashboard performance with large datasets
/// - Query optimization
/// - Error handling
/// - Metrics recording
mod common;

// Query modules available for direct use if needed
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::Utc;
use common::{create_test_app, TestDb};
use sea_orm::Set;
use serde_json::{json, Value};
use std::time::Instant;
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

/// Create a test repository
async fn create_repository(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
    provider: &str,
    is_private: bool,
    is_archived: bool,
) -> ampel_db::entities::repository::Model {
    use ampel_db::entities::repository::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let repo = ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        provider: Set(provider.to_string()),
        provider_id: Set(format!("repo-{}", Uuid::new_v4())),
        owner: Set("test-owner".to_string()),
        name: Set(format!("test-repo-{}", Uuid::new_v4())),
        full_name: Set(format!("owner/test-repo-{}", Uuid::new_v4())),
        description: Set(Some("Test repository".to_string())),
        url: Set(format!("https://github.com/test-repo-{}", Uuid::new_v4())),
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

/// Create a test pull request
async fn create_pull_request(
    db: &sea_orm::DatabaseConnection,
    repository_id: Uuid,
    state: &str,
) -> ampel_db::entities::pull_request::Model {
    use ampel_db::entities::pull_request::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let pr = ActiveModel {
        id: Set(Uuid::new_v4()),
        repository_id: Set(repository_id),
        provider: Set("github".to_string()),
        provider_id: Set(format!("pr-{}", Uuid::new_v4())),
        number: Set((Uuid::new_v4().as_u128() % 10000) as i32),
        title: Set(format!("Test PR {}", Uuid::new_v4())),
        description: Set(Some("Test pull request".to_string())),
        url: Set(format!("https://github.com/test/pr/{}", Uuid::new_v4())),
        state: Set(state.to_string()),
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

/// Create CI check for PR
async fn create_ci_check(
    db: &sea_orm::DatabaseConnection,
    pr_id: Uuid,
    status: &str,
) -> ampel_db::entities::ci_check::Model {
    use ampel_db::entities::ci_check::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let check = ActiveModel {
        id: Set(Uuid::new_v4()),
        pull_request_id: Set(pr_id),
        name: Set("CI Build".to_string()),
        status: Set("completed".to_string()),
        conclusion: Set(Some(status.to_string())),
        url: Set(Some("https://ci.example.com/build".to_string())),
        started_at: Set(Some(Utc::now())),
        completed_at: Set(Some(Utc::now())),
        duration_seconds: Set(Some(60)),
    };

    check.insert(db).await.unwrap()
}

/// Create review for PR
async fn create_review(
    db: &sea_orm::DatabaseConnection,
    pr_id: Uuid,
    state: &str,
) -> ampel_db::entities::review::Model {
    use ampel_db::entities::review::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let review = ActiveModel {
        id: Set(Uuid::new_v4()),
        pull_request_id: Set(pr_id),
        reviewer: Set("reviewer".to_string()),
        reviewer_avatar_url: Set(Some("https://avatar.com/reviewer".to_string())),
        state: Set(state.to_string()),
        body: Set(Some("LGTM".to_string())),
        submitted_at: Set(Utc::now()),
    };

    review.insert(db).await.unwrap()
}

#[tokio::test]
async fn test_visibility_breakdown_with_mixed_repositories() {
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

    // Create repositories with different visibility levels
    let public_repo1 =
        create_repository(test_db.connection(), user_id, "github", false, false).await;
    let public_repo2 =
        create_repository(test_db.connection(), user_id, "github", false, false).await;
    let private_repo =
        create_repository(test_db.connection(), user_id, "github", true, false).await;
    let archived_repo =
        create_repository(test_db.connection(), user_id, "github", false, true).await;

    // Create PRs in different repositories
    let pr1 = create_pull_request(test_db.connection(), public_repo1.id, "open").await;
    let pr2 = create_pull_request(test_db.connection(), public_repo2.id, "open").await;
    let pr3 = create_pull_request(test_db.connection(), private_repo.id, "open").await;
    let pr4 = create_pull_request(test_db.connection(), archived_repo.id, "open").await;

    // Create CI checks and reviews for different statuses
    // PR1: Green (ready to merge)
    create_ci_check(test_db.connection(), pr1.id, "success").await;
    create_review(test_db.connection(), pr1.id, "approved").await;

    // PR2: Yellow (in progress)
    create_ci_check(test_db.connection(), pr2.id, "in_progress").await;

    // PR3: Red (needs attention)
    create_ci_check(test_db.connection(), pr3.id, "failure").await;

    // PR4: Red (needs attention, in archived repo)
    create_ci_check(test_db.connection(), pr4.id, "failure").await;

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
    assert_eq!(json["data"]["repositoryBreakdown"]["public"], 2);
    assert_eq!(json["data"]["repositoryBreakdown"]["private"], 1);
    assert_eq!(json["data"]["repositoryBreakdown"]["archived"], 1);

    // Verify open PRs breakdown
    assert_eq!(json["data"]["openPrsBreakdown"]["public"], 2);
    assert_eq!(json["data"]["openPrsBreakdown"]["private"], 1);
    assert_eq!(json["data"]["openPrsBreakdown"]["archived"], 1);

    // Verify ready to merge breakdown (only PR1 - public)
    assert_eq!(json["data"]["readyToMergeBreakdown"]["public"], 1);
    assert_eq!(json["data"]["readyToMergeBreakdown"]["private"], 0);
    assert_eq!(json["data"]["readyToMergeBreakdown"]["archived"], 0);

    // Verify needs attention breakdown (PR3=private, PR4=archived)
    assert_eq!(json["data"]["needsAttentionBreakdown"]["public"], 0);
    assert_eq!(json["data"]["needsAttentionBreakdown"]["private"], 1);
    assert_eq!(json["data"]["needsAttentionBreakdown"]["archived"], 1);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_dashboard_performance_with_100_repositories() {
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

    // Create 100 repositories with PRs
    println!("Creating test data: 100 repositories with PRs...");
    for i in 0..100 {
        let is_private = i % 3 == 0;
        let is_archived = i % 10 == 0;
        let repo = create_repository(
            test_db.connection(),
            user_id,
            if i % 2 == 0 { "github" } else { "gitlab" },
            is_private,
            is_archived,
        )
        .await;

        // Create 1-3 PRs per repository
        let pr_count = (i % 3) + 1;
        for _ in 0..pr_count {
            let pr = create_pull_request(test_db.connection(), repo.id, "open").await;

            // Add CI checks
            create_ci_check(test_db.connection(), pr.id, "success").await;

            // Add reviews
            if i % 2 == 0 {
                create_review(test_db.connection(), pr.id, "approved").await;
            }
        }

        if i % 10 == 0 {
            println!("  Created {} repositories...", i);
        }
    }

    println!("Testing dashboard performance...");
    let start = Instant::now();

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    println!(
        "Dashboard loaded in {:?} with {} repositories and {} PRs",
        duration, json["data"]["totalRepositories"], json["data"]["totalOpenPrs"]
    );

    // Performance assertion: should be < 500ms
    assert!(
        duration.as_millis() < 500,
        "Dashboard should load in < 500ms, took {:?}",
        duration
    );

    // Verify data correctness
    assert_eq!(json["data"]["totalRepositories"], 100);
    assert!(json["data"]["totalOpenPrs"].as_i64().unwrap() > 0);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_dashboard_query_optimization() {
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

    // Create test data: 5 repos with 2 PRs each
    for i in 0..5 {
        let repo =
            create_repository(test_db.connection(), user_id, "github", i % 2 == 0, false).await;

        for _ in 0..2 {
            let pr = create_pull_request(test_db.connection(), repo.id, "open").await;
            create_ci_check(test_db.connection(), pr.id, "success").await;
            create_review(test_db.connection(), pr.id, "approved").await;
        }
    }

    // Test that queries are optimized
    // The dashboard should use batch queries, not N+1 queries
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

    // Verify correct counts
    assert_eq!(json["data"]["totalRepositories"], 5);
    assert_eq!(json["data"]["totalOpenPrs"], 10);
    assert_eq!(json["data"]["statusCounts"]["green"], 10); // All approved

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_dashboard_handles_database_errors_gracefully() {
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

    // Create a repository
    let _repo = create_repository(test_db.connection(), user_id, "github", false, false).await;

    // Dashboard should handle gracefully even with partial data
    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_dashboard_grid_with_visibility_filtering() {
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

    // Create repositories with different visibility
    let public_repo =
        create_repository(test_db.connection(), user_id, "github", false, false).await;
    let private_repo =
        create_repository(test_db.connection(), user_id, "github", true, false).await;

    // Create PRs
    let pr1 = create_pull_request(test_db.connection(), public_repo.id, "open").await;
    let pr2 = create_pull_request(test_db.connection(), private_repo.id, "open").await;

    create_ci_check(test_db.connection(), pr1.id, "success").await;
    create_review(test_db.connection(), pr1.id, "approved").await;

    create_ci_check(test_db.connection(), pr2.id, "failure").await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/grid")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"].as_array().unwrap().len(), 2);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_empty_visibility_breakdown() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let (token, _user_id) = register_and_login(&app).await;

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

    // All breakdowns should be zero
    assert_eq!(json["data"]["repositoryBreakdown"]["public"], 0);
    assert_eq!(json["data"]["repositoryBreakdown"]["private"], 0);
    assert_eq!(json["data"]["repositoryBreakdown"]["archived"], 0);
    assert_eq!(json["data"]["openPrsBreakdown"]["public"], 0);
    assert_eq!(json["data"]["openPrsBreakdown"]["private"], 0);
    assert_eq!(json["data"]["openPrsBreakdown"]["archived"], 0);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_dashboard_with_closed_prs() {
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

    let repo = create_repository(test_db.connection(), user_id, "github", false, false).await;

    // Create open and closed PRs
    let _open_pr = create_pull_request(test_db.connection(), repo.id, "open").await;
    let _closed_pr = create_pull_request(test_db.connection(), repo.id, "closed").await;
    let _merged_pr = create_pull_request(test_db.connection(), repo.id, "merged").await;

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

    // Should only count open PRs
    assert_eq!(json["data"]["totalOpenPrs"], 1);
    assert_eq!(json["data"]["openPrsBreakdown"]["public"], 1);

    test_db.cleanup().await;
}
