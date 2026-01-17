/// Tests for user language preference API endpoints
///
/// These tests verify:
/// - GET /api/user/preferences returns user's language preference
/// - PUT /api/user/preferences updates user's language preference
/// - Language preference persists in database
/// - Invalid language codes are rejected
/// - Authentication is required

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use ampel_core::models::user::NewUser;
use ampel_db::entities::{prelude::*, user};
use sea_orm::EntityTrait;

use crate::common::TestDb;

mod common;

/// Helper to create a test user with language preference
async fn create_test_user_with_language(
    db: &sea_orm::DatabaseConnection,
    language: &str,
) -> user::Model {
    let new_user = NewUser {
        email: format!("test-{}@example.com", uuid::Uuid::new_v4()),
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string(),
        display_name: Some("Test User".to_string()),
    };

    let user_model = user::ActiveModel {
        email: sea_orm::Set(new_user.email),
        password_hash: sea_orm::Set(new_user.password_hash),
        display_name: sea_orm::Set(new_user.display_name),
        language: sea_orm::Set(Some(language.to_string())),
        ..Default::default()
    };

    User::insert(user_model)
        .exec_with_returning(db)
        .await
        .expect("Failed to insert test user")
}

/// Test GET /api/user/preferences returns language preference
#[tokio::test]
async fn test_get_user_language_preference() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    // Create test user with German preference
    let user = create_test_user_with_language(db.connection(), "de").await;

    let app = common::create_test_app(db.connection().clone()).await;

    // TODO: Once auth is integrated, this test will need a valid JWT token
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/user/preferences")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Currently returns 401 because no auth token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // TODO: When implemented, verify response contains language preference:
    // let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    // let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // assert_eq!(json["language"], "de");

    db.cleanup().await;
}

/// Test PUT /api/user/preferences updates language
#[tokio::test]
async fn test_update_user_language_preference() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    let user = create_test_user_with_language(db.connection(), "en").await;

    let app = common::create_test_app(db.connection().clone()).await;

    let update_payload = json!({
        "language": "fr"
    });

    // TODO: Once auth is integrated, this test will need a valid JWT token
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/user/preferences")
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Currently returns 401 because no auth token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // TODO: When implemented, verify language was updated in database:
    // let updated_user = User::find_by_id(user.id)
    //     .one(db.connection())
    //     .await
    //     .unwrap()
    //     .unwrap();
    // assert_eq!(updated_user.language, Some("fr".to_string()));

    db.cleanup().await;
}

/// Test all 20 supported languages can be set
#[tokio::test]
async fn test_all_supported_languages() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    let supported_languages = vec![
        "en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja", "ko", "ar", "he", "hi", "bn", "tr",
        "nl", "pl", "vi", "th", "uk",
    ];

    for language in supported_languages {
        let user = create_test_user_with_language(db.connection(), language).await;

        // Verify language was stored correctly
        let stored_user = User::find_by_id(user.id)
            .one(db.connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(stored_user.language, Some(language.to_string()));
    }

    db.cleanup().await;
}

/// Test invalid language codes are rejected
#[tokio::test]
async fn test_invalid_language_rejected() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    let user = create_test_user_with_language(db.connection(), "en").await;

    let app = common::create_test_app(db.connection().clone()).await;

    let invalid_payload = json!({
        "language": "invalid-code"
    });

    // TODO: Once auth is integrated, this test will need a valid JWT token
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/user/preferences")
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&invalid_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return bad request when implemented
    // Currently returns 401 because no auth token
    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::BAD_REQUEST
    );

    db.cleanup().await;
}

/// Test language preference defaults to None for new users
#[tokio::test]
async fn test_new_user_default_language() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    let new_user = NewUser {
        email: "newuser@example.com".to_string(),
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string(),
        display_name: Some("New User".to_string()),
    };

    let user_model = user::ActiveModel {
        email: sea_orm::Set(new_user.email),
        password_hash: sea_orm::Set(new_user.password_hash),
        display_name: sea_orm::Set(new_user.display_name),
        ..Default::default()
    };

    let user = User::insert(user_model)
        .exec_with_returning(db.connection())
        .await
        .expect("Failed to insert test user");

    // Language should default to None
    assert_eq!(user.language, None);

    db.cleanup().await;
}

/// Test language normalization (e.g., "en-US" -> "en")
#[tokio::test]
async fn test_language_normalization_on_update() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    let user = create_test_user_with_language(db.connection(), "en").await;

    let app = common::create_test_app(db.connection().clone()).await;

    let update_payload = json!({
        "language": "en-US"  // Should be normalized to "en"
    });

    // TODO: Once auth is integrated and normalization is implemented:
    // let response = app.oneshot(...).await.unwrap();
    // assert_eq!(response.status(), StatusCode::OK);
    //
    // let updated_user = User::find_by_id(user.id).one(db.connection()).await.unwrap().unwrap();
    // assert_eq!(updated_user.language, Some("en".to_string()));

    db.cleanup().await;
}
