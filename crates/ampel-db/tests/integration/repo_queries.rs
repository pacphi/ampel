/// Integration tests for repository queries
///
/// These tests verify the database layer works correctly with real databases.
/// Each test runs in complete isolation with its own database instance.
///
/// Note: These tests require PostgreSQL because migrations use PostgreSQL-specific
/// features (ALTER TABLE ADD FOREIGN KEY, partial unique indexes). Tests are
/// automatically skipped when running in SQLite mode.
use ampel_db::queries::RepoQueries;
use sea_orm::DbErr;

// Import test utilities from parent module
use super::common::fixtures::create_test_user;
use super::common::TestDb;

/// Test creating a tracked repository
#[tokio::test]
async fn test_create_repository() {
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
        .unwrap();

    let repo = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "provider_123".to_string(),
        "testowner".to_string(),
        "test-repo".to_string(),
        "testowner/test-repo".to_string(),
        Some("A test repository".to_string()),
        "https://github.com/testowner/test-repo".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .expect("Failed to create repository");

    assert_eq!(repo.user_id, user.id);
    assert_eq!(repo.provider, "github");
    assert_eq!(repo.provider_id, "provider_123");
    assert_eq!(repo.owner, "testowner");
    assert_eq!(repo.name, "test-repo");
    assert_eq!(repo.full_name, "testowner/test-repo");
    assert_eq!(repo.description, Some("A test repository".to_string()));
    assert_eq!(repo.default_branch, "main");
    assert!(!repo.is_private);
    assert!(!repo.is_archived);
    assert_eq!(repo.poll_interval_seconds, 300);
    assert!(repo.last_polled_at.is_none());

    test_db.cleanup().await;
}

/// Test finding repositories by user ID
#[tokio::test]
async fn test_find_by_user() {
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
        .unwrap();
    let other_user = create_test_user(db, "other@example.com", "Other User")
        .await
        .unwrap();

    // Create repos for main user
    RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "repo1".to_string(),
        "owner".to_string(),
        "repo-a".to_string(),
        "owner/repo-a".to_string(),
        None,
        "https://github.com/owner/repo-a".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "repo2".to_string(),
        "owner".to_string(),
        "repo-b".to_string(),
        "owner/repo-b".to_string(),
        None,
        "https://github.com/owner/repo-b".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    // Create repo for other user
    RepoQueries::create(
        db,
        other_user.id,
        "github".to_string(),
        "repo3".to_string(),
        "owner".to_string(),
        "repo-c".to_string(),
        "owner/repo-c".to_string(),
        None,
        "https://github.com/owner/repo-c".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    // Find repos for main user
    let repos = RepoQueries::find_by_user_id(db, user.id)
        .await
        .expect("Failed to find repos");

    assert_eq!(repos.len(), 2, "User should have 2 repos");
    for repo in &repos {
        assert_eq!(repo.user_id, user.id);
    }

    // Verify alphabetical ordering by full_name
    assert_eq!(repos[0].full_name, "owner/repo-a");
    assert_eq!(repos[1].full_name, "owner/repo-b");

    test_db.cleanup().await;
}

/// Test finding a repository by UUID
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

    let user = create_test_user(db, "test@example.com", "Test User")
        .await
        .unwrap();

    let created = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "provider_123".to_string(),
        "owner".to_string(),
        "test-repo".to_string(),
        "owner/test-repo".to_string(),
        None,
        "https://github.com/owner/test-repo".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    // Find by ID
    let found = RepoQueries::find_by_id(db, created.id)
        .await
        .expect("Failed to find repo");

    assert!(found.is_some());
    let repo = found.unwrap();
    assert_eq!(repo.id, created.id);
    assert_eq!(repo.name, "test-repo");

    test_db.cleanup().await;
}

/// Test updating last_polled_at timestamp
#[tokio::test]
async fn test_update_last_polled() {
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
        .unwrap();

    let repo = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "provider_123".to_string(),
        "owner".to_string(),
        "test-repo".to_string(),
        "owner/test-repo".to_string(),
        None,
        "https://github.com/owner/test-repo".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    assert!(repo.last_polled_at.is_none(), "Initially not polled");

    // Update last_polled_at
    let updated = RepoQueries::update_last_polled(db, repo.id)
        .await
        .expect("Failed to update last_polled_at");

    assert!(
        updated.last_polled_at.is_some(),
        "Should have last_polled_at set"
    );
    assert!(updated.updated_at > repo.updated_at);

    test_db.cleanup().await;
}

/// Test finding repository by provider ID
#[tokio::test]
async fn test_find_by_provider_id() {
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
        .unwrap();

    let repo = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "provider_456".to_string(),
        "owner".to_string(),
        "test-repo".to_string(),
        "owner/test-repo".to_string(),
        None,
        "https://github.com/owner/test-repo".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    // Find by provider ID
    let found = RepoQueries::find_by_provider_id(db, user.id, "github", "provider_456")
        .await
        .expect("Failed to find repo");

    assert!(found.is_some());
    let found_repo = found.unwrap();
    assert_eq!(found_repo.id, repo.id);
    assert_eq!(found_repo.provider_id, "provider_456");

    // Try with wrong provider
    let not_found = RepoQueries::find_by_provider_id(db, user.id, "gitlab", "provider_456")
        .await
        .expect("Query should succeed");
    assert!(not_found.is_none());

    test_db.cleanup().await;
}

/// Test updating poll interval
#[tokio::test]
async fn test_update_poll_interval() {
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
        .unwrap();

    let repo = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "provider_123".to_string(),
        "owner".to_string(),
        "test-repo".to_string(),
        "owner/test-repo".to_string(),
        None,
        "https://github.com/owner/test-repo".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    assert_eq!(repo.poll_interval_seconds, 300);

    // Update poll interval
    let updated = RepoQueries::update_poll_interval(db, repo.id, 600)
        .await
        .expect("Failed to update poll interval");

    assert_eq!(updated.poll_interval_seconds, 600);
    assert!(updated.updated_at > repo.updated_at);

    test_db.cleanup().await;
}

/// Test finding repositories due for polling
#[tokio::test]
async fn test_find_due_for_polling() {
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
        .unwrap();

    // Create repos - some never polled, some recently polled
    let repo1 = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "repo1".to_string(),
        "owner".to_string(),
        "never-polled".to_string(),
        "owner/never-polled".to_string(),
        None,
        "https://github.com/owner/never-polled".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    let repo2 = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "repo2".to_string(),
        "owner".to_string(),
        "also-never-polled".to_string(),
        "owner/also-never-polled".to_string(),
        None,
        "https://github.com/owner/also-never-polled".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    // Create and immediately poll repo3
    let repo3 = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "repo3".to_string(),
        "owner".to_string(),
        "recently-polled".to_string(),
        "owner/recently-polled".to_string(),
        None,
        "https://github.com/owner/recently-polled".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    RepoQueries::update_last_polled(db, repo3.id).await.unwrap();

    // Find repos due for polling
    let due_repos = RepoQueries::find_due_for_polling(db, 10)
        .await
        .expect("Failed to find due repos");

    // Should include never-polled repos
    assert!(due_repos.len() >= 2, "Should find at least 2 due repos");
    let due_ids: Vec<_> = due_repos.iter().map(|r| r.id).collect();
    assert!(due_ids.contains(&repo1.id));
    assert!(due_ids.contains(&repo2.id));

    test_db.cleanup().await;
}

/// Test deleting a repository
#[tokio::test]
async fn test_delete_repository() {
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
        .unwrap();

    let repo = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "provider_123".to_string(),
        "owner".to_string(),
        "test-repo".to_string(),
        "owner/test-repo".to_string(),
        None,
        "https://github.com/owner/test-repo".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
    .unwrap();

    // Delete repo
    RepoQueries::delete(db, repo.id)
        .await
        .expect("Failed to delete repository");

    // Verify repo is deleted
    let result = RepoQueries::find_by_id(db, repo.id).await.unwrap();
    assert!(result.is_none(), "Repository should be deleted");

    test_db.cleanup().await;
}

/// Test update operations on non-existent repository return error
#[tokio::test]
async fn test_update_nonexistent_repository() {
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

    // Try to update non-existent repo
    let result = RepoQueries::update_last_polled(db, fake_id).await;

    assert!(result.is_err(), "Should fail for non-existent repo");
    match result {
        Err(DbErr::RecordNotFound(msg)) => assert_eq!(msg, "Repository not found"),
        _ => panic!("Expected RecordNotFound error"),
    }

    test_db.cleanup().await;
}

/// Test creating repository with provider account
#[tokio::test]
async fn test_create_with_provider_account() {
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
        .unwrap();

    // Create a provider account
    use super::common::fixtures::create_test_provider_account;
    let provider_account = create_test_provider_account(db, user.id, "github", "Work GitHub", true)
        .await
        .unwrap();

    // Create repo with provider account
    let repo = RepoQueries::create(
        db,
        user.id,
        "github".to_string(),
        "provider_123".to_string(),
        "owner".to_string(),
        "test-repo".to_string(),
        "owner/test-repo".to_string(),
        None,
        "https://github.com/owner/test-repo".to_string(),
        "main".to_string(),
        false,
        false,
        300,
        Some(provider_account.id),
    )
    .await
    .unwrap();

    assert_eq!(repo.provider_account_id, Some(provider_account.id));

    test_db.cleanup().await;
}
