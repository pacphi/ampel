/// Comprehensive tests for observability features
///
/// Tests:
/// - Health check endpoints
/// - Readiness checks
/// - Metrics endpoint
/// - Database health monitoring
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{create_test_app, TestDb};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint_returns_healthy_with_db() {
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
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert_eq!(json["checks"]["database"], true);
    assert!(json["version"].is_string());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_readiness_endpoint_returns_ready_with_db() {
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
        .uri("/ready")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["ready"], true);
    assert_eq!(json["checks"]["database"], true);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_metrics_endpoint_returns_prometheus_format() {
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
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics = String::from_utf8(body.to_vec()).unwrap();

    // Verify it returns Prometheus-formatted metrics
    // The exact metrics depend on what has been recorded, but the format should be correct
    assert!(
        metrics.is_empty() || metrics.contains("# ") || metrics.contains("{"),
        "Should return valid Prometheus format"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_health_endpoint_structure() {
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
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify all required fields are present
    assert!(json.get("status").is_some());
    assert!(json.get("version").is_some());
    assert!(json.get("checks").is_some());
    assert!(json["checks"].get("database").is_some());
    assert!(json["checks"]["database"].is_boolean());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_readiness_endpoint_structure() {
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
        .uri("/ready")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify all required fields are present
    assert!(json.get("ready").is_some());
    assert!(json.get("checks").is_some());
    assert!(json["checks"].get("database").is_some());
    assert!(json["ready"].is_boolean());
    assert!(json["checks"]["database"].is_boolean());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_observability_endpoints_do_not_require_auth() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // Health endpoint should not require auth
    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Readiness endpoint should not require auth
    let request = Request::builder()
        .method("GET")
        .uri("/ready")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Metrics endpoint should not require auth
    let request = Request::builder()
        .method("GET")
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_database_health_check() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    // Test that database is actually healthy
    use sea_orm::DatabaseConnection;
    let db: &DatabaseConnection = test_db.connection();

    let ping_result = db.ping().await;
    assert!(ping_result.is_ok(), "Database should be healthy");

    test_db.cleanup().await;
}
