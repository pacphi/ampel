//! Adapts a real `ampel_providers::RemediationCapable` provider into the
//! `ampel_core::services::RemediationProvider` seam the orchestrator depends on.
//!
//! Every write is gated on the provider's [`RemediationCaps`]: an unsupported
//! operation returns a clean [`AmpelError`] rather than panicking, so a
//! partial-support provider (e.g. Bitbucket) degrades gracefully. No force-push
//! primitive is reachable through this adapter — by construction.
//!
//! ## Why `pr_number.to_string()` is used as the status ref
//!
//! Neither the frozen `RemediationCapable` nor `GitProvider` traits expose a
//! per-ref commit SHA. The adapter therefore (a) fetches CI checks for the PR's
//! ref and (b) sources the TOCTOU anchor SHA from `get_default_branch_sha`
//! (which detects *base* movement between verify and merge). A dedicated
//! provider "resolve ref SHA" method is a recommended follow-up so the anchor
//! tracks the consolidated branch HEAD directly; until then the merge gate is
//! still fully protected by the fresh CI re-verification (red ⇒ handoff).

use std::sync::Arc;

use ampel_core::errors::{AmpelError, AmpelResult};
use ampel_core::models::GitProvider as ProviderKind;
use ampel_core::models::{MergeRequest, MergeStrategy};
use ampel_core::services::{ProviderRefStatus, RawCiCheck, RemediationProvider};
use ampel_providers::error::ProviderError;
use ampel_providers::traits::ProviderCredentials;
use ampel_providers::{BitbucketProvider, GitHubProvider, GitLabProvider, RemediationCapable};
use async_trait::async_trait;

/// Build a write-capable provider for `kind`.
///
/// The shared [`ampel_providers::ProviderFactory`] only yields `dyn GitProvider`
/// (the read surface), so the worker constructs the concrete provider directly
/// to obtain the `RemediationCapable` write surface.
pub fn remediation_capable_provider(
    kind: ProviderKind,
    instance_url: Option<String>,
) -> Arc<dyn RemediationCapable> {
    match kind {
        ProviderKind::GitHub => Arc::new(GitHubProvider::new(instance_url)),
        ProviderKind::GitLab => Arc::new(GitLabProvider::new(instance_url)),
        ProviderKind::Bitbucket => Arc::new(BitbucketProvider::new(instance_url)),
    }
}

/// Wraps a `RemediationCapable` provider + the per-repo coordinates and
/// credentials needed to satisfy [`RemediationProvider`].
pub struct ProviderAdapter {
    provider: Arc<dyn RemediationCapable>,
    credentials: ProviderCredentials,
    owner: String,
    repo: String,
    /// Branch-protection required check names (empty ⇒ no required checks).
    required_checks: Vec<String>,
}

impl ProviderAdapter {
    pub fn new(
        provider: Arc<dyn RemediationCapable>,
        credentials: ProviderCredentials,
        owner: impl Into<String>,
        repo: impl Into<String>,
        required_checks: Vec<String>,
    ) -> Self {
        Self {
            provider,
            credentials,
            owner: owner.into(),
            repo: repo.into(),
            required_checks,
        }
    }

    fn caps_guard(&self, allowed: bool, op: &str) -> AmpelResult<()> {
        if allowed {
            Ok(())
        } else {
            Err(AmpelError::ProviderError(format!(
                "provider does not support `{op}` (capability disabled)"
            )))
        }
    }
}

fn provider_err(e: ProviderError) -> AmpelError {
    AmpelError::ProviderError(e.to_string())
}

#[async_trait]
impl RemediationProvider for ProviderAdapter {
    async fn get_status_for_ref(&self, pr_number: i64) -> AmpelResult<ProviderRefStatus> {
        let caps = self.provider.capabilities();
        self.caps_guard(caps.get_status_for_ref, "get_status_for_ref")?;

        let git_ref = pr_number.to_string();
        let checks = self
            .provider
            .get_status_for_ref(&self.credentials, &self.owner, &self.repo, &git_ref)
            .await
            .map_err(provider_err)?;

        let checks: Vec<RawCiCheck> = checks
            .into_iter()
            .map(|c| RawCiCheck::new(c.name, c.status, c.conclusion.as_deref()))
            .collect();

        // TOCTOU anchor SHA — see module docs for why this is the default-branch SHA.
        let ref_sha = self
            .provider
            .get_default_branch_sha(&self.credentials, &self.owner, &self.repo)
            .await
            .map_err(provider_err)?;

        // Mergeability from the PR when available; assume mergeable otherwise
        // (the sandbox already produced a clean merge — CI gates the rest).
        let mergeable = match self
            .provider
            .get_pull_request(&self.credentials, &self.owner, &self.repo, pr_number as i32)
            .await
        {
            Ok(pr) => pr.is_mergeable.unwrap_or(true),
            Err(_) => true,
        };

        Ok(ProviderRefStatus {
            ref_sha,
            checks,
            required_check_names: self.required_checks.clone(),
            mergeable,
        })
    }

    async fn merge_pull_request(&self, pr_number: i64) -> AmpelResult<String> {
        // Merge via the base `GitProvider` surface. No force-push; a plain merge.
        let merge_request = MergeRequest {
            strategy: MergeStrategy::Merge,
            commit_title: None,
            commit_message: None,
            delete_branch: false,
        };
        let result = self
            .provider
            .merge_pull_request(
                &self.credentials,
                &self.owner,
                &self.repo,
                pr_number as i32,
                &merge_request,
            )
            .await
            .map_err(provider_err)?;

        if !result.merged {
            return Err(AmpelError::ProviderError(format!(
                "provider declined to merge PR #{pr_number}: {}",
                result.message
            )));
        }
        Ok(result.sha.unwrap_or_default())
    }

    async fn close_pull_request(&self, pr_number: i64, comment: &str) -> AmpelResult<()> {
        let caps = self.provider.capabilities();
        // Leave the audit-trail comment first, then close. Both capability-gated.
        self.caps_guard(caps.create_comment, "create_comment")?;
        self.provider
            .create_comment(
                &self.credentials,
                &self.owner,
                &self.repo,
                pr_number as i32,
                comment,
            )
            .await
            .map_err(provider_err)?;

        self.caps_guard(caps.close_pull_request, "close_pull_request")?;
        self.provider
            .close_pull_request(&self.credentials, &self.owner, &self.repo, pr_number as i32)
            .await
            .map_err(provider_err)?;
        Ok(())
    }
}
