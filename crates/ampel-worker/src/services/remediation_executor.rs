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
    pub async fn execute(
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
}
