use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::DisplayName).string())
                    .col(ColumnDef::new(Users::AvatarUrl).string())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Organizations table
        manager
            .create_table(
                Table::create()
                    .table(Organizations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Organizations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Organizations::OwnerId).uuid().not_null())
                    .col(ColumnDef::new(Organizations::Name).string().not_null())
                    .col(
                        ColumnDef::new(Organizations::Slug)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Organizations::Description).string())
                    .col(ColumnDef::new(Organizations::LogoUrl).string())
                    .col(
                        ColumnDef::new(Organizations::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Organizations::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Organizations::Table, Organizations::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Repositories table
        manager
            .create_table(
                Table::create()
                    .table(Repositories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Repositories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Repositories::UserId).uuid().not_null())
                    .col(ColumnDef::new(Repositories::Provider).string().not_null())
                    .col(ColumnDef::new(Repositories::ProviderId).string().not_null())
                    .col(ColumnDef::new(Repositories::Owner).string().not_null())
                    .col(ColumnDef::new(Repositories::Name).string().not_null())
                    .col(ColumnDef::new(Repositories::FullName).string().not_null())
                    .col(ColumnDef::new(Repositories::Description).string())
                    .col(ColumnDef::new(Repositories::Url).string().not_null())
                    .col(
                        ColumnDef::new(Repositories::DefaultBranch)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Repositories::IsPrivate)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Repositories::IsArchived)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Repositories::PollIntervalSeconds)
                            .integer()
                            .not_null()
                            .default(300),
                    )
                    .col(ColumnDef::new(Repositories::LastPolledAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Repositories::GroupId).uuid())
                    .col(
                        ColumnDef::new(Repositories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Repositories::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Repositories::Table, Repositories::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one repo per provider per user
        manager
            .create_index(
                Index::create()
                    .name("idx_repositories_user_provider_id")
                    .table(Repositories::Table)
                    .col(Repositories::UserId)
                    .col(Repositories::Provider)
                    .col(Repositories::ProviderId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Pull requests table
        manager
            .create_table(
                Table::create()
                    .table(PullRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PullRequests::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PullRequests::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(PullRequests::Provider).string().not_null())
                    .col(ColumnDef::new(PullRequests::ProviderId).string().not_null())
                    .col(ColumnDef::new(PullRequests::Number).integer().not_null())
                    .col(ColumnDef::new(PullRequests::Title).string().not_null())
                    .col(ColumnDef::new(PullRequests::Description).text())
                    .col(ColumnDef::new(PullRequests::Url).string().not_null())
                    .col(ColumnDef::new(PullRequests::State).string().not_null())
                    .col(
                        ColumnDef::new(PullRequests::SourceBranch)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PullRequests::TargetBranch)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PullRequests::Author).string().not_null())
                    .col(ColumnDef::new(PullRequests::AuthorAvatarUrl).string())
                    .col(
                        ColumnDef::new(PullRequests::IsDraft)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(PullRequests::IsMergeable).boolean())
                    .col(
                        ColumnDef::new(PullRequests::HasConflicts)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(PullRequests::Additions)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PullRequests::Deletions)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PullRequests::ChangedFiles)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PullRequests::CommitsCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PullRequests::CommentsCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PullRequests::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PullRequests::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PullRequests::MergedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(PullRequests::ClosedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(PullRequests::LastSyncedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PullRequests::Table, PullRequests::RepositoryId)
                            .to(Repositories::Table, Repositories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for finding PRs by repository
        manager
            .create_index(
                Index::create()
                    .name("idx_pull_requests_repository_id")
                    .table(PullRequests::Table)
                    .col(PullRequests::RepositoryId)
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one PR per number per repository
        manager
            .create_index(
                Index::create()
                    .name("idx_pull_requests_repo_number")
                    .table(PullRequests::Table)
                    .col(PullRequests::RepositoryId)
                    .col(PullRequests::Number)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // CI checks table
        manager
            .create_table(
                Table::create()
                    .table(CIChecks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CIChecks::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(CIChecks::PullRequestId).uuid().not_null())
                    .col(ColumnDef::new(CIChecks::Name).string().not_null())
                    .col(ColumnDef::new(CIChecks::Status).string().not_null())
                    .col(ColumnDef::new(CIChecks::Conclusion).string())
                    .col(ColumnDef::new(CIChecks::Url).string())
                    .col(ColumnDef::new(CIChecks::StartedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(CIChecks::CompletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(CIChecks::DurationSeconds).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .from(CIChecks::Table, CIChecks::PullRequestId)
                            .to(PullRequests::Table, PullRequests::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for finding checks by PR
        manager
            .create_index(
                Index::create()
                    .name("idx_ci_checks_pull_request_id")
                    .table(CIChecks::Table)
                    .col(CIChecks::PullRequestId)
                    .to_owned(),
            )
            .await?;

        // Reviews table
        manager
            .create_table(
                Table::create()
                    .table(Reviews::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Reviews::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Reviews::PullRequestId).uuid().not_null())
                    .col(ColumnDef::new(Reviews::Reviewer).string().not_null())
                    .col(ColumnDef::new(Reviews::ReviewerAvatarUrl).string())
                    .col(ColumnDef::new(Reviews::State).string().not_null())
                    .col(ColumnDef::new(Reviews::Body).text())
                    .col(
                        ColumnDef::new(Reviews::SubmittedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Reviews::Table, Reviews::PullRequestId)
                            .to(PullRequests::Table, PullRequests::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for finding reviews by PR
        manager
            .create_index(
                Index::create()
                    .name("idx_reviews_pull_request_id")
                    .table(Reviews::Table)
                    .col(Reviews::PullRequestId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Reviews::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CIChecks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PullRequests::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Repositories::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Organizations::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    PasswordHash,
    DisplayName,
    AvatarUrl,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
    OwnerId,
    Name,
    Slug,
    Description,
    LogoUrl,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    Id,
    UserId,
    Provider,
    ProviderId,
    Owner,
    Name,
    FullName,
    Description,
    Url,
    DefaultBranch,
    IsPrivate,
    IsArchived,
    PollIntervalSeconds,
    LastPolledAt,
    GroupId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum PullRequests {
    Table,
    Id,
    RepositoryId,
    Provider,
    ProviderId,
    Number,
    Title,
    Description,
    Url,
    State,
    SourceBranch,
    TargetBranch,
    Author,
    AuthorAvatarUrl,
    IsDraft,
    IsMergeable,
    HasConflicts,
    Additions,
    Deletions,
    ChangedFiles,
    CommitsCount,
    CommentsCount,
    CreatedAt,
    UpdatedAt,
    MergedAt,
    ClosedAt,
    LastSyncedAt,
}

#[derive(DeriveIden)]
enum CIChecks {
    Table,
    Id,
    PullRequestId,
    Name,
    Status,
    Conclusion,
    Url,
    StartedAt,
    CompletedAt,
    DurationSeconds,
}

#[derive(DeriveIden)]
enum Reviews {
    Table,
    Id,
    PullRequestId,
    Reviewer,
    ReviewerAvatarUrl,
    State,
    Body,
    SubmittedAt,
}
