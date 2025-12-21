/// Integration tests for provider account queries
///
/// These tests verify the database layer works correctly with real SQLite databases.
/// Each test runs in complete isolation with its own database instance.

use ampel_db::entities::provider_account::Entity;
use ampel_db::queries::ProviderAccountQueries;
use sea_orm::{DbErr, EntityTrait};

// Import test utilities from parent module
use super::common::fixtures::{create_test_provider_account, create_test_user};
use super::common::TestDb;

#[tokio::test]
async fn test_find_by_user() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .expect("Failed to create user");

    // Create multiple accounts for the user
    create_test_provider_account(db, user.id, "github", "Work GitHub", true)
        .await
        .unwrap();
    create_test_provider_account(db, user.id, "github", "Personal GitHub", false)
        .await
        .unwrap();
    create_test_provider_account(db, user.id, "gitlab", "Work GitLab", true)
        .await
        .unwrap();

    // Create account for different user
    let other_user = create_test_user(db, "other@example.com", "otheruser")
        .await
        .unwrap();
    create_test_provider_account(db, other_user.id, "github", "Other Account", true)
        .await
        .unwrap();

    // Test find_by_user
    let accounts = ProviderAccountQueries::find_by_user(db, user.id)
        .await
        .expect("Failed to find accounts");

    assert_eq!(accounts.len(), 3, "User should have 3 accounts");

    // Verify all accounts belong to the user
    for account in &accounts {
        assert_eq!(account.user_id, user.id);
    }

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_find_default_for_provider() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();

    // Create two GitHub accounts, one default
    let default_account = create_test_provider_account(db, user.id, "github", "Work GitHub", true)
        .await
        .unwrap();
    create_test_provider_account(db, user.id, "github", "Personal GitHub", false)
        .await
        .unwrap();

    // Test find_default_for_provider
    let result = ProviderAccountQueries::find_default_for_provider(db, user.id, "github")
        .await
        .expect("Failed to find default account");

    assert!(result.is_some(), "Should find a default account");
    let found_account = result.unwrap();
    assert_eq!(found_account.id, default_account.id);
    assert!(found_account.is_default);
    assert_eq!(found_account.account_label, "Work GitHub");

    // Test for provider with no default
    let result_gitlab = ProviderAccountQueries::find_default_for_provider(db, user.id, "gitlab")
        .await
        .expect("Failed to query");

    assert!(
        result_gitlab.is_none(),
        "Should not find default for GitLab"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_set_default_clears_previous() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();

    // Create two GitHub accounts
    let account1 = create_test_provider_account(db, user.id, "github", "Account 1", true)
        .await
        .unwrap();
    let account2 = create_test_provider_account(db, user.id, "github", "Account 2", false)
        .await
        .unwrap();

    // Verify account1 is default
    let default = ProviderAccountQueries::find_default_for_provider(db, user.id, "github")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(default.id, account1.id);

    // Set account2 as default
    let updated = ProviderAccountQueries::set_default(db, account2.id, user.id)
        .await
        .expect("Failed to set default");

    assert!(updated.is_default);
    assert_eq!(updated.id, account2.id);

    // Verify account1 is no longer default
    let account1_refreshed = Entity::find_by_id(account1.id)
        .one(db)
        .await
        .unwrap()
        .unwrap();
    assert!(!account1_refreshed.is_default);

    // Verify only account2 is default
    let new_default = ProviderAccountQueries::find_default_for_provider(db, user.id, "github")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(new_default.id, account2.id);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_set_default_unauthorized() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();
    let other_user = create_test_user(db, "other@example.com", "otheruser")
        .await
        .unwrap();

    // Create account for user_id
    let account = create_test_provider_account(db, user.id, "github", "Test Account", true)
        .await
        .unwrap();

    // Try to set default as different user
    let result = ProviderAccountQueries::set_default(db, account.id, other_user.id).await;

    assert!(result.is_err(), "Should fail with unauthorized error");
    match result {
        Err(DbErr::Custom(msg)) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Expected Custom error with 'Unauthorized' message"),
    }

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_count_by_user_and_provider() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();

    // Create multiple GitHub accounts
    create_test_provider_account(db, user.id, "github", "Account 1", true)
        .await
        .unwrap();
    create_test_provider_account(db, user.id, "github", "Account 2", false)
        .await
        .unwrap();
    create_test_provider_account(db, user.id, "github", "Account 3", false)
        .await
        .unwrap();

    // Create GitLab account
    create_test_provider_account(db, user.id, "gitlab", "GitLab Account", true)
        .await
        .unwrap();

    // Test count
    let github_count = ProviderAccountQueries::count_by_user_and_provider(db, user.id, "github")
        .await
        .expect("Failed to count");

    assert_eq!(github_count, 3, "Should have 3 GitHub accounts");

    let gitlab_count = ProviderAccountQueries::count_by_user_and_provider(db, user.id, "gitlab")
        .await
        .expect("Failed to count");

    assert_eq!(gitlab_count, 1, "Should have 1 GitLab account");

    let bitbucket_count =
        ProviderAccountQueries::count_by_user_and_provider(db, user.id, "bitbucket")
            .await
            .expect("Failed to count");

    assert_eq!(bitbucket_count, 0, "Should have 0 Bitbucket accounts");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_update_validation_status() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();

    let account = create_test_provider_account(db, user.id, "github", "Test Account", true)
        .await
        .unwrap();

    assert_eq!(account.validation_status, "valid");

    // Update to invalid
    let updated = ProviderAccountQueries::update_validation_status(db, account.id, "invalid")
        .await
        .expect("Failed to update status");

    assert_eq!(updated.validation_status, "invalid");
    assert!(updated.last_validated_at.is_some());

    // Verify timestamp was updated
    assert!(updated.updated_at > account.created_at);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_find_active_by_user() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();

    // Create active and inactive accounts
    let active1 = create_test_provider_account(db, user.id, "github", "Active 1", true)
        .await
        .unwrap();
    let active2 = create_test_provider_account(db, user.id, "gitlab", "Active 2", true)
        .await
        .unwrap();
    let inactive = create_test_provider_account(db, user.id, "bitbucket", "Inactive", false)
        .await
        .unwrap();

    // Mark inactive account as inactive
    ProviderAccountQueries::set_active_status(db, inactive.id, false)
        .await
        .expect("Failed to set inactive");

    // Query active accounts
    let active_accounts = ProviderAccountQueries::find_active_by_user(db, user.id)
        .await
        .expect("Failed to find active accounts");

    assert_eq!(active_accounts.len(), 2, "Should have 2 active accounts");

    let active_ids: Vec<_> = active_accounts.iter().map(|a| a.id).collect();
    assert!(active_ids.contains(&active1.id));
    assert!(active_ids.contains(&active2.id));
    assert!(!active_ids.contains(&inactive.id));

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_delete_account_unauthorized() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();
    let other_user = create_test_user(db, "other@example.com", "otheruser")
        .await
        .unwrap();

    let account = create_test_provider_account(db, user.id, "github", "Test Account", true)
        .await
        .unwrap();

    // Try to delete as different user
    let result = ProviderAccountQueries::delete(db, account.id, other_user.id).await;

    assert!(result.is_err(), "Should fail with unauthorized error");
    match result {
        Err(DbErr::Custom(msg)) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Expected Custom error with 'Unauthorized' message"),
    }

    // Verify account still exists
    let exists = Entity::find_by_id(account.id).one(db).await.unwrap();
    assert!(exists.is_some(), "Account should still exist");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_delete_account_success() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();

    let account = create_test_provider_account(db, user.id, "github", "Test Account", true)
        .await
        .unwrap();

    // Delete as owner
    ProviderAccountQueries::delete(db, account.id, user.id)
        .await
        .expect("Failed to delete account");

    // Verify account is deleted
    let exists = Entity::find_by_id(account.id).one(db).await.unwrap();
    assert!(exists.is_none(), "Account should be deleted");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_find_by_user_and_provider() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();

    // Create accounts
    create_test_provider_account(db, user.id, "github", "GitHub 1", true)
        .await
        .unwrap();
    create_test_provider_account(db, user.id, "github", "GitHub 2", false)
        .await
        .unwrap();
    create_test_provider_account(db, user.id, "gitlab", "GitLab 1", true)
        .await
        .unwrap();

    // Query GitHub accounts
    let github_accounts =
        ProviderAccountQueries::find_by_user_and_provider(db, user.id, "github")
            .await
            .expect("Failed to find accounts");

    assert_eq!(github_accounts.len(), 2, "Should have 2 GitHub accounts");

    for account in &github_accounts {
        assert_eq!(account.provider, "github");
        assert_eq!(account.user_id, user.id);
    }

    // Query GitLab accounts
    let gitlab_accounts =
        ProviderAccountQueries::find_by_user_and_provider(db, user.id, "gitlab")
            .await
            .expect("Failed to find accounts");

    assert_eq!(gitlab_accounts.len(), 1, "Should have 1 GitLab account");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_parallel_test_isolation() {
    // This test verifies that parallel test execution doesn't cause conflicts
    let test_db1 = TestDb::new().await.expect("Failed to create test DB 1");
    let test_db2 = TestDb::new().await.expect("Failed to create test DB 2");

    test_db1.run_migrations().await.expect("Failed to run migrations 1");
    test_db2.run_migrations().await.expect("Failed to run migrations 2");

    let user1 = create_test_user(test_db1.connection(), "user1@example.com", "user1")
        .await
        .unwrap();
    let user2 = create_test_user(test_db2.connection(), "user2@example.com", "user2")
        .await
        .unwrap();

    create_test_provider_account(test_db1.connection(), user1.id, "github", "Account 1", true)
        .await
        .unwrap();
    create_test_provider_account(test_db2.connection(), user2.id, "github", "Account 2", true)
        .await
        .unwrap();

    // Verify each database has only its own data
    let accounts1 = ProviderAccountQueries::find_by_user(test_db1.connection(), user1.id)
        .await
        .unwrap();
    let accounts2 = ProviderAccountQueries::find_by_user(test_db2.connection(), user2.id)
        .await
        .unwrap();

    assert_eq!(accounts1.len(), 1);
    assert_eq!(accounts2.len(), 1);
    assert_eq!(accounts1[0].account_label, "Account 1");
    assert_eq!(accounts2[0].account_label, "Account 2");

    test_db1.cleanup().await;
    test_db2.cleanup().await;
}
