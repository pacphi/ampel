/// Integration tests for metrics_collection job
///
/// These tests verify that the metrics collection job correctly:
/// - Identifies merged PRs without metrics
/// - Calculates time to first review
/// - Calculates time to approval
/// - Calculates time to merge
/// - Counts review rounds
/// - Detects bot authors
/// - Stores metrics in pr_metrics table
///
/// Note: These tests require PostgreSQL because migrations use PostgreSQL-specific
/// features. Tests are automatically skipped when running in SQLite mode.
mod common;

use ampel_db::entities::{pr_metrics, provider_account, pull_request, repository, review};
use ampel_worker::jobs::metrics_collection::MetricsCollectionJob;
use chrono::{Duration, Utc};
use common::{create_test_encryption_service, create_test_user, TestDb};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
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
    author: &str,
    created_at: chrono::DateTime<Utc>,
    merged_at: Option<chrono::DateTime<Utc>>,
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
        author: Set(author.to_string()),
        author_avatar_url: Set(Some("https://example.com/avatar.png".to_string())),
        is_draft: Set(false),
        is_mergeable: Set(Some(true)),
        has_conflicts: Set(false),
        additions: Set(100),
        deletions: Set(50),
        changed_files: Set(5),
        commits_count: Set(3),
        comments_count: Set(2),
        created_at: Set(created_at),
        updated_at: Set(now),
        merged_at: Set(merged_at),
        closed_at: Set(merged_at),
        last_synced_at: Set(now),
    };

    Ok(pr.insert(db).await?)
}

/// Helper to create a test review
async fn create_test_review(
    db: &sea_orm::DatabaseConnection,
    pr_id: Uuid,
    reviewer: &str,
    state: &str,
    submitted_at: chrono::DateTime<Utc>,
) -> anyhow::Result<review::Model> {
    let review = review::ActiveModel {
        id: Set(Uuid::new_v4()),
        pull_request_id: Set(pr_id),
        reviewer: Set(reviewer.to_string()),
        reviewer_avatar_url: Set(Some("https://example.com/avatar.png".to_string())),
        state: Set(state.to_string()),
        body: Set(Some("Review comment".to_string())),
        submitted_at: Set(submitted_at),
    };

    Ok(review.insert(db).await?)
}

#[tokio::test]
async fn test_metrics_collection_basic() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "metrics1@example.com", "metricsuser1")
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

    // Create merged PR
    // Created 10 hours ago, merged 10 hours later (now) = 10 hour merge time
    let created = Utc::now() - Duration::hours(10);
    let merged = created + Duration::hours(10); // 10 hours after creation
    let pr = create_test_pull_request(db, repo.id, 1, "merged", "author1", created, Some(merged))
        .await
        .expect("Failed to create PR");

    // Add review 1 hour after creation
    let review_time = created + Duration::hours(1);
    create_test_review(db, pr.id, "reviewer1", "approved", review_time)
        .await
        .expect("Failed to create review");

    // Run metrics collection
    let job = MetricsCollectionJob;
    job.execute(db).await.expect("Metrics collection failed");

    // Verify metrics were created
    let metrics = pr_metrics::Entity::find()
        .filter(pr_metrics::Column::PullRequestId.eq(pr.id))
        .one(db)
        .await
        .expect("Failed to fetch metrics");

    assert!(metrics.is_some(), "Metrics should be created");
    let m = metrics.unwrap();

    assert_eq!(m.repository_id, repo.id);
    assert!(m.time_to_merge.is_some());
    assert!(m.time_to_first_review.is_some());
    assert!(!m.is_bot);

    // Verify time calculations (approximately 10 hours to merge)
    let merge_time = m.time_to_merge.unwrap();
    assert!(
        merge_time > 8 * 3600 && merge_time < 11 * 3600,
        "Merge time should be ~10 hours"
    );

    // Verify review time (approximately 1 hour)
    let review_time_sec = m.time_to_first_review.unwrap();
    assert!(
        review_time_sec > 3000 && review_time_sec < 4000,
        "Review time should be ~1 hour"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_metrics_collection_skips_existing() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "metrics2@example.com", "metricsuser2")
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

    let created = Utc::now() - Duration::hours(10);
    let merged = Utc::now() - Duration::hours(2);
    let pr = create_test_pull_request(db, repo.id, 1, "merged", "author1", created, Some(merged))
        .await
        .expect("Failed to create PR");

    // Create metrics manually
    let existing_metrics = pr_metrics::ActiveModel {
        id: Set(Uuid::new_v4()),
        pull_request_id: Set(pr.id),
        repository_id: Set(repo.id),
        time_to_first_review: Set(Some(3600)),
        time_to_approval: Set(Some(7200)),
        time_to_merge: Set(Some(10800)),
        review_rounds: Set(Some(1)),
        comments_count: Set(Some(2)),
        is_bot: Set(false),
        merged_at: Set(Some(merged)),
        recorded_at: Set(Utc::now()),
    };
    existing_metrics
        .insert(db)
        .await
        .expect("Failed to insert metrics");

    // Run metrics collection
    let job = MetricsCollectionJob;
    job.execute(db).await.expect("Metrics collection failed");

    // Verify only one metrics record exists (not duplicated)
    let count = pr_metrics::Entity::find()
        .filter(pr_metrics::Column::PullRequestId.eq(pr.id))
        .count(db)
        .await
        .expect("Failed to count metrics");

    assert_eq!(count, 1, "Should not create duplicate metrics");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_metrics_collection_bot_detection() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "metrics3@example.com", "metricsuser3")
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

    // Create PR from dependabot
    let created = Utc::now() - Duration::hours(5);
    let merged = Utc::now() - Duration::hours(1);
    let pr = create_test_pull_request(
        db,
        repo.id,
        1,
        "merged",
        "dependabot[bot]",
        created,
        Some(merged),
    )
    .await
    .expect("Failed to create PR");

    // Run metrics collection
    let job = MetricsCollectionJob;
    job.execute(db).await.expect("Metrics collection failed");

    // Verify bot flag is set
    let metrics = pr_metrics::Entity::find()
        .filter(pr_metrics::Column::PullRequestId.eq(pr.id))
        .one(db)
        .await
        .expect("Failed to fetch metrics")
        .expect("Metrics should exist");

    assert!(metrics.is_bot, "Should detect bot author");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_metrics_collection_review_rounds() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "metrics4@example.com", "metricsuser4")
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

    let created = Utc::now() - Duration::hours(20);
    let merged = Utc::now() - Duration::hours(2);
    let pr = create_test_pull_request(db, repo.id, 1, "merged", "author1", created, Some(merged))
        .await
        .expect("Failed to create PR");

    // Add multiple review rounds (changes requested, then approved)
    create_test_review(
        db,
        pr.id,
        "reviewer1",
        "changes_requested",
        created + Duration::hours(1),
    )
    .await
    .expect("Failed to create review");
    create_test_review(
        db,
        pr.id,
        "reviewer2",
        "changes_requested",
        created + Duration::hours(5),
    )
    .await
    .expect("Failed to create review");
    create_test_review(
        db,
        pr.id,
        "reviewer1",
        "approved",
        created + Duration::hours(10),
    )
    .await
    .expect("Failed to create review");

    // Run metrics collection
    let job = MetricsCollectionJob;
    job.execute(db).await.expect("Metrics collection failed");

    // Verify review rounds are counted
    let metrics = pr_metrics::Entity::find()
        .filter(pr_metrics::Column::PullRequestId.eq(pr.id))
        .one(db)
        .await
        .expect("Failed to fetch metrics")
        .expect("Metrics should exist");

    assert_eq!(
        metrics.review_rounds.unwrap(),
        2,
        "Should count 2 changes_requested reviews"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_metrics_collection_no_reviews() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "metrics5@example.com", "metricsuser5")
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

    // Create merged PR without reviews
    let created = Utc::now() - Duration::hours(5);
    let merged = Utc::now() - Duration::hours(1);
    let pr = create_test_pull_request(db, repo.id, 1, "merged", "author1", created, Some(merged))
        .await
        .expect("Failed to create PR");

    // Run metrics collection
    let job = MetricsCollectionJob;
    job.execute(db).await.expect("Metrics collection failed");

    // Verify metrics exist with null review times
    let metrics = pr_metrics::Entity::find()
        .filter(pr_metrics::Column::PullRequestId.eq(pr.id))
        .one(db)
        .await
        .expect("Failed to fetch metrics")
        .expect("Metrics should exist");

    assert!(
        metrics.time_to_first_review.is_none(),
        "Should have no review time"
    );
    assert!(
        metrics.time_to_approval.is_none(),
        "Should have no approval time"
    );
    assert!(
        metrics.time_to_merge.is_some(),
        "Should still have merge time"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_metrics_collection_multiple_prs() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "metrics6@example.com", "metricsuser6")
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

    // Create 5 merged PRs
    let now = Utc::now();
    for i in 1..=5 {
        let created = now - Duration::hours(10 + i);
        let merged = now - Duration::hours(i);
        create_test_pull_request(
            db,
            repo.id,
            i as i32,
            "merged",
            "author1",
            created,
            Some(merged),
        )
        .await
        .expect("Failed to create PR");
    }

    // Run metrics collection
    let job = MetricsCollectionJob;
    job.execute(db).await.expect("Metrics collection failed");

    // Verify metrics created for all PRs
    let count = pr_metrics::Entity::find()
        .filter(pr_metrics::Column::RepositoryId.eq(repo.id))
        .count(db)
        .await
        .expect("Failed to count metrics");

    assert_eq!(count, 5, "Should create metrics for all 5 merged PRs");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_metrics_collection_ignores_open_prs() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "metrics7@example.com", "metricsuser7")
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

    // Create open PR (should be ignored)
    let created = Utc::now() - Duration::hours(5);
    create_test_pull_request(db, repo.id, 1, "open", "author1", created, None)
        .await
        .expect("Failed to create PR");

    // Run metrics collection
    let job = MetricsCollectionJob;
    job.execute(db).await.expect("Metrics collection failed");

    // Verify no metrics created for open PR
    let count = pr_metrics::Entity::find()
        .filter(pr_metrics::Column::RepositoryId.eq(repo.id))
        .count(db)
        .await
        .expect("Failed to count metrics");

    assert_eq!(count, 0, "Should not create metrics for open PRs");

    test_db.cleanup().await;
}
