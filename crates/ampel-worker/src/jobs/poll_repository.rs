use chrono::{DateTime, Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::models::GitProvider;
use ampel_db::encryption::EncryptionService;
use ampel_db::entities::{provider_connection, repository};
use ampel_db::queries::{CICheckQueries, PrQueries, RepoQueries, ReviewQueries};
use ampel_providers::ProviderFactory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollRepositoryJob;

impl From<DateTime<Utc>> for PollRepositoryJob {
    fn from(_: DateTime<Utc>) -> Self {
        Self
    }
}

impl PollRepositoryJob {
    pub async fn execute(
        &self,
        db: &DatabaseConnection,
        encryption_service: &EncryptionService,
        provider_factory: &ProviderFactory,
    ) -> anyhow::Result<()> {
        // Find repositories due for polling
        let repos = self.find_repos_to_poll(db).await?;

        tracing::info!("Found {} repositories to poll", repos.len());

        for repo in repos {
            if let Err(e) = self
                .poll_single_repo(db, encryption_service, provider_factory, &repo)
                .await
            {
                tracing::error!(
                    "Failed to poll repository {}/{}: {}",
                    repo.owner,
                    repo.name,
                    e
                );
            }
        }

        Ok(())
    }

    async fn find_repos_to_poll(
        &self,
        db: &DatabaseConnection,
    ) -> anyhow::Result<Vec<repository::Model>> {
        let now = Utc::now();

        // Find repos where:
        // 1. Never polled, OR
        // 2. Last polled + poll_interval < now
        let repos = repository::Entity::find()
            .order_by_asc(repository::Column::LastPolledAt)
            .limit(50)
            .all(db)
            .await?;

        // Filter to repos actually due for polling
        let due_repos: Vec<_> = repos
            .into_iter()
            .filter(|r| {
                match r.last_polled_at {
                    None => true, // Never polled
                    Some(last) => {
                        let next_poll = last + Duration::seconds(r.poll_interval_seconds as i64);
                        now > next_poll
                    }
                }
            })
            .collect();

        Ok(due_repos)
    }

    async fn poll_single_repo(
        &self,
        db: &DatabaseConnection,
        encryption_service: &EncryptionService,
        provider_factory: &ProviderFactory,
        repo: &repository::Model,
    ) -> anyhow::Result<()> {
        tracing::debug!("Polling repository {}/{}", repo.owner, repo.name);

        let provider_type: GitProvider = repo
            .provider
            .parse()
            .map_err(|e: String| anyhow::anyhow!(e))?;

        // Get provider connection - prefer the repo's bound connection, fall back to any connection
        let connection = if let Some(connection_id) = repo.connection_id {
            provider_connection::Entity::find_by_id(connection_id)
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Bound connection not found"))?
        } else {
            // Fall back to first connection for this user + provider
            provider_connection::Entity::find()
                .filter(provider_connection::Column::UserId.eq(repo.user_id))
                .filter(provider_connection::Column::Provider.eq(&repo.provider))
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("No connection found for provider"))?
        };

        // Decrypt access token
        let access_token = encryption_service.decrypt(&connection.access_token_encrypted)?;

        let provider = provider_factory.create(provider_type);

        // Fetch open PRs from provider
        let prs = provider
            .list_pull_requests(&access_token, &repo.owner, &repo.name, Some("open"))
            .await?;

        tracing::debug!(
            "Found {} open PRs for {}/{}",
            prs.len(),
            repo.owner,
            repo.name
        );

        for pr in prs {
            // Upsert PR
            let pr_model = PrQueries::upsert(
                db,
                repo.id,
                provider_type.to_string(),
                pr.provider_id.clone(),
                pr.number,
                pr.title,
                pr.description,
                pr.url,
                pr.state,
                pr.source_branch,
                pr.target_branch,
                pr.author,
                pr.author_avatar_url,
                pr.is_draft,
                pr.is_mergeable,
                pr.has_conflicts,
                pr.additions,
                pr.deletions,
                pr.changed_files,
                pr.commits_count,
                pr.comments_count,
                pr.created_at,
                pr.updated_at,
                pr.merged_at,
                pr.closed_at,
            )
            .await?;

            // Fetch and update CI checks
            match provider
                .get_ci_checks(&access_token, &repo.owner, &repo.name, pr.number)
                .await
            {
                Ok(checks) => {
                    // Delete old checks and insert new ones
                    CICheckQueries::delete_by_pull_request(db, pr_model.id).await?;

                    for check in checks {
                        CICheckQueries::upsert(
                            db,
                            pr_model.id,
                            check.name,
                            check.status,
                            check.conclusion,
                            check.url,
                            check.started_at,
                            check.completed_at,
                            check.completed_at.and_then(|c| {
                                check.started_at.map(|s| (c - s).num_seconds() as i32)
                            }),
                        )
                        .await?;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch CI checks for PR #{}: {}", pr.number, e);
                }
            }

            // Fetch and update reviews
            match provider
                .get_reviews(&access_token, &repo.owner, &repo.name, pr.number)
                .await
            {
                Ok(reviews) => {
                    // Delete old reviews and insert new ones
                    ReviewQueries::delete_by_pull_request(db, pr_model.id).await?;

                    for review in reviews {
                        ReviewQueries::upsert(
                            db,
                            Uuid::new_v4(),
                            pr_model.id,
                            review.reviewer,
                            review.reviewer_avatar_url,
                            review.state,
                            review.body,
                            review.submitted_at,
                        )
                        .await?;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch reviews for PR #{}: {}", pr.number, e);
                }
            }
        }

        // Update last polled timestamp
        RepoQueries::update_last_polled(db, repo.id).await?;

        tracing::debug!("Finished polling {}/{}", repo.owner, repo.name);

        Ok(())
    }
}
