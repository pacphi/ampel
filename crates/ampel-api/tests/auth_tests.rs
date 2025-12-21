/// Integration tests for authentication handlers
///
/// These tests verify the auth endpoints work correctly with real databases.
/// Tests use Axum's testing utilities to directly test the handler functions
/// without starting a full server.
///
/// Note: These tests require PostgreSQL because migrations use PostgreSQL-specific
/// features. Tests are automatically skipped when running in SQLite mode.
mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use common::{create_test_app, TestDb};
use serde_json::{json, Value};
use tower::ServiceExt;

/// Test successful user registration with valid data
#[tokio::test]
async fn test_register_success() {
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
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "newuser@example.com",
                "password": "SecurePassword123!",
                "displayName": "New User"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Should return 201 CREATED"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["accessToken"].is_string());
    assert!(json["data"]["refreshToken"].is_string());
    assert_eq!(json["data"]["tokenType"], "Bearer");
    assert!(json["data"]["expiresIn"].is_number());

    test_db.cleanup().await;
}

/// Test registration fails with duplicate email
#[tokio::test]
async fn test_register_duplicate_email() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // First registration
    let request1 = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "duplicate@example.com",
                "password": "SecurePassword123!",
                "displayName": "User One"
            })
            .to_string(),
        ))
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::CREATED);

    // Second registration with same email
    let request2 = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "duplicate@example.com",
                "password": "DifferentPassword123!",
                "displayName": "User Two"
            })
            .to_string(),
        ))
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();

    assert_eq!(
        response2.status(),
        StatusCode::BAD_REQUEST,
        "Should return 400 for duplicate email"
    );

    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Email already registered"));

    test_db.cleanup().await;
}

/// Test registration fails with invalid email format
#[tokio::test]
async fn test_register_invalid_email() {
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
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "not-an-email",
                "password": "SecurePassword123!",
                "displayName": "Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should return 400 for invalid email"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["error"], "Validation failed");
    assert!(json["details"].is_array());

    test_db.cleanup().await;
}

/// Test registration fails with weak password (less than 8 characters)
#[tokio::test]
async fn test_register_weak_password() {
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
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "user@example.com",
                "password": "short",
                "displayName": "Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should return 400 for weak password"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["error"], "Validation failed");
    assert!(json["details"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d.as_str().unwrap().contains("at least 8 characters")));

    test_db.cleanup().await;
}

/// Test successful login with valid credentials
#[tokio::test]
async fn test_login_success() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // Register user first
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "logintest@example.com",
                "password": "SecurePassword123!",
                "displayName": "Login Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Now login
    let login_request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "logintest@example.com",
                "password": "SecurePassword123!"
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = app.oneshot(login_request).await.unwrap();

    assert_eq!(
        login_response.status(),
        StatusCode::OK,
        "Should return 200 OK"
    );

    let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["accessToken"].is_string());
    assert!(json["data"]["refreshToken"].is_string());
    assert_eq!(json["data"]["tokenType"], "Bearer");

    test_db.cleanup().await;
}

/// Test login fails with invalid password
#[tokio::test]
async fn test_login_invalid_password() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // Register user first
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "passwordtest@example.com",
                "password": "CorrectPassword123!",
                "displayName": "Password Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Try to login with wrong password
    let login_request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "passwordtest@example.com",
                "password": "WrongPassword123!"
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = app.oneshot(login_request).await.unwrap();

    assert_eq!(
        login_response.status(),
        StatusCode::UNAUTHORIZED,
        "Should return 401 for invalid password"
    );

    let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Invalid email or password"));

    test_db.cleanup().await;
}

/// Test login fails for unknown user
#[tokio::test]
async fn test_login_unknown_user() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    let login_request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "nonexistent@example.com",
                "password": "AnyPassword123!"
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = app.oneshot(login_request).await.unwrap();

    assert_eq!(
        login_response.status(),
        StatusCode::UNAUTHORIZED,
        "Should return 401 for unknown user"
    );

    let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Invalid email or password"));

    test_db.cleanup().await;
}

/// Test refresh token successfully generates new access token
#[tokio::test]
async fn test_refresh_valid_token() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // Register and get tokens
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "refreshtest@example.com",
                "password": "SecurePassword123!",
                "displayName": "Refresh Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(register_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let refresh_token = json["data"]["refreshToken"].as_str().unwrap();

    // Use refresh token to get new tokens
    let refresh_request = Request::builder()
        .method("POST")
        .uri("/api/auth/refresh")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "refreshToken": refresh_token
            })
            .to_string(),
        ))
        .unwrap();

    let refresh_response = app.oneshot(refresh_request).await.unwrap();

    assert_eq!(
        refresh_response.status(),
        StatusCode::OK,
        "Should return 200 OK"
    );

    let body = axum::body::to_bytes(refresh_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["accessToken"].is_string());
    assert!(json["data"]["refreshToken"].is_string());

    test_db.cleanup().await;
}

/// Test refresh endpoint rejects access token (wrong token type)
#[tokio::test]
async fn test_refresh_with_access_token() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // Register and get tokens
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "accesstokentest@example.com",
                "password": "SecurePassword123!",
                "displayName": "Access Token Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    let body = axum::body::to_bytes(register_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let access_token = json["data"]["accessToken"].as_str().unwrap();

    // Try to use access token for refresh (should fail)
    let refresh_request = Request::builder()
        .method("POST")
        .uri("/api/auth/refresh")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "refreshToken": access_token
            })
            .to_string(),
        ))
        .unwrap();

    let refresh_response = app.oneshot(refresh_request).await.unwrap();

    assert_eq!(
        refresh_response.status(),
        StatusCode::UNAUTHORIZED,
        "Should return 401 when using access token for refresh"
    );

    let body = axum::body::to_bytes(refresh_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Invalid or expired refresh token"));

    test_db.cleanup().await;
}

/// Test refresh endpoint rejects expired token
#[tokio::test]
async fn test_refresh_expired_token() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // Create an expired token (this is a fake token that will fail validation)
    let expired_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjF9.invalid";

    let refresh_request = Request::builder()
        .method("POST")
        .uri("/api/auth/refresh")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "refreshToken": expired_token
            })
            .to_string(),
        ))
        .unwrap();

    let refresh_response = app.oneshot(refresh_request).await.unwrap();

    assert_eq!(
        refresh_response.status(),
        StatusCode::UNAUTHORIZED,
        "Should return 401 for expired/invalid token"
    );

    let body = axum::body::to_bytes(refresh_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Invalid or expired refresh token"));

    test_db.cleanup().await;
}

/// Test authenticated user can get their profile
#[tokio::test]
async fn test_me_authenticated() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // Register user
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "profiletest@example.com",
                "password": "SecurePassword123!",
                "displayName": "Profile Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    let body = axum::body::to_bytes(register_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let access_token = json["data"]["accessToken"].as_str().unwrap();

    // Get profile
    let me_request = Request::builder()
        .method("GET")
        .uri("/api/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let me_response = app.oneshot(me_request).await.unwrap();

    assert_eq!(me_response.status(), StatusCode::OK, "Should return 200 OK");

    let body = axum::body::to_bytes(me_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["email"], "profiletest@example.com");
    assert_eq!(json["data"]["displayName"], "Profile Test User");
    assert!(json["data"]["id"].is_string());
    assert!(json["data"]["createdAt"].is_string());

    test_db.cleanup().await;
}

/// Test unauthenticated request to /me returns 401
#[tokio::test]
async fn test_me_unauthenticated() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // Try to get profile without auth header
    let me_request = Request::builder()
        .method("GET")
        .uri("/api/auth/me")
        .body(Body::empty())
        .unwrap();

    let me_response = app.oneshot(me_request).await.unwrap();

    assert_eq!(
        me_response.status(),
        StatusCode::UNAUTHORIZED,
        "Should return 401 without auth header"
    );

    test_db.cleanup().await;
}

/// Test authenticated user can update their profile
#[tokio::test]
async fn test_update_profile_success() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    // Register user
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "updatetest@example.com",
                "password": "SecurePassword123!",
                "displayName": "Original Name"
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    let body = axum::body::to_bytes(register_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let access_token = json["data"]["accessToken"].as_str().unwrap();

    // Update profile
    let update_request = Request::builder()
        .method("PUT")
        .uri("/api/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "displayName": "Updated Name"
            })
            .to_string(),
        ))
        .unwrap();

    let update_response = app.clone().oneshot(update_request).await.unwrap();

    assert_eq!(
        update_response.status(),
        StatusCode::OK,
        "Should return 200 OK"
    );

    let body = axum::body::to_bytes(update_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["displayName"], "Updated Name");
    assert_eq!(json["data"]["email"], "updatetest@example.com");

    test_db.cleanup().await;
}

/// Test logout endpoint returns 204 No Content
#[tokio::test]
async fn test_logout() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let app = create_test_app(test_db.connection().clone()).await;

    let logout_request = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .body(Body::empty())
        .unwrap();

    let logout_response = app.oneshot(logout_request).await.unwrap();

    assert_eq!(
        logout_response.status(),
        StatusCode::NO_CONTENT,
        "Should return 204 No Content"
    );

    test_db.cleanup().await;
}
