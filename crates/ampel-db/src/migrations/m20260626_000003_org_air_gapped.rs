use sea_orm_migration::prelude::*;

/// ADR-014: org-level air-gapped ceiling.
///
/// Adds a non-nullable `air_gapped` boolean to `organizations`. When set, the
/// `PolicyResolver` forces the effective policy's `air_gapped` to `true`
/// regardless of the matched policy value (a non-bypassable ceiling).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Organizations::Table)
                    .add_column(
                        ColumnDef::new(Organizations::AirGapped)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Organizations::Table)
                    .drop_column(Organizations::AirGapped)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    AirGapped,
}
