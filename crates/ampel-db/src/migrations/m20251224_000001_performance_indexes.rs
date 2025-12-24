use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Index on repositories.user_id for faster user-specific repository lookups
        // This is critical for the main dashboard view where we fetch all repositories for a user
        // Expected performance improvement: 10-100x for users with many repositories
        manager
            .create_index(
                Index::create()
                    .name("idx_repositories_user_id")
                    .table(Repositories::Table)
                    .col(Repositories::UserId)
                    .to_owned(),
            )
            .await?;

        // Composite index on pull_requests(repository_id, state) for filtered PR queries
        // This enables efficient queries like "find all open PRs for repository X"
        // which is the most common query pattern in the application
        // Expected performance improvement: 5-50x for repositories with many PRs
        // The state column is included to support WHERE clauses like state = 'open'
        manager
            .create_index(
                Index::create()
                    .name("idx_pull_requests_repository_state")
                    .table(PullRequests::Table)
                    .col(PullRequests::RepositoryId)
                    .col(PullRequests::State)
                    .to_owned(),
            )
            .await?;

        // Note: The following indexes already exist from the initial migration:
        // - idx_repositories_user_provider_id (composite unique index)
        // - idx_pull_requests_repository_id (single column index)
        // - idx_pull_requests_repo_number (composite unique index)
        // - idx_ci_checks_pull_request_id (for CI checks per PR)
        // - idx_reviews_pull_request_id (for reviews per PR)

        // The new composite index on (repository_id, state) complements the existing
        // single-column index on repository_id and provides better performance for
        // state-filtered queries without requiring a full table scan.

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes in reverse order
        manager
            .drop_index(
                Index::drop()
                    .name("idx_pull_requests_repository_state")
                    .table(PullRequests::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_repositories_user_id")
                    .table(Repositories::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum PullRequests {
    Table,
    RepositoryId,
    State,
}
