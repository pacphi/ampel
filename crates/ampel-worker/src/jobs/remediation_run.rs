//! `RemediationRunJob` — drives a single, already-created remediation run.
//!
//! Given a `run_id`, it rebuilds the per-repo execution context (authenticated
//! provider, selected PRs, clone coordinates) and hands off to the
//! [`RemediationExecutor`]. Run *creation* happens in the sweep; this job only
//! executes an existing run, so it is safe to (re)dispatch idempotently — the
//! orchestrator's CAS transitions reject stale work.

use std::sync::Arc;

use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::models::GitProvider as ProviderKind;
use ampel_core::services::{
    CredentialHandle, PolicyResolver, RemediationProvider, RemediationRunRepository,
    RemediationService, RepoContext, SandboxRunner, VerificationService,
};
use ampel_db::encryption::EncryptionService;
use ampel_db::entities::{provider_account, repository};
use ampel_db::repositories::SeaOrmRemediationRunRepository;
use ampel_providers::traits::ProviderCredentials;

use crate::services::notifier::{LoggingNotifier, RemediationNotifier, SlackNotifier};
use crate::services::{
    remediation_capable_provider, ProviderAdapter, RemediationExecutor, RunOutcome,
};

/// Build the notification delivery channel from the environment. When
/// `REMEDIATION_SLACK_WEBHOOK_URL` is set, events are delivered to Slack via the
/// shared `NotificationService`; otherwise they are logged (no network).
fn notifier_from_env() -> Arc<dyn RemediationNotifier> {
    match std::env::var("REMEDIATION_SLACK_WEBHOOK_URL") {
        Ok(url) if !url.is_empty() => {
            let channel = std::env::var("REMEDIATION_SLACK_CHANNEL").ok();
            Arc::new(SlackNotifier::new(url, channel))
        }
        _ => Arc::new(LoggingNotifier),
    }
}

/// Drives one remediation run identified by `run_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationRunJob {
    pub run_id: Uuid,
}

impl RemediationRunJob {
    pub fn new(run_id: Uuid) -> Self {
        Self { run_id }
    }

    /// Execute the run end-to-end. Returns the terminal [`RunOutcome`].
    pub async fn execute(
        &self,
        db: &DatabaseConnection,
        encryption_service: &EncryptionService,
        sandbox: Arc<dyn SandboxRunner>,
    ) -> anyhow::Result<RunOutcome> {
        let run_repo: Arc<dyn RemediationRunRepository> =
            Arc::new(SeaOrmRemediationRunRepository::new(db.clone()));

        let run = run_repo
            .get_run(self.run_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("remediation run {} not found", self.run_id))?;

        // Repository + provider account.
        let repo = repository::Entity::find_by_id(run.repository_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("repository {} not found", run.repository_id))?;

        let provider_kind: ProviderKind = repo
            .provider
            .parse()
            .map_err(|e: String| anyhow::anyhow!(e))?;

        let account_id = repo
            .provider_account_id
            .ok_or_else(|| anyhow::anyhow!("repository {} has no provider account", repo.id))?;
        let account = provider_account::Entity::find_by_id(account_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("provider account {account_id} not found"))?;

        let access_token = encryption_service.decrypt(&account.access_token_encrypted)?;
        let credentials = ProviderCredentials::Pat {
            token: access_token.clone(),
            username: account.auth_username.clone(),
        };

        // Selected PRs (resolve the effective policy, then select).
        let resolver = PolicyResolver::new(db.clone());
        let criteria = resolver
            .resolve(repo.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no remediation policy for repository {}", repo.id))?;
        let prs = RemediationService::new(db.clone())
            .select_prs(repo.id, &criteria)
            .await?;

        // Authenticated, capability-gated provider adapter.
        let provider = remediation_capable_provider(provider_kind, account.instance_url.clone());
        // Required-check names are not yet sourced from branch protection (Phase 2
        // follow-up); an empty set means the verifier gates purely on observed CI.
        let adapter: Arc<dyn RemediationProvider> = Arc::new(ProviderAdapter::new(
            provider,
            credentials,
            repo.owner.clone(),
            repo.name.clone(),
            Vec::new(),
        ));

        let executor =
            RemediationExecutor::new(run_repo, sandbox, VerificationService::new(), adapter)
                .with_provider_label(repo.provider.clone())
                .with_notifier(notifier_from_env());

        let repo_ctx = RepoContext {
            clone_url: repo.url.clone(),
            default_branch: repo.default_branch.clone(),
            credential: CredentialHandle::new(access_token),
        };

        let outcome = executor.execute(self.run_id, prs, repo_ctx).await?;
        tracing::info!(
            run_id = %self.run_id,
            repo = %repo.full_name,
            ?outcome,
            "remediation run finished"
        );
        Ok(outcome)
    }
}
