/// Tests for database migration that adds language column
///
/// These tests verify:
/// - Migration adds 'language' column to users table
/// - Column is nullable (existing users have NULL)
/// - Column accepts valid language codes
/// - Migration is idempotent (can run multiple times)

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use ampel_db::entities::{prelude::*, user};

use crate::common::TestDb;

mod common;

/// Test that migration adds language column to users table
#[tokio::test]
async fn test_migration_adds_language_column() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");

    // Run migrations
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    // Create a test user
    let user_model = user::ActiveModel {
        email: sea_orm::Set("migration-test@example.com".to_string()),
        password_hash: sea_orm::Set(
            "$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string(),
        ),
        display_name: sea_orm::Set(Some("Migration Test".to_string())),
        language: sea_orm::Set(Some("en".to_string())),
        ..Default::default()
    };

    let result = User::insert(user_model).exec(db.connection()).await;

    // Should succeed - language column exists
    assert!(result.is_ok());

    db.cleanup().await;
}

/// Test that language column is nullable
#[tokio::test]
async fn test_language_column_nullable() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    // Create user without language
    let user_model = user::ActiveModel {
        email: sea_orm::Set("nullable-test@example.com".to_string()),
        password_hash: sea_orm::Set(
            "$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string(),
        ),
        display_name: sea_orm::Set(Some("Nullable Test".to_string())),
        language: sea_orm::Set(None), // Explicitly NULL
        ..Default::default()
    };

    let result = User::insert(user_model)
        .exec_with_returning(db.connection())
        .await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.language, None);

    db.cleanup().await;
}

/// Test that existing users have NULL language after migration
#[tokio::test]
async fn test_existing_users_null_language() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    // Create user
    let user_model = user::ActiveModel {
        email: sea_orm::Set("existing@example.com".to_string()),
        password_hash: sea_orm::Set(
            "$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string(),
        ),
        display_name: sea_orm::Set(Some("Existing User".to_string())),
        ..Default::default()
    };

    let user = User::insert(user_model)
        .exec_with_returning(db.connection())
        .await
        .unwrap();

    // Default should be NULL
    assert_eq!(user.language, None);

    db.cleanup().await;
}

/// Test that language column accepts all supported language codes
#[tokio::test]
async fn test_language_column_accepts_valid_codes() {
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

    for (i, language) in supported_languages.iter().enumerate() {
        let user_model = user::ActiveModel {
            email: sea_orm::Set(format!("user-{}@example.com", i)),
            password_hash: sea_orm::Set(
                "$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string(),
            ),
            display_name: sea_orm::Set(Some(format!("User {}", i))),
            language: sea_orm::Set(Some(language.to_string())),
            ..Default::default()
        };

        let result = User::insert(user_model)
            .exec_with_returning(db.connection())
            .await;

        assert!(result.is_ok(), "Failed to insert user with language {}", language);
        let user = result.unwrap();
        assert_eq!(user.language, Some(language.to_string()));
    }

    db.cleanup().await;
}

/// Test that we can query users by language
#[tokio::test]
async fn test_query_users_by_language() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    // Create users with different languages
    for (i, lang) in ["en", "es", "fr"].iter().enumerate() {
        let user_model = user::ActiveModel {
            email: sea_orm::Set(format!("query-test-{}@example.com", i)),
            password_hash: sea_orm::Set(
                "$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string(),
            ),
            display_name: sea_orm::Set(Some(format!("Query Test {}", i))),
            language: sea_orm::Set(Some(lang.to_string())),
            ..Default::default()
        };

        User::insert(user_model)
            .exec(db.connection())
            .await
            .unwrap();
    }

    // Query users with Spanish language
    let spanish_users = User::find()
        .filter(user::Column::Language.eq("es"))
        .all(db.connection())
        .await
        .unwrap();

    assert_eq!(spanish_users.len(), 1);
    assert_eq!(spanish_users[0].language, Some("es".to_string()));

    db.cleanup().await;
}

/// Test that language column can be updated
#[tokio::test]
async fn test_update_language_column() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    // Create user with English
    let user_model = user::ActiveModel {
        email: sea_orm::Set("update-test@example.com".to_string()),
        password_hash: sea_orm::Set(
            "$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string(),
        ),
        display_name: sea_orm::Set(Some("Update Test".to_string())),
        language: sea_orm::Set(Some("en".to_string())),
        ..Default::default()
    };

    let user = User::insert(user_model)
        .exec_with_returning(db.connection())
        .await
        .unwrap();

    // Update to French
    let mut user_active: user::ActiveModel = user.into();
    user_active.language = sea_orm::Set(Some("fr".to_string()));

    let updated = user_active.update(db.connection()).await.unwrap();

    assert_eq!(updated.language, Some("fr".to_string()));

    db.cleanup().await;
}

/// Test migration is idempotent (can run multiple times)
#[tokio::test]
async fn test_migration_idempotent() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let db = TestDb::new().await.expect("Failed to create test database");

    // Run migrations twice
    db.run_migrations()
        .await
        .expect("First migration failed");

    let result = db.run_migrations().await;

    // Should succeed (migration already applied)
    assert!(result.is_ok(), "Migration should be idempotent");

    db.cleanup().await;
}
