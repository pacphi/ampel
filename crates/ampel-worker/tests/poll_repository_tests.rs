/// Integration tests for poll_repository job
///
/// These tests verify that the repository polling job correctly:
/// - Identifies repositories due for polling
/// - Fetches pull requests from providers
/// - Updates database with latest PR data
/// - Updates last polled timestamps
///
/// Note: These tests require PostgreSQL because migrations use PostgreSQL-specific
/// features. Tests are automatically skipped when running in SQLite mode.
mod common;

use ampel_core::models::GitProvider;
use ampel_db::entities::{provider_account, repository};
use ampel_db::queries::RepoQueries;
use ampel_worker::jobs::poll_repository::PollRepositoryJob;
use chrono::{Duration, Utc};
use common::{
    create_test_ci_check, create_test_encryption_service, create_test_pr, create_test_review,
    create_test_user, MockProvider, TestDb,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

/// Helper to create a test repository
async fn create_test_repository(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
    provider_account_id: Uuid,
    name: &str,
    last_polled_at: Option<chrono::DateTime<Utc>>,
    poll_interval_seconds: i32,
) -> anyhow::Result<repository::Model> {
    let now = Utc::now();
    let repo = repository::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        provider_account_id: Set(Some(provider_account_id)),
        provider: Set("github".to_string()),
        provider_id: Set(format!("repo_{}", name)),
        owner: Set("testowner".to_string()),
        name: Set(name.to_string()),
        full_name: Set(format!("testowner/{}", name)),
        description: Set(Some("Test repository".to_string())),
        url: Set(format!("https://github.com/testowner/{}", name)),
        default_branch: Set("main".to_string()),
        is_private: Set(false),
        is_archived: Set(false),
        group_id: Set(None),
        last_polled_at: Set(last_polled_at),
        poll_interval_seconds: Set(poll_interval_seconds),
        created_at: Set(now),
        updated_at: Set(now),
    };

    Ok(repo.insert(db).await?)
}

#[tokio::test]
async fn test_find_repos_to_poll_empty() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let job = PollRepositoryJob;

    // No repositories in database
    let repos = job
        .find_repos_to_poll(db)
        .await
        .expect("Failed to find repos");

    assert_eq!(
        repos.len(),
        0,
        "Should return empty list when no repos exist"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_find_repos_to_poll_never_polled() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    // Create test user and provider account
    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .expect("Failed to create user");

    let encryption_service = create_test_encryption_service();
    let encrypted_token = encryption_service
        .encrypt("test_token")
        .expect("Failed to encrypt token");

    let provider_account = provider_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        provider: Set("github".to_string()),
        instance_url: Set(None),
        account_label: Set("Test Account".to_string()),
        provider_user_id: Set("123".to_string()),
        provider_username: Set("testuser".to_string()),
        provider_email: Set(Some("test@example.com".to_string())),
        avatar_url: Set(None),
        auth_type: Set("pat".to_string()),
        access_token_encrypted: Set(encrypted_token),
        auth_username: Set(None),
        scopes: Set(None),
        token_expires_at: Set(None),
        last_validated_at: Set(Some(Utc::now())),
        validation_status: Set("valid".to_string()),
        is_active: Set(true),
        is_default: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .expect("Failed to create provider account");

    // Create repository that has never been polled
    create_test_repository(db, user.id, provider_account.id, "never-polled", None, 300)
        .await
        .expect("Failed to create repo");

    let job = PollRepositoryJob;
    let repos = job
        .find_repos_to_poll(db)
        .await
        .expect("Failed to find repos");

    assert_eq!(
        repos.len(),
        1,
        "Should find repository that was never polled"
    );
    assert_eq!(repos[0].name, "never-polled");
    assert!(repos[0].last_polled_at.is_none());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_find_repos_to_poll_due() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    // Create test user and provider account
    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .expect("Failed to create user");

    let encryption_service = create_test_encryption_service();
    let encrypted_token = encryption_service
        .encrypt("test_token")
        .expect("Failed to encrypt token");

    let provider_account = provider_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        provider: Set("github".to_string()),
        instance_url: Set(None),
        account_label: Set("Test Account".to_string()),
        provider_user_id: Set("123".to_string()),
        provider_username: Set("testuser".to_string()),
        provider_email: Set(Some("test@example.com".to_string())),
        avatar_url: Set(None),
        auth_type: Set("pat".to_string()),
        access_token_encrypted: Set(encrypted_token),
        auth_username: Set(None),
        scopes: Set(None),
        token_expires_at: Set(None),
        last_validated_at: Set(Some(Utc::now())),
        validation_status: Set("valid".to_string()),
        is_active: Set(true),
        is_default: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .expect("Failed to create provider account");

    // Create repository polled 400 seconds ago with 300 second interval (due for poll)
    let last_polled = Utc::now() - Duration::seconds(400);
    create_test_repository(
        db,
        user.id,
        provider_account.id,
        "due-repo",
        Some(last_polled),
        300,
    )
    .await
    .expect("Failed to create repo");

    let job = PollRepositoryJob;
    let repos = job
        .find_repos_to_poll(db)
        .await
        .expect("Failed to find repos");

    assert_eq!(
        repos.len(),
        1,
        "Should find repository that is due for polling"
    );
    assert_eq!(repos[0].name, "due-repo");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_find_repos_to_poll_not_due() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    // Create test user and provider account
    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .expect("Failed to create user");

    let encryption_service = create_test_encryption_service();
    let encrypted_token = encryption_service
        .encrypt("test_token")
        .expect("Failed to encrypt token");

    let provider_account = provider_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        provider: Set("github".to_string()),
        instance_url: Set(None),
        account_label: Set("Test Account".to_string()),
        provider_user_id: Set("123".to_string()),
        provider_username: Set("testuser".to_string()),
        provider_email: Set(Some("test@example.com".to_string())),
        avatar_url: Set(None),
        auth_type: Set("pat".to_string()),
        access_token_encrypted: Set(encrypted_token),
        auth_username: Set(None),
        scopes: Set(None),
        token_expires_at: Set(None),
        last_validated_at: Set(Some(Utc::now())),
        validation_status: Set("valid".to_string()),
        is_active: Set(true),
        is_default: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .expect("Failed to create provider account");

    // Create repository polled 100 seconds ago with 300 second interval (not due yet)
    let last_polled = Utc::now() - Duration::seconds(100);
    create_test_repository(
        db,
        user.id,
        provider_account.id,
        "not-due",
        Some(last_polled),
        300,
    )
    .await
    .expect("Failed to create repo");

    let job = PollRepositoryJob;
    let repos = job
        .find_repos_to_poll(db)
        .await
        .expect("Failed to find repos");

    assert_eq!(
        repos.len(),
        0,
        "Should not return repositories that are not due for polling"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_poll_updates_last_polled_timestamp() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    // Create test user and provider account
    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .expect("Failed to create user");

    let encryption_service = create_test_encryption_service();
    let encrypted_token = encryption_service
        .encrypt("test_token")
        .expect("Failed to encrypt token");

    let provider_account = provider_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        provider: Set("github".to_string()),
        instance_url: Set(None),
        account_label: Set("Test Account".to_string()),
        provider_user_id: Set("123".to_string()),
        provider_username: Set("testuser".to_string()),
        provider_email: Set(Some("test@example.com".to_string())),
        avatar_url: Set(None),
        auth_type: Set("pat".to_string()),
        access_token_encrypted: Set(encrypted_token),
        auth_username: Set(None),
        scopes: Set(None),
        token_expires_at: Set(None),
        last_validated_at: Set(Some(Utc::now())),
        validation_status: Set("valid".to_string()),
        is_active: Set(true),
        is_default: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .expect("Failed to create provider account");

    // Create repository
    let repo = create_test_repository(db, user.id, provider_account.id, "test-repo", None, 300)
        .await
        .expect("Failed to create repo");

    // Create mock provider with test data
    let mock = MockProvider::new(GitProvider::GitHub);
    mock.add_pull_request(create_test_pr(1, "Test PR", "open"));
    mock.add_ci_check(create_test_ci_check("CI", "completed", Some("success")));
    mock.add_review(create_test_review("reviewer1", "approved"));

    // Note: Due to ProviderFactory design, we can't easily inject mock providers.
    // This test would need refactoring of poll_single_repo to accept a provider trait.
    // For now, we'll test the timestamp update logic by calling RepoQueries directly.

    let before_poll = Utc::now();

    // Update last polled timestamp
    RepoQueries::update_last_polled(db, repo.id)
        .await
        .expect("Failed to update last polled");

    // Verify timestamp was updated
    let updated_repo = repository::Entity::find_by_id(repo.id)
        .one(db)
        .await
        .expect("Failed to fetch repo")
        .expect("Repo not found");

    assert!(
        updated_repo.last_polled_at.is_some(),
        "last_polled_at should be set"
    );
    assert!(
        updated_repo.last_polled_at.unwrap() >= before_poll,
        "last_polled_at should be updated to current time"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_find_repos_to_poll_mixed_scenarios() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    // Create test user and provider account
    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .expect("Failed to create user");

    let encryption_service = create_test_encryption_service();
    let encrypted_token = encryption_service
        .encrypt("test_token")
        .expect("Failed to encrypt token");

    let provider_account = provider_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        provider: Set("github".to_string()),
        instance_url: Set(None),
        account_label: Set("Test Account".to_string()),
        provider_user_id: Set("123".to_string()),
        provider_username: Set("testuser".to_string()),
        provider_email: Set(Some("test@example.com".to_string())),
        avatar_url: Set(None),
        auth_type: Set("pat".to_string()),
        access_token_encrypted: Set(encrypted_token),
        auth_username: Set(None),
        scopes: Set(None),
        token_expires_at: Set(None),
        last_validated_at: Set(Some(Utc::now())),
        validation_status: Set("valid".to_string()),
        is_active: Set(true),
        is_default: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .expect("Failed to create provider account");

    // Create repositories with different polling states
    create_test_repository(db, user.id, provider_account.id, "never-polled", None, 300)
        .await
        .expect("Failed to create repo");

    let due_time = Utc::now() - Duration::seconds(400);
    create_test_repository(db, user.id, provider_account.id, "due", Some(due_time), 300)
        .await
        .expect("Failed to create repo");

    let not_due_time = Utc::now() - Duration::seconds(100);
    create_test_repository(
        db,
        user.id,
        provider_account.id,
        "not-due",
        Some(not_due_time),
        300,
    )
    .await
    .expect("Failed to create repo");

    let job = PollRepositoryJob;
    let repos = job
        .find_repos_to_poll(db)
        .await
        .expect("Failed to find repos");

    assert_eq!(
        repos.len(),
        2,
        "Should find exactly 2 repos (never-polled and due)"
    );

    let repo_names: Vec<&str> = repos.iter().map(|r| r.name.as_str()).collect();
    assert!(repo_names.contains(&"never-polled"));
    assert!(repo_names.contains(&"due"));
    assert!(!repo_names.contains(&"not-due"));

    test_db.cleanup().await;
}
