/// Integration tests for pull request queries
///
/// These tests verify the database layer works correctly with real databases.
/// Each test runs in complete isolation with its own database instance.
///
/// Note: These tests require PostgreSQL because migrations use PostgreSQL-specific
/// features (ALTER TABLE ADD FOREIGN KEY, partial unique indexes). Tests are
/// automatically skipped when running in SQLite mode.
use ampel_db::queries::PrQueries;
use chrono::Utc;
use sea_orm::DbErr;

// Import test utilities from parent module
use super::common::fixtures::create_test_user;
use super::common::TestDb;

/// Helper to create a test repository
async fn create_test_repo(
    db: &sea_orm::DatabaseConnection,
    user_id: uuid::Uuid,
    name: &str,
) -> Result<ampel_db::entities::repository::Model, sea_orm::DbErr> {
    use ampel_db::queries::RepoQueries;

    RepoQueries::create(
        db,
        user_id,
        "github".to_string(),
        format!("provider_{}", name),
        "testowner".to_string(),
        name.to_string(),
        format!("testowner/{}", name),
        Some("Test repository".to_string()),
        format!("https://github.com/testowner/{}", name),
        "main".to_string(),
        false,
        false,
        300,
        None,
    )
    .await
}

/// Helper to create a test PR
#[allow(clippy::too_many_arguments)]
async fn create_test_pr(
    db: &sea_orm::DatabaseConnection,
    repository_id: uuid::Uuid,
    number: i32,
    title: &str,
    state: &str,
) -> Result<ampel_db::entities::pull_request::Model, sea_orm::DbErr> {
    let now = Utc::now();
    PrQueries::upsert(
        db,
        repository_id,
        "github".to_string(),
        format!("pr_{}", number),
        number,
        title.to_string(),
        Some("Test PR description".to_string()),
        format!("https://github.com/owner/repo/pull/{}", number),
        state.to_string(),
        "feature-branch".to_string(),
        "main".to_string(),
        "testauthor".to_string(),
        Some("https://github.com/testauthor.png".to_string()),
        false,
        Some(true),
        false,
        50,
        20,
        3,
        5,
        2,
        now,
        now,
        None,
        None,
    )
    .await
}

/// Test finding a PR by UUID
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
    let repo = create_test_repo(db, user.id, "test-repo").await.unwrap();
    let pr = create_test_pr(db, repo.id, 1, "Test PR", "open")
        .await
        .unwrap();

    // Find PR by ID
    let found = PrQueries::find_by_id(db, pr.id)
        .await
        .expect("Failed to find PR");

    assert!(found.is_some());
    let found_pr = found.unwrap();
    assert_eq!(found_pr.id, pr.id);
    assert_eq!(found_pr.title, "Test PR");
    assert_eq!(found_pr.number, 1);

    test_db.cleanup().await;
}

/// Test finding open PRs for a repository
#[tokio::test]
async fn test_find_open_by_repository() {
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
    let repo = create_test_repo(db, user.id, "test-repo").await.unwrap();

    // Create multiple PRs with different states
    create_test_pr(db, repo.id, 1, "Open PR 1", "open")
        .await
        .unwrap();
    create_test_pr(db, repo.id, 2, "Open PR 2", "open")
        .await
        .unwrap();
    create_test_pr(db, repo.id, 3, "Closed PR", "closed")
        .await
        .unwrap();
    create_test_pr(db, repo.id, 4, "Merged PR", "merged")
        .await
        .unwrap();

    // Find open PRs
    let open_prs = PrQueries::find_open_by_repository(db, repo.id)
        .await
        .expect("Failed to find open PRs");

    assert_eq!(open_prs.len(), 2, "Should find 2 open PRs");
    for pr in &open_prs {
        assert_eq!(pr.state, "open");
        assert_eq!(pr.repository_id, repo.id);
    }

    test_db.cleanup().await;
}

/// Test finding open PRs for a user with pagination
#[tokio::test]
async fn test_find_open_by_user() {
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
    let repo1 = create_test_repo(db, user.id, "repo1").await.unwrap();
    let repo2 = create_test_repo(db, user.id, "repo2").await.unwrap();

    // Create open PRs across multiple repos
    create_test_pr(db, repo1.id, 1, "PR 1", "open")
        .await
        .unwrap();
    create_test_pr(db, repo1.id, 2, "PR 2", "open")
        .await
        .unwrap();
    create_test_pr(db, repo2.id, 1, "PR 3", "open")
        .await
        .unwrap();
    create_test_pr(db, repo2.id, 2, "PR 4", "closed")
        .await
        .unwrap();

    // Find open PRs for user (page 1, 2 per page)
    let (prs, total) = PrQueries::find_open_by_user(db, user.id, 1, 2)
        .await
        .expect("Failed to find user PRs");

    assert_eq!(prs.len(), 2, "Should find 2 PRs on page 1");
    assert_eq!(total, 3, "Total should be 3 open PRs");

    // Page 2
    let (prs_page2, _) = PrQueries::find_open_by_user(db, user.id, 2, 2)
        .await
        .expect("Failed to find user PRs page 2");

    assert_eq!(prs_page2.len(), 1, "Should find 1 PR on page 2");

    test_db.cleanup().await;
}

/// Test creating a new PR via upsert
#[tokio::test]
async fn test_upsert_create() {
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
    let repo = create_test_repo(db, user.id, "test-repo").await.unwrap();

    let now = Utc::now();
    let pr = PrQueries::upsert(
        db,
        repo.id,
        "github".to_string(),
        "pr_123".to_string(),
        123,
        "New Feature".to_string(),
        Some("Adds new feature".to_string()),
        "https://github.com/owner/repo/pull/123".to_string(),
        "open".to_string(),
        "feature".to_string(),
        "main".to_string(),
        "author123".to_string(),
        Some("https://github.com/author123.png".to_string()),
        false,
        Some(true),
        false,
        100,
        50,
        5,
        10,
        3,
        now,
        now,
        None,
        None,
    )
    .await
    .expect("Failed to create PR");

    assert_eq!(pr.number, 123);
    assert_eq!(pr.title, "New Feature");
    assert_eq!(pr.state, "open");
    assert_eq!(pr.additions, 100);
    assert_eq!(pr.deletions, 50);

    test_db.cleanup().await;
}

/// Test updating an existing PR via upsert
#[tokio::test]
async fn test_upsert_update() {
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
    let repo = create_test_repo(db, user.id, "test-repo").await.unwrap();

    // Create initial PR
    let original = create_test_pr(db, repo.id, 1, "Original Title", "open")
        .await
        .unwrap();

    // Update via upsert
    let now = Utc::now();
    let updated = PrQueries::upsert(
        db,
        repo.id,
        "github".to_string(),
        format!("pr_{}", 1),
        1,
        "Updated Title".to_string(),
        Some("Updated description".to_string()),
        "https://github.com/owner/repo/pull/1".to_string(),
        "open".to_string(),
        "feature-branch".to_string(),
        "main".to_string(),
        "testauthor".to_string(),
        Some("https://github.com/testauthor.png".to_string()),
        false,
        Some(false),
        true,
        150,
        75,
        8,
        12,
        5,
        original.created_at,
        now,
        None,
        None,
    )
    .await
    .expect("Failed to update PR");

    assert_eq!(updated.id, original.id, "Should be same PR");
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.description, Some("Updated description".to_string()));
    assert_eq!(updated.is_mergeable, Some(false));
    assert!(updated.has_conflicts);
    assert_eq!(updated.additions, 150);
    assert_eq!(updated.deletions, 75);

    test_db.cleanup().await;
}

/// Test updating PR state to merged
#[tokio::test]
async fn test_update_state() {
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
    let repo = create_test_repo(db, user.id, "test-repo").await.unwrap();
    let pr = create_test_pr(db, repo.id, 1, "Test PR", "open")
        .await
        .unwrap();

    assert_eq!(pr.state, "open");
    assert!(pr.merged_at.is_none());

    // Update to merged
    let merged_time = Utc::now();
    let updated = PrQueries::update_state(db, pr.id, "merged".to_string(), Some(merged_time), None)
        .await
        .expect("Failed to update state");

    assert_eq!(updated.state, "merged");
    assert!(updated.merged_at.is_some());
    // PostgreSQL TIMESTAMPTZ has microsecond precision, so truncate nanoseconds
    assert_eq!(
        updated.merged_at.unwrap().trunc_subsecs(6),
        merged_time.trunc_subsecs(6)
    );
    assert!(updated.closed_at.is_none());

    test_db.cleanup().await;
}

/// Test updating PR state to closed
#[tokio::test]
async fn test_update_state_closed() {
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
    let repo = create_test_repo(db, user.id, "test-repo").await.unwrap();
    let pr = create_test_pr(db, repo.id, 1, "Test PR", "open")
        .await
        .unwrap();

    // Update to closed
    let closed_time = Utc::now();
    let updated = PrQueries::update_state(db, pr.id, "closed".to_string(), None, Some(closed_time))
        .await
        .expect("Failed to update state");

    assert_eq!(updated.state, "closed");
    assert!(updated.merged_at.is_none());
    assert!(updated.closed_at.is_some());
    // PostgreSQL TIMESTAMPTZ has microsecond precision, so truncate nanoseconds
    assert_eq!(
        updated.closed_at.unwrap().trunc_subsecs(6),
        closed_time.trunc_subsecs(6)
    );

    test_db.cleanup().await;
}

/// Test update_state for non-existent PR returns error
#[tokio::test]
async fn test_update_state_not_found() {
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

    let result = PrQueries::update_state(db, fake_id, "closed".to_string(), None, None).await;

    assert!(result.is_err(), "Should fail for non-existent PR");
    match result {
        Err(DbErr::RecordNotFound(msg)) => assert_eq!(msg, "PR not found"),
        _ => panic!("Expected RecordNotFound error"),
    }

    test_db.cleanup().await;
}

/// Test counting open PRs by repository
#[tokio::test]
async fn test_count_open_by_repository() {
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
    let repo = create_test_repo(db, user.id, "test-repo").await.unwrap();

    // Create PRs
    create_test_pr(db, repo.id, 1, "Open 1", "open")
        .await
        .unwrap();
    create_test_pr(db, repo.id, 2, "Open 2", "open")
        .await
        .unwrap();
    create_test_pr(db, repo.id, 3, "Open 3", "open")
        .await
        .unwrap();
    create_test_pr(db, repo.id, 4, "Closed", "closed")
        .await
        .unwrap();

    let count = PrQueries::count_open_by_repository(db, repo.id)
        .await
        .expect("Failed to count");

    assert_eq!(count, 3, "Should count 3 open PRs");

    test_db.cleanup().await;
}

/// Test deleting a PR
#[tokio::test]
async fn test_delete_pr() {
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
    let repo = create_test_repo(db, user.id, "test-repo").await.unwrap();
    let pr = create_test_pr(db, repo.id, 1, "Test PR", "open")
        .await
        .unwrap();

    // Delete PR
    PrQueries::delete(db, pr.id)
        .await
        .expect("Failed to delete PR");

    // Verify PR is deleted
    let result = PrQueries::find_by_id(db, pr.id).await.unwrap();
    assert!(result.is_none(), "PR should be deleted");

    test_db.cleanup().await;
}

/// Test finding PR by repository and number
#[tokio::test]
async fn test_find_by_number() {
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
    let repo = create_test_repo(db, user.id, "test-repo").await.unwrap();
    let pr = create_test_pr(db, repo.id, 42, "Test PR", "open")
        .await
        .unwrap();

    // Find by number
    let found = PrQueries::find_by_number(db, repo.id, 42)
        .await
        .expect("Failed to find PR");

    assert!(found.is_some());
    let found_pr = found.unwrap();
    assert_eq!(found_pr.id, pr.id);
    assert_eq!(found_pr.number, 42);

    // Try non-existent number
    let not_found = PrQueries::find_by_number(db, repo.id, 999)
        .await
        .expect("Query should succeed");
    assert!(not_found.is_none());

    test_db.cleanup().await;
}
