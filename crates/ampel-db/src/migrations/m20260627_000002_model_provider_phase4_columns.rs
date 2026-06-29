//! Phase-4 (Agentic Remediation Tier) columns for `model_provider_account`
//! and `remediation_agent_session`.
//!
//! The Phase-1 tables predate the agentic tier and lack the columns the
//! credential/validation/spend-accounting flow (ADR-008) and the failure
//! classifier (ADR-012) need to persist. This migration adds them with plain
//! `ADD COLUMN`s (defaults, no FKs / partial indexes) so it applies on SQLite as
//! well as PostgreSQL.
//!
//! Money values (`spend_cap_usd`, `spend_used_usd`) are stored as strings and
//! parsed to [`rust_decimal::Decimal`] at the service layer — never `f64` — for
//! cross-DB exactness, mirroring `remediation_agent_session.cost_usd`.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ---- model_provider_account -----------------------------------------
        for mut col in [
            ColumnDef::new(ModelProviderAccount::AuthType)
                .string()
                .not_null()
                .default("api_key")
                .to_owned(),
            ColumnDef::new(ModelProviderAccount::SpendCapUsd)
                .string()
                .to_owned(),
            ColumnDef::new(ModelProviderAccount::SpendUsedUsd)
                .string()
                .not_null()
                .default("0")
                .to_owned(),
            ColumnDef::new(ModelProviderAccount::ValidationStatus)
                .string()
                .not_null()
                .default("unvalidated")
                .to_owned(),
            ColumnDef::new(ModelProviderAccount::LastValidatedAt)
                .timestamp_with_time_zone()
                .to_owned(),
            ColumnDef::new(ModelProviderAccount::ModelPath)
                .string()
                .to_owned(),
            ColumnDef::new(ModelProviderAccount::IsDefault)
                .boolean()
                .not_null()
                .default(false)
                .to_owned(),
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(ModelProviderAccount::Table)
                        .add_column(&mut col)
                        .to_owned(),
                )
                .await?;
        }

        // ---- remediation_agent_session --------------------------------------
        for mut col in [
            ColumnDef::new(RemediationAgentSession::FailureClass)
                .string()
                .to_owned(),
            ColumnDef::new(RemediationAgentSession::ClassifierSource)
                .string()
                .to_owned(),
            ColumnDef::new(RemediationAgentSession::ClassifierConfidence)
                .double()
                .to_owned(),
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(RemediationAgentSession::Table)
                        .add_column(&mut col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [
            RemediationAgentSession::ClassifierConfidence,
            RemediationAgentSession::ClassifierSource,
            RemediationAgentSession::FailureClass,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(RemediationAgentSession::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }
        for col in [
            ModelProviderAccount::IsDefault,
            ModelProviderAccount::ModelPath,
            ModelProviderAccount::LastValidatedAt,
            ModelProviderAccount::ValidationStatus,
            ModelProviderAccount::SpendUsedUsd,
            ModelProviderAccount::SpendCapUsd,
            ModelProviderAccount::AuthType,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(ModelProviderAccount::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ModelProviderAccount {
    Table,
    AuthType,
    SpendCapUsd,
    SpendUsedUsd,
    ValidationStatus,
    LastValidatedAt,
    ModelPath,
    IsDefault,
}

#[derive(DeriveIden)]
enum RemediationAgentSession {
    Table,
    FailureClass,
    ClassifierSource,
    ClassifierConfidence,
}
