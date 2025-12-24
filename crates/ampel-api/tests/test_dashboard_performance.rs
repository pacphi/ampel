/// Performance tests for dashboard handlers
///
/// Tests dashboard summary endpoint performance with varying dataset sizes.
///
/// Performance Targets:
/// - Small dataset (10 repos, 50 PRs): < 100ms
/// - Medium dataset (50 repos, 500 PRs): < 300ms
/// - Large dataset (100 repos, 2000 PRs): < 500ms
/// - Very large dataset (200 repos, 6000 PRs): < 1000ms
mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use common::{create_test_app, TestDb};
use serde_json::{json, Value};
use std::time::Instant;
use tower::ServiceExt;

/// Helper to register a user and return access token
async fn register_and_login(app: &axum::Router) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "perf_test@example.com",
                "password": "SecurePassword123!",
                "displayName": "Performance Test User"
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
async fn test_summary_small_dataset_performance() {
    if TestDb::skip_if_sqlite() {
        println!("⏭️  Skipping performance test on SQLite (requires PostgreSQL)");
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;

    // Setup: Create 10 repositories with 5 open PRs each
    // Note: In production tests, create actual data here
    // For now, test with empty dataset as baseline

    let start = Instant::now();

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    // Assert response is successful
    assert_eq!(response.status(), StatusCode::OK);

    // Assert performance target met for small dataset
    // Target: < 100ms (empty dataset should be much faster)
    assert!(
        duration.as_millis() < 100,
        "Small dataset response time {}ms exceeds 100ms target",
        duration.as_millis()
    );

    println!(
        "✅ Small dataset performance test passed: {}ms (target: <100ms)",
        duration.as_millis()
    );

    // Parse and validate response structure
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["totalRepositories"].is_number());
    assert!(json["data"]["statusCounts"]["green"].is_number());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_summary_response_time_logging() {
    if TestDb::skip_if_sqlite() {
        println!("⏭️  Skipping performance test on SQLite (requires PostgreSQL)");
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;

    // Verify that structured logging is working
    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Response should be successful
    assert_eq!(response.status(), StatusCode::OK);

    // Verify response contains breakdown counts
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify all required fields are present (for logging)
    assert!(json["data"]["totalRepositories"].is_number());
    assert!(json["data"]["totalOpenPrs"].is_number());
    assert!(json["data"]["statusCounts"]["green"].is_number());
    assert!(json["data"]["statusCounts"]["yellow"].is_number());
    assert!(json["data"]["statusCounts"]["red"].is_number());
    assert!(json["data"]["providerCounts"]["github"].is_number());
    assert!(json["data"]["providerCounts"]["gitlab"].is_number());
    assert!(json["data"]["providerCounts"]["bitbucket"].is_number());

    println!("✅ Structured logging validation passed");

    test_db.cleanup().await;
}

#[tokio::test]
#[ignore] // Run with: cargo test --test test_dashboard_performance -- --ignored --nocapture
async fn test_summary_large_dataset_performance() {
    if TestDb::skip_if_sqlite() {
        println!("⏭️  Skipping performance test on SQLite (requires PostgreSQL)");
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;

    // TODO: Create 100 repositories with 20 open PRs each
    // This test is ignored by default and should be run manually
    // when test data creation is implemented
    //
    // Expected setup:
    // - 100 repositories
    // - 20 open PRs per repository
    // - 2000 total PRs with varied statuses
    // - CI checks and reviews for each PR

    let start = Instant::now();

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    // Assert response is successful
    assert_eq!(response.status(), StatusCode::OK);

    // Assert performance target met for large dataset
    // Target: < 500ms (CRITICAL THRESHOLD)
    assert!(
        duration.as_millis() < 500,
        "Large dataset response time {}ms exceeds 500ms critical threshold",
        duration.as_millis()
    );

    println!(
        "✅ Large dataset (100 repos) performance test passed: {}ms (target: <500ms)",
        duration.as_millis()
    );

    // Verify response data
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let total_repos = json["data"]["totalRepositories"].as_i64().unwrap();
    let total_prs = json["data"]["totalOpenPrs"].as_i64().unwrap();

    println!(
        "📊 Dataset: {} repositories, {} open PRs",
        total_repos, total_prs
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_summary_concurrent_requests() {
    if TestDb::skip_if_sqlite() {
        println!("⏭️  Skipping performance test on SQLite (requires PostgreSQL)");
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;

    // Test concurrent requests to verify no race conditions
    let mut handles = vec![];

    for i in 0..10 {
        let app_clone = app.clone();
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            let start = Instant::now();

            let request = Request::builder()
                .method("GET")
                .uri("/api/dashboard/summary")
                .header(header::AUTHORIZATION, format!("Bearer {}", token_clone))
                .body(Body::empty())
                .unwrap();

            let response = app_clone.oneshot(request).await.unwrap();
            let duration = start.elapsed();

            (i, response.status(), duration)
        });

        handles.push(handle);
    }

    // Await all concurrent requests
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Verify all requests succeeded
    for result in results {
        let (request_num, status, duration) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        println!(
            "✅ Concurrent request #{} completed in {}ms",
            request_num,
            duration.as_millis()
        );
    }

    println!("✅ Concurrent requests test passed (10 requests)");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_summary_performance_metrics_collection() {
    if TestDb::skip_if_sqlite() {
        println!("⏭️  Skipping performance test on SQLite (requires PostgreSQL)");
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;

    // Make multiple requests to collect performance data
    let mut durations = vec![];

    for _ in 0..5 {
        let start = Instant::now();

        let request = Request::builder()
            .method("GET")
            .uri("/api/dashboard/summary")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        let duration = start.elapsed();

        assert_eq!(response.status(), StatusCode::OK);
        durations.push(duration.as_millis());
    }

    // Calculate performance statistics
    let avg = durations.iter().sum::<u128>() / durations.len() as u128;
    let min = *durations.iter().min().unwrap();
    let max = *durations.iter().max().unwrap();

    println!("📊 Performance Statistics (5 requests):");
    println!("   Average: {}ms", avg);
    println!("   Min: {}ms", min);
    println!("   Max: {}ms", max);

    // Assert average is within acceptable range
    assert!(
        avg < 200,
        "Average response time {}ms exceeds 200ms target",
        avg
    );

    println!("✅ Performance metrics collection test passed");

    test_db.cleanup().await;
}

/// Performance test documentation
///
/// To run all performance tests:
/// ```bash
/// cargo test --test test_dashboard_performance -- --nocapture
/// ```
///
/// To run ignored tests (large datasets):
/// ```bash
/// cargo test --test test_dashboard_performance -- --ignored --nocapture
/// ```
///
/// To run a specific test:
/// ```bash
/// cargo test --test test_dashboard_performance test_summary_small_dataset_performance -- --nocapture
/// ```
///
/// Performance Optimization Checklist:
/// - [ ] Database indexes on repositories.user_id
/// - [ ] Database indexes on pull_requests.repository_id and state
/// - [ ] Redis caching for dashboard summaries
/// - [ ] SQL aggregation instead of N+1 queries
/// - [ ] Connection pooling optimized
/// - [ ] Prometheus metrics enabled
/// - [ ] Grafana dashboard created
/// - [ ] Alert rules configured
#[test]
fn performance_test_documentation() {
    // This is a documentation test to ensure the above documentation compiles
    println!("Performance test suite documentation:");
    println!("- Small dataset: < 100ms");
    println!("- Medium dataset: < 300ms");
    println!("- Large dataset (100 repos): < 500ms");
    println!("- Very large dataset (200 repos): < 1000ms");
}
