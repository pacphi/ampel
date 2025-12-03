use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Remove password_hash from users (OAuth-only authentication)
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::PasswordHash)
                    .to_owned(),
            )
            .await?;

        // 2. Create user_oauth_accounts table for social login
        manager
            .create_table(
                Table::create()
                    .table(UserOauthAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserOauthAccounts::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserOauthAccounts::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserOauthAccounts::Provider)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserOauthAccounts::ProviderUserId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserOauthAccounts::ProviderEmail)
                            .string_len(255),
                    )
                    .col(
                        ColumnDef::new(UserOauthAccounts::ProviderUsername)
                            .string_len(255),
                    )
                    .col(
                        ColumnDef::new(UserOauthAccounts::AvatarUrl)
                            .string_len(500),
                    )
                    .col(
                        ColumnDef::new(UserOauthAccounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserOauthAccounts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserOauthAccounts::Table, UserOauthAccounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one OAuth account per provider per provider_user_id
        manager
            .create_index(
                Index::create()
                    .name("idx_user_oauth_accounts_provider_user")
                    .table(UserOauthAccounts::Table)
                    .col(UserOauthAccounts::Provider)
                    .col(UserOauthAccounts::ProviderUserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index for looking up OAuth accounts by user
        manager
            .create_index(
                Index::create()
                    .name("idx_user_oauth_accounts_user_id")
                    .table(UserOauthAccounts::Table)
                    .col(UserOauthAccounts::UserId)
                    .to_owned(),
            )
            .await?;

        // 3. Modify git_providers for PAT-based multi-connection support
        // Add connection_name column
        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .add_column(
                        ColumnDef::new(GitProviders::ConnectionName)
                            .string_len(100)
                            .not_null()
                            .default("default"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add base_url column (for self-hosted instances)
        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .add_column(
                        ColumnDef::new(GitProviders::BaseUrl)
                            .string_len(500),
                    )
                    .to_owned(),
            )
            .await?;

        // Add is_validated column
        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .add_column(
                        ColumnDef::new(GitProviders::IsValidated)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        // Add validation_error column
        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .add_column(
                        ColumnDef::new(GitProviders::ValidationError)
                            .string_len(500),
                    )
                    .to_owned(),
            )
            .await?;

        // Drop OAuth-specific columns (PATs don't need these)
        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .drop_column(GitProviders::RefreshTokenEncrypted)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .drop_column(GitProviders::TokenExpiresAt)
                    .to_owned(),
            )
            .await?;

        // Drop old unique constraint (one per provider per user)
        manager
            .drop_index(
                Index::drop()
                    .name("idx_git_providers_user_provider")
                    .table(GitProviders::Table)
                    .to_owned(),
            )
            .await?;

        // Add new unique constraint (multiple connections per provider allowed, unique by name)
        manager
            .create_index(
                Index::create()
                    .name("idx_provider_connections_user_provider_name")
                    .table(GitProviders::Table)
                    .col(GitProviders::UserId)
                    .col(GitProviders::Provider)
                    .col(GitProviders::ConnectionName)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 4. Rename git_providers table to provider_connections
        manager
            .rename_table(
                Table::rename()
                    .table(GitProviders::Table, ProviderConnections::Table)
                    .to_owned(),
            )
            .await?;

        // 5. Add connection_id to repositories table
        manager
            .alter_table(
                Table::alter()
                    .table(Repositories::Table)
                    .add_column(ColumnDef::new(Repositories::ConnectionId).uuid())
                    .to_owned(),
            )
            .await?;

        // Add foreign key for connection_id
        manager
            .alter_table(
                Table::alter()
                    .table(Repositories::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_repositories_connection_id")
                            .from_col(Repositories::ConnectionId)
                            .to_tbl(ProviderConnections::Table)
                            .to_col(ProviderConnections::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for efficient connection_id lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_repositories_connection_id")
                    .table(Repositories::Table)
                    .col(Repositories::ConnectionId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove connection_id foreign key and index from repositories
        manager
            .drop_index(
                Index::drop()
                    .name("idx_repositories_connection_id")
                    .table(Repositories::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Repositories::Table)
                    .drop_foreign_key(Alias::new("fk_repositories_connection_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Repositories::Table)
                    .drop_column(Repositories::ConnectionId)
                    .to_owned(),
            )
            .await?;

        // Rename provider_connections back to git_providers
        manager
            .rename_table(
                Table::rename()
                    .table(ProviderConnections::Table, GitProviders::Table)
                    .to_owned(),
            )
            .await?;

        // Drop new unique constraint
        manager
            .drop_index(
                Index::drop()
                    .name("idx_provider_connections_user_provider_name")
                    .table(GitProviders::Table)
                    .to_owned(),
            )
            .await?;

        // Re-add old unique constraint
        manager
            .create_index(
                Index::create()
                    .name("idx_git_providers_user_provider")
                    .table(GitProviders::Table)
                    .col(GitProviders::UserId)
                    .col(GitProviders::Provider)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Re-add OAuth columns
        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .add_column(ColumnDef::new(GitProviders::TokenExpiresAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .add_column(ColumnDef::new(GitProviders::RefreshTokenEncrypted).binary())
                    .to_owned(),
            )
            .await?;

        // Drop new columns
        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .drop_column(GitProviders::ValidationError)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .drop_column(GitProviders::IsValidated)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .drop_column(GitProviders::BaseUrl)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GitProviders::Table)
                    .drop_column(GitProviders::ConnectionName)
                    .to_owned(),
            )
            .await?;

        // Drop user_oauth_accounts table
        manager
            .drop_table(Table::drop().table(UserOauthAccounts::Table).to_owned())
            .await?;

        // Re-add password_hash column to users
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    PasswordHash,
}

#[derive(DeriveIden)]
enum UserOauthAccounts {
    Table,
    Id,
    UserId,
    Provider,
    ProviderUserId,
    ProviderEmail,
    ProviderUsername,
    AvatarUrl,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum GitProviders {
    Table,
    UserId,
    Provider,
    ConnectionName,
    BaseUrl,
    IsValidated,
    ValidationError,
    RefreshTokenEncrypted,
    TokenExpiresAt,
}

#[derive(DeriveIden)]
enum ProviderConnections {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    ConnectionId,
}
