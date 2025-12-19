use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create provider_accounts table
        manager
            .create_table(
                Table::create()
                    .table(ProviderAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProviderAccounts::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProviderAccounts::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(ProviderAccounts::Provider)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProviderAccounts::InstanceUrl).string())
                    .col(
                        ColumnDef::new(ProviderAccounts::AccountLabel)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProviderAccounts::ProviderUserId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProviderAccounts::ProviderUsername)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProviderAccounts::ProviderEmail).string())
                    .col(ColumnDef::new(ProviderAccounts::AvatarUrl).string())
                    .col(
                        ColumnDef::new(ProviderAccounts::AuthType)
                            .string()
                            .not_null()
                            .default("pat"),
                    )
                    .col(
                        ColumnDef::new(ProviderAccounts::AccessTokenEncrypted)
                            .binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProviderAccounts::AuthUsername).string())
                    .col(ColumnDef::new(ProviderAccounts::Scopes).text())
                    .col(ColumnDef::new(ProviderAccounts::TokenExpiresAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(ProviderAccounts::LastValidatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(ProviderAccounts::ValidationStatus)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(ProviderAccounts::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(ProviderAccounts::IsDefault)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ProviderAccounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ProviderAccounts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProviderAccounts::Table, ProviderAccounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on user_id for efficient lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_provider_accounts_user_id")
                    .table(ProviderAccounts::Table)
                    .col(ProviderAccounts::UserId)
                    .to_owned(),
            )
            .await?;

        // Index on provider for filtering
        manager
            .create_index(
                Index::create()
                    .name("idx_provider_accounts_provider")
                    .table(ProviderAccounts::Table)
                    .col(ProviderAccounts::Provider)
                    .to_owned(),
            )
            .await?;

        // Partial index on is_active for efficient active account lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_provider_accounts_active")
                    .table(ProviderAccounts::Table)
                    .col(ProviderAccounts::IsActive)
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one label per provider per user (accounting for instance_url)
        manager
            .create_index(
                Index::create()
                    .name("idx_provider_accounts_unique_label")
                    .table(ProviderAccounts::Table)
                    .col(ProviderAccounts::UserId)
                    .col(ProviderAccounts::Provider)
                    .col(ProviderAccounts::InstanceUrl)
                    .col(ProviderAccounts::AccountLabel)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one provider account per user (accounting for instance_url)
        manager
            .create_index(
                Index::create()
                    .name("idx_provider_accounts_unique_account")
                    .table(ProviderAccounts::Table)
                    .col(ProviderAccounts::UserId)
                    .col(ProviderAccounts::Provider)
                    .col(ProviderAccounts::InstanceUrl)
                    .col(ProviderAccounts::ProviderUserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Unique partial index: only one default account per provider per user per instance
        // Note: SeaORM doesn't have direct support for partial unique indexes with WHERE clause
        // We'll need to create this with raw SQL
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE UNIQUE INDEX idx_provider_accounts_default
                ON provider_accounts(user_id, provider, COALESCE(instance_url, ''))
                WHERE is_default = true
                "#,
            )
            .await?;

        // Add provider_account_id column to repositories table
        manager
            .alter_table(
                Table::alter()
                    .table(Repositories::Table)
                    .add_column(ColumnDef::new(Repositories::ProviderAccountId).uuid())
                    .to_owned(),
            )
            .await?;

        // Add foreign key from repositories to provider_accounts
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_repositories_provider_account")
                    .from(Repositories::Table, Repositories::ProviderAccountId)
                    .to(ProviderAccounts::Table, ProviderAccounts::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // Add index for provider_account_id lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_repositories_provider_account")
                    .table(Repositories::Table)
                    .col(Repositories::ProviderAccountId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index on repositories
        manager
            .drop_index(
                Index::drop()
                    .name("idx_repositories_provider_account")
                    .table(Repositories::Table)
                    .to_owned(),
            )
            .await?;

        // Drop foreign key from repositories
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Repositories::Table)
                    .name("fk_repositories_provider_account")
                    .to_owned(),
            )
            .await?;

        // Drop provider_account_id column from repositories
        manager
            .alter_table(
                Table::alter()
                    .table(Repositories::Table)
                    .drop_column(Repositories::ProviderAccountId)
                    .to_owned(),
            )
            .await?;

        // Drop partial unique index (raw SQL)
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_provider_accounts_default")
            .await?;

        // Drop provider_accounts table (indexes drop automatically)
        manager
            .drop_table(Table::drop().table(ProviderAccounts::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ProviderAccounts {
    Table,
    Id,
    UserId,
    Provider,
    InstanceUrl,
    AccountLabel,
    ProviderUserId,
    ProviderUsername,
    ProviderEmail,
    AvatarUrl,
    AuthType,
    AccessTokenEncrypted,
    AuthUsername,
    Scopes,
    TokenExpiresAt,
    LastValidatedAt,
    ValidationStatus,
    IsActive,
    IsDefault,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    ProviderAccountId,
}
