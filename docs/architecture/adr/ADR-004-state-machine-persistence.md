# ADR-004: State Machine Persistence for RemediationRun

**Status**: Accepted  
**Date**: 2026-06-24  
**Deciders**: Architecture Team  
**Technical Story**: Each per-repo remediation run is a multi-step operation that must
survive worker restarts, Fly.io pod evictions, and concurrent trigger attempts without
losing progress or producing duplicate consolidated branches.

---

## Context

### Problem Statement

A `RemediationRunJob` executes a sequence of slow, side-effecting operations: cloning
a repository (seconds to minutes), waiting for CI (minutes to hours), and calling
provider APIs (seconds each). If the Apalis worker restarts mid-run, the in-progress
run must be resumable from the last persisted checkpoint, not restarted from scratch.

Concurrent triggers compound the problem: the outer `RemediationSweepJob` fires on a
cron schedule; operators can also trigger runs manually via API. Two triggers for the
same repository must not produce two consolidated PRs for the same set of source PRs.

The entire audit trail (which PRs were selected, what conflicts were found, which CI
checks passed, what the agent did) must be queryable after the run completes.

### Technical Context

- Apalis 0.6 jobs are PostgreSQL-backed; a worker crash causes the job to be retried
  from the beginning by a surviving worker unless the job records its own progress.
- Fly.io workers are restarted on new deploys (rolling), on OOM, and on machine
  recycling.
- The existing `merge_operation` entity uses a parallel pattern: a `state` column
  updated per-transition, with `merge_operation_item` as child rows.
- SeaORM 1.1 `ActiveModel` partial updates map cleanly to the state-transition pattern.
- PostgreSQL `SELECT FOR UPDATE SKIP LOCKED` is used by Apalis for job dequeuing; the
  same primitive enforces the one-active-run-per-repo constraint.

---

## Decision

**Use a custom SeaORM-persisted state machine: a `state` VARCHAR column on
`remediation_run`, updated via explicit DB transactions on each transition. Each
transition follows: load (SELECT FOR UPDATE) → guard (validate expected state) → act
(side-effecting work) → commit (write new state in same transaction).**

### States

```
pending → selecting → consolidating → remediating → verifying
  → merging → closing_sources → completed
  → awaiting_approval → (approve) → merging
  → handoff_human | failed | cancelled | no_op    (terminal)
```

### `RunState` Enum

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Pending, Selecting, Consolidating, Remediating, Verifying,
    Merging, ClosingSources, AwaitingApproval,
    Completed, HandoffHuman, Failed, Cancelled, NoOp,
}

impl RunState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::HandoffHuman
            | Self::Failed | Self::Cancelled | Self::NoOp)
    }
    pub fn is_active(&self) -> bool { !self.is_terminal() }

    pub fn valid_predecessors(&self) -> &'static [RunState] {
        match self {
            Self::Selecting       => &[Self::Pending],
            Self::Consolidating   => &[Self::Selecting],
            Self::Remediating     => &[Self::Consolidating],
            Self::Verifying       => &[Self::Remediating],
            Self::Merging         => &[Self::Verifying, Self::AwaitingApproval],
            Self::ClosingSources  => &[Self::Merging],
            Self::AwaitingApproval => &[Self::Verifying],
            Self::Completed       => &[Self::ClosingSources],
            Self::HandoffHuman    => &[Self::Consolidating, Self::Remediating,
                                       Self::Verifying],
            Self::Failed          => RunState::ACTIVE_STATES,
            Self::Cancelled       => RunState::ACTIVE_STATES,
            Self::NoOp            => &[Self::Selecting],
            Self::Pending         => &[],
        }
    }
}
```

### State Transition Protocol

```rust
// In RemediationService::advance(run_id, new_state, updates)
db.transaction(|txn| async {
    let run = remediation_run::Entity::find_by_id(run_id)
        .lock(LockType::Update, LockBehavior::SkipLocked)
        .one(txn).await?
        .ok_or(NotFound)?;

    let current = RunState::try_from(run.state.as_str())?;
    if !new_state.valid_predecessors().contains(&current) {
        return Err(InvalidTransition { from: current, to: new_state });
    }

    let mut active: remediation_run::ActiveModel = run.into();
    active.state = Set(new_state.to_string());
    // Apply additional updates (branch name, pr_id, ci_status, etc.)
    for (col, val) in updates { active.set(col, val); }
    active.update(txn).await
})
```

### One-Active-Run-Per-Repo Constraint

```sql
-- PostgreSQL partial unique index (prevents concurrent active runs per repo)
CREATE UNIQUE INDEX idx_remediation_run_one_active_per_repo
  ON remediation_run (repository_id)
  WHERE state NOT IN ('completed','handoff_human','failed','cancelled','no_op');
```

### Resumability

`RemediationRunJob::execute()` reads the persisted state on entry and branches:

```rust
match RunState::try_from(run.state.as_str())? {
    RunState::Pending | RunState::Selecting   => self.do_select(run).await,
    RunState::Consolidating                   => self.do_consolidate(run).await,
    RunState::Remediating                     => self.do_remediate(run).await,
    RunState::Verifying                       => self.do_verify(run).await,
    RunState::Merging                         => self.do_merge(run).await,
    RunState::ClosingSources                  => self.do_close_sources(run).await,
    RunState::AwaitingApproval                => Ok(()), // wait for API approval
    s if s.is_terminal()                      => Ok(()), // idempotent no-op
    _                                         => unreachable!(),
}
```

The deterministic branch name `ampel/remediation/<run_id[..8]>` means a re-trigger
finds the existing consolidated branch rather than creating a duplicate.

---

## Alternatives Considered

### Option A: In-memory state machine, persist only at checkpoints (Rejected)

**Approach**: A typed state machine (hand-rolled enum) drives transitions in memory. The
DB is written only at significant checkpoints.

**Cons**:
- ❌ Worker crash between checkpoints loses progress; run restarts from scratch
- ❌ Concurrent runs mid-flight are not detected
- ❌ Audit log is incomplete
- ❌ Does not match the `merge_operation` pattern

**Verdict**: REJECTED.

### Option B: External state machine crate + SeaORM (Rejected)

**Approach**: Use `statig` or `sm` crate for compile-time state machine definition;
serialize state to DB.

**Cons**:
- ❌ `statig` (pre-1.0) has limited async support; `sm` is similar
- ❌ Adds a dependency that provides no reduction in the DB write code needed
- ❌ Custom DB update logic is still required for each transition

**Verdict**: REJECTED.

### Option C: Custom SeaORM state column with explicit transitions (ACCEPTED)

**Pros**:
- ✅ Matches `merge_operation` pattern — no new concepts for contributors
- ✅ Every transition is durable and auditable
- ✅ Resumability is trivial — read `state`, branch on it
- ✅ One-active-run-per-repo via partial unique index
- ✅ No new crate dependencies

**Cons**:
- ⚠️ State is a string at the DB layer; mitigated by `RunState::try_from` catching
  invalid values at runtime

**Verdict**: ACCEPTED.

---

## Trade-off Analysis

| Aspect | Option A (in-memory) | Option B (crate + SeaORM) | Option C (SeaORM column) ⭐ |
|--------|---------------------|--------------------------|---------------------------|
| **Durability** | ❌ Checkpoint only | ✅ Per transition | ✅ Per transition |
| **Resumability** | ❌ Restart from 0 | ✅ Yes | ✅ Yes |
| **Concurrent-run safety** | ⚠️ Partial | ✅ Yes | ✅ Yes (partial unique index) |
| **Type safety** | ✅ Compile-time | ✅ Compile-time | ⚠️ Runtime (mitigated) |
| **Audit trail** | ❌ Incomplete | ✅ Full | ✅ Full |
| **Codebase consistency** | ⚠️ New pattern | ❌ New pattern | ✅ Matches merge_operation |
| **Dependencies** | ✅ None | ❌ New crate | ✅ None |

---

## Consequences

### Positive

- Every state transition is durable; worker restarts resume from last state
- `remediation_run` table is a complete audit log
- Concurrent trigger races resolved by partial unique index
- No new crate dependencies

### Negative

- Each transition incurs a DB round-trip; acceptable for IO-bound operations
- Invalid state strings caught at runtime; mitigated by `RunState::try_from`

### Neutral

- Future contributors learn the pattern from `merge_operation`

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| DB lock contention on high-volume fleets | Medium | `SKIP LOCKED` keeps contention low; per-repo lock is short-lived |
| Worker restart between DB write and side-effect | Low | Transition written before side-effect; on resume, side-effect retried idempotently |
| Invalid state string in DB | Low | `RunState::try_from` panics on unknown value in debug; returns `Err` in release |

---

## Related ADRs

- ADR-002: `RemediationCapable` supertrait — called during `consolidating` and
  `closing_sources` transitions
- ADR-003: Sandbox isolation — `consolidating` transition spawns the Podman container
- ADR-010: CI verification TOCTOU guard — `merging` transition re-verifies before the
  merge API call within the same transaction context
