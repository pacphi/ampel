//! Persistence abstraction for remediation runs.
//!
//! `ampel-core` cannot depend on `ampel-db` (dependency cycle), so the write
//! side of a remediation run is expressed as a trait here and implemented in the
//! outer layer (`ampel-db` / worker) via dependency injection. The orchestrator
//! ([`crate::services::RemediationOrchestrator`]) holds an
//! `Arc<dyn RemediationRunRepository>` and never sees a concrete DB type.
//!
//! State changes go exclusively through
//! [`RemediationRunRepository::transition_state`], which performs a
//! compare-and-swap on the run's current state (SQL: `WHERE state = $from`).
//! A `false` return means the CAS lost a race (concurrent modification) and the
//! caller MUST re-read and decide — it must never assume the write landed.

use crate::errors::AmpelResult;
use crate::remediation::{AutonomyLevel, ConsolidationPlan, MergeDisposition, RunState};
use async_trait::async_trait;
use uuid::Uuid;

/// Read model of a remediation run (the columns the orchestrator needs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemediationRun {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub state: RunState,
    pub autonomy_level: AutonomyLevel,
    pub consolidated_pr_number: Option<i64>,
    /// The CI SHA snapshot captured at `verify` time — the TOCTOU anchor.
    pub head_sha: Option<String>,
    /// Last persisted error / handoff reason (secret-scrubbed), if any.
    pub error_message: Option<String>,
}

/// Optional column updates applied atomically with a state transition.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunUpdate {
    pub consolidated_pr_number: Option<i64>,
    pub head_sha: Option<String>,
    pub error_message: Option<String>,
}

impl RunUpdate {
    /// No-op update (a bare transition).
    pub fn none() -> Self {
        Self::default()
    }

    /// Update carrying the verified head SHA into `merging`.
    pub fn with_head_sha(sha: impl Into<String>) -> Self {
        Self {
            head_sha: Some(sha.into()),
            ..Self::default()
        }
    }

    /// Update carrying an error/handoff reason for observability on a terminal
    /// or handoff transition. The caller is responsible for scrubbing secrets.
    pub fn with_error_message(message: impl Into<String>) -> Self {
        Self {
            error_message: Some(message.into()),
            ..Self::default()
        }
    }
}

/// Write-side persistence for remediation runs (CAS state machine).
#[async_trait]
pub trait RemediationRunRepository: Send + Sync {
    /// Create a fresh run in [`RunState::Created`].
    async fn create_run(
        &self,
        repository_id: Uuid,
        autonomy_level: AutonomyLevel,
    ) -> AmpelResult<RemediationRun>;

    /// Fetch a run by id.
    async fn get_run(&self, id: Uuid) -> AmpelResult<Option<RemediationRun>>;

    /// Compare-and-swap the run state from `from` to `to`, applying `updates`.
    ///
    /// Returns `Ok(true)` when the swap landed, `Ok(false)` when the current
    /// state was not `from` (a concurrent modification — the caller must
    /// re-read). The transition must be legal per [`RunState::can_transition_to`].
    async fn transition_state(
        &self,
        id: Uuid,
        from: RunState,
        to: RunState,
        updates: RunUpdate,
    ) -> AmpelResult<bool>;

    /// Record the final disposition for one source PR.
    async fn record_disposition(
        &self,
        run_id: Uuid,
        pr_number: i64,
        disposition: MergeDisposition,
    ) -> AmpelResult<()>;

    /// Persist the consolidation plan snapshot for the run.
    async fn set_consolidation_plan(
        &self,
        run_id: Uuid,
        plan: ConsolidationPlan,
    ) -> AmpelResult<()>;

    /// Record the consolidated PR number produced by the run.
    async fn set_consolidated_pr(&self, run_id: Uuid, pr_number: i64) -> AmpelResult<()>;

    /// Source PR numbers already recorded as *closed* (superseded) for this run.
    ///
    /// Used by `finalize` to stay idempotent across re-entry: a partial close
    /// failure can be re-run without double-closing PRs already handled.
    async fn closed_source_prs(&self, run_id: Uuid) -> AmpelResult<Vec<i64>>;
}

#[cfg(any(test, feature = "test-utils"))]
pub use in_memory::InMemoryRemediationRunRepository;

#[cfg(any(test, feature = "test-utils"))]
mod in_memory {
    //! In-process fake for unit tests — Mutex<HashMap>, honors CAS semantics.
    //! No DB, no network.

    use super::*;
    use crate::errors::AmpelError;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Inner {
        runs: HashMap<Uuid, RemediationRun>,
        plans: HashMap<Uuid, ConsolidationPlan>,
        dispositions: HashMap<Uuid, Vec<(i64, MergeDisposition)>>,
    }

    /// A thread-safe in-memory [`RemediationRunRepository`].
    #[derive(Default)]
    pub struct InMemoryRemediationRunRepository {
        inner: Mutex<Inner>,
    }

    impl InMemoryRemediationRunRepository {
        pub fn new() -> Self {
            Self::default()
        }

        /// Test helper: dispositions recorded for a run, in insertion order.
        pub fn dispositions_for(&self, run_id: Uuid) -> Vec<(i64, MergeDisposition)> {
            self.inner
                .lock()
                .unwrap()
                .dispositions
                .get(&run_id)
                .cloned()
                .unwrap_or_default()
        }

        /// Test helper: the stored consolidation plan, if any.
        pub fn plan_for(&self, run_id: Uuid) -> Option<ConsolidationPlan> {
            self.inner.lock().unwrap().plans.get(&run_id).cloned()
        }
    }

    #[async_trait]
    impl RemediationRunRepository for InMemoryRemediationRunRepository {
        async fn create_run(
            &self,
            repository_id: Uuid,
            autonomy_level: AutonomyLevel,
        ) -> AmpelResult<RemediationRun> {
            let run = RemediationRun {
                id: Uuid::new_v4(),
                repository_id,
                state: RunState::Created,
                autonomy_level,
                consolidated_pr_number: None,
                head_sha: None,
                error_message: None,
            };
            self.inner.lock().unwrap().runs.insert(run.id, run.clone());
            Ok(run)
        }

        async fn get_run(&self, id: Uuid) -> AmpelResult<Option<RemediationRun>> {
            Ok(self.inner.lock().unwrap().runs.get(&id).cloned())
        }

        async fn transition_state(
            &self,
            id: Uuid,
            from: RunState,
            to: RunState,
            updates: RunUpdate,
        ) -> AmpelResult<bool> {
            if !from.can_transition_to(to) {
                return Err(AmpelError::ValidationError(format!(
                    "illegal run transition: {from} -> {to}"
                )));
            }
            let mut inner = self.inner.lock().unwrap();
            let run = inner
                .runs
                .get_mut(&id)
                .ok_or_else(|| AmpelError::NotFound(format!("remediation run {id}")))?;
            // CAS: only swap when the observed state still matches `from`.
            if run.state != from {
                return Ok(false);
            }
            run.state = to;
            if let Some(pr) = updates.consolidated_pr_number {
                run.consolidated_pr_number = Some(pr);
            }
            if let Some(sha) = updates.head_sha {
                run.head_sha = Some(sha);
            }
            if let Some(msg) = updates.error_message {
                run.error_message = Some(msg);
            }
            Ok(true)
        }

        async fn record_disposition(
            &self,
            run_id: Uuid,
            pr_number: i64,
            disposition: MergeDisposition,
        ) -> AmpelResult<()> {
            self.inner
                .lock()
                .unwrap()
                .dispositions
                .entry(run_id)
                .or_default()
                .push((pr_number, disposition));
            Ok(())
        }

        async fn set_consolidation_plan(
            &self,
            run_id: Uuid,
            plan: ConsolidationPlan,
        ) -> AmpelResult<()> {
            self.inner.lock().unwrap().plans.insert(run_id, plan);
            Ok(())
        }

        async fn set_consolidated_pr(&self, run_id: Uuid, pr_number: i64) -> AmpelResult<()> {
            let mut inner = self.inner.lock().unwrap();
            if let Some(run) = inner.runs.get_mut(&run_id) {
                run.consolidated_pr_number = Some(pr_number);
            }
            Ok(())
        }

        async fn closed_source_prs(&self, run_id: Uuid) -> AmpelResult<Vec<i64>> {
            Ok(self
                .inner
                .lock()
                .unwrap()
                .dispositions
                .get(&run_id)
                .map(|ds| {
                    ds.iter()
                        .filter(|(_, d)| matches!(d, MergeDisposition::ClosedWithRef { .. }))
                        .map(|(pr, _)| *pr)
                        .collect()
                })
                .unwrap_or_default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_create_run_in_created_state() {
        // Arrange
        let repo = InMemoryRemediationRunRepository::new();

        // Act
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Assert
        assert_eq!(run.state, RunState::Created);
        assert_eq!(run.autonomy_level, AutonomyLevel::FullyAutonomous);
    }

    #[tokio::test]
    async fn should_transition_state_when_from_matches() {
        // Arrange
        let repo = InMemoryRemediationRunRepository::new();
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Act
        let swapped = repo
            .transition_state(
                run.id,
                RunState::Created,
                RunState::Selecting,
                RunUpdate::none(),
            )
            .await
            .unwrap();

        // Assert
        assert!(swapped);
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::Selecting
        );
    }

    #[tokio::test]
    async fn should_fail_cas_when_state_already_advanced() {
        // Arrange: advance the run past `created`.
        let repo = InMemoryRemediationRunRepository::new();
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        repo.transition_state(
            run.id,
            RunState::Created,
            RunState::Selecting,
            RunUpdate::none(),
        )
        .await
        .unwrap();

        // Act: a second writer still believes the state is `created`.
        let swapped = repo
            .transition_state(
                run.id,
                RunState::Created,
                RunState::Selecting,
                RunUpdate::none(),
            )
            .await
            .unwrap();

        // Assert: CAS loses the race.
        assert!(!swapped);
    }

    #[tokio::test]
    async fn should_reject_illegal_transition() {
        // Arrange
        let repo = InMemoryRemediationRunRepository::new();
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Act: created -> merging is not a legal edge.
        let result = repo
            .transition_state(
                run.id,
                RunState::Created,
                RunState::Merging,
                RunUpdate::none(),
            )
            .await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_apply_head_sha_update_with_transition() {
        // Arrange
        let repo = InMemoryRemediationRunRepository::new();
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Act: walk to verifying then carry a head sha into merging.
        repo.transition_state(
            run.id,
            RunState::Created,
            RunState::Selecting,
            RunUpdate::none(),
        )
        .await
        .unwrap();
        repo.transition_state(
            run.id,
            RunState::Selecting,
            RunState::Consolidating,
            RunUpdate::none(),
        )
        .await
        .unwrap();
        repo.transition_state(
            run.id,
            RunState::Consolidating,
            RunState::Verifying,
            RunUpdate::none(),
        )
        .await
        .unwrap();
        repo.transition_state(
            run.id,
            RunState::Verifying,
            RunState::Merging,
            RunUpdate::with_head_sha("deadbeef"),
        )
        .await
        .unwrap();

        // Assert
        let fetched = repo.get_run(run.id).await.unwrap().unwrap();
        assert_eq!(fetched.head_sha.as_deref(), Some("deadbeef"));
    }

    #[tokio::test]
    async fn should_record_dispositions_in_order() {
        // Arrange
        let repo = InMemoryRemediationRunRepository::new();
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Act
        repo.record_disposition(run.id, 1, MergeDisposition::Consolidated)
            .await
            .unwrap();
        repo.record_disposition(
            run.id,
            2,
            MergeDisposition::SkippedConflict {
                reason: "lockfile".into(),
            },
        )
        .await
        .unwrap();

        // Assert
        let recorded = repo.dispositions_for(run.id);
        assert_eq!(recorded.len(), 2);
        assert_eq!(recorded[0].0, 1);
        assert_eq!(recorded[1].0, 2);
    }
}
