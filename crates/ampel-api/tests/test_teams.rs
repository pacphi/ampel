/// Integration tests for team handlers
///
/// Tests team CRUD operations, member management, and authorization.
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
async fn test_list_teams_empty() {
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
        .uri("/api/teams")
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
async fn test_list_teams_requires_auth() {
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
        .uri("/api/teams")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_team_not_found() {
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

    let fake_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/teams/{}", fake_uuid))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"].as_str().unwrap().contains("not found"));

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_team_requires_auth() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    let fake_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/teams/{}", fake_uuid))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_add_member_not_found() {
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

    let fake_team_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let fake_user_uuid = "550e8400-e29b-41d4-a716-446655440001";

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/teams/{}/members", fake_team_uuid))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "userId": fake_user_uuid,
                "role": "member"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_add_member_requires_auth() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    let fake_team_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let fake_user_uuid = "550e8400-e29b-41d4-a716-446655440001";

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/teams/{}/members", fake_team_uuid))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "userId": fake_user_uuid,
                "role": "member"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_remove_member_not_found() {
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

    let fake_team_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let fake_user_uuid = "550e8400-e29b-41d4-a716-446655440001";

    let request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/teams/{}/members/{}",
            fake_team_uuid, fake_user_uuid
        ))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_remove_member_requires_auth() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    let fake_team_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let fake_user_uuid = "550e8400-e29b-41d4-a716-446655440001";

    let request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/teams/{}/members/{}",
            fake_team_uuid, fake_user_uuid
        ))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}
