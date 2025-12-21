/// Integration tests for cleanup job
///
/// These tests verify that the cleanup job correctly:
/// - Deletes old closed pull requests
/// - Leaves recent closed PRs untouched
/// - Leaves open PRs untouched
/// - Respects the 30-day cutoff period
///
/// Note: These tests require PostgreSQL because migrations use PostgreSQL-specific
/// features. Tests are automatically skipped when running in SQLite mode.
mod common;

use ampel_db::entities::{provider_account, pull_request, repository};
use ampel_worker::jobs::cleanup::CleanupJob;
use chrono::{Duration, Utc};
use common::{create_test_encryption_service, create_test_user, TestDb};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

/// Helper to create a test repository
async fn create_test_repository(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
    provider_account_id: Uuid,
    name: &str,
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
        last_polled_at: Set(None),
        poll_interval_seconds: Set(300),
        created_at: Set(now),
        updated_at: Set(now),
    };

    Ok(repo.insert(db).await?)
}

/// Helper to create a test pull request
async fn create_test_pull_request(
    db: &sea_orm::DatabaseConnection,
    repo_id: Uuid,
    number: i32,
    state: &str,
    closed_at: Option<chrono::DateTime<Utc>>,
) -> anyhow::Result<pull_request::Model> {
    let now = Utc::now();
    let pr = pull_request::ActiveModel {
        id: Set(Uuid::new_v4()),
        repository_id: Set(repo_id),
        provider: Set("github".to_string()),
        provider_id: Set(format!("pr_{}", number)),
        number: Set(number),
        title: Set(format!("Test PR #{}", number)),
        description: Set(Some("Test description".to_string())),
        url: Set(format!("https://github.com/test/repo/pull/{}", number)),
        state: Set(state.to_string()),
        source_branch: Set("feature".to_string()),
        target_branch: Set("main".to_string()),
        author: Set("testauthor".to_string()),
        author_avatar_url: Set(Some("https://example.com/avatar.png".to_string())),
        is_draft: Set(false),
        is_mergeable: Set(Some(true)),
        has_conflicts: Set(false),
        additions: Set(100),
        deletions: Set(50),
        changed_files: Set(5),
        commits_count: Set(3),
        comments_count: Set(2),
        created_at: Set(now),
        updated_at: Set(now),
        merged_at: Set(None),
        closed_at: Set(closed_at),
        last_synced_at: Set(now),
    };

    Ok(pr.insert(db).await?)
}

#[tokio::test]
async fn test_cleanup_deletes_old_closed_prs() {
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
    let user = create_test_user(db, "cleanup1@example.com", "cleanupuser1")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "test-repo")
        .await
        .expect("Failed to create repo");

    // Create old closed PR (40 days ago)
    let old_closed_at = Utc::now() - Duration::days(40);
    let old_pr = create_test_pull_request(db, repo.id, 1, "closed", Some(old_closed_at))
        .await
        .expect("Failed to create old PR");

    // Verify PR exists before cleanup
    let pr_before = pull_request::Entity::find_by_id(old_pr.id)
        .one(db)
        .await
        .expect("Failed to query PR");
    assert!(pr_before.is_some(), "PR should exist before cleanup");

    // Run cleanup
    let job = CleanupJob;
    let result: Result<(), anyhow::Error> = job.execute(db).await;
    assert!(result.is_ok(), "Cleanup should succeed");

    // Verify PR was deleted
    let pr_after = pull_request::Entity::find_by_id(old_pr.id)
        .one(db)
        .await
        .expect("Failed to query PR");
    assert!(
        pr_after.is_none(),
        "Old closed PR should be deleted after cleanup"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_cleanup_preserves_recent_closed_prs() {
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
    let user = create_test_user(db, "cleanup2@example.com", "cleanupuser2")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "test-repo2")
        .await
        .expect("Failed to create repo");

    // Create recent closed PR (10 days ago)
    let recent_closed_at = Utc::now() - Duration::days(10);
    let recent_pr = create_test_pull_request(db, repo.id, 1, "closed", Some(recent_closed_at))
        .await
        .expect("Failed to create recent PR");

    // Run cleanup
    let job = CleanupJob;
    let result: Result<(), anyhow::Error> = job.execute(db).await;
    assert!(result.is_ok(), "Cleanup should succeed");

    // Verify PR was NOT deleted (it's recent)
    let pr_after = pull_request::Entity::find_by_id(recent_pr.id)
        .one(db)
        .await
        .expect("Failed to query PR");
    assert!(
        pr_after.is_some(),
        "Recent closed PR should NOT be deleted after cleanup"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_cleanup_preserves_open_prs() {
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
    let user = create_test_user(db, "cleanup3@example.com", "cleanupuser3")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "test-repo3")
        .await
        .expect("Failed to create repo");

    // Create open PR (no closed_at)
    let open_pr = create_test_pull_request(db, repo.id, 1, "open", None)
        .await
        .expect("Failed to create open PR");

    // Run cleanup
    let job = CleanupJob;
    let result: Result<(), anyhow::Error> = job.execute(db).await;
    assert!(result.is_ok(), "Cleanup should succeed");

    // Verify open PR was NOT deleted
    let pr_after = pull_request::Entity::find_by_id(open_pr.id)
        .one(db)
        .await
        .expect("Failed to query PR");
    assert!(
        pr_after.is_some(),
        "Open PR should NOT be deleted after cleanup"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_cleanup_boundary_30_days() {
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
    let user = create_test_user(db, "cleanup4@example.com", "cleanupuser4")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "test-repo4")
        .await
        .expect("Failed to create repo");

    // Create PR closed exactly at 30 days (should be preserved - boundary case)
    let at_boundary = Utc::now() - Duration::days(30);
    let boundary_pr = create_test_pull_request(db, repo.id, 1, "closed", Some(at_boundary))
        .await
        .expect("Failed to create boundary PR");

    // Create PR closed at 31 days (should be deleted - just past boundary)
    let past_boundary = Utc::now() - Duration::days(31);
    let old_pr = create_test_pull_request(db, repo.id, 2, "closed", Some(past_boundary))
        .await
        .expect("Failed to create old PR");

    // Run cleanup
    let job = CleanupJob;
    let result: Result<(), anyhow::Error> = job.execute(db).await;
    assert!(result.is_ok(), "Cleanup should succeed");

    // Verify boundary PR was NOT deleted (exactly 30 days)
    let boundary_pr_after = pull_request::Entity::find_by_id(boundary_pr.id)
        .one(db)
        .await
        .expect("Failed to query PR");
    assert!(
        boundary_pr_after.is_some(),
        "PR at exactly 30 days should NOT be deleted"
    );

    // Verify old PR WAS deleted (31 days)
    let old_pr_after = pull_request::Entity::find_by_id(old_pr.id)
        .one(db)
        .await
        .expect("Failed to query PR");
    assert!(
        old_pr_after.is_none(),
        "PR older than 30 days should be deleted"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_cleanup_multiple_prs_mixed() {
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
    let user = create_test_user(db, "cleanup5@example.com", "cleanupuser5")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "test-repo5")
        .await
        .expect("Failed to create repo");

    // Create mix of PRs
    let open_pr = create_test_pull_request(db, repo.id, 1, "open", None)
        .await
        .expect("Failed to create open PR");

    let recent_closed = Utc::now() - Duration::days(15);
    let recent_pr = create_test_pull_request(db, repo.id, 2, "closed", Some(recent_closed))
        .await
        .expect("Failed to create recent closed PR");

    let old_closed = Utc::now() - Duration::days(45);
    let old_pr = create_test_pull_request(db, repo.id, 3, "closed", Some(old_closed))
        .await
        .expect("Failed to create old closed PR");

    let very_old_closed = Utc::now() - Duration::days(90);
    let very_old_pr = create_test_pull_request(db, repo.id, 4, "closed", Some(very_old_closed))
        .await
        .expect("Failed to create very old closed PR");

    // Run cleanup
    let job = CleanupJob;
    let result: Result<(), anyhow::Error> = job.execute(db).await;
    assert!(result.is_ok(), "Cleanup should succeed");

    // Verify open PR still exists
    let open_after = pull_request::Entity::find_by_id(open_pr.id)
        .one(db)
        .await
        .expect("Failed to query");
    assert!(open_after.is_some(), "Open PR should still exist");

    // Verify recent closed PR still exists
    let recent_after = pull_request::Entity::find_by_id(recent_pr.id)
        .one(db)
        .await
        .expect("Failed to query");
    assert!(
        recent_after.is_some(),
        "Recent closed PR should still exist"
    );

    // Verify old PRs were deleted
    let old_after = pull_request::Entity::find_by_id(old_pr.id)
        .one(db)
        .await
        .expect("Failed to query");
    assert!(old_after.is_none(), "Old PR should be deleted");

    let very_old_after = pull_request::Entity::find_by_id(very_old_pr.id)
        .one(db)
        .await
        .expect("Failed to query");
    assert!(very_old_after.is_none(), "Very old PR should be deleted");

    test_db.cleanup().await;
}
