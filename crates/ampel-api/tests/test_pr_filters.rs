/// Integration tests for PR filter handlers
///
/// Tests PR filter settings management and defaults.
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
async fn test_get_pr_filters_returns_defaults() {
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
        .uri("/api/pr-filters")
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
    assert!(json["data"]["allowedActors"].is_array());
    assert!(json["data"]["skipLabels"].is_array());

    // Verify default allowed actors
    let allowed_actors = json["data"]["allowedActors"].as_array().unwrap();
    assert!(allowed_actors.contains(&json!("dependabot[bot]")));
    assert!(allowed_actors.contains(&json!("renovate[bot]")));
    assert!(allowed_actors.contains(&json!("snyk-bot")));

    // Verify default skip labels
    let skip_labels = json["data"]["skipLabels"].as_array().unwrap();
    assert!(skip_labels.contains(&json!("do-not-merge")));
    assert!(skip_labels.contains(&json!("wip")));
    assert!(skip_labels.contains(&json!("draft")));
    assert!(skip_labels.contains(&json!("hold")));

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_pr_filters_requires_auth() {
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
        .uri("/api/pr-filters")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_update_pr_filters_success() {
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
        .method("PUT")
        .uri("/api/pr-filters")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "allowedActors": ["custom-bot[bot]"],
                "skipLabels": ["blocked"],
                "maxAgeDays": 30
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["allowedActors"], json!(["custom-bot[bot]"]));
    assert_eq!(json["data"]["skipLabels"], json!(["blocked"]));
    assert_eq!(json["data"]["maxAgeDays"], 30);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_update_pr_filters_partial_update() {
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

    // First update to set custom values
    let request1 = Request::builder()
        .method("PUT")
        .uri("/api/pr-filters")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "allowedActors": ["custom-bot[bot]"],
                "skipLabels": ["blocked"],
                "maxAgeDays": 30
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(request1).await.unwrap();

    // Partial update - only update maxAgeDays
    let request2 = Request::builder()
        .method("PUT")
        .uri("/api/pr-filters")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "maxAgeDays": 60
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request2).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    // Should preserve previous values
    assert_eq!(json["data"]["allowedActors"], json!(["custom-bot[bot]"]));
    assert_eq!(json["data"]["skipLabels"], json!(["blocked"]));
    // Should update maxAgeDays
    assert_eq!(json["data"]["maxAgeDays"], 60);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_reset_pr_filters_success() {
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

    // First update to custom values
    let request1 = Request::builder()
        .method("PUT")
        .uri("/api/pr-filters")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "allowedActors": ["custom-bot[bot]"],
                "maxAgeDays": 30
            })
            .to_string(),
        ))
        .unwrap();

    app.clone().oneshot(request1).await.unwrap();

    // Reset to defaults
    let request2 = Request::builder()
        .method("POST")
        .uri("/api/pr-filters/reset")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request2).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);

    // Verify defaults are restored
    let allowed_actors = json["data"]["allowedActors"].as_array().unwrap();
    assert!(allowed_actors.contains(&json!("dependabot[bot]")));
    assert!(allowed_actors.contains(&json!("renovate[bot]")));
    assert!(allowed_actors.contains(&json!("snyk-bot")));

    assert_eq!(json["data"]["maxAgeDays"], serde_json::Value::Null);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_update_pr_filters_requires_auth() {
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
        .method("PUT")
        .uri("/api/pr-filters")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({"maxAgeDays": 30}).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}
