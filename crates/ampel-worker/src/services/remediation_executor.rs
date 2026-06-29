//! Drives one remediation run through the Phase-2/3 state machine.
//!
//! Wires the injected collaborators (repository CAS, sandbox runner, verifier,
//! provider adapter) into an [`RemediationOrchestrator`] and walks a single
//! `run_id` from `created` to a terminal state, returning the [`RunOutcome`].
//! All state mutation flows through the orchestrator's CAS transitions; the
//! executor only owns the *sequencing* and the one transition the orchestrator
//! leaves to the caller (a non-safe `verify` ⇒ `handoff_human`).
//!
//! ## Human-approval gate (Phase 3)
//! Under the `auto_with_approval` autonomy tier a safe `verify` parks the run in
//! `awaiting_approval` (the orchestrator stops short of merging). The executor
//! detects this and returns [`RunOutcome::AwaitingApproval`] cleanly — this is
//! NOT a failure, so the C1 error→`Failed` wrapper must never see it.
//!
//! A human approves out-of-band: the API `approve` endpoint CAS-advances the run
//! `awaiting_approval → merging`. The next time the worker drives the run, the
//! executor finds it already in `merging` and **resumes at `do_merge`** (with the
//! orchestrator's TOCTOU re-verify), then finalizes.
//!
//! ## Observability (Phase 3)
//! Metric emission lives here (the worker layer) so `ampel-core` stays
//! dependency-light. The executor records run terminal counts/durations, merge
//! counts, skipped-conflict counts, and handoff counts at the points where each
//! outcome is observed. The happy path also emits `RemediationRunMerged` and
//! `SourcePrsClosed` notifications via the injected [`RemediationNotifier`].

use std::sync::Arc;
use std::time::Instant;

use ampel_core::errors::AmpelResult;
use ampel_core::remediation::{MergeDisposition, PrRef, RemediationTier, RunState};
use ampel_core::services::{
    ConsolidateResult, HandoffReason, MergeOutcome, RemediationOrchestrator, RemediationProvider,
    RemediationRunRepository, RepoContext, RunUpdate, SandboxRunner, VerificationService,
};
use uuid::Uuid;

use crate::observability;
use crate::services::agentic_tier::{tier_allows_agentic, AgentTierOutcome, AgenticTier};
use crate::services::notifier::{
    NoopNotifier, RemediationNotifier, RunMergedNotification, SourcePrsClosedNotification,
};

/// Terminal (or parked) outcome of an executor run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    /// Autonomy did not permit writes — parked in `no_op`.
    NoOp,
    /// Consolidated, verified, merged and finalized — reached `completed`.
    Completed,
    /// Withheld at a safety gate (non-green verify or TOCTOU) — `handoff_human`.
    HandoffHuman,
    /// Parked at the human-approval gate (`awaiting_approval`). Not terminal and
    /// not a failure — the run resumes once a human approves.
    AwaitingApproval,
}

/// Sequences a single remediation run.
pub struct RemediationExecutor {
    repo: Arc<dyn RemediationRunRepository>,
    orchestrator: RemediationOrchestrator,
    /// Provider kind used as the `provider` label on merge metrics + payloads.
    provider_label: String,
    notifier: Arc<dyn RemediationNotifier>,
    /// Optional Tier-2 agentic seam (Phase 4). When present and the resolved
    /// `remediation_tier` permits it, a RED verify triggers an autonomous
    /// recovery attempt before handing off to a human.
    agentic_tier: Option<Arc<dyn AgenticTier>>,
    remediation_tier: RemediationTier,
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
        Self {
            repo,
            orchestrator,
            provider_label: "unknown".to_string(),
            notifier: Arc::new(NoopNotifier),
            agentic_tier: None,
            remediation_tier: RemediationTier::ConsolidateOnly,
        }
    }

    /// Inject the Tier-2 agentic seam together with the run's resolved
    /// `remediation_tier`. Only `fix_and_consolidate` / `full_remediation` will
    /// actually invoke the model (see [`tier_allows_agentic`]).
    ///
    /// Exercised by the executor integration tests; the bin's run job does not
    /// yet construct the sandbox-backed tier (see `agentic_tier` module note).
    #[allow(dead_code)]
    pub fn with_agentic_tier(
        mut self,
        tier: Arc<dyn AgenticTier>,
        remediation_tier: RemediationTier,
    ) -> Self {
        self.agentic_tier = Some(tier);
        self.remediation_tier = remediation_tier;
        self
    }

    /// Set the provider-kind label used on merge metrics and notification
    /// payloads (e.g. `github`).
    pub fn with_provider_label(mut self, label: impl Into<String>) -> Self {
        self.provider_label = label.into();
        self
    }

    /// Inject the notification delivery seam (defaults to a no-op).
    pub fn with_notifier(mut self, notifier: Arc<dyn RemediationNotifier>) -> Self {
        self.notifier = notifier;
        self
    }

    /// Execute `run_id`: consolidate → verify → (gate?) → (re-verify) merge →
    /// finalize. Short-circuits to [`RunOutcome::NoOp`] under read-only autonomy,
    /// to [`RunOutcome::AwaitingApproval`] at the human gate, and to
    /// [`RunOutcome::HandoffHuman`] at any safety gate. A run already in `merging`
    /// (human-approved) resumes at `do_merge`.
    ///
    /// Crash safety (chaos DoD): ANY error from the orchestration drives a
    /// best-effort CAS of the run from its current active state into
    /// [`RunState::Failed`] (with a secret-scrubbed `error_message`) BEFORE the
    /// error propagates — the run is never left in a non-terminal active state.
    /// The `awaiting_approval` parked state is an `Ok` outcome and is therefore
    /// never treated as a failure by this wrapper.
    pub async fn execute(
        &self,
        run_id: Uuid,
        prs: Vec<PrRef>,
        repo_ctx: RepoContext,
    ) -> AmpelResult<RunOutcome> {
        // Capture the secret up front so we can scrub it from any error message
        // before persisting — `repo_ctx` is moved into the run below.
        let secret = repo_ctx.credential.expose().to_string();
        let started = Instant::now();
        match self.execute_inner(run_id, prs, repo_ctx).await {
            Ok(outcome) => {
                self.record_terminal_metric(outcome, started.elapsed().as_secs_f64());
                Ok(outcome)
            }
            Err(e) => {
                self.mark_failed(run_id, &e, &secret).await;
                observability::record_run_terminal("failed", started.elapsed().as_secs_f64());
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
        // Resume a human-approved run already advanced to `merging` (the API
        // approve endpoint CAS-advanced awaiting_approval → merging). Pick up at
        // do_merge; consolidate/verify already ran on the prior pass.
        if self.run_state(run_id).await? == Some(RunState::Merging) {
            return self.merge_and_finalize(run_id, &prs).await;
        }

        // 1. Consolidate (sandbox). Read-only autonomy parks the run in no_op.
        let consolidated = match self
            .orchestrator
            .consolidate(run_id, prs.clone(), repo_ctx)
            .await?
        {
            ConsolidateResult::NoOp => return Ok(RunOutcome::NoOp),
            ConsolidateResult::Consolidated(outcome) => outcome,
        };
        // Count any skipped-conflict dispositions the sandbox surfaced.
        for (_, disposition) in &consolidated.dispositions {
            if let MergeDisposition::SkippedConflict { reason } = disposition {
                observability::record_conflict(observability::classify_conflict(reason));
            }
        }

        // 2. Verify (ADR-010). A non-safe result leaves the run in `verifying`,
        //    so we hand it off. A safe result advances the run — but the *next*
        //    state depends on the autonomy tier (the orchestrator decides):
        //    `auto_with_approval` parks in `awaiting_approval`; otherwise it
        //    moves to `merging`.
        let verification = self.orchestrator.verify(run_id).await?;
        if !verification.is_safe_to_merge() {
            // Tier-2 (Phase 4): a RED verify under an agentic tier may be
            // recovered by an autonomous fix attempt. The run is still in
            // `verifying` (a non-safe verify does not transition), so a
            // successful recovery can legally re-verify and advance to merge.
            if let Some(tier) = &self.agentic_tier {
                if tier_allows_agentic(self.remediation_tier) {
                    match tier.attempt(run_id).await? {
                        AgentTierOutcome::Recovered => {
                            // The agent pushed fixes — re-verify the consolidated
                            // PR. A now-safe verify advances `verifying → merging`
                            // (or parks at the human gate).
                            let reverify = self.orchestrator.verify(run_id).await?;
                            if reverify.is_safe_to_merge() {
                                if self.run_state(run_id).await? == Some(RunState::AwaitingApproval)
                                {
                                    return Ok(RunOutcome::AwaitingApproval);
                                }
                                return self.merge_and_finalize(run_id, &prs).await;
                            }
                            // Still not safe after recovery → hand off below.
                        }
                        AgentTierOutcome::Exhausted => {
                            // Budget/egress/error — hand off below.
                        }
                    }
                }
            }

            self.repo
                .transition_state(
                    run_id,
                    RunState::Verifying,
                    RunState::HandoffHuman,
                    RunUpdate::none(),
                )
                .await?;
            observability::record_handoff("verification_unsafe");
            return Ok(RunOutcome::HandoffHuman);
        }
        // Parked at the human gate? Stop cleanly (NOT a failure, NOT terminal).
        if self.run_state(run_id).await? == Some(RunState::AwaitingApproval) {
            return Ok(RunOutcome::AwaitingApproval);
        }

        // 3 + 4. Re-verify (TOCTOU) + merge, then finalize.
        self.merge_and_finalize(run_id, &prs).await
    }

    /// Drive `do_merge` (with the orchestrator's TOCTOU re-verify) and, on a
    /// successful merge, finalize the run. Emits merge metrics and the merge /
    /// sources-closed notifications. A TOCTOU/unsafe re-verify hands off.
    async fn merge_and_finalize(&self, run_id: Uuid, prs: &[PrRef]) -> AmpelResult<RunOutcome> {
        match self.orchestrator.do_merge(run_id).await? {
            MergeOutcome::HandedOff { reason, .. } => {
                observability::record_handoff(handoff_reason_label(reason));
                return Ok(RunOutcome::HandoffHuman);
            }
            MergeOutcome::Merged { .. } => {
                observability::record_merge(&self.provider_label);
            }
        }

        let consolidated_pr = self
            .repo
            .get_run(run_id)
            .await?
            .and_then(|r| r.consolidated_pr_number)
            .unwrap_or_default();

        self.notifier
            .run_merged(RunMergedNotification {
                run_id,
                consolidated_pr_number: consolidated_pr,
                provider: self.provider_label.clone(),
            })
            .await;

        // Finalize: close each source PR with a "Superseded by #N" comment.
        let source_prs: Vec<i64> = prs.iter().map(|p| p.number as i64).collect();
        self.orchestrator.finalize(run_id, &source_prs).await?;

        self.notifier
            .sources_closed(SourcePrsClosedNotification {
                run_id,
                consolidated_pr_number: consolidated_pr,
                closed_pr_numbers: source_prs,
            })
            .await;

        Ok(RunOutcome::Completed)
    }

    /// Record the terminal run counter + duration histogram for a finished run.
    /// `awaiting_approval` is parked, not terminal, so it is intentionally skipped.
    fn record_terminal_metric(&self, outcome: RunOutcome, duration_secs: f64) {
        let state = match outcome {
            RunOutcome::Completed => "completed",
            RunOutcome::NoOp => "no_op",
            RunOutcome::HandoffHuman => "handoff_human",
            RunOutcome::AwaitingApproval => return,
        };
        observability::record_run_terminal(state, duration_secs);
    }

    async fn run_state(&self, run_id: Uuid) -> AmpelResult<Option<RunState>> {
        Ok(self.repo.get_run(run_id).await?.map(|r| r.state))
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

/// Bounded label for a [`HandoffReason`] (Prometheus-safe).
fn handoff_reason_label(reason: HandoffReason) -> &'static str {
    match reason {
        HandoffReason::ShaChanged => "sha_changed",
        HandoffReason::NotSafe => "not_safe",
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
