# Value Objects — Fleet PR Remediation Bounded Context

This document is the canonical reference for value objects used in the Fleet
Remediation bounded context. Value objects are immutable, identity-free, and
defined entirely by their attributes. They carry invariants that must hold at
construction time; a value object must never be in an invalid state once
created.

The Rust sketches below are illustrative. Each type lives in
`crates/ampel-core/src/remediation/` unless noted otherwise.

---

## Table of Contents

1. [RemediationCriteria](#1-remediationcriteria)
2. [AgentBudget](#2-agentbudget)
3. [MergeDisposition](#3-mergedisposition)
4. [CiVerificationResult](#4-civersificationresult)
5. [AmpelStatus (reference)](#5-ampelstatus-reference)
6. [ConsolidationPlan](#6-consolidationplan)
7. [RemediationScope](#7-remediationscope)
8. [EgressClass](#8-egressclass)
9. [FailureClass](#9-failureclass)
10. [PlaybookRef](#10-playbookref)

---

## 1. RemediationCriteria

### Definition

An immutable snapshot of the effective filter set derived from resolving the
policy hierarchy at run time. Represents *what the agent is allowed to act on*
for one remediation session. Because it is a resolved snapshot, it does not
change during a session even if the underlying policy records are updated
mid-run.

The resolution order follows the hierarchical config: Repository → Team →
Organization → User, with narrower scopes winning.

### Attributes

| Attribute | Type | Description |
|---|---|---|
| `min_open_prs` | `u32` | Trigger threshold. Remediation activates only when a repo has strictly more than this many open PRs. |
| `pr_selection` | `PrSelectionStrategy` | Which PRs to act on: `AllOpen`, `OldestFirst { max: u32 }`, `ByLabel { labels: Vec<String> }`, `ExplicitIds { ids: Vec<u64> }`. |
| `autonomy_level` | `AutonomyLevel` | `DryRunOnly` | `SuggestOnly` | `AutoWithApproval` | `FullyAutonomous`. |
| `remediation_tier` | `RemediationTier` | `ConsolidateOnly` | `FixAndConsolidate` | `FullRemediation`. Controls how deeply the agent may modify code. |
| `max_prs_per_run` | `u32` | Hard cap on PRs touched per single session. |
| `allowed_targets` | `Vec<String>` | Base branch names the agent may target (e.g., `["main", "develop"]`). Empty means all branches. |
| `skip_draft` | `bool` | When `true`, draft PRs are excluded from selection. |
| `require_green_before_merge` | `bool` | If `true`, the agent must confirm `AmpelStatus::Green` immediately before any merge attempt. |
| `air_gapped` | `bool` | When `true`, only `EgressClass::LocalOnly` model providers are eligible. |
| `resolved_at` | `DateTime<Utc>` | Wall-clock time at which the policy was resolved. Recorded in the session for audit. |

### Validation Rules

- `min_open_prs` must be ≥ 1. A threshold of 0 would trigger on any repo, which is disallowed by policy.
- `max_prs_per_run` must be ≥ 1 and must be ≥ the effective `max` inside `PrSelectionStrategy::OldestFirst` when that variant is active.
- `allowed_targets` elements must be non-empty strings.
- `autonomy_level` of `FullyAutonomous` requires `remediation_tier` to be explicitly set (not defaulted); the policy resolver must surface an error if an org-level policy enables `FullyAutonomous` without a tier override.

### Rust Sketch

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemediationCriteria {
    pub min_open_prs: u32,
    pub pr_selection: PrSelectionStrategy,
    pub autonomy_level: AutonomyLevel,
    pub remediation_tier: RemediationTier,
    pub max_prs_per_run: u32,
    pub allowed_targets: Vec<String>,
    pub skip_draft: bool,
    pub require_green_before_merge: bool,
    pub air_gapped: bool,
    pub resolved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum PrSelectionStrategy {
    AllOpen,
    OldestFirst { max: u32 },
    ByLabel { labels: Vec<String> },
    ExplicitIds { ids: Vec<u64> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyLevel {
    DryRunOnly,
    SuggestOnly,
    AutoWithApproval,
    FullyAutonomous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationTier {
    ConsolidateOnly,
    FixAndConsolidate,
    FullRemediation,
}

impl RemediationCriteria {
    /// Construct a validated snapshot. Returns Err if any invariant is violated.
    pub fn new(/* fields */) -> Result<Self, RemediationCriteriaError> {
        // ... validation ...
    }
}
```

---

## 2. AgentBudget

### Definition

Hard resource limits for one agentic session. The session executor checks
remaining budget before each iteration and must abort cleanly when any limit is
reached. All three limits are active simultaneously; the first to be exhausted
ends the session.

### Attributes

| Attribute | Type | Description |
|---|---|---|
| `max_iterations` | `u32` | Maximum number of agent decision loops (tool calls + reasoning cycles) before the session is halted. |
| `max_seconds` | `u64` | Wall-clock time budget in seconds. The session start timestamp is captured at construction. |
| `max_cost_usd` | `Decimal` | Maximum spend in US dollars across all model inference calls. Uses `rust_decimal::Decimal` to avoid floating-point error in cost accounting. |

### Invariants

- All three values must be strictly greater than zero.
- `max_cost_usd` must have at most 4 decimal places (currency precision).

### Default

`{ max_iterations: 6, max_seconds: 900, max_cost_usd: 2.00 }`

This default is conservative: 6 loops is enough to analyze, plan, apply a fix,
verify CI, and merge one batch; 15 minutes and $2 are ceiling guards.

### Validation Rules

- Reject `max_iterations == 0`, `max_seconds == 0`, or `max_cost_usd <= 0`.
- `max_cost_usd` is validated against an org-level hard ceiling
  (`org_settings.agent_max_cost_usd`); the resolver must clip at the ceiling
  and surface a warning if the requested budget exceeds it.

### Rust Sketch

```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentBudget {
    pub max_iterations: u32,
    pub max_seconds: u64,
    pub max_cost_usd: Decimal,
}

impl Default for AgentBudget {
    fn default() -> Self {
        Self {
            max_iterations: 6,
            max_seconds: 900,
            max_cost_usd: Decimal::new(200, 2), // 2.00
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AgentBudgetError {
    #[error("max_iterations must be > 0")]
    ZeroIterations,
    #[error("max_seconds must be > 0")]
    ZeroSeconds,
    #[error("max_cost_usd must be > 0")]
    ZeroCost,
    #[error("max_cost_usd {requested} exceeds org ceiling {ceiling}")]
    ExceedsOrgCeiling { requested: Decimal, ceiling: Decimal },
}

impl AgentBudget {
    pub fn new(
        max_iterations: u32,
        max_seconds: u64,
        max_cost_usd: Decimal,
        org_ceiling: Option<Decimal>,
    ) -> Result<Self, AgentBudgetError> {
        if max_iterations == 0 { return Err(AgentBudgetError::ZeroIterations); }
        if max_seconds == 0    { return Err(AgentBudgetError::ZeroSeconds); }
        if max_cost_usd <= Decimal::ZERO { return Err(AgentBudgetError::ZeroCost); }
        if let Some(ceil) = org_ceiling {
            if max_cost_usd > ceil {
                return Err(AgentBudgetError::ExceedsOrgCeiling {
                    requested: max_cost_usd,
                    ceiling: ceil,
                });
            }
        }
        Ok(Self { max_iterations, max_seconds, max_cost_usd })
    }
}
```

---

## 3. MergeDisposition

### Definition

Records the final decision made by the agent for a single source PR. Set once
at the end of a remediation run and is immutable thereafter. Used to populate
the session audit log and drive SSE progress events to the frontend.

`ClosedWithRef` is used when the PR's content has been incorporated into a
consolidation PR and the original was closed (not merged directly).

### Variants

| Variant | Payload | Meaning |
|---|---|---|
| `Consolidated` | — | This PR's commits were included in the consolidated PR and the original was closed. |
| `ClosedWithRef` | `consolidated_pr_number: u64` | The PR was closed; the reference PR number is recorded for traceability. |
| `SkippedConflict` | `reason: String` | Skipped because a merge conflict was detected that the agent could not resolve within budget or tier constraints. |
| `LeftOpen` | `reason: String` | No action taken; records why (e.g., `"draft"`, `"excluded by label"`, `"budget exhausted"`). |

### Validation Rules

- `reason` fields must be non-empty strings (max 512 chars). They are displayed
  in the UI and written to the audit log verbatim.
- `consolidated_pr_number` must be > 0.
- Once constructed and stored, no mutation is permitted. Implement via
  `#[non_exhaustive]` on the enum to guard against accidental addition of
  mutable variants.

### Rust Sketch

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "disposition", rename_all = "snake_case")]
#[non_exhaustive]
pub enum MergeDisposition {
    Consolidated,
    ClosedWithRef {
        consolidated_pr_number: u64,
    },
    SkippedConflict {
        reason: String,
    },
    LeftOpen {
        reason: String,
    },
}

impl MergeDisposition {
    /// Returns true if the PR was acted upon (closed or merged).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Consolidated | Self::ClosedWithRef { .. })
    }

    /// Returns the reason text if present.
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::SkippedConflict { reason } | Self::LeftOpen { reason } => Some(reason),
            _ => None,
        }
    }
}
```

---

## 4. CiVerificationResult

### Definition

A point-in-time snapshot of the CI state for a specific commit SHA on a PR.
Used both during the agent's pre-action assessment and as the TOCTOU guard
immediately before merge. Two snapshots at different times must be compared
by `ref_sha`; if the SHA has changed, the older snapshot is stale and must
be discarded.

`NormalizedCiCheck` collapses provider-specific check representations
(GitHub Actions, GitLab pipelines, Bitbucket pipelines) into a common shape.

### Attributes

| Attribute | Type | Description |
|---|---|---|
| `ref_sha` | `String` | The commit SHA this snapshot applies to. 40-char hex for Git. |
| `checks` | `Vec<NormalizedCiCheck>` | All checks observed at this instant. |
| `all_required_green` | `bool` | Derived: `true` iff every check with `required == true` has `status == CheckStatus::Green`. |
| `mergeable` | `bool` | Provider-reported mergeability at this instant. |
| `ampel_status` | `AmpelStatus` | Aggregate traffic-light status computed from `checks` using `AmpelStatus::for_repository`. |
| `captured_at` | `DateTime<Utc>` | Wall clock at snapshot creation. |

### NormalizedCiCheck

| Attribute | Type | Description |
|---|---|---|
| `name` | `String` | Human-readable check name (e.g., `"ci / build"`, `"test"`, `"lint"`). |
| `status` | `CheckStatus` | `Pending` | `Running` | `Green` | `Red` | `Skipped` | `Cancelled`. |
| `required` | `bool` | Whether this check blocks mergeability per the branch protection rules. |
| `url` | `Option<String>` | Link to the check run detail page. |

### CheckStatus Variants

| Variant | Meaning |
|---|---|
| `Pending` | Not yet started (queued). |
| `Running` | In progress. |
| `Green` | Completed successfully. |
| `Red` | Completed with failure or timeout. |
| `Skipped` | Skipped by provider (e.g., path filters). |
| `Cancelled` | Run was cancelled. |

### Derived Field Rules

- `all_required_green` is recomputed on construction from `checks`; it is a
  derived convenience field and must never be set independently.
- `ampel_status` is similarly derived: use `AmpelStatus::for_pull_request`
  after mapping `checks` back to `CICheck` structs, or compute directly from
  the `CheckStatus` values.
- A snapshot where `checks` is empty and `mergeable == true` results in
  `all_required_green == true` only if the repo has no required checks
  configured. Callers must pass a `has_required_checks: bool` flag to
  `new()` to resolve this ambiguity.

### Rust Sketch

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::AmpelStatus;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CiVerificationResult {
    pub ref_sha: String,
    pub checks: Vec<NormalizedCiCheck>,
    pub all_required_green: bool,
    pub mergeable: bool,
    pub ampel_status: AmpelStatus,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedCiCheck {
    pub name: String,
    pub status: CheckStatus,
    pub required: bool,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pending,
    Running,
    Green,
    Red,
    Skipped,
    Cancelled,
}

impl CiVerificationResult {
    pub fn new(
        ref_sha: String,
        checks: Vec<NormalizedCiCheck>,
        mergeable: bool,
        has_required_checks: bool,
    ) -> Self {
        let all_required_green = if has_required_checks {
            checks
                .iter()
                .filter(|c| c.required)
                .all(|c| c.status == CheckStatus::Green)
        } else {
            true
        };

        // Derive AmpelStatus from normalized checks.
        let ampel_status = derive_ampel_status(&checks, mergeable);

        Self {
            ref_sha,
            checks,
            all_required_green,
            mergeable,
            ampel_status,
            captured_at: Utc::now(),
        }
    }

    /// Returns true if this snapshot is still valid for the given commit.
    pub fn is_current_for(&self, sha: &str) -> bool {
        self.ref_sha == sha
    }

    /// True when the agent may proceed to merge.
    pub fn ready_to_merge(&self) -> bool {
        self.all_required_green && self.mergeable
    }
}

fn derive_ampel_status(checks: &[NormalizedCiCheck], mergeable: bool) -> AmpelStatus {
    if !mergeable {
        return AmpelStatus::Red;
    }
    let has_red = checks.iter().any(|c| c.status == CheckStatus::Red);
    let has_pending = checks
        .iter()
        .any(|c| matches!(c.status, CheckStatus::Pending | CheckStatus::Running));
    if has_red { AmpelStatus::Red }
    else if has_pending { AmpelStatus::Yellow }
    else { AmpelStatus::Green }
}
```

---

## 5. AmpelStatus (reference)

### Definition

The project's core traffic-light metaphor. Defined in
`crates/ampel-core/src/models/ampel_status.rs`. Reproduced here for
completeness because it participates in several Fleet Remediation value
objects.

### Variants

| Variant | Meaning |
|---|---|
| `Green` | All required CI checks pass + approved + no conflicts. Ready to merge. |
| `Yellow` | Checks pending or awaiting review. Not yet actionable. |
| `Red` | Checks failed, conflicts present, or changes requested. Blocked. |
| `None` | No open PRs in scope. |

### Aggregation Rule

**Red beats Yellow beats Green.** In any collection of statuses, the aggregate
is the worst individual status. This is implemented in
`AmpelStatus::for_repository`.

### Relevance to Fleet Remediation

- `RemediationCriteria.require_green_before_merge` references this type to
  define the gate condition.
- `CiVerificationResult.ampel_status` carries the computed status of a PR at
  verification time.
- The agent emits `AmpelStatus` in SSE progress events so the frontend traffic
  light updates live during a remediation run.

```rust
// Existing definition — do not duplicate.
// crates/ampel-core/src/models/ampel_status.rs
pub enum AmpelStatus { Green, Yellow, Red, None }
```

---

## 6. ConsolidationPlan

### Definition

The output of the dry-run phase. Computed before any destructive action is
taken and surfaced to users when `autonomy_level` is `DryRunOnly` or
`SuggestOnly`. Immutable once produced. Recorded in the agent session for
audit and A/B analytics.

### Attributes

| Attribute | Type | Description |
|---|---|---|
| `would_select` | `Vec<PrRef>` | The PRs that would be included in consolidation, in order of application. |
| `predicted_conflicts` | `Vec<ConflictPrediction>` | Files predicted to conflict, with the pair of PRs causing the conflict. |
| `lockfile_regen_commands` | `Vec<String>` | Shell commands the agent will run to regenerate lockfiles after merge (e.g., `cargo update -p foo`, `pnpm install --frozen-lockfile=false`). |
| `estimated_duration_secs` | `u32` | Rough estimate of wall-clock time for the full consolidation. Used to surface a warning if `AgentBudget.max_seconds` would be exceeded. |

### PrRef

A minimal cross-provider PR reference.

| Attribute | Type | Description |
|---|---|---|
| `provider` | `GitProvider` | `GitHub` | `GitLab` | `Bitbucket`. |
| `repo_full_name` | `String` | e.g., `"org/repo"`. |
| `number` | `u64` | PR number as reported by the provider. |
| `title` | `String` | PR title at plan time (snapshot). |
| `head_sha` | `String` | Head commit SHA at plan time. |

### ConflictPrediction

| Attribute | Type | Description |
|---|---|---|
| `file_path` | `String` | Repo-relative path of the conflicting file. |
| `pr_a` | `u64` | First PR number involved. |
| `pr_b` | `u64` | Second PR number involved. |
| `confidence` | `f32` | 0.0–1.0. Derived from diff-range overlap heuristic. Not a guarantee. |

### Validation Rules

- `would_select` must be non-empty; a plan with no PRs is meaningless and
  should not be constructed.
- `confidence` must be in `[0.0, 1.0]`.
- `estimated_duration_secs` must be > 0.
- `lockfile_regen_commands` elements must be non-empty strings (shell-injection
  risk: these are passed through `minijinja` rendering and then executed in the
  sandbox container; the PolicyResolver must validate commands against an
  allowlist before the plan is persisted).

### Rust Sketch

```rust
use serde::{Deserialize, Serialize};
use crate::models::GitProvider;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsolidationPlan {
    pub would_select: Vec<PrRef>,
    pub predicted_conflicts: Vec<ConflictPrediction>,
    pub lockfile_regen_commands: Vec<String>,
    pub estimated_duration_secs: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrRef {
    pub provider: GitProvider,
    pub repo_full_name: String,
    pub number: u64,
    pub title: String,
    pub head_sha: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictPrediction {
    pub file_path: String,
    pub pr_a: u64,
    pub pr_b: u64,
    pub confidence: f32,
}

impl ConsolidationPlan {
    pub fn new(
        would_select: Vec<PrRef>,
        predicted_conflicts: Vec<ConflictPrediction>,
        lockfile_regen_commands: Vec<String>,
        estimated_duration_secs: u32,
    ) -> Result<Self, ConsolidationPlanError> {
        if would_select.is_empty() {
            return Err(ConsolidationPlanError::EmptySelection);
        }
        if estimated_duration_secs == 0 {
            return Err(ConsolidationPlanError::ZeroDuration);
        }
        for pred in &predicted_conflicts {
            if !(0.0..=1.0).contains(&pred.confidence) {
                return Err(ConsolidationPlanError::InvalidConfidence(pred.confidence));
            }
        }
        Ok(Self {
            would_select,
            predicted_conflicts,
            lockfile_regen_commands,
            estimated_duration_secs,
        })
    }

    /// Whether the plan is conflict-free.
    pub fn is_clean(&self) -> bool {
        self.predicted_conflicts.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConsolidationPlanError {
    #[error("would_select must not be empty")]
    EmptySelection,
    #[error("estimated_duration_secs must be > 0")]
    ZeroDuration,
    #[error("confidence {0} is out of range [0.0, 1.0]")]
    InvalidConfidence(f32),
}
```

---

## 7. RemediationScope

### Definition

Identifies the organizational entity at which a policy, trigger, or session is
anchored. Used by `PolicyResolver` when traversing the hierarchy and by the
session audit log to record who authorized the run.

### Attributes

| Attribute | Type | Description |
|---|---|---|
| `scope_type` | `ScopeType` | The tier in the hierarchy this scope refers to. |
| `scope_id` | `Uuid` | The UUID of the specific entity (user, org, team, or repository row). |

### ScopeType Variants

| Variant | Resolves from table |
|---|---|
| `User` | `users.id` |
| `Org` | `organizations.id` |
| `Team` | `teams.id` |
| `Repository` | `repositories.id` |

### Resolution Order

PolicyResolver walks Repository → Team → Org → User. A policy set at a
narrower scope (Repository) overrides a broader scope (Org). An `air_gapped`
flag set at the Org level cannot be overridden by a Repository-level policy —
it is a hard ceiling, not a default.

### Validation Rules

- `scope_id` must be a non-nil UUID (`Uuid::nil()` is rejected).
- The combination `(scope_type, scope_id)` must refer to an existing row;
  validation is performed at policy resolution time, not at value object
  construction time (construction is cheap).

### Rust Sketch

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RemediationScope {
    pub scope_type: ScopeType,
    pub scope_id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeType {
    User,
    Org,
    Team,
    Repository,
}

impl RemediationScope {
    pub fn new(scope_type: ScopeType, scope_id: Uuid) -> Result<Self, ScopeError> {
        if scope_id.is_nil() {
            return Err(ScopeError::NilId);
        }
        Ok(Self { scope_type, scope_id })
    }

    /// Ordinal rank for hierarchy resolution (lower = narrower = higher priority).
    pub fn resolution_rank(&self) -> u8 {
        match self.scope_type {
            ScopeType::Repository => 0,
            ScopeType::Team       => 1,
            ScopeType::Org        => 2,
            ScopeType::User       => 3,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    #[error("scope_id must not be the nil UUID")]
    NilId,
}
```

---

## 8. EgressClass

### Definition

Classifies whether a model provider account is permitted to send data outside
the local network. Carried on `ModelProviderAccount` and enforced by
`PolicyResolver` when `RemediationCriteria.air_gapped == true`.

### Variants

| Variant | Meaning | Examples |
|---|---|---|
| `External` | Traffic leaves the local network (calls a third-party API). | Claude (Anthropic API), Gemini (Google AI API) |
| `LocalOnly` | All inference stays within the deployment boundary. | Ollama (self-hosted), ONNX (embedded WASM runtime) |

### Enforcement Rule

When `air_gapped == true` on the effective `RemediationCriteria`, the agent
session startup check must:

1. Enumerate all `ModelProviderAccount` records available for the session.
2. Filter to only those with `egress_class == EgressClass::LocalOnly`.
3. If no `LocalOnly` provider is available, abort with
   `RemediationError::NoLocalProviderAvailable`.

The enforcement lives in `PolicyResolver::validate_providers`, called before
any agent loop begins.

### Rust Sketch

```rust
use serde::{Deserialize, Serialize};

/// Network egress classification for a model provider.
///
/// Stored as a string column on `model_provider_accounts.egress_class`
/// with values `"external"` and `"local_only"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EgressClass {
    External,
    LocalOnly,
}

impl EgressClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::External  => "external",
            Self::LocalOnly => "local_only",
        }
    }

    pub fn is_permitted_when_air_gapped(self) -> bool {
        self == Self::LocalOnly
    }
}

impl std::fmt::Display for EgressClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for EgressClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "external"   => Ok(Self::External),
            "local_only" => Ok(Self::LocalOnly),
            other        => Err(format!("unknown EgressClass: {other}")),
        }
    }
}
```

---

## 9. FailureClass

### Definition

Classifies the type of CI or build failure observed on a PR. Used by the
playbook engine to select the appropriate remediation task (e.g., a
`BuildError` routes to the `fix-build` playbook task; a `FlakyTest` routes to
the `retry-ci` task). Also used by the router model to pick the right inference
strategy (lightweight ONNX classifier for `FlakyTest` vs. full Sonnet for
`TypeError`).

### Variants

| Variant | Description | Typical Signal |
|---|---|---|
| `BuildError` | Compilation or link failure. | `cargo build` exit ≠ 0, `rustc` error output. |
| `TestFailure` | One or more tests failed (not flaky). | `cargo test` / `nextest` non-zero exit with deterministic failure. |
| `TypeError` | Type system error (Rust type errors, TypeScript tsc errors). | `error[E...]` in rustc output; `tsc --noEmit` errors. |
| `Lint` | Linter or formatter violation. | `clippy::deny`, ESLint error, `rustfmt --check` diff. |
| `LockfileConflict` | `Cargo.lock` or `pnpm-lock.yaml` is out of sync or has merge conflicts. | Lock file conflict markers or `--locked` failure. |
| `FlakyTest` | Test failure that is non-deterministic (passes on retry). | Detected via retry policy in nextest or historical pass rate. |
| `MissingDependency` | A required crate or package is absent. | `error[E0432]`, `Module not found`. |
| `Unknown` | Could not be classified. | Fallback when no pattern matches. |

### Usage in Playbook Task Selection

The playbook engine maps `FailureClass` to task tags:

| FailureClass | Playbook task tag |
|---|---|
| `BuildError` | `fix:build` |
| `TestFailure` | `fix:tests` |
| `TypeError` | `fix:types` |
| `Lint` | `fix:lint` |
| `LockfileConflict` | `fix:lockfile` |
| `FlakyTest` | `retry:ci` |
| `MissingDependency` | `fix:deps` |
| `Unknown` | `escalate:human` |

### Rust Sketch

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    BuildError,
    TestFailure,
    TypeError,
    Lint,
    LockfileConflict,
    FlakyTest,
    MissingDependency,
    Unknown,
}

impl FailureClass {
    /// Returns the playbook task tag for this failure class.
    pub fn playbook_tag(self) -> &'static str {
        match self {
            Self::BuildError        => "fix:build",
            Self::TestFailure       => "fix:tests",
            Self::TypeError         => "fix:types",
            Self::Lint              => "fix:lint",
            Self::LockfileConflict  => "fix:lockfile",
            Self::FlakyTest         => "retry:ci",
            Self::MissingDependency => "fix:deps",
            Self::Unknown           => "escalate:human",
        }
    }

    /// Whether this class is amenable to automated fixing (as opposed to
    /// requiring human escalation or a simple retry).
    pub fn is_auto_fixable(self) -> bool {
        matches!(
            self,
            Self::Lint | Self::LockfileConflict | Self::FlakyTest
        )
    }

    /// Classify from raw CI log output. Applies patterns in priority order.
    /// Returns `Unknown` if no pattern matches.
    pub fn classify_from_log(log: &str) -> Self {
        if log.contains("error[E") && (log.contains("rustc") || log.contains("tsc")) {
            return Self::TypeError;
        }
        if log.contains("error") && log.contains("linking") {
            return Self::BuildError;
        }
        if log.contains("Cargo.lock") || log.contains("pnpm-lock") {
            return Self::LockfileConflict;
        }
        // ... additional pattern matching ...
        Self::Unknown
    }
}
```

---

## 10. PlaybookRef

### Definition

A stable reference to the specific playbook version that was loaded for a
remediation session. Recorded in the agent session audit log to enable A/B
analytics (comparing outcomes across playbook versions and sources) and
reproducibility (re-running a session with the same playbook).

### Attributes

| Attribute | Type | Description |
|---|---|---|
| `playbook_id` | `String` | Logical identifier for the playbook (e.g., `"consolidate-prs"`, `"fix-and-consolidate"`). Stable across versions. |
| `version` | `u32` | Monotonically increasing version number. `1` is the initial version. |
| `source` | `PlaybookSource` | Where this playbook was loaded from. |

### PlaybookSource Variants

| Variant | Description |
|---|---|
| `Builtin` | Embedded in the binary via `rust-embed`. Immutable at runtime. |
| `Db` | Loaded from the `playbooks` database table. Org/team override of a builtin. |
| `RepoLocal` | Loaded from `.ampel/remediation.yaml` in the repository. Highest-priority override. |

### Resolution Priority

`RepoLocal` > `Db` > `Builtin`. The `PolicyResolver` records which source was
used so that session analytics can attribute outcome differences to playbook
source, not just playbook content.

### Validation Rules

- `playbook_id` must match `^[a-z0-9][a-z0-9-]{0,63}$` (lowercase, hyphens,
  no leading hyphen, max 64 chars). This is the same convention used for
  Docker image names and ensures safe filesystem embedding.
- `version` must be ≥ 1.
- `playbook_id` + `version` + `source` together form the natural key for
  analytics queries. The triple should be unique within a session.

### Rust Sketch

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaybookRef {
    pub playbook_id: String,
    pub version: u32,
    pub source: PlaybookSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybookSource {
    Builtin,
    Db,
    RepoLocal,
}

impl PlaybookRef {
    /// Validation regex: lowercase alphanumeric + hyphens, 1–64 chars,
    /// no leading hyphen.
    const ID_PATTERN: &'static str = r"^[a-z0-9][a-z0-9-]{0,63}$";

    pub fn new(
        playbook_id: impl Into<String>,
        version: u32,
        source: PlaybookSource,
    ) -> Result<Self, PlaybookRefError> {
        let playbook_id = playbook_id.into();

        // Validate ID format.
        let re = regex::Regex::new(Self::ID_PATTERN).expect("static pattern");
        if !re.is_match(&playbook_id) {
            return Err(PlaybookRefError::InvalidId(playbook_id));
        }

        if version == 0 {
            return Err(PlaybookRefError::ZeroVersion);
        }

        Ok(Self { playbook_id, version, source })
    }

    /// Returns a stable string key suitable for use as a metrics label.
    pub fn metrics_key(&self) -> String {
        format!("{}/{}/{}", self.source.as_str(), self.playbook_id, self.version)
    }
}

impl PlaybookSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Builtin   => "builtin",
            Self::Db        => "db",
            Self::RepoLocal => "repo_local",
        }
    }

    /// Priority for resolution (lower = higher priority).
    pub fn priority(self) -> u8 {
        match self {
            Self::RepoLocal => 0,
            Self::Db        => 1,
            Self::Builtin   => 2,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlaybookRefError {
    #[error("playbook_id '{0}' does not match ^[a-z0-9][a-z0-9-]{{0,63}}$")]
    InvalidId(String),
    #[error("version must be >= 1")]
    ZeroVersion,
}
```

---

## Cross-Cutting Notes for Implementors

### Immutability convention

All value objects in this bounded context use only owned data (no `&` fields)
and derive `Clone`. Construction goes through a `new()` or `try_new()` method
that enforces invariants. There are no setters.

### Serialization

All value objects derive `serde::Serialize` + `serde::Deserialize`. Enums use
`#[serde(tag = "...", rename_all = "snake_case")]` for tagged JSON to match
the existing project convention (see `AmpelStatus`).

### Error types

Each value object that can fail construction defines its own `Error` enum (via
`thiserror`) rather than using a shared error type. This keeps error messages
precise and avoids coupling unrelated types.

### Crate placement

| Value object | Module |
|---|---|
| `RemediationCriteria`, `AgentBudget`, `RemediationScope`, `EgressClass`, `PlaybookRef` | `crates/ampel-core/src/remediation/policy.rs` |
| `MergeDisposition`, `ConsolidationPlan`, `PrRef`, `ConflictPrediction` | `crates/ampel-core/src/remediation/consolidation.rs` |
| `CiVerificationResult`, `NormalizedCiCheck`, `CheckStatus` | `crates/ampel-core/src/remediation/verification.rs` |
| `FailureClass` | `crates/ampel-core/src/remediation/classification.rs` |
| `AmpelStatus` | `crates/ampel-core/src/models/ampel_status.rs` (existing) |

### Dependency on `rust_decimal`

`AgentBudget.max_cost_usd` uses `rust_decimal::Decimal`. Add to
`crates/ampel-core/Cargo.toml`:

```toml
rust_decimal = { version = "1", features = ["serde-float"] }
```

### Dependency on `regex` in PlaybookRef

`PlaybookRef::new` compiles a regex at call time for the ID pattern. In
production code, wrap the `Regex` in a `once_cell::sync::Lazy` to compile it
once:

```rust
use once_cell::sync::Lazy;
use regex::Regex;

static PLAYBOOK_ID_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z0-9][a-z0-9-]{0,63}$").unwrap());
```
