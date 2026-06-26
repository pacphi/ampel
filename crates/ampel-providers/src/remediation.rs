//! Provider write primitives for Fleet PR Remediation.
//!
//! This module defines [`RemediationCapable`], a **supertrait** of [`GitProvider`]
//! that adds the branch/PR/comment/label write operations required by the autonomous
//! remediation loops. Per **ADR-002**, write capability is opt-in: read-only providers
//! and air-gapped/org-ceiling deployments simply never acquire an
//! `Arc<dyn RemediationCapable>`, so the existing [`GitProvider`] contract is untouched.
//!
//! ## Async trait strategy (ADR-013)
//!
//! `RemediationCapable` is stored behind `Arc<dyn RemediationCapable + Send + Sync>`, so it
//! is annotated with `#[async_trait]` exactly like [`GitProvider`]. Do **not** convert this
//! to native `async fn in trait` — it must remain `dyn`-compatible.
//!
//! ## Capability introspection
//!
//! Providers declare which operations they support via [`RemediationCapable::capabilities`],
//! which returns a [`RemediationCaps`] descriptor. The job layer checks the relevant flag
//! before issuing a write and routes unsupported operations to sandbox clone-push fallbacks
//! (Phase 5) rather than panicking. This is a synchronous, zero-cost struct-field comparison.
//!
//! ## Signature adaptation
//!
//! ADR-002 sketches signatures using a `repo_id`. The live [`GitProvider`] contract instead
//! threads `credentials: &ProviderCredentials` plus `owner`/`repo` through every call (PATs
//! are per-call, never stored in the provider). `RemediationCapable` follows that established
//! convention. Likewise, the ADR's `CiStatus` return type does not exist in the codebase; the
//! closest faithful type is the existing [`ProviderCICheck`] list returned by
//! `GitProvider::get_ci_checks`, so [`RemediationCapable::get_status_for_ref`] returns
//! `Vec<ProviderCICheck>`.

use async_trait::async_trait;

use crate::error::ProviderResult;
use crate::traits::{GitProvider, ProviderCICheck, ProviderCredentials, ProviderPullRequest};

/// Capability flags returned by [`RemediationCapable::capabilities`].
///
/// All fields default to `false`; providers set only what they support. New flags are
/// additive and non-breaking as long as `Default` is derived (ADR-002). Keep fields flat
/// and boolean; deprecate rather than remove.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemediationCaps {
    /// Provider can create a new branch from a known SHA.
    pub create_branch: bool,
    /// Provider can fast-forward/merge a base branch into a working branch via its API.
    pub update_branch_from_base: bool,
    /// Provider can open a pull/merge request.
    pub create_pull_request: bool,
    /// Provider can edit the title/body of an existing pull/merge request.
    pub update_pull_request: bool,
    /// Provider can close (decline) a pull/merge request without merging.
    pub close_pull_request: bool,
    /// Provider can author a comment on a pull/merge request.
    pub create_comment: bool,
    /// Provider can attach labels to a pull/merge request.
    pub add_labels: bool,
    /// Provider can return CI/status for an arbitrary ref (SHA or branch), not just a PR.
    pub get_status_for_ref: bool,
    /// Provider can delete a branch.
    pub delete_branch: bool,
}

impl RemediationCaps {
    /// A descriptor with every capability enabled (used by GitHub/GitLab/Mock).
    pub fn all() -> Self {
        Self {
            create_branch: true,
            update_branch_from_base: true,
            create_pull_request: true,
            update_pull_request: true,
            close_pull_request: true,
            create_comment: true,
            add_labels: true,
            get_status_for_ref: true,
            delete_branch: true,
        }
    }
}

/// Write-capable extension of [`GitProvider`] for Fleet PR Remediation (ADR-002).
///
/// Implemented only by providers that support write operations. The job layer holds
/// instances as `Arc<dyn RemediationCapable + Send + Sync>` and consults
/// [`capabilities`](RemediationCapable::capabilities) before each write.
///
/// Object safety: every method here must remain `dyn`-compatible. Adding a generic or an
/// `impl Trait` return would break the `Arc<dyn RemediationCapable>` coercion — don't.
#[async_trait]
pub trait RemediationCapable: GitProvider {
    /// Static capability declaration. Reflects the provider's **API surface**, not the PAT
    /// scope — scope failures surface as [`ProviderError::PermissionDenied`] at call time.
    ///
    /// [`ProviderError::PermissionDenied`]: crate::error::ProviderError::PermissionDenied
    fn capabilities(&self) -> RemediationCaps;

    /// Resolve the commit SHA at the tip of the repository's default branch.
    async fn get_default_branch_sha(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
    ) -> ProviderResult<String>;

    /// Create `branch_name` pointing at `from_sha`.
    async fn create_branch(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        branch_name: &str,
        from_sha: &str,
    ) -> ProviderResult<()>;

    /// Bring `branch_name` up to date with `base_branch` (merge/rebase base into branch).
    async fn update_branch_from_base(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        branch_name: &str,
        base_branch: &str,
    ) -> ProviderResult<()>;

    /// Open a pull/merge request from `head` into `base`.
    #[allow(clippy::too_many_arguments)]
    async fn create_pull_request(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> ProviderResult<ProviderPullRequest>;

    /// Edit the title and/or body of an existing pull/merge request.
    async fn update_pull_request(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
        title: Option<&str>,
        body: Option<&str>,
    ) -> ProviderResult<()>;

    /// Close (decline) a pull/merge request without merging.
    async fn close_pull_request(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<()>;

    /// Author a comment on a pull/merge request. Returns the provider comment ID.
    async fn create_comment(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
        body: &str,
    ) -> ProviderResult<i64>;

    /// Attach labels to a pull/merge request.
    async fn add_labels(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
        labels: &[String],
    ) -> ProviderResult<()>;

    /// CI/status check for an arbitrary ref (SHA or branch name), not just a PR.
    async fn get_status_for_ref(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        git_ref: &str,
    ) -> ProviderResult<Vec<ProviderCICheck>>;

    /// Delete a branch.
    async fn delete_branch(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        branch_name: &str,
    ) -> ProviderResult<()>;
}
