//! Ownership scope columns for `remediation_playbook` (authz fix).
//!
//! Playbooks drive the agentic remediation prompts, so write/read access must be
//! gated on ownership rather than mere authentication. This migration adds the
//! same `(scope_type, scope_id)` pair used by `remediation_policy`:
//!
//! - `scope_type` — `org` | `team` | `user` | `repository`. Defaults to `org`.
//! - `scope_id`   — the owning scope's UUID. **Nullable**: a NULL `scope_id`
//!   marks a built-in / global sentinel playbook (the pre-existing rows), which
//!   is readable by any authenticated caller but not mutable by anyone.
//!
//! Plain `ADD COLUMN`s (defaults, no FKs / partial indexes) so it applies on
//! SQLite as well as PostgreSQL, matching the other Phase-4 migrations.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for mut col in [
            ColumnDef::new(RemediationPlaybook::ScopeType)
                .string()
                .not_null()
                .default("org")
                .to_owned(),
            ColumnDef::new(RemediationPlaybook::ScopeId)
                .uuid()
                .to_owned(),
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(RemediationPlaybook::Table)
                        .add_column(&mut col)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for col in [RemediationPlaybook::ScopeId, RemediationPlaybook::ScopeType] {
            manager
                .alter_table(
                    Table::alter()
                        .table(RemediationPlaybook::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum RemediationPlaybook {
    Table,
    ScopeType,
    ScopeId,
}
