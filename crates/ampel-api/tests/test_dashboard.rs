/// Integration tests for dashboard handlers
///
/// Tests dashboard summary statistics and grid/list views.
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

#[tokio::test]
async fn test_get_summary_empty() {
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

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["totalRepositories"], 0);
    assert_eq!(json["data"]["totalOpenPrs"], 0);
    assert_eq!(json["data"]["statusCounts"]["green"], 0);
    assert_eq!(json["data"]["statusCounts"]["yellow"], 0);
    assert_eq!(json["data"]["statusCounts"]["red"], 0);
    assert_eq!(json["data"]["providerCounts"]["github"], 0);
    assert_eq!(json["data"]["providerCounts"]["gitlab"], 0);
    assert_eq!(json["data"]["providerCounts"]["bitbucket"], 0);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_summary_requires_auth() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_grid_empty() {
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

    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 0);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_grid_requires_auth() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/grid")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_summary_has_correct_structure() {
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

    // Verify structure
    assert!(json["data"]["totalRepositories"].is_number());
    assert!(json["data"]["totalOpenPrs"].is_number());
    assert!(json["data"]["statusCounts"].is_object());
    assert!(json["data"]["statusCounts"]["green"].is_number());
    assert!(json["data"]["statusCounts"]["yellow"].is_number());
    assert!(json["data"]["statusCounts"]["red"].is_number());
    assert!(json["data"]["providerCounts"].is_object());
    assert!(json["data"]["providerCounts"]["github"].is_number());
    assert!(json["data"]["providerCounts"]["gitlab"].is_number());
    assert!(json["data"]["providerCounts"]["bitbucket"].is_number());

    // Verify visibility breakdown structure
    assert!(json["data"]["repositoryBreakdown"].is_object());
    assert!(json["data"]["repositoryBreakdown"]["public"].is_number());
    assert!(json["data"]["repositoryBreakdown"]["private"].is_number());
    assert!(json["data"]["repositoryBreakdown"]["archived"].is_number());
    assert!(json["data"]["openPrsBreakdown"].is_object());
    assert!(json["data"]["openPrsBreakdown"]["public"].is_number());
    assert!(json["data"]["openPrsBreakdown"]["private"].is_number());
    assert!(json["data"]["openPrsBreakdown"]["archived"].is_number());
    assert!(json["data"]["readyToMergeBreakdown"].is_object());
    assert!(json["data"]["readyToMergeBreakdown"]["public"].is_number());
    assert!(json["data"]["readyToMergeBreakdown"]["private"].is_number());
    assert!(json["data"]["readyToMergeBreakdown"]["archived"].is_number());
    assert!(json["data"]["needsAttentionBreakdown"].is_object());
    assert!(json["data"]["needsAttentionBreakdown"]["public"].is_number());
    assert!(json["data"]["needsAttentionBreakdown"]["private"].is_number());
    assert!(json["data"]["needsAttentionBreakdown"]["archived"].is_number());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_grid_returns_array_of_repositories() {
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

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/grid")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());

    test_db.cleanup().await;
}
