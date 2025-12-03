use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Teams table
        manager
            .create_table(
                Table::create()
                    .table(Teams::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Teams::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Teams::OrganizationId).uuid().not_null())
                    .col(ColumnDef::new(Teams::Name).string().not_null())
                    .col(ColumnDef::new(Teams::Slug).string().not_null())
                    .col(ColumnDef::new(Teams::Description).string())
                    .col(
                        ColumnDef::new(Teams::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Teams::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Team members table
        manager
            .create_table(
                Table::create()
                    .table(TeamMembers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TeamMembers::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TeamMembers::TeamId).uuid().not_null())
                    .col(ColumnDef::new(TeamMembers::UserId).uuid().not_null())
                    .col(ColumnDef::new(TeamMembers::Role).string().not_null()) // admin, member, viewer
                    .col(
                        ColumnDef::new(TeamMembers::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one membership per team per user
        manager
            .create_index(
                Index::create()
                    .name("idx_team_members_team_user")
                    .table(TeamMembers::Table)
                    .col(TeamMembers::TeamId)
                    .col(TeamMembers::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Team repositories (which repos are assigned to which team)
        manager
            .create_table(
                Table::create()
                    .table(TeamRepositories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TeamRepositories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TeamRepositories::TeamId).uuid().not_null())
                    .col(
                        ColumnDef::new(TeamRepositories::RepositoryId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TeamRepositories::AddedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_team_repositories_team_repo")
                    .table(TeamRepositories::Table)
                    .col(TeamRepositories::TeamId)
                    .col(TeamRepositories::RepositoryId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Notification preferences
        manager
            .create_table(
                Table::create()
                    .table(NotificationPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NotificationPreferences::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NotificationPreferences::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NotificationPreferences::EmailEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(NotificationPreferences::SlackEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(NotificationPreferences::SlackWebhookUrl).string())
                    .col(
                        ColumnDef::new(NotificationPreferences::PushEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(NotificationPreferences::NotifyOnPrReady)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(NotificationPreferences::NotifyOnPrFailed)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(NotificationPreferences::NotifyOnReviewRequested)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(NotificationPreferences::DigestFrequency)
                            .string()
                            .not_null()
                            .default("daily"),
                    ) // none, daily, weekly
                    .col(
                        ColumnDef::new(NotificationPreferences::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Auto-merge rules for bot PRs
        manager
            .create_table(
                Table::create()
                    .table(AutoMergeRules::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AutoMergeRules::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AutoMergeRules::RepositoryId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AutoMergeRules::Enabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AutoMergeRules::BotAuthors)
                            .string()
                            .not_null()
                            .default("[]"),
                    ) // JSON array
                    .col(
                        ColumnDef::new(AutoMergeRules::RequireAllChecks)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AutoMergeRules::RequireApproval)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AutoMergeRules::MergeStrategy)
                            .string()
                            .not_null()
                            .default("squash"),
                    )
                    .col(
                        ColumnDef::new(AutoMergeRules::DeleteBranch)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AutoMergeRules::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AutoMergeRules::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // PR metrics for analytics
        manager
            .create_table(
                Table::create()
                    .table(PrMetrics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PrMetrics::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PrMetrics::PullRequestId).uuid().not_null())
                    .col(ColumnDef::new(PrMetrics::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(PrMetrics::TimeToFirstReview).integer()) // seconds
                    .col(ColumnDef::new(PrMetrics::TimeToApproval).integer())
                    .col(ColumnDef::new(PrMetrics::TimeToMerge).integer())
                    .col(ColumnDef::new(PrMetrics::ReviewRounds).integer())
                    .col(ColumnDef::new(PrMetrics::CommentsCount).integer())
                    .col(
                        ColumnDef::new(PrMetrics::IsBot)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(PrMetrics::MergedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(PrMetrics::RecordedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_pr_metrics_repository_merged")
                    .table(PrMetrics::Table)
                    .col(PrMetrics::RepositoryId)
                    .col(PrMetrics::MergedAt)
                    .to_owned(),
            )
            .await?;

        // Repository health scores
        manager
            .create_table(
                Table::create()
                    .table(HealthScores::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HealthScores::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(HealthScores::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(HealthScores::Score).integer().not_null()) // 0-100
                    .col(ColumnDef::new(HealthScores::AvgTimeToMerge).integer()) // seconds
                    .col(ColumnDef::new(HealthScores::AvgReviewTime).integer())
                    .col(ColumnDef::new(HealthScores::StalePrCount).integer())
                    .col(ColumnDef::new(HealthScores::FailedCheckRate).float())
                    .col(ColumnDef::new(HealthScores::PrThroughput).integer()) // PRs merged in period
                    .col(
                        ColumnDef::new(HealthScores::CalculatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_health_scores_repository_date")
                    .table(HealthScores::Table)
                    .col(HealthScores::RepositoryId)
                    .col(HealthScores::CalculatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HealthScores::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PrMetrics::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AutoMergeRules::Table).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(NotificationPreferences::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(TeamRepositories::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TeamMembers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Teams::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Teams {
    Table,
    Id,
    OrganizationId,
    Name,
    Slug,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TeamMembers {
    Table,
    Id,
    TeamId,
    UserId,
    Role,
    JoinedAt,
}

#[derive(DeriveIden)]
enum TeamRepositories {
    Table,
    Id,
    TeamId,
    RepositoryId,
    AddedAt,
}

#[derive(DeriveIden)]
enum NotificationPreferences {
    Table,
    Id,
    UserId,
    EmailEnabled,
    SlackEnabled,
    SlackWebhookUrl,
    PushEnabled,
    NotifyOnPrReady,
    NotifyOnPrFailed,
    NotifyOnReviewRequested,
    DigestFrequency,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AutoMergeRules {
    Table,
    Id,
    RepositoryId,
    Enabled,
    BotAuthors,
    RequireAllChecks,
    RequireApproval,
    MergeStrategy,
    DeleteBranch,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum PrMetrics {
    Table,
    Id,
    PullRequestId,
    RepositoryId,
    TimeToFirstReview,
    TimeToApproval,
    TimeToMerge,
    ReviewRounds,
    CommentsCount,
    IsBot,
    MergedAt,
    RecordedAt,
}

#[derive(DeriveIden)]
enum HealthScores {
    Table,
    Id,
    RepositoryId,
    Score,
    AvgTimeToMerge,
    AvgReviewTime,
    StalePrCount,
    FailedCheckRate,
    PrThroughput,
    CalculatedAt,
}
