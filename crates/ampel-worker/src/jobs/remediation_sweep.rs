//! `RemediationSweepJob` — periodic discovery of repositories due for an
//! autonomous remediation run.
//!
//! Mirrors `poll_repository`'s sweep shape (oldest-first by `last_polled_at`,
//! `limit(50)`, due-filter), then narrows to repos carrying an *enabled*
//! remediation policy, caps the batch at `AMPEL_MAX_CONCURRENT_REPOS`, creates
//! one run per qualifying repo, and drives it.
//!
//! ## Enqueue decision
//!
//! The existing worker has no storage-backed Apalis queue (only `CronStream`
//! cron jobs), so there is no enqueue primitive to chain a per-run job onto.
//! Per the brief's fallback, the sweep therefore **drives each run inline**
//! (sequentially, capped) by calling [`RemediationRunJob::execute`] directly.
//! When a durable queue lands, swap the inline call for an enqueue of
//! `RemediationRunJob { run_id }`.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};

use ampel_core::services::{PolicyResolver, RemediationRunRepository};
use ampel_db::encryption::EncryptionService;
use ampel_db::entities::{remediation_policy, repository};
use ampel_db::repositories::SeaOrmRemediationRunRepository;

use ampel_core::services::SandboxRunner;

use super::remediation_run::RemediationRunJob;

/// Default cap on runs started per sweep tick.
const DEFAULT_MAX_CONCURRENT_REPOS: usize = 3;

/// Cron-driven sweep that starts remediation runs for due repositories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationSweepJob;

impl From<DateTime<Utc>> for RemediationSweepJob {
    fn from(_: DateTime<Utc>) -> Self {
        Self
    }
}

impl RemediationSweepJob {
    pub async fn execute(
        &self,
        db: &DatabaseConnection,
        encryption_service: &EncryptionService,
        sandbox: Arc<dyn SandboxRunner>,
    ) -> anyhow::Result<()> {
        let max_concurrent = std::env::var("AMPEL_MAX_CONCURRENT_REPOS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_CONCURRENT_REPOS);

        let candidates = self.find_repos_to_remediate(db).await?;
        tracing::info!(
            "Remediation sweep: {} candidate repo(s), cap {}",
            candidates.len(),
            max_concurrent
        );

        for repo in candidates.into_iter().take(max_concurrent) {
            if let Err(e) = self
                .start_run_for_repo(db, encryption_service, sandbox.clone(), &repo)
                .await
            {
                tracing::error!(
                    "Failed to start remediation run for {}: {}",
                    repo.full_name,
                    e
                );
            }
        }

        Ok(())
    }

    /// Repos due for polling (oldest-first, capped) that also carry an enabled
    /// repository-scoped remediation policy.
    pub async fn find_repos_to_remediate(
        &self,
        db: &DatabaseConnection,
    ) -> anyhow::Result<Vec<repository::Model>> {
        let now = Utc::now();
        let repos = repository::Entity::find()
            .order_by_asc(repository::Column::LastPolledAt)
            .limit(50)
            .all(db)
            .await?;

        let mut out = Vec::new();
        for repo in repos {
            // Due-filter (reuses the poll interval as the run cadence for now).
            let due = match repo.last_polled_at {
                None => true,
                Some(last) => now > last + Duration::seconds(repo.poll_interval_seconds as i64),
            };
            if !due {
                continue;
            }
            // Enabled repository-scoped policy?
            let has_enabled_policy = remediation_policy::Entity::find()
                .filter(remediation_policy::Column::ScopeType.eq("repository"))
                .filter(remediation_policy::Column::ScopeId.eq(repo.id))
                .filter(remediation_policy::Column::Enabled.eq(true))
                .one(db)
                .await?
                .is_some();
            if has_enabled_policy {
                out.push(repo);
            }
        }
        Ok(out)
    }

    async fn start_run_for_repo(
        &self,
        db: &DatabaseConnection,
        encryption_service: &EncryptionService,
        sandbox: Arc<dyn SandboxRunner>,
        repo: &repository::Model,
    ) -> anyhow::Result<()> {
        // Resolve the effective policy to learn the granted autonomy level.
        let criteria = PolicyResolver::new(db.clone())
            .resolve(repo.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no remediation policy resolved for {}", repo.id))?;

        // Create the run (read-only autonomy still produces a run that no-ops).
        let run_repo = SeaOrmRemediationRunRepository::new(db.clone());
        let run = run_repo
            .create_run(repo.id, criteria.autonomy_level)
            .await?;

        // Inline drive (see "Enqueue decision" in the module docs).
        let outcome = RemediationRunJob::new(run.id)
            .execute(db, encryption_service, sandbox)
            .await?;
        tracing::info!(run_id = %run.id, repo = %repo.full_name, ?outcome, "sweep started run");
        Ok(())
    }
}
