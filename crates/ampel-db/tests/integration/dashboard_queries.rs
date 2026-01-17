/// Integration tests for dashboard-specific database queries
///
/// Tests query efficiency and correctness for dashboard operations
use crate::common::TestDb;
use ampel_db::queries::{CICheckQueries, PrQueries, RepoQueries, ReviewQueries};
use chrono::Utc;
use sea_orm::Set;
use std::time::Instant;
use uuid::Uuid;

/// Create a test user
async fn create_test_user(db: &sea_orm::DatabaseConnection) -> Uuid {
    use ampel_db::entities::user::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let user = ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(format!("test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("$argon2id$test".to_string()),
        display_name: Set(Some("Test User".to_string())),
        avatar_url: Set(None),
        language: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let user = user.insert(db).await.unwrap();
    user.id
}

/// Create a test repository
async fn create_repository(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
    provider: &str,
) -> ampel_db::entities::repository::Model {
    use ampel_db::entities::repository::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let repo = ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        provider: Set(provider.to_string()),
        provider_id: Set(format!("repo-{}", Uuid::new_v4())),
        owner: Set("test-owner".to_string()),
        name: Set(format!("test-repo-{}", Uuid::new_v4())),
        full_name: Set(format!("owner/test-repo-{}", Uuid::new_v4())),
        description: Set(Some("Test repository".to_string())),
        url: Set(format!("https://github.com/test-repo-{}", Uuid::new_v4())),
        default_branch: Set("main".to_string()),
        is_private: Set(false),
        is_archived: Set(false),
        poll_interval_seconds: Set(300),
        last_polled_at: Set(Some(Utc::now())),
        group_id: Set(None),
        provider_account_id: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    repo.insert(db).await.unwrap()
}

#[tokio::test]
async fn test_find_repositories_by_user_performance() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let user_id = create_test_user(test_db.connection()).await;

    // Create 50 repositories
    for _ in 0..50 {
        create_repository(test_db.connection(), user_id, "github").await;
    }

    let start = Instant::now();
    let repos = RepoQueries::find_by_user_id(test_db.connection(), user_id)
        .await
        .unwrap();
    let duration = start.elapsed();

    assert_eq!(repos.len(), 50);
    assert!(
        duration.as_millis() < 100,
        "Query should be fast, took {:?}",
        duration
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_find_open_prs_by_repository_performance() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let user_id = create_test_user(test_db.connection()).await;
    let repo = create_repository(test_db.connection(), user_id, "github").await;

    // Create 20 open PRs and 10 closed PRs
    for i in 0..30 {
        use ampel_db::entities::pull_request::ActiveModel;
        use sea_orm::ActiveModelTrait;

        let state = if i < 20 { "open" } else { "closed" };

        let pr = ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(repo.id),
            provider: Set("github".to_string()),
            provider_id: Set(format!("pr-{}", i)),
            number: Set(i),
            title: Set(format!("Test PR {}", i)),
            description: Set(Some("Test".to_string())),
            url: Set(format!("https://github.com/test/pr/{}", i)),
            state: Set(state.to_string()),
            source_branch: Set("feature".to_string()),
            target_branch: Set("main".to_string()),
            author: Set("testuser".to_string()),
            author_avatar_url: Set(Some("https://avatar.com".to_string())),
            is_draft: Set(false),
            is_mergeable: Set(Some(true)),
            has_conflicts: Set(false),
            additions: Set(10),
            deletions: Set(5),
            changed_files: Set(2),
            commits_count: Set(1),
            comments_count: Set(0),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            merged_at: Set(None),
            closed_at: Set(None),
            last_synced_at: Set(Utc::now()),
        };

        pr.insert(test_db.connection()).await.unwrap();
    }

    let start = Instant::now();
    let open_prs = PrQueries::find_open_by_repository(test_db.connection(), repo.id)
        .await
        .unwrap();
    let duration = start.elapsed();

    assert_eq!(open_prs.len(), 20);
    assert!(
        duration.as_millis() < 50,
        "Query should be fast, took {:?}",
        duration
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_batch_ci_check_queries() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let user_id = create_test_user(test_db.connection()).await;
    let repo = create_repository(test_db.connection(), user_id, "github").await;

    // Create PR with multiple CI checks
    use ampel_db::entities::pull_request::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let pr = ActiveModel {
        id: Set(Uuid::new_v4()),
        repository_id: Set(repo.id),
        provider: Set("github".to_string()),
        provider_id: Set("pr-1".to_string()),
        number: Set(1),
        title: Set("Test PR".to_string()),
        description: Set(Some("Test".to_string())),
        url: Set("https://github.com/test/pr/1".to_string()),
        state: Set("open".to_string()),
        source_branch: Set("feature".to_string()),
        target_branch: Set("main".to_string()),
        author: Set("testuser".to_string()),
        author_avatar_url: Set(Some("https://avatar.com".to_string())),
        is_draft: Set(false),
        is_mergeable: Set(Some(true)),
        has_conflicts: Set(false),
        additions: Set(10),
        deletions: Set(5),
        changed_files: Set(2),
        commits_count: Set(1),
        comments_count: Set(0),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        merged_at: Set(None),
        closed_at: Set(None),
        last_synced_at: Set(Utc::now()),
    };

    let pr = pr.insert(test_db.connection()).await.unwrap();

    // Create 5 CI checks
    for i in 0..5 {
        use ampel_db::entities::ci_check::ActiveModel as CIActiveModel;

        let check = CIActiveModel {
            id: Set(Uuid::new_v4()),
            pull_request_id: Set(pr.id),
            name: Set(format!("CI Check {}", i)),
            status: Set("completed".to_string()),
            conclusion: Set(Some("success".to_string())),
            url: Set(Some("https://ci.example.com".to_string())),
            started_at: Set(Some(Utc::now())),
            completed_at: Set(Some(Utc::now())),
            duration_seconds: Set(Some(60)),
        };

        check.insert(test_db.connection()).await.unwrap();
    }

    let start = Instant::now();
    let checks = CICheckQueries::find_by_pull_request(test_db.connection(), pr.id)
        .await
        .unwrap();
    let duration = start.elapsed();

    assert_eq!(checks.len(), 5);
    assert!(
        duration.as_millis() < 20,
        "CI check query should be fast, took {:?}",
        duration
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_batch_review_queries() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let user_id = create_test_user(test_db.connection()).await;
    let repo = create_repository(test_db.connection(), user_id, "github").await;

    // Create PR with multiple reviews
    use ampel_db::entities::pull_request::ActiveModel;
    use sea_orm::ActiveModelTrait;

    let pr = ActiveModel {
        id: Set(Uuid::new_v4()),
        repository_id: Set(repo.id),
        provider: Set("github".to_string()),
        provider_id: Set("pr-1".to_string()),
        number: Set(1),
        title: Set("Test PR".to_string()),
        description: Set(Some("Test".to_string())),
        url: Set("https://github.com/test/pr/1".to_string()),
        state: Set("open".to_string()),
        source_branch: Set("feature".to_string()),
        target_branch: Set("main".to_string()),
        author: Set("testuser".to_string()),
        author_avatar_url: Set(Some("https://avatar.com".to_string())),
        is_draft: Set(false),
        is_mergeable: Set(Some(true)),
        has_conflicts: Set(false),
        additions: Set(10),
        deletions: Set(5),
        changed_files: Set(2),
        commits_count: Set(1),
        comments_count: Set(0),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        merged_at: Set(None),
        closed_at: Set(None),
        last_synced_at: Set(Utc::now()),
    };

    let pr = pr.insert(test_db.connection()).await.unwrap();

    // Create 3 reviews
    for i in 0..3 {
        use ampel_db::entities::review::ActiveModel as ReviewActiveModel;

        let review = ReviewActiveModel {
            id: Set(Uuid::new_v4()),
            pull_request_id: Set(pr.id),
            reviewer: Set(format!("reviewer{}", i)),
            reviewer_avatar_url: Set(Some("https://avatar.com".to_string())),
            state: Set("approved".to_string()),
            body: Set(Some("LGTM".to_string())),
            submitted_at: Set(Utc::now()),
        };

        review.insert(test_db.connection()).await.unwrap();
    }

    let start = Instant::now();
    let reviews = ReviewQueries::find_by_pull_request(test_db.connection(), pr.id)
        .await
        .unwrap();
    let duration = start.elapsed();

    assert_eq!(reviews.len(), 3);
    assert!(
        duration.as_millis() < 20,
        "Review query should be fast, took {:?}",
        duration
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_repository_query_correctness() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    let user1 = create_test_user(test_db.connection()).await;
    let user2 = create_test_user(test_db.connection()).await;

    // Create repos for different users
    create_repository(test_db.connection(), user1, "github").await;
    create_repository(test_db.connection(), user1, "gitlab").await;
    create_repository(test_db.connection(), user2, "github").await;

    // Verify user1 only sees their repos
    let user1_repos = RepoQueries::find_by_user_id(test_db.connection(), user1)
        .await
        .unwrap();
    assert_eq!(user1_repos.len(), 2);

    // Verify user2 only sees their repos
    let user2_repos = RepoQueries::find_by_user_id(test_db.connection(), user2)
        .await
        .unwrap();
    assert_eq!(user2_repos.len(), 1);

    test_db.cleanup().await;
}
