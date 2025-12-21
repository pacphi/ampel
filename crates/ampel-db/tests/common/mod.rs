/// Common test utilities for database testing
///
/// This module provides helpers for setting up isolated test databases,
/// running migrations, and cleaning up after tests.
pub mod fixtures;

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
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
    ///
    /// This creates a completely isolated PostgreSQL database for testing.
    /// Uses DATABASE_URL or TEST_DATABASE_URL environment variable.
    pub async fn new_postgres() -> Result<Self, DbErr> {
        // Check for DATABASE_URL first (used in CI), then TEST_DATABASE_URL
        let base_url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("TEST_DATABASE_URL"))
            .unwrap_or_else(|_| "postgres://ampel:ampel@localhost:5432".to_string());

        // Extract base URL without database name
        let base_url = if let Some(last_slash) = base_url.rfind('/') {
            &base_url[..last_slash]
        } else {
            &base_url
        };

        let db_id = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_name = format!("ampel_test_{}_{}", db_id, Uuid::new_v4().simple());

        // Connect to postgres database to create test database
        let postgres_url = format!("{}/postgres", base_url);
        let postgres_conn = Database::connect(&postgres_url).await?;

        // Create test database
        postgres_conn
            .execute_unprepared(&format!("CREATE DATABASE {}", db_name))
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to create test database: {}", e)))?;

        // Connect to the new test database
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
    ///
    /// This creates a completely isolated SQLite database using a temporary file,
    /// ensuring no state is shared between tests.
    pub async fn new_sqlite() -> Result<Self, DbErr> {
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
    ///
    /// Uses PostgreSQL if TEST_DATABASE_TYPE=postgres or DATABASE_URL points to postgres,
    /// otherwise falls back to SQLite. This ensures compatibility with CI environments.
    pub async fn new() -> Result<Self, DbErr> {
        if Self::should_use_postgres() {
            Self::new_postgres().await
        } else {
            Self::new_sqlite().await
        }
    }

    /// Check if we should use PostgreSQL for tests
    fn should_use_postgres() -> bool {
        // Check TEST_DATABASE_TYPE (primary method used in CI)
        if let Ok(db_type) = std::env::var("TEST_DATABASE_TYPE") {
            return db_type.to_lowercase().contains("postgres");
        }

        // Check if DATABASE_URL points to PostgreSQL
        if let Ok(url) = std::env::var("DATABASE_URL") {
            return url.starts_with("postgres://") || url.starts_with("postgresql://");
        }

        // Check TEST_DATABASE_URL
        if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
            return url.starts_with("postgres://") || url.starts_with("postgresql://");
        }

        // Explicit opt-in
        std::env::var("USE_POSTGRES_TESTS").is_ok()
    }

    /// Check if the current test environment supports migrations
    ///
    /// Migrations require PostgreSQL because they use features not supported by SQLite:
    /// - ALTER TABLE ADD COLUMN with FOREIGN KEY
    /// - Partial unique indexes with WHERE clause
    ///
    /// Returns true if PostgreSQL is being used, false if SQLite.
    pub fn supports_migrations() -> bool {
        Self::should_use_postgres()
    }

    /// Skip test if migrations are not supported
    ///
    /// Call this at the start of tests that require migrations.
    /// Returns true if the test should be skipped (SQLite mode).
    pub fn skip_if_sqlite() -> bool {
        if !Self::supports_migrations() {
            eprintln!("Skipping test: requires PostgreSQL (migrations not SQLite-compatible)");
            true
        } else {
            false
        }
    }

    /// Run database migrations
    ///
    /// This sets up all required tables for testing.
    /// Note: Migrations require PostgreSQL. Use `skip_if_sqlite()` to skip tests
    /// that require migrations when running in SQLite mode.
    pub async fn run_migrations(&self) -> Result<(), DbErr> {
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

        // Explicitly drop self to close connection
        drop(self);

        // For PostgreSQL, drop the test database
        if backend == DbBackend::Postgres {
            if let Some(name) = db_name {
                let base_url = std::env::var("DATABASE_URL")
                    .or_else(|_| std::env::var("TEST_DATABASE_URL"))
                    .unwrap_or_else(|_| "postgres://ampel:ampel@localhost:5432".to_string());

                // Extract base URL without database name
                let base_url = if let Some(last_slash) = base_url.rfind('/') {
                    &base_url[..last_slash]
                } else {
                    &base_url
                };
                let postgres_url = format!("{}/postgres", base_url);

                // Connect to postgres database to drop test database
                if let Ok(postgres_conn) = Database::connect(&postgres_url).await {
                    // Terminate existing connections to the test database
                    let _ = postgres_conn
                        .execute_unprepared(&format!(
                            "SELECT pg_terminate_backend(pg_stat_activity.pid) \
                             FROM pg_stat_activity \
                             WHERE pg_stat_activity.datname = '{}' \
                             AND pid <> pg_backend_pid()",
                            name
                        ))
                        .await;

                    // Drop the database
                    if let Err(e) = postgres_conn
                        .execute_unprepared(&format!("DROP DATABASE IF EXISTS {}", name))
                        .await
                    {
                        eprintln!("Failed to drop test database {}: {}", name, e);
                    }
                }
            }
        }

        // For SQLite, delete file-based databases
        if let Some(path) = file_path {
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    eprintln!("Failed to delete test database file {:?}: {}", path, e);
                }
            }
        }
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        // Best-effort cleanup of file-based databases
        if let Some(ref file_path) = self.file_path {
            if file_path.exists() {
                let _ = std::fs::remove_file(file_path);
            }
        }
        // Note: PostgreSQL databases are cleaned up in the cleanup() method
        // because Drop cannot be async
    }
}

/// Convenience macro for creating a test database and running migrations
#[macro_export]
macro_rules! setup_test_db {
    () => {{
        let test_db = $crate::common::TestDb::new()
            .await
            .expect("Failed to create test database");
        test_db
            .run_migrations()
            .await
            .expect("Failed to run migrations");
        test_db
    }};
}

/// Convenience macro for test database with custom setup
#[macro_export]
macro_rules! setup_test_db_with {
    ($setup:expr) => {{
        let test_db = $crate::common::TestDb::new()
            .await
            .expect("Failed to create test database");
        test_db
            .run_migrations()
            .await
            .expect("Failed to run migrations");
        $setup(&test_db.connection)
            .await
            .expect("Failed to run custom setup");
        test_db
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_sqlite_db() {
        let test_db = TestDb::new_sqlite().await.unwrap();
        assert_eq!(test_db.backend, DbBackend::Sqlite);
        assert!(test_db.file_path.is_some());
        assert!(test_db.db_name.is_none());

        let file_path = test_db.file_path.clone().unwrap();
        assert!(file_path.exists());

        test_db.cleanup().await;
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_multiple_databases_are_isolated() {
        let test_db1 = TestDb::new().await.unwrap();
        let test_db2 = TestDb::new().await.unwrap();

        // Each should have its own unique identifier (db_name for Postgres, file_path for SQLite)
        // We verify isolation by checking these identifiers differ
        match (&test_db1.db_name, &test_db2.db_name) {
            (Some(name1), Some(name2)) => {
                // PostgreSQL: database names should differ
                assert_ne!(
                    name1, name2,
                    "PostgreSQL databases should have different names"
                );
            }
            (None, None) => {
                // SQLite: file paths should differ
                match (&test_db1.file_path, &test_db2.file_path) {
                    (Some(path1), Some(path2)) => {
                        assert_ne!(
                            path1, path2,
                            "SQLite databases should have different file paths"
                        );
                    }
                    _ => panic!("SQLite databases should have file paths"),
                }
            }
            _ => panic!("Both databases should use the same backend"),
        }

        test_db1.cleanup().await;
        test_db2.cleanup().await;
    }
}
