/// Common test utilities for API integration testing
///
/// This module provides helpers for setting up test applications with
/// real database connections and all required services.
use axum::Router;
use sea_orm::{Database, DatabaseConnection};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use ampel_api::{routes::create_router, AppState, Config};
use ampel_core::services::AuthService;
use ampel_db::encryption::EncryptionService;
use ampel_providers::ProviderFactory;
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

/// Create a test encryption key (32 bytes)
fn create_test_encryption_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    for (i, byte) in key.iter_mut().enumerate() {
        *byte = (i as u8).wrapping_add(42); // Deterministic test key
    }
    key
}

/// Create a test application with all required services
///
/// This sets up:
/// - Auth service with test JWT secret
/// - Encryption service with test key
/// - Provider factory
/// - App state with test configuration
/// - Full router with all routes
pub async fn create_test_app(db: DatabaseConnection) -> Router {
    let config = create_test_config();

    let auth_service = AuthService::new(
        config.jwt_secret.clone(),
        config.jwt_access_expiry_minutes,
        config.jwt_refresh_expiry_days,
    );

    let encryption_key = create_test_encryption_key();
    let encryption_service = EncryptionService::new(&encryption_key);

    let provider_factory = ProviderFactory::new();

    let state = AppState::new(
        db,
        auth_service,
        encryption_service,
        provider_factory,
        config,
    );

    create_router(state)
}

/// Create test configuration with safe defaults
fn create_test_config() -> Config {
    Config {
        database_url: "postgresql://test".to_string(), // Not used in tests (we pass connection directly)
        host: "127.0.0.1".to_string(),
        port: 8080,
        jwt_secret: "test-secret-key-at-least-32-chars-long!!!".to_string(),
        jwt_access_expiry_minutes: 15,
        jwt_refresh_expiry_days: 7,
        encryption_key: "test-encryption-key-32-bytes!!!!".to_string(),
        cors_origins: vec!["http://localhost:3000".to_string()],
    }
}
