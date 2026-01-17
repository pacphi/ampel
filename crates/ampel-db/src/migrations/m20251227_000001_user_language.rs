use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add language column to users table
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::Language)
                            .string_len(10)
                            .null()
                            .default("en"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add index on language column for analytics queries
        manager
            .create_index(
                Index::create()
                    .name("idx_users_language")
                    .table(Users::Table)
                    .col(Users::Language)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_language")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        // Remove language column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Language)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Language,
}
