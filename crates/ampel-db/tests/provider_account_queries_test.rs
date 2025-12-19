use ampel_db::entities::provider_account::{ActiveModel, Entity, Model};
use ampel_db::queries::ProviderAccountQueries;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, DbErr, EntityTrait, Set};
use uuid::Uuid;

/// Helper to create a test database connection
async fn setup_test_db() -> DatabaseConnection {
    // Use in-memory SQLite for testing
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Run migrations or create tables manually for tests
    // Note: In a real implementation, you would run actual migrations here
    db
}

/// Helper to create a test user
async fn create_test_user(_db: &DatabaseConnection) -> Uuid {
    // In a real implementation, this would use the user queries
    Uuid::new_v4()
}

/// Helper to create a test provider account
async fn create_test_provider_account(
    db: &DatabaseConnection,
    user_id: Uuid,
    provider: &str,
    account_label: &str,
    is_default: bool,
) -> Model {
    let now = Utc::now();
    let account = ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        provider: Set(provider.to_string()),
        instance_url: Set(None),
        account_label: Set(account_label.to_string()),
        provider_user_id: Set(format!("provider_user_id_{}", account_label)),
        provider_username: Set(format!("username_{}", account_label)),
        provider_email: Set(Some(format!("{}@example.com", account_label))),
        avatar_url: Set(Some("https://example.com/avatar.png".to_string())),
        auth_type: Set("pat".to_string()),
        access_token_encrypted: Set(vec![1, 2, 3, 4, 5]),
        auth_username: Set(None),
        scopes: Set(Some(r#"["repo","read:user"]"#.to_string())),
        token_expires_at: Set(None),
        last_validated_at: Set(Some(now)),
        validation_status: Set("valid".to_string()),
        is_active: Set(true),
        is_default: Set(is_default),
        created_at: Set(now),
        updated_at: Set(now),
    };

    account
        .insert(db)
        .await
        .expect("Failed to create test account")
}

#[tokio::test]
async fn test_find_by_user() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    // Create multiple accounts for the user
    create_test_provider_account(&db, user_id, "github", "Work GitHub", true).await;
    create_test_provider_account(&db, user_id, "github", "Personal GitHub", false).await;
    create_test_provider_account(&db, user_id, "gitlab", "Work GitLab", true).await;

    // Create account for different user
    let other_user_id = create_test_user(&db).await;
    create_test_provider_account(&db, other_user_id, "github", "Other Account", true).await;

    // Test find_by_user
    let accounts = ProviderAccountQueries::find_by_user(&db, user_id)
        .await
        .expect("Failed to find accounts");

    assert_eq!(accounts.len(), 3, "User should have 3 accounts");

    // Verify all accounts belong to the user
    for account in &accounts {
        assert_eq!(account.user_id, user_id);
    }
}

#[tokio::test]
async fn test_find_default_for_provider() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    // Create two GitHub accounts, one default
    let default_account =
        create_test_provider_account(&db, user_id, "github", "Work GitHub", true).await;
    create_test_provider_account(&db, user_id, "github", "Personal GitHub", false).await;

    // Test find_default_for_provider
    let result = ProviderAccountQueries::find_default_for_provider(&db, user_id, "github")
        .await
        .expect("Failed to find default account");

    assert!(result.is_some(), "Should find a default account");
    let found_account = result.unwrap();
    assert_eq!(found_account.id, default_account.id);
    assert!(found_account.is_default);
    assert_eq!(found_account.account_label, "Work GitHub");

    // Test for provider with no default
    let result_gitlab = ProviderAccountQueries::find_default_for_provider(&db, user_id, "gitlab")
        .await
        .expect("Failed to query");

    assert!(
        result_gitlab.is_none(),
        "Should not find default for GitLab"
    );
}

#[tokio::test]
async fn test_set_default_clears_previous() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    // Create two GitHub accounts
    let account1 = create_test_provider_account(&db, user_id, "github", "Account 1", true).await;
    let account2 = create_test_provider_account(&db, user_id, "github", "Account 2", false).await;

    // Verify account1 is default
    let default = ProviderAccountQueries::find_default_for_provider(&db, user_id, "github")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(default.id, account1.id);

    // Set account2 as default
    let updated = ProviderAccountQueries::set_default(&db, account2.id, user_id)
        .await
        .expect("Failed to set default");

    assert!(updated.is_default);
    assert_eq!(updated.id, account2.id);

    // Verify account1 is no longer default
    let account1_refreshed = Entity::find_by_id(account1.id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert!(!account1_refreshed.is_default);

    // Verify only account2 is default
    let new_default = ProviderAccountQueries::find_default_for_provider(&db, user_id, "github")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(new_default.id, account2.id);
}

#[tokio::test]
async fn test_set_default_unauthorized() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;
    let other_user_id = create_test_user(&db).await;

    // Create account for user_id
    let account = create_test_provider_account(&db, user_id, "github", "Test Account", true).await;

    // Try to set default as different user
    let result = ProviderAccountQueries::set_default(&db, account.id, other_user_id).await;

    assert!(result.is_err(), "Should fail with unauthorized error");
    match result {
        Err(DbErr::Custom(msg)) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Expected Custom error with 'Unauthorized' message"),
    }
}

#[tokio::test]
async fn test_count_by_user_and_provider() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    // Create multiple GitHub accounts
    create_test_provider_account(&db, user_id, "github", "Account 1", true).await;
    create_test_provider_account(&db, user_id, "github", "Account 2", false).await;
    create_test_provider_account(&db, user_id, "github", "Account 3", false).await;

    // Create GitLab account
    create_test_provider_account(&db, user_id, "gitlab", "GitLab Account", true).await;

    // Test count
    let github_count = ProviderAccountQueries::count_by_user_and_provider(&db, user_id, "github")
        .await
        .expect("Failed to count");

    assert_eq!(github_count, 3, "Should have 3 GitHub accounts");

    let gitlab_count = ProviderAccountQueries::count_by_user_and_provider(&db, user_id, "gitlab")
        .await
        .expect("Failed to count");

    assert_eq!(gitlab_count, 1, "Should have 1 GitLab account");

    let bitbucket_count =
        ProviderAccountQueries::count_by_user_and_provider(&db, user_id, "bitbucket")
            .await
            .expect("Failed to count");

    assert_eq!(bitbucket_count, 0, "Should have 0 Bitbucket accounts");
}

#[tokio::test]
async fn test_update_validation_status() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let account = create_test_provider_account(&db, user_id, "github", "Test Account", true).await;

    assert_eq!(account.validation_status, "valid");

    // Update to invalid
    let updated = ProviderAccountQueries::update_validation_status(&db, account.id, "invalid")
        .await
        .expect("Failed to update status");

    assert_eq!(updated.validation_status, "invalid");
    assert!(updated.last_validated_at.is_some());

    // Verify timestamp was updated
    assert!(updated.updated_at > account.created_at);
}

#[tokio::test]
async fn test_find_active_by_user() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    // Create active and inactive accounts
    let active1 = create_test_provider_account(&db, user_id, "github", "Active 1", true).await;
    let active2 = create_test_provider_account(&db, user_id, "gitlab", "Active 2", true).await;
    let inactive = create_test_provider_account(&db, user_id, "bitbucket", "Inactive", false).await;

    // Mark inactive account as inactive
    ProviderAccountQueries::set_active_status(&db, inactive.id, false)
        .await
        .expect("Failed to set inactive");

    // Query active accounts
    let active_accounts = ProviderAccountQueries::find_active_by_user(&db, user_id)
        .await
        .expect("Failed to find active accounts");

    assert_eq!(active_accounts.len(), 2, "Should have 2 active accounts");

    let active_ids: Vec<Uuid> = active_accounts.iter().map(|a| a.id).collect();
    assert!(active_ids.contains(&active1.id));
    assert!(active_ids.contains(&active2.id));
    assert!(!active_ids.contains(&inactive.id));
}

#[tokio::test]
async fn test_delete_account_unauthorized() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;
    let other_user_id = create_test_user(&db).await;

    let account = create_test_provider_account(&db, user_id, "github", "Test Account", true).await;

    // Try to delete as different user
    let result = ProviderAccountQueries::delete(&db, account.id, other_user_id).await;

    assert!(result.is_err(), "Should fail with unauthorized error");
    match result {
        Err(DbErr::Custom(msg)) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Expected Custom error with 'Unauthorized' message"),
    }

    // Verify account still exists
    let exists = Entity::find_by_id(account.id).one(&db).await.unwrap();
    assert!(exists.is_some(), "Account should still exist");
}

#[tokio::test]
async fn test_delete_account_success() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    let account = create_test_provider_account(&db, user_id, "github", "Test Account", true).await;

    // Delete as owner
    ProviderAccountQueries::delete(&db, account.id, user_id)
        .await
        .expect("Failed to delete account");

    // Verify account is deleted
    let exists = Entity::find_by_id(account.id).one(&db).await.unwrap();
    assert!(exists.is_none(), "Account should be deleted");
}

#[tokio::test]
async fn test_find_by_user_and_provider() {
    let db = setup_test_db().await;
    let user_id = create_test_user(&db).await;

    // Create accounts
    create_test_provider_account(&db, user_id, "github", "GitHub 1", true).await;
    create_test_provider_account(&db, user_id, "github", "GitHub 2", false).await;
    create_test_provider_account(&db, user_id, "gitlab", "GitLab 1", true).await;

    // Query GitHub accounts
    let github_accounts = ProviderAccountQueries::find_by_user_and_provider(&db, user_id, "github")
        .await
        .expect("Failed to find accounts");

    assert_eq!(github_accounts.len(), 2, "Should have 2 GitHub accounts");

    for account in &github_accounts {
        assert_eq!(account.provider, "github");
        assert_eq!(account.user_id, user_id);
    }

    // Query GitLab accounts
    let gitlab_accounts = ProviderAccountQueries::find_by_user_and_provider(&db, user_id, "gitlab")
        .await
        .expect("Failed to find accounts");

    assert_eq!(gitlab_accounts.len(), 1, "Should have 1 GitLab account");
}
