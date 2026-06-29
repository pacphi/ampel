//! Phase-2 columns for `remediation_run`.
//!
//! The Phase-1 `remediation_run` table predates the write-path state machine and
//! has no place to persist (a) the granted autonomy level snapshot the
//! orchestrator gates on, nor (b) the verified consolidated-ref HEAD SHA used as
//! the ADR-010 TOCTOU anchor. Both are required by the
//! `ampel_core::services::RemediationRunRepository` contract, so this migration
//! adds them. Plain `ADD COLUMN`s (no FKs / partial indexes), so they apply on
//! SQLite as well as PostgreSQL.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RemediationRun::Table)
                    .add_column(
                        ColumnDef::new(RemediationRun::AutonomyLevel)
                            .string()
                            .not_null()
                            .default("dry_run_only"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(RemediationRun::Table)
                    .add_column(ColumnDef::new(RemediationRun::HeadSha).string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RemediationRun::Table)
                    .drop_column(RemediationRun::HeadSha)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(RemediationRun::Table)
                    .drop_column(RemediationRun::AutonomyLevel)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum RemediationRun {
    Table,
    AutonomyLevel,
    HeadSha,
}
