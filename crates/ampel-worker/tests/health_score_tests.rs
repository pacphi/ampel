/// Integration tests for health_score job
///
/// These tests verify that the health score calculation job correctly:
/// - Calculates health scores for repositories
/// - Computes average time to merge and review metrics
/// - Identifies stale PRs (open > 7 days)
/// - Calculates PR throughput
/// - Applies correct penalty and bonus scoring
///
/// Note: These tests require PostgreSQL because migrations use PostgreSQL-specific
/// features. Tests are automatically skipped when running in SQLite mode.
mod common;

use ampel_db::entities::{health_score, pr_metrics, provider_account, pull_request, repository};
use ampel_worker::jobs::health_score::HealthScoreJob;
use chrono::{Duration, Utc};
use common::{create_test_encryption_service, create_test_user, TestDb};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
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
    created_at: chrono::DateTime<Utc>,
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
        created_at: Set(created_at),
        updated_at: Set(now),
        merged_at: Set(None),
        closed_at: Set(None),
        last_synced_at: Set(now),
    };

    Ok(pr.insert(db).await?)
}

/// Helper to create test PR metrics
async fn create_test_pr_metrics(
    db: &sea_orm::DatabaseConnection,
    pr_id: Uuid,
    repo_id: Uuid,
    time_to_merge: Option<i32>,
    time_to_first_review: Option<i32>,
    merged_at: chrono::DateTime<Utc>,
) -> anyhow::Result<pr_metrics::Model> {
    let metrics = pr_metrics::ActiveModel {
        id: Set(Uuid::new_v4()),
        pull_request_id: Set(pr_id),
        repository_id: Set(repo_id),
        time_to_first_review: Set(time_to_first_review),
        time_to_approval: Set(None),
        time_to_merge: Set(time_to_merge),
        review_rounds: Set(Some(1)),
        comments_count: Set(Some(2)),
        is_bot: Set(false),
        merged_at: Set(Some(merged_at)),
        recorded_at: Set(Utc::now()),
    };

    Ok(metrics.insert(db).await?)
}

#[tokio::test]
async fn test_health_score_empty_repository() {
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
    let user = create_test_user(db, "health1@example.com", "healthuser1")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "empty-repo")
        .await
        .expect("Failed to create repo");

    // Run health score job
    let job = HealthScoreJob;
    let result = job.execute(db).await;
    assert!(result.is_ok(), "Health score job should succeed");

    // Verify health score was created with default values
    let scores = health_score::Entity::find()
        .filter(health_score::Column::RepositoryId.eq(repo.id))
        .all(db)
        .await
        .expect("Failed to fetch health scores");

    assert_eq!(scores.len(), 1, "Should create one health score entry");
    let score = &scores[0];

    // Empty repo should have baseline score of 100 (no penalties, no bonuses)
    assert_eq!(score.score, 100, "Empty repo should have score of 100");
    assert!(score.avg_time_to_merge.is_none());
    assert!(score.avg_review_time.is_none());
    assert_eq!(score.stale_pr_count, Some(0));
    assert_eq!(score.pr_throughput, Some(0));

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_health_score_with_stale_prs() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "health2@example.com", "healthuser2")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "stale-repo")
        .await
        .expect("Failed to create repo");

    // Create 3 stale PRs (open for > 7 days)
    let stale_time = Utc::now() - Duration::days(10);
    for i in 1..=3 {
        create_test_pull_request(db, repo.id, i, "open", stale_time)
            .await
            .expect("Failed to create stale PR");
    }

    // Create 1 recent PR (not stale)
    let recent_time = Utc::now() - Duration::days(2);
    create_test_pull_request(db, repo.id, 4, "open", recent_time)
        .await
        .expect("Failed to create recent PR");

    // Run health score job
    let job = HealthScoreJob;
    job.execute(db).await.expect("Health score job failed");

    // Verify health score reflects stale PRs
    let scores = health_score::Entity::find()
        .filter(health_score::Column::RepositoryId.eq(repo.id))
        .all(db)
        .await
        .expect("Failed to fetch health scores");

    assert_eq!(scores.len(), 1);
    let score = &scores[0];

    assert_eq!(score.stale_pr_count, Some(3), "Should count 3 stale PRs");
    // Score = 100 - (3 * 2) = 94 (penalty for 3 stale PRs)
    assert_eq!(score.score, 94, "Should penalize for stale PRs");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_health_score_with_good_metrics() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "health3@example.com", "healthuser3")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "healthy-repo")
        .await
        .expect("Failed to create repo");

    // Create PRs with good metrics (fast merge/review times)
    // merged_at must be within last 7 days for throughput calculation
    let now = Utc::now();
    for i in 1..=15 {
        let pr = create_test_pull_request(db, repo.id, i, "merged", now - Duration::days(10))
            .await
            .expect("Failed to create PR");

        // Fast merge time: 2 hours (7200 seconds)
        // Fast review time: 30 minutes (1800 seconds)
        // Merged within last 7 days to count toward throughput
        create_test_pr_metrics(
            db,
            pr.id,
            repo.id,
            Some(7200),
            Some(1800),
            now - Duration::days(3),
        )
        .await
        .expect("Failed to create metrics");
    }

    // Run health score job
    let job = HealthScoreJob;
    job.execute(db).await.expect("Health score job failed");

    // Verify excellent health score
    let scores = health_score::Entity::find()
        .filter(health_score::Column::RepositoryId.eq(repo.id))
        .all(db)
        .await
        .expect("Failed to fetch health scores");

    assert_eq!(scores.len(), 1);
    let score = &scores[0];

    // Score = 100 + 10 (bonus for throughput >= 10) = 110, clamped to 100
    assert_eq!(score.score, 100, "Excellent repo should have max score");
    assert!(score.avg_time_to_merge.is_some());
    assert!(score.avg_review_time.is_some());
    assert_eq!(score.stale_pr_count, Some(0));
    // 15 PRs merged within last 7 days should all be counted in throughput
    assert_eq!(
        score.pr_throughput,
        Some(15),
        "Should have throughput of 15 (all PRs merged in last 7 days)"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_health_score_with_slow_merge_times() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "health4@example.com", "healthuser4")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "slow-repo")
        .await
        .expect("Failed to create repo");

    // Create PRs with slow merge times (> 72 hours = severe penalty)
    let now = Utc::now();
    for i in 1..=5 {
        let pr = create_test_pull_request(db, repo.id, i, "merged", now - Duration::days(20))
            .await
            .expect("Failed to create PR");

        // Very slow merge time: 80 hours (288000 seconds)
        // Slow review time: 26 hours (93600 seconds)
        create_test_pr_metrics(
            db,
            pr.id,
            repo.id,
            Some(288000), // 80 hours
            Some(93600),  // 26 hours
            now - Duration::days(15),
        )
        .await
        .expect("Failed to create metrics");
    }

    // Run health score job
    let job = HealthScoreJob;
    job.execute(db).await.expect("Health score job failed");

    // Verify poor health score
    let scores = health_score::Entity::find()
        .filter(health_score::Column::RepositoryId.eq(repo.id))
        .all(db)
        .await
        .expect("Failed to fetch health scores");

    assert_eq!(scores.len(), 1);
    let score = &scores[0];

    // Score = 100 - 30 (merge > 72h) - 20 (review > 24h) = 50
    assert_eq!(score.score, 50, "Slow repo should have low score");
    assert!(
        score.avg_time_to_merge.unwrap() > 24 * 3600,
        "Should have slow merge time"
    );
    assert!(
        score.avg_review_time.unwrap() > 24 * 3600,
        "Should have slow review time"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_health_score_multiple_repositories() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "health5@example.com", "healthuser5")
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

    // Create 3 repositories
    let repo1 = create_test_repository(db, user.id, provider_account.id, "repo1")
        .await
        .expect("Failed to create repo1");
    let repo2 = create_test_repository(db, user.id, provider_account.id, "repo2")
        .await
        .expect("Failed to create repo2");
    let repo3 = create_test_repository(db, user.id, provider_account.id, "repo3")
        .await
        .expect("Failed to create repo3");

    // Run health score job
    let job = HealthScoreJob;
    job.execute(db).await.expect("Health score job failed");

    // Verify all repositories have health scores
    let all_scores = health_score::Entity::find()
        .all(db)
        .await
        .expect("Failed to fetch scores");

    assert_eq!(all_scores.len(), 3, "Should create scores for all 3 repos");

    // Check that each repo has exactly one score
    for repo_id in [repo1.id, repo2.id, repo3.id] {
        let repo_scores: Vec<_> = all_scores
            .iter()
            .filter(|s| s.repository_id == repo_id)
            .collect();
        assert_eq!(
            repo_scores.len(),
            1,
            "Each repo should have exactly one score"
        );
    }

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_health_score_boundary_conditions() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "health6@example.com", "healthuser6")
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

    let repo = create_test_repository(db, user.id, provider_account.id, "boundary-repo")
        .await
        .expect("Failed to create repo");

    // Create exactly 5 stale PRs (boundary for penalty tier)
    let stale_time = Utc::now() - Duration::days(10);
    for i in 1..=5 {
        create_test_pull_request(db, repo.id, i, "open", stale_time)
            .await
            .expect("Failed to create stale PR");
    }

    // Run health score job
    let job = HealthScoreJob;
    job.execute(db).await.expect("Health score job failed");

    let scores = health_score::Entity::find()
        .filter(health_score::Column::RepositoryId.eq(repo.id))
        .all(db)
        .await
        .expect("Failed to fetch health scores");

    assert_eq!(scores.len(), 1);
    let score = &scores[0];

    assert_eq!(score.stale_pr_count, Some(5));
    // Exactly 5 stale PRs triggers -10 penalty (5 * 2), but at 5 we're at boundary
    // According to code: if stale_prs > 5 { -15 } else if stale_prs > 0 { stale_prs * 2 }
    // So 5 stale PRs = 5 * 2 = -10
    assert_eq!(score.score, 90, "5 stale PRs should result in score of 90");

    test_db.cleanup().await;
}
