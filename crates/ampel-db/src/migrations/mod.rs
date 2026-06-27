mod m20250101_000001_initial;
mod m20250102_000002_teams;
mod m20250103_000003_pr_filters;
mod m20250104_000004_merge_notifications;
mod m20250105_000005_skip_review_setting;
mod m20250120_000001_provider_accounts;
mod m20251223_000001_repository_filters;
mod m20251224_000001_performance_indexes;
mod m20251227_000001_user_language;
mod m20260626_000001_remediation_loops;
mod m20260626_000002_model_provider_account;
mod m20260626_000003_org_air_gapped;
mod m20260627_000001_remediation_run_phase2_columns;

use sea_orm_migration::prelude::*;

pub struct Migrator;

/// Reusable schema builders for SQLite-backed tests (in this crate and in
/// downstream crates such as `ampel-worker`).
///
/// The full [`Migrator`] cannot run against SQLite (the `provider_accounts`
/// migration uses `ALTER TABLE ... ADD FOREIGN KEY` + a partial unique index).
/// The remediation migrations, however, are self-contained, so this helper
/// applies exactly those — the loops tables plus the Phase-2 columns — directly
/// via a [`SchemaManager`]. It is intentionally `pub` (not `#[cfg(test)]`) so
/// integration tests in other crates can reuse it.
pub mod test_support {
    use sea_orm_migration::prelude::DbErr;
    use sea_orm_migration::{MigrationTrait, SchemaManager};

    /// Apply the remediation schema (loops tables + Phase-2 columns) to `manager`.
    pub async fn apply_remediation_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        super::m20260626_000001_remediation_loops::Migration
            .up(manager)
            .await?;
        super::m20260627_000001_remediation_run_phase2_columns::Migration
            .up(manager)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_initial::Migration),
            Box::new(m20250102_000002_teams::Migration),
            Box::new(m20250103_000003_pr_filters::Migration),
            Box::new(m20250104_000004_merge_notifications::Migration),
            Box::new(m20250105_000005_skip_review_setting::Migration),
            Box::new(m20250120_000001_provider_accounts::Migration),
            Box::new(m20251223_000001_repository_filters::Migration),
            Box::new(m20251224_000001_performance_indexes::Migration),
            Box::new(m20251227_000001_user_language::Migration),
            Box::new(m20260626_000001_remediation_loops::Migration),
            Box::new(m20260626_000002_model_provider_account::Migration),
            Box::new(m20260626_000003_org_air_gapped::Migration),
            Box::new(m20260627_000001_remediation_run_phase2_columns::Migration),
        ]
    }
}

#[cfg(test)]
mod tests {
    //! SQLite migration tests for the Phase 1 Fleet PR Remediation tables.
    //!
    //! The full `Migrator` cannot run against SQLite because the pre-existing
    //! `provider_accounts` migration uses `ALTER TABLE ... ADD FOREIGN KEY` and a
    //! partial unique index (documented in `tests/common/mod.rs`). The two new
    //! remediation migrations are self-contained (no FKs to pre-existing tables),
    //! so we apply them directly via a `SchemaManager` against a fresh in-memory
    //! SQLite database.

    use crate::entities::{
        model_provider_account, remediation_agent_session, remediation_playbook,
        remediation_policy, remediation_run, remediation_run_pr,
    };
    use sea_orm::{Database, DatabaseConnection, EntityTrait};
    use sea_orm_migration::{MigrationTrait, SchemaManager};

    async fn apply_remediation_migrations() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect sqlite");
        let manager = SchemaManager::new(&conn);

        super::m20260626_000001_remediation_loops::Migration
            .up(&manager)
            .await
            .expect("up remediation_loops");
        super::m20260627_000001_remediation_run_phase2_columns::Migration
            .up(&manager)
            .await
            .expect("up remediation_run_phase2_columns");
        super::m20260626_000002_model_provider_account::Migration
            .up(&manager)
            .await
            .expect("up model_provider_account");

        conn
    }

    #[tokio::test]
    async fn should_create_all_six_remediation_tables_on_sqlite() {
        // Arrange + Act
        let conn = apply_remediation_migrations().await;

        // Assert: each table is queryable, proving it exists with the mapped columns.
        remediation_policy::Entity::find()
            .all(&conn)
            .await
            .expect("remediation_policy table exists");
        remediation_run::Entity::find()
            .all(&conn)
            .await
            .expect("remediation_run table exists");
        remediation_run_pr::Entity::find()
            .all(&conn)
            .await
            .expect("remediation_run_pr table exists");
        remediation_agent_session::Entity::find()
            .all(&conn)
            .await
            .expect("remediation_agent_session table exists");
        model_provider_account::Entity::find()
            .all(&conn)
            .await
            .expect("model_provider_account table exists");
        remediation_playbook::Entity::find()
            .all(&conn)
            .await
            .expect("remediation_playbook table exists");
    }

    #[tokio::test]
    async fn should_drop_all_remediation_tables_on_down() {
        // Arrange
        let conn = apply_remediation_migrations().await;
        let manager = SchemaManager::new(&conn);

        // Act: reverse the migrations (children first, mirroring FK order).
        super::m20260626_000002_model_provider_account::Migration
            .down(&manager)
            .await
            .expect("down model_provider_account");
        super::m20260626_000001_remediation_loops::Migration
            .down(&manager)
            .await
            .expect("down remediation_loops");

        // Assert: a representative table from each migration is gone.
        assert!(
            remediation_run::Entity::find().all(&conn).await.is_err(),
            "remediation_run should be dropped"
        );
        assert!(
            remediation_playbook::Entity::find()
                .all(&conn)
                .await
                .is_err(),
            "remediation_playbook should be dropped"
        );
    }
}
