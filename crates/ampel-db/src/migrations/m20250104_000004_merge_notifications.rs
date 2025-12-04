use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add new columns to notification_preferences table
        manager
            .alter_table(
                Table::alter()
                    .table(NotificationPreferences::Table)
                    .add_column(ColumnDef::new(NotificationPreferences::SmtpHost).string())
                    .add_column(ColumnDef::new(NotificationPreferences::SmtpPort).integer())
                    .add_column(ColumnDef::new(NotificationPreferences::SmtpUsername).string())
                    .add_column(
                        ColumnDef::new(NotificationPreferences::SmtpPasswordEncrypted).binary(),
                    )
                    .add_column(ColumnDef::new(NotificationPreferences::SmtpFromEmail).string())
                    .add_column(ColumnDef::new(NotificationPreferences::SmtpToEmails).text())
                    .add_column(
                        ColumnDef::new(NotificationPreferences::SmtpUseTls)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .add_column(
                        ColumnDef::new(NotificationPreferences::NotifyOnMergeSuccess)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .add_column(
                        ColumnDef::new(NotificationPreferences::NotifyOnMergeFailure)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .add_column(ColumnDef::new(NotificationPreferences::SlackChannel).string())
                    .to_owned(),
            )
            .await?;

        // Create user_settings table for behavior config
        manager
            .create_table(
                Table::create()
                    .table(UserSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSettings::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserSettings::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(UserSettings::MergeDelaySeconds)
                            .integer()
                            .not_null()
                            .default(5),
                    )
                    .col(
                        ColumnDef::new(UserSettings::RequireApproval)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserSettings::DeleteBranchesDefault)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserSettings::DefaultMergeStrategy)
                            .string()
                            .not_null()
                            .default("squash"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserSettings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserSettings::Table, UserSettings::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one settings row per user
        manager
            .create_index(
                Index::create()
                    .name("idx_user_settings_user_id")
                    .table(UserSettings::Table)
                    .col(UserSettings::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create merge_operations table
        manager
            .create_table(
                Table::create()
                    .table(MergeOperations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MergeOperations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MergeOperations::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(MergeOperations::StartedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(MergeOperations::CompletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(MergeOperations::TotalCount)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MergeOperations::SuccessCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(MergeOperations::FailedCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(MergeOperations::SkippedCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(MergeOperations::Status)
                            .string()
                            .not_null()
                            .default("in_progress"),
                    )
                    .col(
                        ColumnDef::new(MergeOperations::NotificationSent)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(MergeOperations::Table, MergeOperations::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for finding operations by user
        manager
            .create_index(
                Index::create()
                    .name("idx_merge_operations_user_id")
                    .table(MergeOperations::Table)
                    .col(MergeOperations::UserId)
                    .to_owned(),
            )
            .await?;

        // Create merge_operation_items table
        manager
            .create_table(
                Table::create()
                    .table(MergeOperationItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MergeOperationItems::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MergeOperationItems::MergeOperationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MergeOperationItems::PullRequestId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MergeOperationItems::RepositoryId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MergeOperationItems::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(MergeOperationItems::ErrorMessage).text())
                    .col(
                        ColumnDef::new(MergeOperationItems::MergeSha)
                            .string()
                            .string_len(40),
                    )
                    .col(ColumnDef::new(MergeOperationItems::MergedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                MergeOperationItems::Table,
                                MergeOperationItems::MergeOperationId,
                            )
                            .to(MergeOperations::Table, MergeOperations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                MergeOperationItems::Table,
                                MergeOperationItems::PullRequestId,
                            )
                            .to(PullRequests::Table, PullRequests::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for finding items by operation
        manager
            .create_index(
                Index::create()
                    .name("idx_merge_operation_items_operation_id")
                    .table(MergeOperationItems::Table)
                    .col(MergeOperationItems::MergeOperationId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(MergeOperationItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MergeOperations::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserSettings::Table).to_owned())
            .await?;

        // Remove added columns from notification_preferences
        manager
            .alter_table(
                Table::alter()
                    .table(NotificationPreferences::Table)
                    .drop_column(NotificationPreferences::SmtpHost)
                    .drop_column(NotificationPreferences::SmtpPort)
                    .drop_column(NotificationPreferences::SmtpUsername)
                    .drop_column(NotificationPreferences::SmtpPasswordEncrypted)
                    .drop_column(NotificationPreferences::SmtpFromEmail)
                    .drop_column(NotificationPreferences::SmtpToEmails)
                    .drop_column(NotificationPreferences::SmtpUseTls)
                    .drop_column(NotificationPreferences::NotifyOnMergeSuccess)
                    .drop_column(NotificationPreferences::NotifyOnMergeFailure)
                    .drop_column(NotificationPreferences::SlackChannel)
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
}

#[derive(DeriveIden)]
enum PullRequests {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum NotificationPreferences {
    Table,
    SmtpHost,
    SmtpPort,
    SmtpUsername,
    SmtpPasswordEncrypted,
    SmtpFromEmail,
    SmtpToEmails,
    SmtpUseTls,
    NotifyOnMergeSuccess,
    NotifyOnMergeFailure,
    SlackChannel,
}

#[derive(DeriveIden)]
enum UserSettings {
    Table,
    Id,
    UserId,
    MergeDelaySeconds,
    RequireApproval,
    DeleteBranchesDefault,
    DefaultMergeStrategy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum MergeOperations {
    Table,
    Id,
    UserId,
    StartedAt,
    CompletedAt,
    TotalCount,
    SuccessCount,
    FailedCount,
    SkippedCount,
    Status,
    NotificationSent,
}

#[derive(DeriveIden)]
enum MergeOperationItems {
    Table,
    Id,
    MergeOperationId,
    PullRequestId,
    RepositoryId,
    Status,
    ErrorMessage,
    MergeSha,
    MergedAt,
}
