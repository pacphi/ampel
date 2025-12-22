/// Integration tests for user queries
///
/// These tests verify the database layer works correctly with real databases.
/// Each test runs in complete isolation with its own database instance.
///
/// Note: These tests require PostgreSQL because migrations use PostgreSQL-specific
/// features (ALTER TABLE ADD FOREIGN KEY, partial unique indexes). Tests are
/// automatically skipped when running in SQLite mode.
use ampel_db::queries::UserQueries;
use sea_orm::DbErr;

// Import test utilities from parent module
use super::common::fixtures::create_test_user;
use super::common::TestDb;

/// Test creating a new user with valid data
#[tokio::test]
async fn test_create_user() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = UserQueries::create(
        db,
        "newuser@example.com".to_string(),
        "$argon2id$v=19$m=19456,t=2,p=1$test$hash".to_string(),
        Some("New User".to_string()),
    )
    .await
    .expect("Failed to create user");

    assert_eq!(user.email, "newuser@example.com");
    assert_eq!(user.display_name, Some("New User".to_string()));
    assert!(user.password_hash.starts_with("$argon2id$"));
    assert!(user.created_at <= user.updated_at);

    test_db.cleanup().await;
}

/// Test finding a user by email returns the correct user
#[tokio::test]
async fn test_find_by_email() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    // Create test user
    let created_user = create_test_user(db, "test@example.com", "Test User")
        .await
        .expect("Failed to create user");

    // Find user by email
    let found_user = UserQueries::find_by_email(db, "test@example.com")
        .await
        .expect("Failed to find user");

    assert!(found_user.is_some(), "User should be found");
    let user = found_user.unwrap();
    assert_eq!(user.id, created_user.id);
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.display_name, Some("Test User".to_string()));

    test_db.cleanup().await;
}

/// Test finding a user by UUID returns the correct user
#[tokio::test]
async fn test_find_by_id() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    // Create test user
    let created_user = create_test_user(db, "test@example.com", "Test User")
        .await
        .expect("Failed to create user");

    // Find user by ID
    let found_user = UserQueries::find_by_id(db, created_user.id)
        .await
        .expect("Failed to find user");

    assert!(found_user.is_some(), "User should be found");
    let user = found_user.unwrap();
    assert_eq!(user.id, created_user.id);
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.display_name, Some("Test User".to_string()));

    test_db.cleanup().await;
}

/// Test finding a user by unknown email returns None
#[tokio::test]
async fn test_find_by_email_not_found() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    // Try to find non-existent user
    let result = UserQueries::find_by_email(db, "nonexistent@example.com")
        .await
        .expect("Query should succeed");

    assert!(result.is_none(), "Should return None for unknown email");

    test_db.cleanup().await;
}

/// Test updating user profile fields (email and display name)
#[tokio::test]
async fn test_update_profile() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    // Create test user
    let user = create_test_user(db, "original@example.com", "Original Name")
        .await
        .expect("Failed to create user");

    let original_updated_at = user.updated_at;

    // Update email and display name
    let updated = UserQueries::update_profile(
        db,
        user.id,
        Some("newemail@example.com".to_string()),
        Some(Some("Updated Name".to_string())),
    )
    .await
    .expect("Failed to update profile");

    assert_eq!(updated.id, user.id);
    assert_eq!(updated.email, "newemail@example.com");
    assert_eq!(updated.display_name, Some("Updated Name".to_string()));
    assert!(updated.updated_at > original_updated_at);

    // Update only display name
    let updated2 = UserQueries::update_profile(db, user.id, None, Some(None))
        .await
        .expect("Failed to update profile");

    assert_eq!(updated2.email, "newemail@example.com"); // Email unchanged
    assert_eq!(updated2.display_name, None); // Display name cleared

    test_db.cleanup().await;
}

/// Test updating user display name only
#[tokio::test]
async fn test_update_display_name() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "Original Name")
        .await
        .expect("Failed to create user");

    let updated = UserQueries::update_display_name(db, user.id, Some("New Name".to_string()))
        .await
        .expect("Failed to update display name");

    assert_eq!(updated.display_name, Some("New Name".to_string()));
    assert!(updated.updated_at > user.updated_at);

    test_db.cleanup().await;
}

/// Test updating user avatar URL
#[tokio::test]
async fn test_update_avatar() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "Test User")
        .await
        .expect("Failed to create user");

    assert_eq!(user.avatar_url, None);

    let updated = UserQueries::update_avatar(
        db,
        user.id,
        Some("https://example.com/new-avatar.png".to_string()),
    )
    .await
    .expect("Failed to update avatar");

    assert_eq!(
        updated.avatar_url,
        Some("https://example.com/new-avatar.png".to_string())
    );
    assert!(updated.updated_at > user.updated_at);

    test_db.cleanup().await;
}

/// Test updating user password hash
#[tokio::test]
async fn test_update_password() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "Test User")
        .await
        .expect("Failed to create user");

    let original_hash = user.password_hash.clone();

    let updated = UserQueries::update_password(db, user.id, "$argon2id$v=19$new$hash".to_string())
        .await
        .expect("Failed to update password");

    assert_ne!(updated.password_hash, original_hash);
    assert_eq!(updated.password_hash, "$argon2id$v=19$new$hash");
    assert!(updated.updated_at > user.updated_at);

    test_db.cleanup().await;
}

/// Test updating non-existent user returns error
#[tokio::test]
async fn test_update_nonexistent_user() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let fake_id = uuid::Uuid::new_v4();

    let result = UserQueries::update_display_name(db, fake_id, Some("Name".to_string())).await;

    assert!(result.is_err(), "Should fail for non-existent user");
    match result {
        Err(DbErr::RecordNotFound(msg)) => assert_eq!(msg, "User not found"),
        _ => panic!("Expected RecordNotFound error"),
    }

    test_db.cleanup().await;
}

/// Test deleting a user
#[tokio::test]
async fn test_delete_user() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "Test User")
        .await
        .expect("Failed to create user");

    // Delete user
    UserQueries::delete(db, user.id)
        .await
        .expect("Failed to delete user");

    // Verify user is deleted
    let result = UserQueries::find_by_id(db, user.id).await.unwrap();
    assert!(result.is_none(), "User should be deleted");

    test_db.cleanup().await;
}
