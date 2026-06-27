//! Drives one remediation run through the Phase-2 state machine.
//!
//! Wires the injected collaborators (repository CAS, sandbox runner, verifier,
//! provider adapter) into an [`RemediationOrchestrator`] and walks a single
//! `run_id` from `created` to a terminal state, returning the [`RunOutcome`].
//! All state mutation flows through the orchestrator's CAS transitions; the
//! executor only owns the *sequencing* and the one transition the orchestrator
//! leaves to the caller (a non-safe `verify` ⇒ `handoff_human`).

use std::sync::Arc;

use ampel_core::errors::AmpelResult;
use ampel_core::remediation::{PrRef, RunState};
use ampel_core::services::{
    ConsolidateResult, MergeOutcome, RemediationOrchestrator, RemediationProvider,
    RemediationRunRepository, RepoContext, RunUpdate, SandboxRunner, VerificationService,
};
use uuid::Uuid;

/// Terminal outcome of an executor run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    /// Autonomy did not permit writes — parked in `no_op`.
    NoOp,
    /// Consolidated, verified, merged and finalized — reached `completed`.
    Completed,
    /// Withheld at a safety gate (non-green verify or TOCTOU) — `handoff_human`.
    HandoffHuman,
}

/// Sequences a single remediation run.
pub struct RemediationExecutor {
    repo: Arc<dyn RemediationRunRepository>,
    orchestrator: RemediationOrchestrator,
}

impl RemediationExecutor {
    pub fn new(
        repo: Arc<dyn RemediationRunRepository>,
        sandbox: Arc<dyn SandboxRunner>,
        verification: VerificationService,
        provider: Arc<dyn RemediationProvider>,
    ) -> Self {
        let orchestrator =
            RemediationOrchestrator::new(repo.clone(), sandbox, verification, provider);
        Self { repo, orchestrator }
    }

    /// Execute `run_id` end-to-end: consolidate → verify → (re-verify) merge →
    /// finalize. Short-circuits to [`RunOutcome::NoOp`] under read-only autonomy
    /// and to [`RunOutcome::HandoffHuman`] at any safety gate.
    ///
    /// Crash safety (chaos DoD): ANY error from the orchestration drives a
    /// best-effort CAS of the run from its current active state into
    /// [`RunState::Failed`] (with a secret-scrubbed `error_message`) BEFORE the
    /// error propagates — the run is never left in a non-terminal active state.
    /// (A TTL reaper for orphaned runs that crash the process entirely is out of
    /// scope here and tracked separately.)
    pub async fn execute(
        &self,
        run_id: Uuid,
        prs: Vec<PrRef>,
        repo_ctx: RepoContext,
    ) -> AmpelResult<RunOutcome> {
        // Capture the secret up front so we can scrub it from any error message
        // before persisting — `repo_ctx` is moved into the run below.
        let secret = repo_ctx.credential.expose().to_string();
        match self.execute_inner(run_id, prs, repo_ctx).await {
            Ok(outcome) => Ok(outcome),
            Err(e) => {
                self.mark_failed(run_id, &e, &secret).await;
                Err(e)
            }
        }
    }

    async fn execute_inner(
        &self,
        run_id: Uuid,
        prs: Vec<PrRef>,
        repo_ctx: RepoContext,
    ) -> AmpelResult<RunOutcome> {
        // 1. Consolidate (sandbox). Read-only autonomy parks the run in no_op.
        match self
            .orchestrator
            .consolidate(run_id, prs.clone(), repo_ctx)
            .await?
        {
            ConsolidateResult::NoOp => return Ok(RunOutcome::NoOp),
            ConsolidateResult::Consolidated(_) => {}
        }

        // 2. Verify (ADR-010). A safe result transitions verifying→merging; a
        //    non-safe result leaves the run in `verifying`, so we hand it off.
        let verification = self.orchestrator.verify(run_id).await?;
        if !verification.is_safe_to_merge() {
            self.repo
                .transition_state(
                    run_id,
                    RunState::Verifying,
                    RunState::HandoffHuman,
                    RunUpdate::none(),
                )
                .await?;
            return Ok(RunOutcome::HandoffHuman);
        }

        // 3. Re-verify immediately before merge (TOCTOU). A moved SHA or a fresh
        //    non-green result hands off without ever calling merge.
        match self.orchestrator.do_merge(run_id).await? {
            MergeOutcome::HandedOff { .. } => return Ok(RunOutcome::HandoffHuman),
            MergeOutcome::Merged { .. } => {}
        }

        // 4. Finalize: close each source PR with a "Superseded by #N" comment.
        let source_prs: Vec<i64> = prs.iter().map(|p| p.number as i64).collect();
        self.orchestrator.finalize(run_id, &source_prs).await?;
        Ok(RunOutcome::Completed)
    }

    /// Best-effort: move an active run into `Failed`, recording a scrubbed error.
    /// Infra/sandbox errors land here; the TOCTOU/unsafe-merge case is already
    /// handled inside the orchestrator (it hands off to a human, not Failed).
    /// Failures of this step itself are logged, never propagated — we must not
    /// mask the original error.
    async fn mark_failed(&self, run_id: Uuid, err: &ampel_core::errors::AmpelError, secret: &str) {
        let scrubbed = scrub_secret(&err.to_string(), secret);
        match self.repo.get_run(run_id).await {
            Ok(Some(run)) if run.state.is_active() => {
                if let Err(cas_err) = self
                    .repo
                    .transition_state(
                        run_id,
                        run.state,
                        RunState::Failed,
                        RunUpdate::with_error_message(scrubbed),
                    )
                    .await
                {
                    tracing::error!(
                        %run_id,
                        error = %cas_err,
                        "failed to mark remediation run as failed after error"
                    );
                }
            }
            Ok(_) => {
                // Already terminal (e.g. handed off) or missing — nothing to do.
            }
            Err(load_err) => {
                tracing::error!(
                    %run_id,
                    error = %load_err,
                    "could not load remediation run to mark it failed"
                );
            }
        }
    }
}

/// Redact a known secret from a message before it is persisted/logged.
fn scrub_secret(message: &str, secret: &str) -> String {
    if secret.is_empty() {
        message.to_string()
    } else {
        message.replace(secret, "***redacted***")
    }
}
