use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add skip_review_requirement column to user_settings table
        manager
            .alter_table(
                Table::alter()
                    .table(UserSettings::Table)
                    .add_column(
                        ColumnDef::new(UserSettings::SkipReviewRequirement)
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
                    .table(UserSettings::Table)
                    .drop_column(UserSettings::SkipReviewRequirement)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum UserSettings {
    Table,
    SkipReviewRequirement,
}
