Created `/Users/cphillipson/Development/active/ai/ampel/docs/architecture/ddd/domain-events.md` — a 330-line domain events catalog for the Fleet Remediation bounded context.

**What was written:**

**Section 1 — Event anatomy.** A generic `DomainEvent<P>` envelope (id, event_type, run_id, repository_id, occurred_at, payload) with the emission rule: events fire only after the state transition is durably committed, so a crashed worker restarts from stable state without re-emitting.

**Section 2 — 20 events, each with:**
- Trigger condition (plain English)
- State transition arrow (e.g. `consolidating → remediating`)
- Typed Rust payload struct with field-level comments
- Subscriber list

Key design details captured per event:
- `PrSelectionCompleted` carries a `HashMap<Uuid, String>` of per-exclusion reasons and a `no_qualifying_prs` flag so consumers can distinguish "below threshold" from "all excluded by label."
- `ConflictDetected` is per-source-PR (not per-run) and includes the `failure_class` classifier output that drives Tier 1 branching.
- `CiVerificationStarted` anchors the TOCTOU guard by recording `consolidated_ref_sha` before polling begins.
- `CiVerificationCompleted` carries the full `Vec<CheckResult>` (name, conclusion, required, url) so the frontend can render the CI matrix without a second API call.
- `SpendCapExceeded` is emitted before the blocked call, not after a failure — the harness checks spend before each model call.
- `ModelProviderAccountValidated` is explicitly NOT on the run SSE stream; it goes to a separate per-user account channel.

**Section 3 — Persistence/SSE disposition table.** All 20 events are persisted (no ephemeral events; all audit-logged before broadcast). SSE reconnect uses `Last-Event-ID` replay. Agent-tier events go to the agent panel sub-stream, not the top-level run timeline.

**Section 4 — Shared Rust enums.** `TriggeredBy`, `AutonomyLevel`, `RemediationTier`, `AgentOutcome` with serde `rename_all = "snake_case"`. Shows the JSON wire format the frontend discriminates on `event_type`.

**Section 5 — Subscriber map.** 9 named subscribers cross-referenced to events: SSE run stream, SSE account stream, audit log writer, `remediation_run` updater, `remediation_run_pr` updater, `remediation_agent_session` updater, notification worker, Prometheus metrics, sandbox teardown, and the sweep-job deduplication guard.

# Fleet Remediation — Domain Events Catalog

> **Bounded context:** Fleet PR Remediation Loops.
> **Reference design:** `docs/planning/autonomous-remediation/REMEDIATION_LOOPS_DESIGN.md`
> **ADRs:** ADR-002 through ADR-010.

This catalog is the authoritative reference for every domain event emitted within the Fleet Remediation bounded context. Implementors use it to wire subscribers, populate the audit log, feed SSE streams to the frontend, and ensure every state transition has a durable, typed record.

---

## Contents

1. [Event anatomy](#1-event-anatomy)
2. [Event inventory](#2-event-inventory)
3. [Persistence and SSE disposition table](#3-persistence-and-sse-disposition-table)
4. [Rust type definitions](#4-rust-type-definitions)
5. [Subscriber map](#5-subscriber-map)

---

## 1. Event Anatomy

Every event shares a common envelope.

```rust
/// Common envelope for all Fleet Remediation domain events.
/// Serialized to JSON and stored in `remediation_event` or published
/// to the SSE broadcast channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent<P> {
    /// Globally unique event identifier.
    pub id: Uuid,
    /// PascalCase event name (the discriminant).
    pub event_type: &'static str,
    /// The remediation_run this event belongs to (None for account-level events).
    pub run_id: Option<Uuid>,
    /// Repository the run is acting on (None for account-level events).
    pub repository_id: Option<Uuid>,
    /// Wall-clock emission time (UTC).
    pub occurred_at: DateTime<Utc>,
    /// Event-specific payload.
    pub payload: P,
}
```

**Emission rule:** events are emitted *after* the state transition has been committed to the database, never before. This ensures a crashed worker restarts from a stable state and does not re-emit an event for a transition that was never durably recorded.

**Naming rule:** event names are PascalCase verb phrases in the past tense, anchored to the aggregate that changed.

---

## 2. Event Inventory

### 2.1 RemediationRunStarted

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | A new `remediation_run` row is created and its state set to `pending`. Occurs on cron sweep, operator-triggered manual run, or preview/dry-run. |
| **State transition** | `(none) → pending` |

**Payload:**

```rust
pub struct RemediationRunStartedPayload {
    pub run_id: Uuid,
    pub repository_id: Uuid,
    pub policy_id: Uuid,
    /// "schedule" | "manual" | "preview"
    pub triggered_by: TriggeredBy,
    /// Resolved effective policy at the moment of trigger.
    pub autonomy_level: AutonomyLevel,
    pub remediation_tier: RemediationTier,
}
```

**Subscribers:** `RemediationSweepJob` (deduplication guard), SSE broadcast (frontend run timeline), audit log.

---

### 2.2 PrSelectionCompleted

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | `RemediationService::select_prs` finishes; state transitions to `selecting`. Emitted whether any PRs were selected or not. |
| **State transition** | `pending → selecting` |

**Payload:**

```rust
pub struct PrSelectionCompletedPayload {
    /// IDs of pull_request rows that will be consolidated.
    pub selected_pr_ids: Vec<Uuid>,
    /// IDs excluded from this run.
    pub excluded_pr_ids: Vec<Uuid>,
    /// Per-excluded-PR reason keyed by pull_request.id.
    /// Possible values: "draft", "label_excluded", "changes_requested",
    /// "too_new", "author_not_in_allowlist", "below_threshold".
    pub exclusion_reasons: HashMap<Uuid, String>,
    /// True when selected_pr_ids.len() == 0; run will proceed to no_op.
    pub no_qualifying_prs: bool,
}
```

**Subscribers:** SSE broadcast (fleet overview eligibility badge), audit log.

---

### 2.3 ConsolidationStarted

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | The sandbox receives control and begins creating the consolidated branch. State transitions to `consolidating`. |
| **State transition** | `selecting → consolidating` |

**Payload:**

```rust
pub struct ConsolidationStartedPayload {
    /// Short-lived sandbox identifier (container/worktree ID).
    pub sandbox_id: String,
    /// The target consolidated branch name (deterministic:
    /// "ampel/remediation/<run-short-id>").
    pub target_branch: String,
    /// Source branches being merged in order.
    pub source_branches: Vec<String>,
    /// Provider-resolved default branch the consolidated branch forks from.
    pub base_branch: String,
    /// SHA of the base branch at the moment the clone was taken.
    pub base_sha: String,
}
```

**Subscribers:** SSE broadcast (run timeline — "Consolidating…" step), audit log.

---

### 2.4 ConsolidationCompleted

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | The sandbox pushes the consolidated branch and the provider creates the consolidated PR. State transitions to `remediating`. |
| **State transition** | `consolidating → remediating` |

**Payload:**

```rust
pub struct ConsolidationCompletedPayload {
    pub consolidated_branch: String,
    /// Provider-assigned PR number.
    pub consolidated_pr_number: i64,
    /// Full URL to the consolidated PR on the provider.
    pub consolidated_pr_url: String,
    /// Per-source-PR outcome after the octopus merge.
    pub per_pr_dispositions: Vec<PrDisposition>,
}

pub struct PrDisposition {
    pub pr_id: Uuid,
    pub pr_number: i64,
    /// "consolidated" | "skipped_conflict" | "skipped_draft"
    pub disposition: String,
    /// Present when disposition == "skipped_conflict".
    pub conflict_files: Option<Vec<String>>,
}
```

**Subscribers:** SSE broadcast (run timeline — consolidated PR link rendered), audit log, `remediation_run` row updated (`consolidated_pr_number`, `consolidated_pr_url`).

---

### 2.5 ConflictDetected

| Field | Value |
|---|---|
| **Emitted by** | `ConsolidationStrategy` (within `RemediationRun` aggregate) |
| **Trigger** | A source branch cannot be octopus-merged cleanly into the consolidated branch. One event is emitted per conflicting source PR. |
| **State transition** | No state change; run continues with remaining branches. If *all* selected branches conflict and agentic tier is off, run transitions to `handoff_human`. |

**Payload:**

```rust
pub struct ConflictDetectedPayload {
    /// The source pull_request.id that introduced the conflict.
    pub pr_id: Uuid,
    pub pr_number: i64,
    /// Paths reported by `git merge` as conflicting.
    pub conflict_files: Vec<String>,
    /// Classifier output. One of:
    /// "lockfile_npm" | "lockfile_cargo" | "lockfile_go" |
    /// "lockfile_python" | "lockfile_ruby" |
    /// "adjacent_lines" | "unknown"
    pub failure_class: String,
    /// True if Tier 1 mechanical resolution will be attempted.
    pub mechanical_resolution_applicable: bool,
}
```

**Subscribers:** SSE broadcast (conflict badge per source PR in run timeline), audit log, `remediation_run_pr.disposition` set to `skipped_conflict`.

---

### 2.6 LockfileRegenerationCompleted

| Field | Value |
|---|---|
| **Emitted by** | `ConsolidationStrategy` Tier 1 path (within `RemediationRun` aggregate) |
| **Trigger** | A lockfile conflict was classified as mechanically resolvable (e.g., `package-lock.json`, `Cargo.lock`, `go.sum`), and the regeneration command exited 0. |
| **State transition** | No state change; run continues in `remediating`. |

**Payload:**

```rust
pub struct LockfileRegenerationCompletedPayload {
    /// Paths that were regenerated (not line-merged).
    pub regenerated_files: Vec<String>,
    /// Commands executed in order, e.g. ["cargo update --workspace"].
    pub commands_run: Vec<String>,
    /// Exit code (always 0 on success; this event is only emitted on success).
    pub exit_code: i32,
    /// Elapsed wall-clock time in milliseconds.
    pub duration_ms: u64,
}
```

**Subscribers:** SSE broadcast (Tier 1 remediation step in run timeline), audit log.

---

### 2.7 CiVerificationStarted

| Field | Value |
|---|---|
| **Emitted by** | `VerificationService` (via `RemediationRun` aggregate) |
| **Trigger** | The consolidated branch has been pushed (and optionally remediated) and the system begins polling provider CI. State transitions to `verifying`. |
| **State transition** | `remediating → verifying` |

**Payload:**

```rust
pub struct CiVerificationStartedPayload {
    /// The exact git SHA being verified. This is the TOCTOU guard anchor.
    pub consolidated_ref_sha: String,
    /// Branch name used to look up CI checks.
    pub consolidated_branch: String,
    /// Required check names resolved from branch protection at this moment.
    pub required_checks: Vec<String>,
    /// Maximum seconds the poller will wait before declaring timeout.
    pub poll_timeout_secs: u64,
}
```

**Subscribers:** SSE broadcast (CI verification step — required checks rendered), audit log.

---

### 2.8 CiVerificationCompleted

| Field | Value |
|---|---|
| **Emitted by** | `VerificationService` (via `RemediationRun` aggregate) |
| **Trigger** | CI polling reaches a terminal conclusion: all required checks are green, at least one required check is red, or the poll timeout is exceeded. |
| **State transition** | `verifying → awaiting_approval` (open-loop) or `verifying → merging` (closed-loop, green) or `verifying → remediating` (red, budget remaining, Tier 2 on) or `verifying → handoff_human` (red, exhausted). |

**Payload:**

```rust
pub struct CiVerificationCompletedPayload {
    pub consolidated_ref_sha: String,
    /// AmpelStatus traffic-light aggregate over all required checks.
    /// "green" | "yellow" | "red"
    pub ampel_status: String,
    /// All required check names and their conclusions at verification time.
    pub required_checks: Vec<CheckResult>,
    /// True iff every required check concluded green.
    pub all_required_green: bool,
    /// True iff the ref is mergeable (no conflicts, not draft, no
    /// changes-requested review). False blocks merge even if CI is green.
    pub mergeable: bool,
    /// False if the verification timed out rather than completing cleanly.
    pub completed_within_timeout: bool,
}

pub struct CheckResult {
    pub name: String,
    /// "success" | "failure" | "pending" | "skipped" | "cancelled"
    pub conclusion: String,
    pub required: bool,
    pub url: Option<String>,
}
```

**Subscribers:** SSE broadcast (CI matrix in run timeline — traffic light coloured), `remediation_run.ci_status` updated, audit log.

---

### 2.9 RemediationRunApproved

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | An operator calls `POST /api/remediation/runs/{id}/approve` on a run in `awaiting_approval` state. State transitions to `merging`. |
| **State transition** | `awaiting_approval → merging` |

**Payload:**

```rust
pub struct RemediationRunApprovedPayload {
    /// Identity of the approver (user ID from JWT).
    pub approved_by: Uuid,
    pub approved_at: DateTime<Utc>,
    /// Snapshot of the CI status at the moment of approval (re-verified
    /// by the TOCTOU guard before merge actually executes).
    pub ci_status_at_approval: String,
}
```

**Subscribers:** SSE broadcast (run timeline — "Approved by X" step), audit log, notification worker (Slack/email).

---

### 2.10 RemediationRunMerged

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | The provider API confirms the consolidated PR was merged. State transitions to `closing_sources`. |
| **State transition** | `merging → closing_sources` |

**Payload:**

```rust
pub struct RemediationRunMergedPayload {
    /// The merge commit SHA returned by the provider.
    pub merged_sha: String,
    /// "merge" | "squash" | "rebase"
    pub merge_strategy_used: String,
    /// Timestamp the provider reports for the merge.
    pub merged_at: DateTime<Utc>,
    pub consolidated_pr_number: i64,
    pub consolidated_pr_url: String,
}
```

**Subscribers:** SSE broadcast (run timeline — "Merged" step with SHA link), audit log, `remediation_run.merged` set to `true`, notification worker.

---

### 2.11 SourcePrsClosed

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | All source PRs have been closed with a back-reference comment. State transitions to `completed`. Emitted once, after all closures succeed. |
| **State transition** | `closing_sources → completed` |

**Payload:**

```rust
pub struct SourcePrsClosedPayload {
    /// IDs of pull_request rows that were closed.
    pub closed_pr_ids: Vec<Uuid>,
    /// Provider PR numbers that were closed.
    pub closed_pr_numbers: Vec<i64>,
    /// The comment body posted on each closed PR, e.g.:
    /// "Superseded by #<consolidated_pr_number> — changes incorporated and merged."
    pub comment_text: String,
    /// True if source branches were also deleted (per policy).
    pub source_branches_deleted: bool,
}
```

**Subscribers:** SSE broadcast (run timeline — "Closed N source PRs"), audit log, `remediation_run.closed_pr_ids` updated, `remediation_run_pr.disposition` set to `closed_with_ref`.

---

### 2.12 RemediationRunCompleted

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | The run reaches the `completed` terminal state (merge + source-PR closure both succeeded). |
| **State transition** | `closing_sources → completed` (terminal) |

**Payload:**

```rust
pub struct RemediationRunCompletedPayload {
    /// Elapsed seconds from run creation to completion.
    pub total_duration_secs: u64,
    /// Number of source PRs that were merged into the consolidated PR.
    pub source_pr_count_merged: u32,
    /// Number of source PRs that were left open (conflict, skipped).
    pub source_pr_count_skipped: u32,
    pub consolidated_pr_number: i64,
    pub consolidated_pr_url: String,
    pub merged_sha: String,
    /// "mechanical_only" | "agentic" — which tier was ultimately used.
    pub remediation_tier_used: String,
}
```

**Subscribers:** SSE broadcast (run timeline — terminal green state), audit log, Prometheus counter `remediation_runs_completed_total`, notification worker.

---

### 2.13 HandoffRequired

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | The run cannot proceed autonomously: unresolvable conflicts with agentic tier off, CI red after budget exhausted, or agent aborted. State transitions to `handoff_human`. |
| **State transition** | `consolidating | remediating | verifying → handoff_human` (terminal for this cycle) |

**Payload:**

```rust
pub struct HandoffRequiredPayload {
    /// Human-readable reason, e.g.:
    /// "All source branches conflict and agentic tier is disabled."
    /// "CI remained red after 6 agent iterations (budget exhausted)."
    pub reason: String,
    /// The state the run was in when it could not proceed.
    pub last_state: String,
    /// Specific, operator-actionable suggestions.
    pub actionable_suggestions: Vec<String>,
    /// Link to the consolidated PR if one was created (may be None if
    /// consolidation itself failed).
    pub consolidated_pr_url: Option<String>,
    /// Conflicting PR numbers, if applicable.
    pub conflicting_pr_numbers: Vec<i64>,
}
```

**Subscribers:** SSE broadcast (run timeline — "Needs attention" terminal state), audit log, notification worker (elevated priority — human action required).

---

### 2.14 RemediationRunFailed

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | An unrecoverable error occurs: provider API returning 5xx, sandbox crash, database write failure, or any panic boundary crossed. State transitions to `failed`. |
| **State transition** | `any → failed` (terminal) |

**Payload:**

```rust
pub struct RemediationRunFailedPayload {
    /// Structured error string (never contains secrets; PATs redacted).
    pub error: String,
    /// Error kind for metrics/alerting.
    /// "provider_api" | "sandbox" | "database" | "timeout" | "internal"
    pub error_kind: String,
    /// State the run was in when the error occurred.
    pub last_state: String,
    /// Number of prior attempts (for retry-aware consumers).
    pub attempt_number: u32,
}
```

**Subscribers:** SSE broadcast (run timeline — error terminal state), audit log, Prometheus counter `remediation_runs_failed_total`, notification worker (alert).

---

### 2.15 RemediationRunCancelled

| Field | Value |
|---|---|
| **Emitted by** | `RemediationRun` aggregate |
| **Trigger** | An operator calls `POST /api/remediation/runs/{id}/cancel`. State transitions to `cancelled`. |
| **State transition** | `pending | selecting | consolidating | remediating | verifying | awaiting_approval → cancelled` (terminal) |

**Payload:**

```rust
pub struct RemediationRunCancelledPayload {
    /// User ID of the operator who cancelled.
    pub cancelled_by: Uuid,
    pub cancelled_at: DateTime<Utc>,
    /// State at the time of cancellation.
    pub state_at_cancellation: String,
    /// True if the sandbox was active and had to be destroyed.
    pub sandbox_destroyed: bool,
}
```

**Subscribers:** SSE broadcast (run timeline — "Cancelled by X"), audit log, sandbox teardown signal.

---

### 2.16 AgentSessionStarted

| Field | Value |
|---|---|
| **Emitted by** | `RemediationAgentHarness` (within `RemediationRun` aggregate) |
| **Trigger** | Tier 2 agentic remediation begins: the harness has resolved the playbook, selected the model provider, and is about to make the first inference or agent-delegation call. |
| **State transition** | `remediating` (no state change; this is an intra-state event) |

**Payload:**

```rust
pub struct AgentSessionStartedPayload {
    pub agent_session_id: Uuid,
    /// model_provider_account.id selected for this run.
    pub provider_id: Uuid,
    /// Provider-specific model identifier, e.g. "claude-sonnet-4-5".
    pub model_id: String,
    /// Resolved playbook identifier.
    pub playbook_id: String,
    /// Source of playbook: "embedded" | "db_override" | "repo_local".
    pub playbook_source: String,
    /// Failure class that triggered agentic escalation.
    /// e.g. "ci_red_after_lockfile_regen" | "merge_conflict_unknown"
    pub failure_class: String,
    /// Budget the harness will enforce.
    pub budget_max_iterations: u32,
    pub budget_max_seconds: u64,
    pub budget_max_cost_usd: f64,
}
```

**Subscribers:** SSE broadcast (agent session panel in run timeline), audit log, `remediation_agent_session` row created.

---

### 2.17 AgentIterationCompleted

| Field | Value |
|---|---|
| **Emitted by** | `RemediationAgentHarness` (within `RemediationRun` aggregate) |
| **Trigger** | One full harness iteration completes: the model was called (or the agent sub-loop finished), edits were applied to the worktree and pushed, and CI status was sampled. |
| **State transition** | `remediating` (no state change; emitted once per iteration) |

**Payload:**

```rust
pub struct AgentIterationCompletedPayload {
    pub agent_session_id: Uuid,
    /// 1-based iteration counter.
    pub iteration_number: u32,
    /// AmpelStatus of the consolidated branch after this iteration's push.
    /// "green" | "yellow" | "red" | "pending"
    pub ci_status_after: String,
    /// Token usage this iteration (input + output).
    pub tokens_used: u64,
    /// Estimated cost in USD for this iteration (0.0 for local models).
    pub cost_usd: f64,
    /// Summary of edits applied (file paths changed).
    pub files_changed: Vec<String>,
    /// True if the harness assessed this as the final iteration
    /// (CI green or budget reached — next event will be AgentSessionCompleted).
    pub is_final: bool,
}
```

**Subscribers:** SSE broadcast (live agent iteration feed in run timeline), audit log, `remediation_agent_session` updated (`iterations`, `tokens`, `cost_usd`).

---

### 2.18 AgentSessionCompleted

| Field | Value |
|---|---|
| **Emitted by** | `RemediationAgentHarness` (within `RemediationRun` aggregate) |
| **Trigger** | The agentic session ends — either the agent produced a green CI, it ran out of budget, or it was aborted (cancelled or internal error). Control returns to `VerificationService`. |
| **State transition** | `remediating → verifying` (passed) or `remediating → handoff_human` (budget_exhausted | aborted) |

**Payload:**

```rust
pub struct AgentSessionCompletedPayload {
    pub agent_session_id: Uuid,
    /// "passed" | "budget_exhausted" | "aborted"
    pub outcome: AgentOutcome,
    pub total_iterations: u32,
    pub total_tokens_used: u64,
    /// Cumulative cost in USD across all iterations.
    pub total_cost_usd: f64,
    /// Elapsed wall-clock seconds for the session.
    pub duration_secs: u64,
    /// Opaque reference to the stored transcript (e.g. S3 key, DB blob ID).
    pub transcript_ref: Option<String>,
}

pub enum AgentOutcome {
    Passed,
    BudgetExhausted,
    Aborted,
}
```

**Subscribers:** SSE broadcast (agent session summary panel), audit log, `remediation_agent_session.outcome` updated, Prometheus histogram `remediation_agent_cost_usd`.

---

### 2.19 ModelProviderAccountValidated

| Field | Value |
|---|---|
| **Emitted by** | `ModelProviderAccountService` aggregate |
| **Trigger** | A `model_provider_account` credential is validated — either on creation, on explicit re-validation request, or at the start of an agent session. Emitted on both success and failure. |
| **State transition** | Credential `status` field set to `valid` or `invalid`. |

**Payload:**

```rust
pub struct ModelProviderAccountValidatedPayload {
    /// model_provider_account.id
    pub account_id: Uuid,
    /// Provider kind: "claude" | "gemini" | "ollama" | "onnx" | "openai_compatible"
    pub provider_kind: String,
    /// True = credentials passed validation; false = failed.
    pub success: bool,
    /// Human-readable failure reason (None on success).
    /// Never contains the credential itself.
    pub failure_reason: Option<String>,
    /// When the validation result was obtained.
    pub validated_at: DateTime<Utc>,
    /// When the credential should be re-validated (None if permanent).
    pub revalidate_after: Option<DateTime<Utc>>,
}
```

**Subscribers:** Audit log (always), SSE broadcast to the credential management UI (account status badge update). Not sent to the run timeline SSE stream.

---

### 2.20 SpendCapExceeded

| Field | Value |
|---|---|
| **Emitted by** | `RemediationAgentHarness` (spend guard, within `RemediationRun` aggregate) |
| **Trigger** | The harness checks the running cost against `agent_budget.max_cost_usd` before issuing a model call, and the accumulated spend meets or exceeds the cap. The call is blocked and this event is emitted. Triggers `AgentSessionCompleted` with `outcome = BudgetExhausted`. |
| **State transition** | Intra-session; leads to `AgentSessionCompleted(BudgetExhausted)`. |

**Payload:**

```rust
pub struct SpendCapExceededPayload {
    pub agent_session_id: Uuid,
    pub run_id: Uuid,
    /// USD accumulated across all iterations in this session.
    pub spend_used_usd: f64,
    /// The policy cap that was hit.
    pub spend_cap_usd: f64,
    /// Iteration number at which the cap was hit.
    pub at_iteration: u32,
    /// The model call that was blocked.
    pub blocked_provider_kind: String,
    pub blocked_model_id: String,
}
```

**Subscribers:** Audit log, SSE broadcast (agent session panel — "Spend cap reached" warning), Prometheus counter `remediation_spend_cap_exceeded_total`, notification worker (alert operator).

---

## 3. Persistence and SSE Disposition Table

| Event | Persisted to audit log | Surfaced via SSE |
|---|---|---|
| `RemediationRunStarted` | Yes | Yes |
| `PrSelectionCompleted` | Yes | Yes |
| `ConsolidationStarted` | Yes | Yes |
| `ConsolidationCompleted` | Yes | Yes |
| `ConflictDetected` | Yes | Yes |
| `LockfileRegenerationCompleted` | Yes | Yes |
| `CiVerificationStarted` | Yes | Yes |
| `CiVerificationCompleted` | Yes | Yes |
| `RemediationRunApproved` | Yes | Yes |
| `RemediationRunMerged` | Yes | Yes |
| `SourcePrsClosed` | Yes | Yes |
| `RemediationRunCompleted` | Yes | Yes |
| `HandoffRequired` | Yes | Yes |
| `RemediationRunFailed` | Yes | Yes |
| `RemediationRunCancelled` | Yes | Yes |
| `AgentSessionStarted` | Yes | Yes (agent panel only) |
| `AgentIterationCompleted` | Yes | Yes (agent panel only) |
| `AgentSessionCompleted` | Yes | Yes (agent panel only) |
| `ModelProviderAccountValidated` | Yes | Yes (credential UI only, not run stream) |
| `SpendCapExceeded` | Yes | Yes (agent panel + alert) |

**Audit log:** The `remediation_event` table stores every event as an append-only JSON row with `(id, run_id, repository_id, event_type, payload JSONB, occurred_at)`. It is never updated or deleted; it is the source of truth for the run audit page and export.

**SSE:** Events are published to a per-run broadcast channel (`/api/remediation/runs/{id}/events`). The frontend subscribes via `EventSource`. The channel reuses the same SSE infrastructure as the existing bulk-merge progress stream. Account-level events (`ModelProviderAccountValidated`) are published to a separate account-scoped channel, not the run stream.

**Ephemeral events:** There are no ephemeral events in this catalog. All events are persisted before they are broadcast. If the SSE connection drops, the client reconnects and replays from `Last-Event-ID`.

---

## 4. Rust Type Definitions

The following types are shared across payload structs above.

```rust
/// How the run was initiated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggeredBy {
    Schedule,
    Manual,
    Preview,
}

/// Autonomy level resolved from the effective policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyLevel {
    Off,
    DryRun,
    ConsolidateOnly,
    AutoMerge,
}

/// Which remediation tier is configured.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationTier {
    MechanicalOnly,
    Agentic,
}

/// Agentic session terminal outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentOutcome {
    Passed,
    BudgetExhausted,
    Aborted,
}
```

The `DomainEvent<P>` envelope (§1) is published to the SSE stream serialized as:

```json
{
  "id": "<uuid>",
  "event_type": "CiVerificationCompleted",
  "run_id": "<uuid>",
  "repository_id": "<uuid>",
  "occurred_at": "2026-06-24T14:32:01.123Z",
  "payload": { ... }
}
```

The frontend uses `event_type` as the discriminant to route incoming SSE messages to the appropriate run timeline step component.

---

## 5. Subscriber Map

| Subscriber | Events consumed | Action |
|---|---|---|
| **SSE broadcast (run stream)** | All run-scoped events | Publish to per-run `BroadcastChannel<DomainEvent>` |
| **SSE broadcast (account stream)** | `ModelProviderAccountValidated` | Publish to per-user account channel |
| **Audit log writer** | All events | Append to `remediation_event` table (JSONB) |
| **`remediation_run` updater** | `ConsolidationCompleted`, `CiVerificationCompleted`, `RemediationRunMerged`, `SourcePrsClosed`, `RemediationRunCompleted`, `RemediationRunFailed`, `RemediationRunCancelled`, `HandoffRequired` | Update state, outcome fields, `closed_pr_ids`, `merged_sha`, `ci_status` |
| **`remediation_run_pr` updater** | `ConflictDetected`, `SourcePrsClosed` | Set `disposition` per source PR |
| **`remediation_agent_session` updater** | `AgentSessionStarted`, `AgentIterationCompleted`, `AgentSessionCompleted` | Create / update session row |
| **Notification worker** | `RemediationRunMerged`, `HandoffRequired`, `RemediationRunFailed`, `SpendCapExceeded`, `RemediationRunApproved` | Enqueue Slack/email notification |
| **Prometheus metrics** | `RemediationRunCompleted`, `RemediationRunFailed`, `AgentSessionCompleted`, `SpendCapExceeded` | Increment/observe counters and histograms |
| **Sandbox teardown** | `RemediationRunCancelled`, `RemediationRunFailed`, `AgentSessionCompleted` | Send SIGTERM to sandbox container; destroy worktree |
| **Deduplication guard** (`RemediationSweepJob`) | `RemediationRunStarted` | Register run in the per-repo active-run index (prevents double-dispatch) |

---

*This catalog reflects the design as of June 2026. As the state machine in `REMEDIATION_LOOPS_DESIGN.md §4` evolves, this document must be updated in the same commit that changes the state transitions.*
