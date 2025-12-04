use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // PR Filters table - user-level global filter settings
        manager
            .create_table(
                Table::create()
                    .table(PrFilters::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PrFilters::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PrFilters::UserId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(PrFilters::AllowedActors)
                            .text()
                            .not_null()
                            .default("[]"),
                    ) // JSON array
                    .col(
                        ColumnDef::new(PrFilters::SkipLabels)
                            .text()
                            .not_null()
                            .default("[]"),
                    ) // JSON array
                    .col(ColumnDef::new(PrFilters::MaxAgeDays).integer()) // Optional max age in days
                    .col(
                        ColumnDef::new(PrFilters::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PrFilters::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PrFilters::Table, PrFilters::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PrFilters::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum PrFilters {
    Table,
    Id,
    UserId,
    AllowedActors,
    SkipLabels,
    MaxAgeDays,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
