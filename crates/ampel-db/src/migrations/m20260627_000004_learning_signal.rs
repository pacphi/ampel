//! `learning_signal` table (Phase 5b — Strategy Learning).
//!
//! One row is appended per completed agentic remediation session, recording the
//! `(provider, failure_class)` pairing, the playbook that drove it, the terminal
//! `outcome` (`passed` | `exhausted`), wall-clock `duration_secs`, and the run
//! `cost_usd` (Decimal-as-string, cross-DB safe). These rows are the training
//! signal the `PolicyResolver` aggregates to bias the `fallback_chain` model
//! ordering toward providers with the highest historical pass-rate per failure
//! class.
//!
//! # Security
//! Signals carry the provider *kind* only (`claude`/`gemini`/`ollama`/`onnx`) —
//! never an API key, endpoint, or any credential material.
//!
//! Plain `CREATE TABLE` + secondary indexes (no FKs / partial indexes), so it
//! applies on SQLite as well as PostgreSQL.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LearningSignal::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LearningSignal::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LearningSignal::Provider).string().not_null())
                    .col(
                        ColumnDef::new(LearningSignal::FailureClass)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LearningSignal::PlaybookId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LearningSignal::PlaybookVersion)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LearningSignal::Outcome).string().not_null())
                    .col(
                        ColumnDef::new(LearningSignal::DurationSecs)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LearningSignal::CostUsd).string())
                    .col(
                        ColumnDef::new(LearningSignal::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Bias lookups filter by (failure_class, provider) to aggregate pass-rate.
        manager
            .create_index(
                Index::create()
                    .name("idx_learning_signal_class_provider")
                    .table(LearningSignal::Table)
                    .col(LearningSignal::FailureClass)
                    .col(LearningSignal::Provider)
                    .to_owned(),
            )
            .await?;
        // Recency windows / pruning scan by created_at.
        manager
            .create_index(
                Index::create()
                    .name("idx_learning_signal_created_at")
                    .table(LearningSignal::Table)
                    .col(LearningSignal::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LearningSignal::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum LearningSignal {
    Table,
    Id,
    Provider,
    FailureClass,
    PlaybookId,
    PlaybookVersion,
    Outcome,
    DurationSecs,
    CostUsd,
    CreatedAt,
}
