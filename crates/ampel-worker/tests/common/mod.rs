/// Common test utilities for worker testing
///
/// This module provides helpers for setting up test databases,
/// mock providers, and encryption services for worker job testing.
use ampel_core::models::GitProvider;
use ampel_db::encryption::EncryptionService;
use ampel_db::entities::{provider_account, user};
use ampel_providers::GitProvider as GitProviderTrait;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

/// Global counter for unique test database IDs
static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Database backend type for tests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbBackend {
    Sqlite,
    Postgres,
}

/// Test database configuration
pub struct TestDb {
    pub connection: DatabaseConnection,
    pub file_path: Option<PathBuf>,
    pub db_name: Option<String>,
    backend: DbBackend,
}

impl TestDb {
    /// Create a new PostgreSQL test database with unique identifier
    pub async fn new_postgres() -> Result<Self, sea_orm::DbErr> {
        let base_url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("TEST_DATABASE_URL"))
            .unwrap_or_else(|_| "postgres://ampel:ampel@localhost:5432".to_string());

        let base_url = if let Some(last_slash) = base_url.rfind('/') {
            &base_url[..last_slash]
        } else {
            &base_url
        };

        let db_id = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_name = format!("ampel_test_{}_{}", db_id, Uuid::new_v4().simple());

        let postgres_url = format!("{}/postgres", base_url);
        let postgres_conn = Database::connect(&postgres_url).await?;

        use sea_orm::ConnectionTrait;
        postgres_conn
            .execute_unprepared(&format!("CREATE DATABASE {}", db_name))
            .await
            .map_err(|e| {
                sea_orm::DbErr::Custom(format!("Failed to create test database: {}", e))
            })?;

        let test_db_url = format!("{}/{}", base_url, db_name);
        let connection = Database::connect(&test_db_url).await?;

        Ok(Self {
            connection,
            file_path: None,
            db_name: Some(db_name),
            backend: DbBackend::Postgres,
        })
    }

    /// Create a new SQLite test database with unique identifier
    pub async fn new_sqlite() -> Result<Self, sea_orm::DbErr> {
        let db_id = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("ampel_test_{}_{}.db", db_id, Uuid::new_v4()));

        let connection_string = format!("sqlite://{}?mode=rwc", file_path.display());
        let connection = Database::connect(&connection_string).await?;

        Ok(Self {
            connection,
            file_path: Some(file_path),
            db_name: None,
            backend: DbBackend::Sqlite,
        })
    }

    /// Create a test database with environment-based backend selection
    pub async fn new() -> Result<Self, sea_orm::DbErr> {
        if Self::should_use_postgres() {
            Self::new_postgres().await
        } else {
            Self::new_sqlite().await
        }
    }

    /// Check if we should use PostgreSQL for tests
    fn should_use_postgres() -> bool {
        if let Ok(db_type) = std::env::var("TEST_DATABASE_TYPE") {
            return db_type.to_lowercase().contains("postgres");
        }

        if let Ok(url) = std::env::var("DATABASE_URL") {
            return url.starts_with("postgres://") || url.starts_with("postgresql://");
        }

        if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
            return url.starts_with("postgres://") || url.starts_with("postgresql://");
        }

        std::env::var("USE_POSTGRES_TESTS").is_ok()
    }

    /// Check if the current test environment supports migrations
    pub fn supports_migrations() -> bool {
        Self::should_use_postgres()
    }

    /// Skip test if migrations are not supported
    pub fn skip_if_sqlite() -> bool {
        if !Self::supports_migrations() {
            eprintln!("Skipping test: requires PostgreSQL (migrations not SQLite-compatible)");
            true
        } else {
            false
        }
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<(), sea_orm::DbErr> {
        use ampel_db::migrations::Migrator;
        use sea_orm_migration::MigratorTrait;

        Migrator::up(&self.connection, None).await
    }

    /// Get a reference to the database connection
    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    /// Clean up the test database
    pub async fn cleanup(self) {
        let file_path = self.file_path.clone();
        let db_name = self.db_name.clone();
        let backend = self.backend;

        drop(self);

        if backend == DbBackend::Postgres {
            if let Some(name) = db_name {
                let base_url = std::env::var("DATABASE_URL")
                    .or_else(|_| std::env::var("TEST_DATABASE_URL"))
                    .unwrap_or_else(|_| "postgres://ampel:ampel@localhost:5432".to_string());

                let base_url = if let Some(last_slash) = base_url.rfind('/') {
                    &base_url[..last_slash]
                } else {
                    &base_url
                };
                let postgres_url = format!("{}/postgres", base_url);

                if let Ok(postgres_conn) = Database::connect(&postgres_url).await {
                    use sea_orm::ConnectionTrait;
                    let _ = postgres_conn
                        .execute_unprepared(&format!(
                            "SELECT pg_terminate_backend(pg_stat_activity.pid) \
                             FROM pg_stat_activity \
                             WHERE pg_stat_activity.datname = '{}' \
                             AND pid <> pg_backend_pid()",
                            name
                        ))
                        .await;

                    let _ = postgres_conn
                        .execute_unprepared(&format!("DROP DATABASE IF EXISTS {}", name))
                        .await;
                }
            }
        }

        if let Some(path) = file_path {
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

/// Helper to create test user
pub async fn create_test_user(
    db: &DatabaseConnection,
    email: &str,
    display_name: &str,
) -> Result<user::Model, sea_orm::DbErr> {
    let now = Utc::now();
    let user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email.to_string()),
        display_name: Set(Some(display_name.to_string())),
        avatar_url: Set(None),
        password_hash: Set("$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };

    user.insert(db).await
}

/// Helper to create test provider account
#[allow(dead_code)]
pub async fn create_test_provider_account(
    db: &DatabaseConnection,
    user_id: Uuid,
    provider: &str,
    label: &str,
    is_default: bool,
) -> Result<provider_account::Model, sea_orm::DbErr> {
    let now = Utc::now();
    let account = provider_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        provider: Set(provider.to_string()),
        instance_url: Set(None),
        account_label: Set(label.to_string()),
        provider_user_id: Set(format!("provider_id_{}", label)),
        provider_username: Set(format!("username_{}", label)),
        provider_email: Set(Some(format!("{}@example.com", label))),
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

    account.insert(db).await
}

/// Create a test encryption service with a predictable key
pub fn create_test_encryption_service() -> EncryptionService {
    // Create a deterministic test key (32 bytes)
    let mut key = [0u8; 32];
    for (i, byte) in key.iter_mut().enumerate() {
        *byte = i as u8;
    }
    EncryptionService::new(&key)
}

/// Mock provider that tracks calls and returns predefined data
#[derive(Clone)]
#[allow(dead_code)]
pub struct MockProvider {
    pub provider_type: GitProvider,
    pub pull_requests: Arc<std::sync::Mutex<Vec<ampel_providers::traits::ProviderPullRequest>>>,
    pub ci_checks: Arc<std::sync::Mutex<Vec<ampel_providers::traits::ProviderCICheck>>>,
    pub reviews: Arc<std::sync::Mutex<Vec<ampel_providers::traits::ProviderReview>>>,
    pub call_log: Arc<std::sync::Mutex<Vec<String>>>,
}

#[allow(dead_code)]
impl MockProvider {
    pub fn new(provider_type: GitProvider) -> Self {
        Self {
            provider_type,
            pull_requests: Arc::new(std::sync::Mutex::new(Vec::new())),
            ci_checks: Arc::new(std::sync::Mutex::new(Vec::new())),
            reviews: Arc::new(std::sync::Mutex::new(Vec::new())),
            call_log: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn add_pull_request(&self, pr: ampel_providers::traits::ProviderPullRequest) {
        self.pull_requests.lock().unwrap().push(pr);
    }

    pub fn add_ci_check(&self, check: ampel_providers::traits::ProviderCICheck) {
        self.ci_checks.lock().unwrap().push(check);
    }

    pub fn add_review(&self, review: ampel_providers::traits::ProviderReview) {
        self.reviews.lock().unwrap().push(review);
    }

    pub fn get_call_log(&self) -> Vec<String> {
        self.call_log.lock().unwrap().clone()
    }
}

#[async_trait]
impl GitProviderTrait for MockProvider {
    fn provider_type(&self) -> GitProvider {
        self.provider_type
    }

    fn instance_url(&self) -> Option<&str> {
        None
    }

    async fn validate_credentials(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
    ) -> ampel_providers::error::ProviderResult<ampel_providers::traits::TokenValidation> {
        self.call_log
            .lock()
            .unwrap()
            .push("validate_credentials".to_string());
        Ok(ampel_providers::traits::TokenValidation {
            is_valid: true,
            user_id: Some("mock_user".to_string()),
            username: Some("mockuser".to_string()),
            email: Some("mock@example.com".to_string()),
            avatar_url: None,
            scopes: vec![],
            expires_at: None,
            error_message: None,
        })
    }

    async fn get_user(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
    ) -> ampel_providers::error::ProviderResult<ampel_providers::traits::ProviderUser> {
        self.call_log.lock().unwrap().push("get_user".to_string());
        Ok(ampel_providers::traits::ProviderUser {
            id: "mock_user".to_string(),
            username: "mockuser".to_string(),
            email: Some("mock@example.com".to_string()),
            avatar_url: None,
        })
    }

    async fn list_repositories(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
        _page: i32,
        _per_page: i32,
    ) -> ampel_providers::error::ProviderResult<Vec<ampel_core::models::DiscoveredRepository>> {
        self.call_log
            .lock()
            .unwrap()
            .push("list_repositories".to_string());
        Ok(vec![])
    }

    async fn get_repository(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
        _owner: &str,
        _repo: &str,
    ) -> ampel_providers::error::ProviderResult<ampel_core::models::DiscoveredRepository> {
        self.call_log
            .lock()
            .unwrap()
            .push("get_repository".to_string());
        Err(ampel_providers::error::ProviderError::NotFound(
            "Mock provider".to_string(),
        ))
    }

    async fn list_pull_requests(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
        _owner: &str,
        _repo: &str,
        _state: Option<&str>,
    ) -> ampel_providers::error::ProviderResult<Vec<ampel_providers::traits::ProviderPullRequest>>
    {
        self.call_log
            .lock()
            .unwrap()
            .push("list_pull_requests".to_string());
        Ok(self.pull_requests.lock().unwrap().clone())
    }

    async fn get_pull_request(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
        _owner: &str,
        _repo: &str,
        _number: i32,
    ) -> ampel_providers::error::ProviderResult<ampel_providers::traits::ProviderPullRequest> {
        self.call_log
            .lock()
            .unwrap()
            .push("get_pull_request".to_string());
        self.pull_requests.lock().unwrap().first().cloned().ok_or(
            ampel_providers::error::ProviderError::NotFound("PR not found".to_string()),
        )
    }

    async fn get_ci_checks(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
        _owner: &str,
        _repo: &str,
        _pr_number: i32,
    ) -> ampel_providers::error::ProviderResult<Vec<ampel_providers::traits::ProviderCICheck>> {
        self.call_log
            .lock()
            .unwrap()
            .push("get_ci_checks".to_string());
        Ok(self.ci_checks.lock().unwrap().clone())
    }

    async fn get_reviews(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
        _owner: &str,
        _repo: &str,
        _pr_number: i32,
    ) -> ampel_providers::error::ProviderResult<Vec<ampel_providers::traits::ProviderReview>> {
        self.call_log
            .lock()
            .unwrap()
            .push("get_reviews".to_string());
        Ok(self.reviews.lock().unwrap().clone())
    }

    async fn merge_pull_request(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
        _owner: &str,
        _repo: &str,
        _pr_number: i32,
        _merge_request: &ampel_core::models::MergeRequest,
    ) -> ampel_providers::error::ProviderResult<ampel_providers::traits::MergeResult> {
        self.call_log
            .lock()
            .unwrap()
            .push("merge_pull_request".to_string());
        Ok(ampel_providers::traits::MergeResult {
            merged: true,
            sha: Some("abc123".to_string()),
            message: "Merged successfully".to_string(),
        })
    }

    async fn get_rate_limit(
        &self,
        _credentials: &ampel_providers::traits::ProviderCredentials,
    ) -> ampel_providers::error::ProviderResult<ampel_providers::traits::RateLimitInfo> {
        self.call_log
            .lock()
            .unwrap()
            .push("get_rate_limit".to_string());
        Ok(ampel_providers::traits::RateLimitInfo {
            limit: 5000,
            remaining: 4999,
            reset_at: Utc::now(),
        })
    }
}

/// Mock provider factory that returns our mock provider
#[allow(dead_code)]
pub struct MockProviderFactory {
    pub provider: Arc<MockProvider>,
}

#[allow(dead_code)]
impl MockProviderFactory {
    pub fn new(provider_type: GitProvider) -> Self {
        Self {
            provider: Arc::new(MockProvider::new(provider_type)),
        }
    }

    pub fn with_provider(provider: MockProvider) -> Self {
        Self {
            provider: Arc::new(provider),
        }
    }

    pub fn get_provider(&self) -> Arc<MockProvider> {
        Arc::clone(&self.provider)
    }
}

/// Helper to create test PR data
#[allow(dead_code)]
pub fn create_test_pr(
    number: i32,
    title: &str,
    state: &str,
) -> ampel_providers::traits::ProviderPullRequest {
    ampel_providers::traits::ProviderPullRequest {
        provider_id: format!("pr_{}", number),
        number,
        title: title.to_string(),
        description: Some(format!("Description for PR #{}", number)),
        url: format!("https://github.com/test/repo/pull/{}", number),
        state: state.to_string(),
        source_branch: "feature-branch".to_string(),
        target_branch: "main".to_string(),
        author: "testauthor".to_string(),
        author_avatar_url: Some("https://example.com/avatar.png".to_string()),
        is_draft: false,
        is_mergeable: Some(true),
        has_conflicts: false,
        additions: 100,
        deletions: 50,
        changed_files: 5,
        commits_count: 3,
        comments_count: 2,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        merged_at: None,
        closed_at: None,
    }
}

/// Helper to create test CI check data
#[allow(dead_code)]
pub fn create_test_ci_check(
    name: &str,
    status: &str,
    conclusion: Option<&str>,
) -> ampel_providers::traits::ProviderCICheck {
    let now = Utc::now();
    ampel_providers::traits::ProviderCICheck {
        name: name.to_string(),
        status: status.to_string(),
        conclusion: conclusion.map(|s| s.to_string()),
        url: Some(format!("https://example.com/checks/{}", name)),
        started_at: Some(now),
        completed_at: if status == "completed" {
            Some(now)
        } else {
            None
        },
    }
}

/// Helper to create test review data
#[allow(dead_code)]
pub fn create_test_review(reviewer: &str, state: &str) -> ampel_providers::traits::ProviderReview {
    ampel_providers::traits::ProviderReview {
        id: format!("review_{}", reviewer),
        reviewer: reviewer.to_string(),
        reviewer_avatar_url: Some("https://example.com/avatar.png".to_string()),
        state: state.to_string(),
        body: Some("Looks good to me".to_string()),
        submitted_at: Utc::now(),
    }
}
