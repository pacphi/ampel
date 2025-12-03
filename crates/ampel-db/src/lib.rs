pub mod encryption;
pub mod entities;
pub mod migrations;
pub mod queries;

pub use entities::*;
pub use migrations::Migrator;

use sea_orm::{Database, DatabaseConnection, DbErr};

/// Initialize database connection pool
pub async fn init_database(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    Ok(db)
}

/// Run all pending migrations
pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm_migration::MigratorTrait;
    Migrator::up(db, None).await?;
    Ok(())
}
