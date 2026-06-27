//! Phase 1 remediation orchestration: PR selection + dry-run preview.
//!
//! Security note: this service is **read-only** in Phase 1. `preview` performs
//! ZERO repository writes by construction — no `RemediationCapable` provider is
//! wired in, so no write primitive (push/merge/comment) is reachable from this
//! code path. Write-capable autonomy tiers are gated off until later phases (see
//! [`AutonomyLevel::allows_writes`]).

use crate::errors::{AmpelError, AmpelResult};
use crate::remediation::db;
use crate::remediation::{
    ConsolidationPlan, PrRef, PrSelectionStrategy, RemediationCriteria, RunState,
};
use crate::services::{
    CiVerificationResult, ConsolidationOutcome, ConsolidationSpec, CredentialHandle,
    PolicyResolver, RawCiCheck, RemediationRun, RemediationRunRepository, RunUpdate, SandboxRunner,
    VerificationService,
};
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// Orchestrates Phase 1 remediation reads (selection + preview).
#[derive(Clone)]
pub struct RemediationService {
    db: DatabaseConnection,
    resolver: PolicyResolver,
}

impl RemediationService {
    pub fn new(db: DatabaseConnection) -> Self {
        let resolver = PolicyResolver::new(db.clone());
        Self { db, resolver }
    }

    /// Construct with an explicit resolver (e.g. a shared one).
    pub fn with_resolver(db: DatabaseConnection, resolver: PolicyResolver) -> Self {
        Self { db, resolver }
    }

    /// Select the open PRs for `repo_id` that satisfy `criteria`.
    ///
    /// Filters applied: open state, optional draft skip, optional target-branch
    /// allow-list, then the [`PrSelectionStrategy`], finally capped at
    /// `max_prs_per_run`. Read-only.
    pub async fn select_prs(
        &self,
        repo_id: Uuid,
        criteria: &RemediationCriteria,
    ) -> AmpelResult<Vec<PrRef>> {
        let mut query = db::pull_requests::Entity::find()
            .filter(db::pull_requests::Column::RepositoryId.eq(repo_id))
            .filter(db::pull_requests::Column::State.eq("open"));

        if criteria.skip_draft {
            query = query.filter(db::pull_requests::Column::IsDraft.eq(false));
        }
        if !criteria.allowed_targets.is_empty() {
            query = query.filter(
                db::pull_requests::Column::TargetBranch.is_in(criteria.allowed_targets.clone()),
            );
        }

        // Deterministic oldest-first ordering; strategies refine from here.
        let mut rows = query
            .order_by_asc(db::pull_requests::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(db_err)?;

        rows = apply_strategy(rows, &criteria.pr_selection);

        // Hard cap regardless of strategy.
        let cap = criteria.max_prs_per_run.max(0) as usize;
        rows.truncate(cap);

        Ok(rows.into_iter().map(to_pr_ref).collect())
    }

    /// Produce a dry-run [`ConsolidationPlan`] for `repo_id`. Read-only.
    ///
    /// Returns [`AmpelError::NotFound`] when no policy resolves for the repo.
    pub async fn preview(&self, repo_id: Uuid) -> AmpelResult<ConsolidationPlan> {
        let criteria =
            self.resolver.resolve(repo_id).await?.ok_or_else(|| {
                AmpelError::NotFound("no remediation policy for repository".into())
            })?;

        let selected = self.select_prs(repo_id, &criteria).await?;
        Ok(ConsolidationPlan::from_selection(
            selected,
            criteria.air_gapped,
        ))
    }
}

fn db_err(e: DbErr) -> AmpelError {
    AmpelError::DatabaseError(e.to_string())
}

fn to_pr_ref(m: db::pull_requests::Model) -> PrRef {
    PrRef {
        number: m.number,
        title: m.title,
        branch: m.source_branch,
    }
}

/// Apply the selection strategy to the already-filtered, oldest-first rows.
fn apply_strategy(
    rows: Vec<db::pull_requests::Model>,
    strategy: &PrSelectionStrategy,
) -> Vec<db::pull_requests::Model> {
    match strategy {
        PrSelectionStrategy::AllOpen => rows,
        PrSelectionStrategy::OldestFirst { max } => {
            let mut rows = rows;
            rows.truncate(*max as usize);
            rows
        }
        // Phase 1: PR labels are not persisted on `pull_requests`, so there is
        // no label data to match against — resolves to an empty selection.
        PrSelectionStrategy::ByLabel { .. } => Vec::new(),
        PrSelectionStrategy::ExplicitIds { ids } => {
            let wanted: HashSet<i64> = ids.iter().copied().collect();
            rows.into_iter()
                .filter(|r| wanted.contains(&(r.number as i64)))
                .collect()
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 2: write-path orchestration.
// ---------------------------------------------------------------------------

/// CI state for a consolidated PR's HEAD, as reported by the provider.
///
/// Provider-agnostic by construction: `ampel-core` does not depend on
/// `ampel-providers`, so the worker adapts a real `RemediationCapable` provider
/// into this shape behind [`RemediationProvider`].
pub struct ProviderRefStatus {
    pub ref_sha: String,
    pub checks: Vec<RawCiCheck>,
    pub required_check_names: Vec<String>,
    pub mergeable: bool,
}

/// Minimal provider write/read surface the orchestrator needs.
///
/// This is the seam that breaks the dependency cycle: the worker implements it
/// over `ampel-providers::RemediationCapable` (gating each call on
/// `capabilities()`), while unit tests implement it with an in-process mock.
/// No force-push primitive is exposed here, by design.
#[async_trait]
pub trait RemediationProvider: Send + Sync {
    /// Fetch CI checks + mergeability + HEAD SHA for the given PR.
    async fn get_status_for_ref(&self, pr_number: i64) -> AmpelResult<ProviderRefStatus>;

    /// Merge the consolidated PR. Returns the resulting merge commit SHA.
    async fn merge_pull_request(&self, pr_number: i64) -> AmpelResult<String>;

    /// Close a source PR, posting `comment` (e.g. "Superseded by #N").
    async fn close_pull_request(&self, pr_number: i64, comment: &str) -> AmpelResult<()>;
}

/// Repository coordinates needed to drive a sandbox consolidation.
pub struct RepoContext {
    pub clone_url: String,
    pub default_branch: String,
    pub credential: CredentialHandle,
}

/// Outcome of [`RemediationOrchestrator::consolidate`].
pub enum ConsolidateResult {
    /// Autonomy did not permit writes; the run was parked in `no_op`.
    NoOp,
    /// Consolidation ran; the run advanced to `verifying`.
    Consolidated(ConsolidationOutcome),
}

/// Why a merge was withheld and the run handed off to a human.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandoffReason {
    /// The HEAD SHA changed between verify and merge (TOCTOU).
    ShaChanged,
    /// Fresh verification was not safe at the merge gate.
    NotSafe,
}

/// Outcome of [`RemediationOrchestrator::do_merge`].
pub enum MergeOutcome {
    /// The consolidated PR was merged. Carries the merge commit SHA.
    Merged { merged_sha: String },
    /// No merge performed; the run moved to `handoff_human`.
    HandedOff {
        reason: HandoffReason,
        verification: CiVerificationResult,
    },
}

/// Drives a single remediation run through the Phase-2 state machine using
/// injected collaborators. All persistence flows through CAS transitions, so a
/// lost race surfaces as an error rather than a silently dropped write.
#[derive(Clone)]
pub struct RemediationOrchestrator {
    repo: Arc<dyn RemediationRunRepository>,
    sandbox: Arc<dyn SandboxRunner>,
    verification: VerificationService,
    provider: Arc<dyn RemediationProvider>,
}

impl RemediationOrchestrator {
    pub fn new(
        repo: Arc<dyn RemediationRunRepository>,
        sandbox: Arc<dyn SandboxRunner>,
        verification: VerificationService,
        provider: Arc<dyn RemediationProvider>,
    ) -> Self {
        Self {
            repo,
            sandbox,
            verification,
            provider,
        }
    }

    /// Consolidate the selected PRs for `run_id`.
    ///
    /// `DryRunOnly`/`SuggestOnly` autonomy short-circuits to `no_op` with zero
    /// sandbox or provider writes. Otherwise: `created`→`selecting`→
    /// `consolidating` (CAS), run the sandbox, persist plan + dispositions +
    /// consolidated PR, then `consolidating`→`verifying`.
    pub async fn consolidate(
        &self,
        run_id: Uuid,
        prs: Vec<PrRef>,
        repo: RepoContext,
    ) -> AmpelResult<ConsolidateResult> {
        let run = self.load(run_id).await?;

        // Autonomy gate: anything below auto-with-approval performs no writes.
        if !run.autonomy_level.allows_writes() {
            self.cas(run.id, run.state, RunState::NoOp, RunUpdate::none())
                .await?;
            return Ok(ConsolidateResult::NoOp);
        }

        // created -> selecting -> consolidating
        if run.state == RunState::Created {
            self.cas(
                run.id,
                RunState::Created,
                RunState::Selecting,
                RunUpdate::none(),
            )
            .await?;
        }
        self.cas(
            run.id,
            RunState::Selecting,
            RunState::Consolidating,
            RunUpdate::none(),
        )
        .await?;

        let spec = ConsolidationSpec::new(
            run.id,
            repo.clone_url,
            repo.default_branch,
            prs.clone(),
            repo.credential,
        );
        let outcome = self.sandbox.run_consolidation(spec).await?;

        // Persist what the sandbox produced.
        self.repo
            .set_consolidation_plan(run.id, ConsolidationPlan::from_selection(prs, false))
            .await?;
        for (pr_number, disposition) in &outcome.dispositions {
            self.repo
                .record_disposition(run.id, *pr_number, disposition.clone())
                .await?;
        }
        if let Some(pr) = outcome.consolidated_pr_number {
            self.repo.set_consolidated_pr(run.id, pr).await?;
        }

        // consolidating -> verifying, anchoring the consolidated HEAD SHA.
        self.cas(
            run.id,
            RunState::Consolidating,
            RunState::Verifying,
            RunUpdate::with_head_sha(outcome.head_sha.clone()),
        )
        .await?;

        Ok(ConsolidateResult::Consolidated(outcome))
    }

    /// Verify the consolidated PR's CI (ADR-010). Transitions
    /// `verifying`→`merging` only when the result is safe to merge, anchoring
    /// the verified SHA for the later TOCTOU re-check.
    pub async fn verify(&self, run_id: Uuid) -> AmpelResult<CiVerificationResult> {
        let run = self.load(run_id).await?;
        let pr = self.consolidated_pr(&run)?;

        let status = self.provider.get_status_for_ref(pr).await?;
        let result = self.verification.verify(
            &status.checks,
            &status.required_check_names,
            status.mergeable,
            status.ref_sha,
        );

        if result.is_safe_to_merge() {
            self.cas(
                run.id,
                RunState::Verifying,
                RunState::Merging,
                RunUpdate::with_head_sha(result.ref_sha.clone()),
            )
            .await?;
        }

        Ok(result)
    }

    /// Re-verify immediately before merge (TOCTOU). If the SHA moved or fresh
    /// verification is not safe, transition to `handoff_human` and DO NOT merge.
    /// Otherwise merge and transition `merging`→`finalizing`.
    pub async fn do_merge(&self, run_id: Uuid) -> AmpelResult<MergeOutcome> {
        let run = self.load(run_id).await?;

        // Guard the irreversible side-effect: refuse to merge unless the run is
        // genuinely in `Merging`. The Merging->Finalizing CAS below is NOT the
        // only guard — an out-of-order/wrong-state call must fail BEFORE the
        // provider merge is ever reached (no merge on stale state).
        if run.state != RunState::Merging {
            return Err(AmpelError::ValidationError(format!(
                "do_merge requires state `merging`, run {run_id} is in `{}`",
                run.state
            )));
        }

        let pr = self.consolidated_pr(&run)?;
        let snapshot_sha = run.head_sha.clone().ok_or_else(|| {
            AmpelError::ValidationError("no verified snapshot sha for merge".into())
        })?;

        let status = self.provider.get_status_for_ref(pr).await?;
        let fresh = self.verification.verify(
            &status.checks,
            &status.required_check_names,
            status.mergeable,
            status.ref_sha,
        );

        let sha_matches = VerificationService::reverify_sha_matches(&snapshot_sha, &fresh.ref_sha);
        if !sha_matches || !fresh.is_safe_to_merge() {
            let reason = if !sha_matches {
                HandoffReason::ShaChanged
            } else {
                HandoffReason::NotSafe
            };
            // M5: persist the handoff reason for observability (no secrets).
            self.cas(
                run.id,
                RunState::Merging,
                RunState::HandoffHuman,
                RunUpdate::with_error_message(format!("handoff: {reason:?}")),
            )
            .await?;
            return Ok(MergeOutcome::HandedOff {
                reason,
                verification: fresh,
            });
        }

        let merged_sha = self.provider.merge_pull_request(pr).await?;
        self.cas(
            run.id,
            RunState::Merging,
            RunState::Finalizing,
            RunUpdate::none(),
        )
        .await?;
        Ok(MergeOutcome::Merged { merged_sha })
    }

    /// Close each source PR with a "Superseded by #N" comment, then transition
    /// `finalizing`→`completed`. Per-PR `Consolidated` dispositions were already
    /// recorded during `consolidate`; this only performs the close + comment.
    pub async fn finalize(&self, run_id: Uuid, source_prs: &[i64]) -> AmpelResult<()> {
        let run = self.load(run_id).await?;

        // Guard the irreversible side-effect: refuse to close any source PR
        // unless the run is genuinely in `Finalizing`. A wrong-state call must
        // fail BEFORE any close happens.
        if run.state != RunState::Finalizing {
            return Err(AmpelError::ValidationError(format!(
                "finalize requires state `finalizing`, run {run_id} is in `{}`",
                run.state
            )));
        }

        let consolidated = self.consolidated_pr(&run)?;
        let comment = format!("Superseded by #{consolidated}");

        // M4: re-entrant + idempotent. Skip PRs already recorded as closed so a
        // partial-close failure is recoverable on re-run without double-closing.
        let already_closed = self.repo.closed_source_prs(run.id).await?;
        for &pr in source_prs {
            if already_closed.contains(&pr) {
                continue;
            }
            self.provider.close_pull_request(pr, &comment).await?;
            // Record the close BEFORE proceeding so a later failure leaves an
            // accurate idempotency ledger for the re-run.
            self.repo
                .record_disposition(
                    run.id,
                    pr,
                    crate::remediation::MergeDisposition::ClosedWithRef {
                        consolidated_pr_number: consolidated as u64,
                    },
                )
                .await?;
        }
        self.cas(
            run.id,
            RunState::Finalizing,
            RunState::Completed,
            RunUpdate::none(),
        )
        .await?;
        Ok(())
    }

    async fn load(&self, id: Uuid) -> AmpelResult<RemediationRun> {
        self.repo
            .get_run(id)
            .await?
            .ok_or_else(|| AmpelError::NotFound(format!("remediation run {id}")))
    }

    fn consolidated_pr(&self, run: &RemediationRun) -> AmpelResult<i64> {
        run.consolidated_pr_number
            .ok_or_else(|| AmpelError::ValidationError("run has no consolidated PR".into()))
    }

    /// CAS transition that treats a lost race as a hard error so the caller
    /// never proceeds on stale state.
    async fn cas(
        &self,
        id: Uuid,
        from: RunState,
        to: RunState,
        updates: RunUpdate,
    ) -> AmpelResult<()> {
        let landed = self.repo.transition_state(id, from, to, updates).await?;
        if !landed {
            return Err(AmpelError::InternalError(format!(
                "concurrent modification: run {id} no longer in state {from}"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remediation::testkit;
    use crate::remediation::{AutonomyLevel, MergeDisposition, RemediationTier};

    fn criteria(strategy: PrSelectionStrategy, skip_draft: bool, max: i32) -> RemediationCriteria {
        RemediationCriteria {
            min_open_prs: 1,
            pr_selection: strategy,
            max_prs_per_run: max,
            allowed_targets: vec![],
            skip_draft,
            require_green_before_merge: true,
            air_gapped: false,
            autonomy_level: AutonomyLevel::DryRunOnly,
            remediation_tier: RemediationTier::ConsolidateOnly,
        }
    }

    async fn seeded_repo(db: &DatabaseConnection) -> Uuid {
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(db, user).await;
        // oldest -> newest by offset (larger offset = older)
        testkit::seed_pr(db, repo, 1, "main", "open", false, 300).await; // oldest
        testkit::seed_pr(db, repo, 2, "main", "open", false, 200).await;
        testkit::seed_pr(db, repo, 3, "develop", "open", true, 100).await; // draft
        testkit::seed_pr(db, repo, 4, "main", "closed", false, 50).await; // closed
        repo
    }

    #[tokio::test]
    async fn should_select_all_open_non_closed_prs() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(repo, &criteria(PrSelectionStrategy::AllOpen, false, 100))
            .await
            .unwrap();

        // Assert: 3 open PRs (the closed one is excluded), drafts included.
        assert_eq!(prs.len(), 3);
    }

    #[tokio::test]
    async fn should_skip_draft_prs_when_skip_draft_set() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(repo, &criteria(PrSelectionStrategy::AllOpen, true, 100))
            .await
            .unwrap();

        // Assert: draft PR #3 excluded -> 2 remain.
        assert_eq!(prs.len(), 2);
        assert!(prs.iter().all(|p| p.number != 3));
    }

    #[tokio::test]
    async fn should_select_oldest_first_with_max() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(
                repo,
                &criteria(PrSelectionStrategy::OldestFirst { max: 1 }, false, 100),
            )
            .await
            .unwrap();

        // Assert: only the single oldest (PR #1) is selected.
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 1);
    }

    #[tokio::test]
    async fn should_cap_at_max_prs_per_run() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act: AllOpen would yield 3, but max cap is 2.
        let prs = svc
            .select_prs(repo, &criteria(PrSelectionStrategy::AllOpen, false, 2))
            .await
            .unwrap();

        // Assert
        assert_eq!(prs.len(), 2);
    }

    #[tokio::test]
    async fn should_select_only_explicit_ids() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(
                repo,
                &criteria(
                    PrSelectionStrategy::ExplicitIds { ids: vec![2] },
                    false,
                    100,
                ),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 2);
    }

    #[tokio::test]
    async fn should_select_nothing_for_by_label_in_phase_1() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(
                repo,
                &criteria(
                    PrSelectionStrategy::ByLabel {
                        labels: vec!["deps".into()],
                    },
                    false,
                    100,
                ),
            )
            .await
            .unwrap();

        // Assert: labels not persisted yet -> empty.
        assert!(prs.is_empty());
    }

    #[tokio::test]
    async fn should_filter_by_allowed_targets() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let mut c = criteria(PrSelectionStrategy::AllOpen, false, 100);
        c.allowed_targets = vec!["develop".into()];
        let svc = RemediationService::new(db);

        // Act
        let prs = svc.select_prs(repo, &c).await.unwrap();

        // Assert: only the develop-targeted PR (#3) matches.
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 3);
    }

    #[tokio::test]
    async fn should_preview_with_correct_pr_count() {
        // Arrange
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        testkit::seed_pr(&db, repo, 1, "main", "open", false, 200).await;
        testkit::seed_pr(&db, repo, 2, "main", "open", false, 100).await;
        testkit::seed_policy(
            &db,
            "repository",
            repo,
            "dry_run_only",
            "consolidate_only",
            "\"all_open\"",
            "[\"main\"]",
            false,
            10,
            false,
        )
        .await;
        let svc = RemediationService::new(db);

        // Act
        let plan = svc.preview(repo).await.unwrap();

        // Assert
        assert_eq!(plan.pr_count, 2);
        assert_eq!(plan.would_select.len(), 2);
        assert!(!plan.blocked_by_air_gap);
    }

    #[tokio::test]
    async fn should_flag_blocked_by_air_gap_in_preview_when_air_gapped() {
        // Arrange: org ceiling makes the resolved criteria air-gapped.
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let _org = testkit::seed_org(&db, user, true).await;
        testkit::seed_pr(&db, repo, 1, "main", "open", false, 100).await;
        testkit::seed_policy(
            &db,
            "repository",
            repo,
            "dry_run_only",
            "consolidate_only",
            "\"all_open\"",
            "[\"main\"]",
            false,
            10,
            false,
        )
        .await;
        let svc = RemediationService::new(db);

        // Act
        let plan = svc.preview(repo).await.unwrap();

        // Assert
        assert!(plan.air_gapped);
        assert!(plan.blocked_by_air_gap);
    }

    #[tokio::test]
    async fn should_error_preview_when_no_policy() {
        // Arrange
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let svc = RemediationService::new(db);

        // Act
        let result = svc.preview(repo).await;

        // Assert
        assert!(matches!(result, Err(AmpelError::NotFound(_))));
    }

    // -----------------------------------------------------------------------
    // Phase 2: orchestration tests (InMemory repo + FakeSandboxRunner + mock
    // provider). No DB, no container, no network.
    // -----------------------------------------------------------------------

    use crate::services::{FakeSandboxRunner, InMemoryRemediationRunRepository, ProviderRefStatus};
    use std::collections::VecDeque;
    use std::sync::Mutex;

    /// In-process [`RemediationProvider`] mock.
    ///
    /// `shas` is consumed front-to-back across `get_status_for_ref` calls (the
    /// last value sticks once drained), letting a test simulate the HEAD SHA
    /// moving between verify and the merge-gate re-verify (TOCTOU).
    struct MockProvider {
        shas: Mutex<VecDeque<String>>,
        last_sha: Mutex<String>,
        checks: Mutex<Vec<RawCiCheck>>,
        required: Vec<String>,
        mergeable: bool,
        merge_calls: Mutex<u32>,
        closed: Mutex<Vec<(i64, String)>>,
        /// PRs whose *next* close attempt should fail once (then succeed).
        fail_close_once: Mutex<HashSet<i64>>,
    }

    impl MockProvider {
        fn green(shas: &[&str]) -> Self {
            Self {
                shas: Mutex::new(shas.iter().map(|s| s.to_string()).collect()),
                last_sha: Mutex::new(shas.last().unwrap_or(&"sha").to_string()),
                checks: Mutex::new(vec![RawCiCheck::new("build", "completed", Some("success"))]),
                required: vec!["build".to_string()],
                mergeable: true,
                merge_calls: Mutex::new(0),
                closed: Mutex::new(Vec::new()),
                fail_close_once: Mutex::new(HashSet::new()),
            }
        }

        /// Arrange for the next `close_pull_request(pr)` to fail exactly once.
        fn fail_next_close(&self, pr: i64) {
            self.fail_close_once.lock().unwrap().insert(pr);
        }

        fn checks_handle(&self) -> std::sync::MutexGuard<'_, Vec<RawCiCheck>> {
            self.checks.lock().unwrap()
        }

        fn next_sha(&self) -> String {
            let mut q = self.shas.lock().unwrap();
            if let Some(s) = q.pop_front() {
                *self.last_sha.lock().unwrap() = s.clone();
                s
            } else {
                self.last_sha.lock().unwrap().clone()
            }
        }

        fn merge_call_count(&self) -> u32 {
            *self.merge_calls.lock().unwrap()
        }

        fn closed_prs(&self) -> Vec<(i64, String)> {
            self.closed.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl RemediationProvider for MockProvider {
        async fn get_status_for_ref(&self, _pr_number: i64) -> AmpelResult<ProviderRefStatus> {
            Ok(ProviderRefStatus {
                ref_sha: self.next_sha(),
                checks: self.checks.lock().unwrap().clone(),
                required_check_names: self.required.clone(),
                mergeable: self.mergeable,
            })
        }

        async fn merge_pull_request(&self, _pr_number: i64) -> AmpelResult<String> {
            *self.merge_calls.lock().unwrap() += 1;
            Ok("merged-sha".to_string())
        }

        async fn close_pull_request(&self, pr_number: i64, comment: &str) -> AmpelResult<()> {
            // Simulate a transient close failure (chaos GAP-2) exactly once.
            if self.fail_close_once.lock().unwrap().remove(&pr_number) {
                return Err(AmpelError::ProviderError(format!(
                    "transient close failure for #{pr_number}"
                )));
            }
            self.closed
                .lock()
                .unwrap()
                .push((pr_number, comment.to_string()));
            Ok(())
        }
    }

    fn pr_ref(n: i32) -> PrRef {
        PrRef {
            number: n,
            title: format!("PR {n}"),
            branch: format!("feature/{n}"),
        }
    }

    fn repo_ctx() -> RepoContext {
        RepoContext {
            clone_url: "https://example.test/repo.git".into(),
            default_branch: "main".into(),
            credential: CredentialHandle::new("pat"),
        }
    }

    fn orchestrator(
        repo: Arc<InMemoryRemediationRunRepository>,
        sandbox: Arc<FakeSandboxRunner>,
        provider: Arc<MockProvider>,
    ) -> RemediationOrchestrator {
        RemediationOrchestrator::new(repo, sandbox, VerificationService::new(), provider)
    }

    #[tokio::test]
    async fn should_drive_happy_path_from_created_to_completed() {
        // Arrange
        let repo = Arc::new(InMemoryRemediationRunRepository::new());
        let sandbox = Arc::new(FakeSandboxRunner::with_outcome(Some(9001), "headsha"));
        let provider = Arc::new(MockProvider::green(&["headsha", "headsha"]));
        let orch = orchestrator(repo.clone(), sandbox.clone(), provider.clone());
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Act: consolidate -> verify -> do_merge -> finalize.
        let consolidated = orch
            .consolidate(run.id, vec![pr_ref(1), pr_ref(2)], repo_ctx())
            .await
            .unwrap();
        let verification = orch.verify(run.id).await.unwrap();
        let merge = orch.do_merge(run.id).await.unwrap();
        orch.finalize(run.id, &[1, 2]).await.unwrap();

        // Assert
        assert!(matches!(consolidated, ConsolidateResult::Consolidated(_)));
        assert!(verification.is_safe_to_merge());
        assert!(matches!(merge, MergeOutcome::Merged { .. }));
        assert_eq!(provider.merge_call_count(), 1);
        assert_eq!(provider.closed_prs().len(), 2);
        assert!(provider
            .closed_prs()
            .iter()
            .all(|(_, c)| c == "Superseded by #9001"));
        let final_run = repo.get_run(run.id).await.unwrap().unwrap();
        assert_eq!(final_run.state, RunState::Completed);
        // 2 `Consolidated` (from consolidate) + 2 `ClosedWithRef` (from finalize).
        let dispositions = repo.dispositions_for(run.id);
        assert_eq!(dispositions.len(), 4);
        assert_eq!(
            dispositions
                .iter()
                .filter(|(_, d)| matches!(d, MergeDisposition::Consolidated))
                .count(),
            2
        );
        assert_eq!(
            dispositions
                .iter()
                .filter(|(_, d)| matches!(d, MergeDisposition::ClosedWithRef { .. }))
                .count(),
            2
        );
    }

    #[tokio::test]
    async fn should_no_op_under_dry_run_only_with_zero_writes() {
        // Arrange
        let repo = Arc::new(InMemoryRemediationRunRepository::new());
        let sandbox = Arc::new(FakeSandboxRunner::new());
        let provider = Arc::new(MockProvider::green(&["headsha"]));
        let orch = orchestrator(repo.clone(), sandbox.clone(), provider.clone());
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::DryRunOnly)
            .await
            .unwrap();

        // Act
        let result = orch
            .consolidate(run.id, vec![pr_ref(1)], repo_ctx())
            .await
            .unwrap();

        // Assert: parked in no_op, sandbox + provider untouched.
        assert!(matches!(result, ConsolidateResult::NoOp));
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::NoOp
        );
        assert!(!sandbox.was_invoked());
        assert_eq!(provider.merge_call_count(), 0);
        assert!(provider.closed_prs().is_empty());
        assert!(repo.dispositions_for(run.id).is_empty());
    }

    #[tokio::test]
    async fn should_handoff_without_merge_when_sha_changes_at_gate() {
        // Arrange: verify sees "sha-old"; the merge-gate re-verify sees "sha-new".
        let repo = Arc::new(InMemoryRemediationRunRepository::new());
        let sandbox = Arc::new(FakeSandboxRunner::with_outcome(Some(9001), "sha-old"));
        let provider = Arc::new(MockProvider::green(&["sha-old", "sha-new"]));
        let orch = orchestrator(repo.clone(), sandbox.clone(), provider.clone());
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Act
        orch.consolidate(run.id, vec![pr_ref(1)], repo_ctx())
            .await
            .unwrap();
        orch.verify(run.id).await.unwrap();
        let merge = orch.do_merge(run.id).await.unwrap();

        // Assert: TOCTOU caught -> handoff, NO merge call.
        assert!(matches!(
            merge,
            MergeOutcome::HandedOff {
                reason: HandoffReason::ShaChanged,
                ..
            }
        ));
        assert_eq!(provider.merge_call_count(), 0);
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::HandoffHuman
        );
    }

    #[tokio::test]
    async fn should_handoff_when_fresh_verification_not_safe() {
        // Arrange: SHA stable but the merge-gate re-verify is red (build failed).
        let repo = Arc::new(InMemoryRemediationRunRepository::new());
        let sandbox = Arc::new(FakeSandboxRunner::with_outcome(Some(9001), "headsha"));
        let provider = Arc::new(MockProvider::green(&["headsha", "headsha"]));
        let orch = orchestrator(repo.clone(), sandbox.clone(), provider.clone());
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        orch.consolidate(run.id, vec![pr_ref(1)], repo_ctx())
            .await
            .unwrap();
        orch.verify(run.id).await.unwrap();
        // Flip the required check red before the merge gate.
        *provider.checks_handle() = vec![RawCiCheck::new("build", "completed", Some("failure"))];

        // Act
        let merge = orch.do_merge(run.id).await.unwrap();

        // Assert
        assert!(matches!(
            merge,
            MergeOutcome::HandedOff {
                reason: HandoffReason::NotSafe,
                ..
            }
        ));
        assert_eq!(provider.merge_call_count(), 0);
    }

    #[tokio::test]
    async fn should_error_on_cas_conflict_mid_flight() {
        // Arrange: a concurrent writer advances the run out from under us.
        let repo = Arc::new(InMemoryRemediationRunRepository::new());
        let sandbox = Arc::new(FakeSandboxRunner::new());
        let provider = Arc::new(MockProvider::green(&["headsha"]));
        let orch = orchestrator(repo.clone(), sandbox.clone(), provider.clone());
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        // Race: someone cancels the run before consolidate transitions it.
        repo.transition_state(
            run.id,
            RunState::Created,
            RunState::Cancelled,
            RunUpdate::none(),
        )
        .await
        .unwrap();

        // Act
        let result = orch.consolidate(run.id, vec![pr_ref(1)], repo_ctx()).await;

        // Assert: the CAS loses and surfaces as an error; no sandbox write.
        assert!(matches!(result, Err(AmpelError::InternalError(_))));
        assert!(!sandbox.was_invoked());
    }

    #[tokio::test]
    async fn should_refuse_merge_when_run_not_in_merging_state() {
        // Arrange: consolidate leaves the run in `verifying`; we skip `verify`,
        // so the run is NOT in `merging` when do_merge is (wrongly) invoked.
        let repo = Arc::new(InMemoryRemediationRunRepository::new());
        let sandbox = Arc::new(FakeSandboxRunner::with_outcome(Some(9001), "headsha"));
        let provider = Arc::new(MockProvider::green(&["headsha", "headsha"]));
        let orch = orchestrator(repo.clone(), sandbox, provider.clone());
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        orch.consolidate(run.id, vec![pr_ref(1)], repo_ctx())
            .await
            .unwrap();

        // Act: call do_merge while still in `verifying`.
        let result = orch.do_merge(run.id).await;

        // Assert: refused BEFORE any merge side-effect.
        assert!(matches!(result, Err(AmpelError::ValidationError(_))));
        assert_eq!(provider.merge_call_count(), 0);
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::Verifying
        );
    }

    #[tokio::test]
    async fn should_refuse_finalize_when_run_not_in_finalizing_state() {
        // Arrange: drive a handoff so the run lands in `handoff_human`, then try
        // to finalize it (illegal — finalize must only run from `finalizing`).
        let repo = Arc::new(InMemoryRemediationRunRepository::new());
        let sandbox = Arc::new(FakeSandboxRunner::with_outcome(Some(9001), "sha-old"));
        let provider = Arc::new(MockProvider::green(&["sha-old", "sha-new"]));
        let orch = orchestrator(repo.clone(), sandbox, provider.clone());
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        orch.consolidate(run.id, vec![pr_ref(1)], repo_ctx())
            .await
            .unwrap();
        orch.verify(run.id).await.unwrap();
        orch.do_merge(run.id).await.unwrap(); // -> handoff_human (SHA changed)

        // Act
        let result = orch.finalize(run.id, &[1]).await;

        // Assert: refused, zero source-PR closes.
        assert!(matches!(result, Err(AmpelError::ValidationError(_))));
        assert!(provider.closed_prs().is_empty());
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::HandoffHuman
        );
    }

    #[tokio::test]
    async fn should_persist_handoff_reason_on_handoff() {
        // Arrange: SHA moves between verify and the merge gate (TOCTOU handoff).
        let repo = Arc::new(InMemoryRemediationRunRepository::new());
        let sandbox = Arc::new(FakeSandboxRunner::with_outcome(Some(9001), "sha-old"));
        let provider = Arc::new(MockProvider::green(&["sha-old", "sha-new"]));
        let orch = orchestrator(repo.clone(), sandbox, provider);
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        orch.consolidate(run.id, vec![pr_ref(1)], repo_ctx())
            .await
            .unwrap();
        orch.verify(run.id).await.unwrap();

        // Act
        orch.do_merge(run.id).await.unwrap();

        // Assert: the handoff reason was persisted (not none).
        let final_run = repo.get_run(run.id).await.unwrap().unwrap();
        assert_eq!(final_run.state, RunState::HandoffHuman);
        let msg = final_run.error_message.expect("handoff reason persisted");
        assert!(msg.contains("ShaChanged"));
    }

    #[tokio::test]
    async fn should_be_reentrant_and_idempotent_on_partial_finalize_failure() {
        // Arrange: walk to `finalizing`, then make the 2nd source close fail once.
        let repo = Arc::new(InMemoryRemediationRunRepository::new());
        let sandbox = Arc::new(FakeSandboxRunner::with_outcome(Some(9001), "headsha"));
        let provider = Arc::new(MockProvider::green(&["headsha", "headsha"]));
        let orch = orchestrator(repo.clone(), sandbox, provider.clone());
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        orch.consolidate(run.id, vec![pr_ref(1), pr_ref(2)], repo_ctx())
            .await
            .unwrap();
        orch.verify(run.id).await.unwrap();
        orch.do_merge(run.id).await.unwrap(); // -> finalizing
        provider.fail_next_close(2);

        // Act 1: first finalize closes #1, then fails on #2 — state stays finalizing.
        let first = orch.finalize(run.id, &[1, 2]).await;
        assert!(first.is_err());
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::Finalizing
        );

        // Act 2: re-run finalize — must skip #1 (already closed) and close #2.
        orch.finalize(run.id, &[1, 2]).await.unwrap();

        // Assert: each PR closed exactly once; #1 not double-closed; completed.
        let closed: Vec<i64> = provider.closed_prs().iter().map(|(n, _)| *n).collect();
        assert_eq!(closed.iter().filter(|&&n| n == 1).count(), 1);
        assert_eq!(closed.iter().filter(|&&n| n == 2).count(), 1);
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::Completed
        );
    }
}
