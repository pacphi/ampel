Domain services reference document written to `/Users/cphillipson/Development/active/ai/ampel/docs/architecture/ddd/domain-services.md`.

The document covers all five domain services in the Fleet Remediation bounded context:

1. **PolicyResolver** — four-level hierarchy walk (repo → team → org → default) with org air_gapped ceiling enforcement. Trait is `#[async_trait]` for `Arc<dyn PolicyResolver>` storage. `default_off()` ensures missing policy = feature off, never permissive.

2. **ConsolidationStrategy** — Podman/Docker sandbox lifecycle (spawn, execute, unconditional destroy), octopus merge via subprocess git sorted oldest-first, lockfile regen command map for six ecosystems (npm/pnpm/cargo/go/poetry/bundler), deterministic branch naming `ampel/remediation/<run_id>`. `TmpfsEnvGuard` on drop ensures `secure_erase` is always called.

3. **VerificationService** — required-check fetch from branch protection, missing required context treated as red (not pending), draft/changes-requested guards, `is_safe_to_merge` predicate. Called twice: once on entering `verifying`, once as TOCTOU guard immediately before the merge API call in `merging`. Prometheus counter emitted on every call.

4. **RemediationAgentHarness** — classify → select task → assemble context → iterate (infer/apply or run_agent) → push → VerificationService loop. `AgentBudget` tracks remaining tokens and wall-clock. Harness never calls the merge API; `BudgetExhausted` routes to `handoff_human`, not failure.

5. **FailureClassifier** — two-level cascade: Level 1 zero-cost heuristic (pure `classify_heuristic` function, priority-ordered regex patterns), Level 2 in-process ONNX (0.7 confidence threshold), fallback to `FailureClass::Unknown`. No network egress at either level. Pattern table and `remediation_agent_session` observability columns documented.

A cross-service dependency map and crate placement table are included at the end.

```
RemediationSweepJob (Apalis cron — outer loop)
  └─ per qualifying repo → RemediationRunJob (inner loop)
        │
        ├─ PolicyResolver        resolve effective policy for this repo
        ├─ ConsolidationStrategy execute octopus merge inside Podman sandbox
        ├─ VerificationService   check CI green + branch protection
        ├─ FailureClassifier     classify why CI is red (Phase 4)
        └─ RemediationAgentHarness  drive agentic fix loop (Phase 4)
```

---

## 1. PolicyResolver

### Purpose

Walk the four-level scope hierarchy (repository → team → organisation → user default) to
produce a single `EffectivePolicy` for a repository. Also applies the org-level
`air_gapped` ceiling so that no policy can route to an external inference provider when
the org forbids it (ADR-014).

### Inputs

| Parameter | Type | Notes |
|---|---|---|
| `repository_id` | `Uuid` | The repo being evaluated |
| `db` | `&DatabaseConnection` | SeaORM connection pool handle |

### Outputs

`EffectivePolicy` — the merged, ceiling-applied policy view:

```rust
#[derive(Debug, Clone)]
pub struct EffectivePolicy {
    /// The raw DB row that won at the deepest matching scope.
    pub source: RemediationPolicy,
    /// Resolved model strategy after scope merging.
    pub model_strategy: ModelStrategy,
    /// True when the org ceiling forces air-gapped mode,
    /// regardless of what the policy row says.
    pub air_gapped: bool,
    /// The scope level that determined air_gapped.
    pub air_gapped_source: AirGappedSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AirGappedSource {
    Org,
    Policy,
    Default, // neither level set it; false
}
```

### Dependencies

| Dependency | Role |
|---|---|
| `RemediationPolicyRepository` | Query `remediation_policy` by scope and scope_id |
| `OrgRepository` | Fetch `org_settings.air_gapped` for the org owning the repo |

### Key Algorithm

```
resolve(repository_id):
  1. query remediation_policy WHERE scope = Repository AND scope_id = repository_id
  2. if found → policy = row; goto step 6
  3. query team for the repo → query remediation_policy WHERE scope = Team AND scope_id = team_id
  4. if found → policy = row; goto step 6
  5. query org for the repo → query remediation_policy WHERE scope = Org AND scope_id = org_id
  6. if found → policy = row
  7. else → policy = RemediationPolicy::default_off()
  8. fetch org_settings for the repo's org
  9. if org_settings.air_gapped AND NOT policy.air_gapped:
       policy.air_gapped = true            // ceiling is authoritative
       air_gapped_source = AirGappedSource::Org
  10. resolve model_strategy from policy + org model_provider_accounts
  11. return EffectivePolicy { source: policy, model_strategy, air_gapped, air_gapped_source }
```

`RemediationPolicy::default_off()` returns a policy with `enabled = false`, so a missing
policy is equivalent to the feature being off rather than defaulting to some permissive
behaviour.

### Rust Trait Sketch

```rust
use async_trait::async_trait;
use uuid::Uuid;
use sea_orm::DatabaseConnection;

/// Resolves the effective remediation policy for a single repository,
/// applying the org-level air_gapped ceiling.
///
/// Implement with `#[async_trait]` because instances are held as
/// `Arc<dyn PolicyResolver>` in `WorkerState`.
#[async_trait]
pub trait PolicyResolver: Send + Sync {
    async fn resolve(
        &self,
        repository_id: Uuid,
        db: &DatabaseConnection,
    ) -> Result<EffectivePolicy, PolicyError>;
}

/// Concrete implementation backed by SeaORM repositories.
pub struct DbPolicyResolver {
    pub policy_repo: Arc<dyn RemediationPolicyRepository>,
    pub org_repo:    Arc<dyn OrgRepository>,
}

#[async_trait]
impl PolicyResolver for DbPolicyResolver {
    async fn resolve(
        &self,
        repository_id: Uuid,
        db: &DatabaseConnection,
    ) -> Result<EffectivePolicy, PolicyError> {
        // walk hierarchy: repo → team → org → default
        let policy = self.find_policy(repository_id, db).await?;
        let org    = self.org_repo.find_for_repo(repository_id, db).await?;
        Ok(self.apply_ceiling(policy, org))
    }
}
```

### Invariants

- `EffectivePolicy.air_gapped` is `true` whenever `org_settings.air_gapped` is `true`,
  regardless of the policy row value.
- `default_off()` policy has `autonomy_level = None` and `enabled = false`; callers must
  check `enabled` before dispatching any work.
- The resolver never mutates DB state; it is purely a read-path query chain.

---

## 2. ConsolidationStrategy

### Purpose

Execute the mechanical merge pipeline inside a rootless Podman (or Docker) sandbox
(ADR-003). Clone the repository, merge the selected PR branches in age order via
subprocess `git` (ADR-005), regenerate lockfiles if needed, push the consolidated branch,
and create a draft PR via the `RemediationCapable` provider supertrait.

### Inputs

| Parameter | Type | Notes |
|---|---|---|
| `clone_url` | `String` | HTTPS URL for the repository |
| `pat` | `ScopedPat` | Short-lived, scoped credential; injected via tmpfs env-file |
| `prs` | `Vec<SelectedPr>` | PRs to merge; sorted oldest-first before use |
| `default_branch` | `String` | Merge base (e.g. `main`) |
| `run_id` | `Uuid` | Used for deterministic branch and container naming |

### Outputs

```rust
#[derive(Debug, Clone)]
pub struct ConsolidationResult {
    /// Name of the consolidated branch pushed to the provider.
    pub branch: String,
    /// Draft PR created by the provider for the consolidated branch.
    pub pr: CreatedPr,
    /// Disposition of each source PR in this consolidation.
    pub per_pr: Vec<(PrId, MergeDisposition)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergeDisposition {
    /// Merged cleanly into the consolidated branch.
    Merged,
    /// Conflict detected; classified and recorded.
    Conflicted { class: ConflictClass },
    /// Excluded by criteria before merge attempt.
    Excluded,
}
```

### Dependencies

| Dependency | Role |
|---|---|
| `ContainerRuntime` | `Podman` or `Docker` (resolved at worker startup) |
| `RemediationCapable` provider | `create_branch`, `create_pr` after push |
| `EncryptionService` | Decrypt PAT before injecting into sandbox |
| Subprocess `git` | Octopus merge, push (git2-rs does not support octopus) |
| `LockfileRegenerator` | Per-ecosystem regen command map |

### Key Algorithm

```
consolidate(clone_url, pat, prs, default_branch, run_id):
  branch_name = format!("ampel/remediation/{run_id}")
  env_path    = write_tmpfs_env(pat, run_id)   // mode 0o600, tmpfs only

  container = spawn_container(
      name    = format!("ampel-run-{run_id}"),
      network = "ampel-egress",                // egress-ACL network
      env_file = env_path,
      args     = ["ampel-consolidate", "--run-id", run_id, "--branch", branch_name],
  )

  // Inside the container (ampel-consolidate subprocess):
  git clone --depth=1 <clone_url> /workspace
  git checkout -b <branch_name>
  sorted_prs = sort_by_age(prs, oldest_first)

  per_pr_results = []
  for pr in sorted_prs:
      result = git merge --no-ff origin/<pr.head_branch>
      if result.conflict:
          class = classify_conflict(result.conflict_files)
          per_pr_results.push((pr.id, MergeDisposition::Conflicted { class }))
          if class == MergeConflictClass::Unresolvable:
              abort // cannot proceed; caller transitions to handoff_human
          // Resolvable (e.g. lockfile): attempt regen
          regen_cmd = LOCKFILE_REGEN[detect_ecosystem(/workspace)]
          run(regen_cmd)
          git add <conflict_files>
          git commit --no-edit
          per_pr_results.push((pr.id, MergeDisposition::Merged))
      else:
          per_pr_results.push((pr.id, MergeDisposition::Merged))

  git push origin <branch_name>
  secure_erase(env_path)
  container.wait() // unconditional cleanup via --rm

  created_pr = provider.create_pr(branch=branch_name, base=default_branch, draft=true)
  return ConsolidationResult { branch: branch_name, pr: created_pr, per_pr: per_pr_results }
```

### Lockfile Regeneration Commands

| Ecosystem | Detection | Regen command |
|---|---|---|
| Node/npm | `package-lock.json` present | `npm install --package-lock-only` |
| Node/pnpm | `pnpm-lock.yaml` present | `pnpm install --frozen-lockfile=false` |
| Rust/Cargo | `Cargo.lock` present | `cargo generate-lockfile` |
| Go | `go.sum` present | `go mod tidy` |
| Python/Poetry | `poetry.lock` present | `poetry lock --no-update` |
| Ruby/Bundler | `Gemfile.lock` present | `bundle lock --update` |

The regen command is attempted only when the conflict file set includes a known lockfile.
After regen, `git add <lockfile> && git commit --no-edit --amend` folds the lockfile into
the merge commit.

### Branch Naming

Deterministic: `ampel/remediation/<run_id>` (UUID). This makes duplicate-run detection
trivial: a branch with this name already existing means a prior run pushed successfully.

### Rust Trait Sketch

```rust
#[async_trait]
pub trait ConsolidationStrategy: Send + Sync {
    /// Execute the full consolidation pipeline for the given run.
    /// Implementors must destroy the sandbox container unconditionally
    /// (success or failure) before returning.
    async fn consolidate(
        &self,
        params: ConsolidationParams,
    ) -> Result<ConsolidationResult, ConsolidationError>;
}

pub struct SandboxConsolidationStrategy {
    pub runtime:          ContainerRuntime,  // Podman | Docker
    pub provider:         Arc<dyn RemediationCapable>,
    pub encryption:       Arc<EncryptionService>,
    pub sandbox_image:    String,            // pinned OCI digest
}

#[async_trait]
impl ConsolidationStrategy for SandboxConsolidationStrategy {
    async fn consolidate(
        &self,
        params: ConsolidationParams,
    ) -> Result<ConsolidationResult, ConsolidationError> {
        let pat      = self.encryption.decrypt(&params.encrypted_pat)?;
        let env_path = write_tmpfs_env(&pat, params.run_id)?;
        let _guard   = TmpfsEnvGuard::new(env_path.clone()); // secure_erase on drop

        let exit = self.runtime
            .run(&self.sandbox_image, &env_path, &params)
            .await?;

        if exit.status != 0 {
            return Err(ConsolidationError::SandboxFailed { exit });
        }

        let result: ConsolidationResult =
            serde_json::from_str(&exit.stdout)?;
        Ok(result)
    }
}
```

---

## 3. VerificationService

### Purpose

Determine whether a consolidated ref is green, satisfies all required branch-protection
checks, is mergeable, and is not in a draft or changes-requested state. Called twice per
run (ADR-010):

1. On entering the `verifying` state — to decide whether to proceed or escalate to the
   agentic tier (Phase 4) or `handoff_human`.
2. Immediately before the merge API call in the `merging` state — the TOCTOU guard.

The service never makes a merge decision. It returns a structured verdict and lets the
state machine act on it.

### Inputs

| Parameter | Type | Notes |
|---|---|---|
| `repo` | `&Repository` | The repository being verified |
| `consolidated_ref_sha` | `&str` | The pushed branch HEAD SHA |
| `credentials` | `&ProviderCredentials` | To authenticate provider API calls |

### Outputs

```rust
#[derive(Debug, Clone)]
pub struct NormalizedCheck {
    /// Status check context name (e.g. "ci/tests", "security/snyk").
    pub context: String,
    /// Whether branch protection marks this check as required.
    pub required: bool,
    /// Normalized traffic-light status.
    pub status: AmpelStatus,
}

#[derive(Debug, Clone)]
pub struct CiVerificationResult {
    pub ref_sha:             String,
    pub checks:              Vec<NormalizedCheck>,
    /// All required checks are present and green.
    pub all_required_green:  bool,
    /// Provider reports the branch as mergeable (no conflicts, not draft).
    pub mergeable:           bool,
    /// Aggregate AmpelStatus (red beats yellow beats green).
    pub ampel_status:        AmpelStatus,
}
```

### Dependencies

| Dependency | Role |
|---|---|
| `GitProvider::get_required_checks` | Fetch required-check contexts from branch protection |
| `GitProvider::get_commit_status` | Fetch check runs for the consolidated ref SHA |
| `RemediationCapable::get_pr_metadata` | Draft state and review decisions |

### Key Algorithm

```
verify(repo, consolidated_ref_sha, credentials):
  required_contexts = provider.get_required_checks(repo, repo.default_branch)
  all_checks        = provider.get_commit_status(repo, consolidated_ref_sha)
  pr_meta           = provider.get_pr_metadata(repo, consolidated_ref_sha)

  // Normalize to NormalizedCheck, marking required
  normalized = []
  for check in all_checks:
      required = check.context IN required_contexts
      normalized.push(NormalizedCheck {
          context: check.context,
          required,
          status: to_ampel_status(check.state),
      })

  // Required context with no result = red (not "pending")
  for ctx in required_contexts:
      if ctx NOT IN all_checks.map(|c| c.context):
          normalized.push(NormalizedCheck {
              context: ctx,
              required: true,
              status: AmpelStatus::Red,
          })

  all_required_green = normalized
      .iter()
      .filter(|c| c.required)
      .all(|c| c.status == AmpelStatus::Green)

  // draft or changes-requested → not mergeable
  mergeable = pr_meta.mergeable
      && !pr_meta.draft
      && !pr_meta.has_changes_requested()

  ampel_status = aggregate(normalized.iter().map(|c| c.status))

  return CiVerificationResult {
      ref_sha: consolidated_ref_sha,
      checks: normalized,
      all_required_green,
      mergeable,
      ampel_status,
  }

safe_to_merge(result):
  result.ampel_status == Green
      AND result.all_required_green
      AND result.mergeable
```

### Re-verification Posture (TOCTOU Guard)

The `merging` state handler must call `verify` and evaluate `safe_to_merge` before
issuing the merge API call. On any non-green result, the run transitions to
`handoff_human` with the `CiVerificationResult` attached. This collapses the check-time
and use-time windows to within a single network round-trip.

```rust
// State machine — merging transition
let pre_merge = verification_service
    .verify(&repo, &consolidated_ref_sha, &creds)
    .await?;

if !is_safe_to_merge(&pre_merge) {
    return transition_to_handoff_human(run, pre_merge).await;
}

provider.merge_pull_request(&consolidated_pr).await?;
```

### Rust Trait Sketch

```rust
#[async_trait]
pub trait VerificationService: Send + Sync {
    /// Query provider CI status for the given SHA and return a
    /// normalized verdict. Does not modify any state.
    async fn verify(
        &self,
        repo: &Repository,
        consolidated_ref_sha: &str,
        credentials: &ProviderCredentials,
    ) -> Result<CiVerificationResult, VerificationError>;
}

pub fn is_safe_to_merge(result: &CiVerificationResult) -> bool {
    result.ampel_status == AmpelStatus::Green
        && result.all_required_green
        && result.mergeable
}

pub struct ProviderVerificationService {
    pub provider: Arc<dyn RemediationCapable>,
}

#[async_trait]
impl VerificationService for ProviderVerificationService {
    async fn verify(
        &self,
        repo: &Repository,
        consolidated_ref_sha: &str,
        credentials: &ProviderCredentials,
    ) -> Result<CiVerificationResult, VerificationError> {
        let required  = self.provider
            .get_required_checks(repo, &repo.default_branch)
            .await?;
        let checks    = self.provider
            .get_commit_status(repo, consolidated_ref_sha)
            .await?;
        let pr_meta   = self.provider
            .get_pr_metadata(repo, consolidated_ref_sha)
            .await?;
        Ok(build_result(consolidated_ref_sha, required, checks, pr_meta))
    }
}
```

### Observability

```
ampel_remediation_pre_merge_verification_total{outcome="green|yellow|red", reason="..."}
```

Emitted on every call to `verify` from the `merging` state. The `verifying` state call
also emits but with a different label (`call_site="verifying"`).

---

## 4. RemediationAgentHarness

### Purpose

Drive the agentic remediation tier (Phase 4). When `VerificationService` returns a red
verdict after mechanical consolidation, the harness classifies the failure, selects the
right Playbook task, assembles context, and runs an iterate-apply-push loop until CI turns
green or the agent budget is exhausted.

The harness **never certifies green**. Every iteration ends with a call to
`VerificationService`. The harness owns the loop control; the provider owns inference.

### Inputs

| Parameter | Type | Notes |
|---|---|---|
| `ci_result` | `CiVerificationResult` | Red result that triggered Phase 4 |
| `run_ctx` | `RemediationRunContext` | Run ID, repo, branch, source PRs |
| `playbook` | `ResolvedPlaybook` | Fully rendered (minijinja substituted) playbook |
| `provider` | `Arc<dyn ModelProvider>` | Selected by PolicyResolver + model router |
| `worktree` | `WorktreeHandle` | Handle to the in-container filesystem |
| `budget` | `AgentBudget` | Max iterations, max tokens, max wall-clock seconds |

### Outputs

```rust
#[derive(Debug, Clone)]
pub struct AgentOutcome {
    /// Whether CI was green at the end of the last iteration.
    pub passed: bool,
    /// Number of inference/agent iterations consumed.
    pub iterations: u32,
    /// Estimated inference cost in USD (from provider token counts).
    pub cost_usd: Decimal,
    /// Optional reference to the stored transcript (for audit/replay).
    pub transcript_ref: Option<String>,
    /// Terminal state reason.
    pub terminal_reason: TerminalReason,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TerminalReason {
    CiGreen,
    BudgetExhausted,
    ProviderError,
    Aborted,
}
```

### Dependencies

| Dependency | Role |
|---|---|
| `FailureClassifier` | Classify the failing CI log before selecting Playbook task |
| `ModelProvider` | `infer()` (inference-kind) or `run_agent()` (agent-kind) |
| `VerificationService` | Re-verify after each push; determines loop exit |
| `PlaybookTaskSelector` | Match `FailureClass` to a Playbook task template |
| `ContextAssembler` | Build the context bundle (diff, CI logs, PR description) |
| `TranscriptStore` | Persist iteration transcripts for audit |

### Key Algorithm

```
run(ci_result, run_ctx, playbook, provider, worktree, budget):
  class, confidence = FailureClassifier::classify(ci_result.failing_log)
  task              = PlaybookTaskSelector::select(playbook, class)
  context_bundle    = ContextAssembler::build(run_ctx, ci_result, task.context_spec)
  total_cost        = Decimal::ZERO
  iterations        = 0

  loop:
    if iterations >= budget.max_iterations:
      return AgentOutcome {
          passed: false, iterations, cost_usd: total_cost,
          terminal_reason: TerminalReason::BudgetExhausted, ...
      }
    if elapsed() >= budget.max_wall_seconds:
      return AgentOutcome { ..., terminal_reason: TerminalReason::BudgetExhausted }

    match provider.kind():
      InferenceKind:
        response = provider.infer(task.prompt, context_bundle, task.output_contract)
        edits    = parse_unified_diff_or_tool_calls(response)
        apply_edits(worktree, edits)
        total_cost += response.cost_usd
      AgentKind:
        outcome  = provider.run_agent(task.prompt, worktree.path, budget.remaining())
        total_cost += outcome.cost_usd

    worktree.commit_and_push("ampel(agent): iteration {iterations + 1}")
    iterations += 1

    verification = VerificationService::verify(run_ctx.repo, worktree.head_sha)
    if is_safe_to_merge(verification):
      store_transcript(run_ctx.run_id, iterations)
      return AgentOutcome {
          passed: true, iterations, cost_usd: total_cost,
          terminal_reason: TerminalReason::CiGreen,
          transcript_ref: Some(transcript_id),
      }

    // Update context bundle with new CI result for next iteration
    context_bundle = ContextAssembler::build(run_ctx, verification, task.context_spec)
```

### Budget Contract

`AgentBudget` is sourced from the resolved Playbook `loop_config` section and may be
tightened (never relaxed) by the org-level `model_provider_account.max_tokens_per_run`
ceiling.

```rust
#[derive(Debug, Clone)]
pub struct AgentBudget {
    pub max_iterations:   u32,
    pub max_tokens:       u64,
    pub max_wall_seconds: u64,
}

impl AgentBudget {
    pub fn remaining(&self, used_tokens: u64, elapsed_secs: u64) -> Self {
        Self {
            max_iterations:   self.max_iterations,
            max_tokens:       self.max_tokens.saturating_sub(used_tokens),
            max_wall_seconds: self.max_wall_seconds.saturating_sub(elapsed_secs),
        }
    }
}
```

### Rust Trait Sketch

```rust
#[async_trait]
pub trait RemediationAgentHarness: Send + Sync {
    /// Run the agentic remediation loop. Returns when CI is green,
    /// the budget is exhausted, or an unrecoverable error occurs.
    async fn run(
        &self,
        ci_result:  CiVerificationResult,
        run_ctx:    RemediationRunContext,
        playbook:   ResolvedPlaybook,
        provider:   Arc<dyn ModelProvider>,
        worktree:   WorktreeHandle,
        budget:     AgentBudget,
    ) -> Result<AgentOutcome, HarnessError>;
}

pub struct StandardRemediationAgentHarness {
    pub classifier:    Arc<dyn FailureClassifier>,
    pub verification:  Arc<dyn VerificationService>,
    pub transcripts:   Arc<dyn TranscriptStore>,
}
```

### Invariants

- The harness never calls the provider's merge API. Only the state machine's `merging`
  handler does, and only after a fresh `VerificationService::verify` call.
- On `BudgetExhausted`, the run transitions to `handoff_human`, not failure. The
  consolidated branch remains open for a human to review.
- `run_agent()` is the agent-kind path (e.g. Claude Code headless); it manages its own
  inner loop. The harness still re-verifies after it returns.

---

## 5. FailureClassifier

### Purpose

Classify a CI failure into a `FailureClass` before model routing and Playbook task
selection. Uses a two-level cascade (ADR-012): a zero-cost heuristic fast-path
(Level 1), followed by a local ONNX classifier (Level 2). No network egress. The
`unknown` class routes to the most capable configured model.

### Inputs

| Parameter | Type | Notes |
|---|---|---|
| `log` | `&str` | First 2 000 tokens of the failing job's combined stdout+stderr |
| `onnx` | `Option<&OnnxClassifier>` | In-process ONNX model; `None` if not configured |

### Outputs

```rust
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub class:      FailureClass,
    pub source:     ClassifierSource,
    /// 1.0 for heuristic matches; ONNX softmax probability otherwise.
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum ClassifierSource {
    Heuristic,
    Onnx,
    // Model-level classification (implicit in Unknown escalation path)
}
```

### Dependencies

| Dependency | Role |
|---|---|
| `OnnxClassifier` | In-process inference; loaded once, held in `Arc` on `AppState` |

### Key Algorithm

```
classify(log, onnx):
  // Level 1: zero-cost heuristic
  class = classify_heuristic(log)
  if class is Some:
    return ClassificationResult { class, source: Heuristic, confidence: 1.0 }

  // Level 2: ONNX classifier (local, no egress)
  if onnx is Some:
    tokens = truncate_to_tokens(log, 2000)
    dist   = onnx.classify(tokens)
    if dist.top_confidence() >= 0.7:
      return ClassificationResult {
          class:      dist.top_class(),
          source:     Onnx,
          confidence: dist.top_confidence(),
      }

  // Both levels below threshold → unknown
  return ClassificationResult {
      class:      FailureClass::Unknown,
      source:     Onnx,
      confidence: 0.0,
  }
```

### Level 1 Heuristic Patterns

| `FailureClass` | Pattern (case-sensitive unless noted) |
|---|---|
| `BuildError` | `error[E` OR `failed to compile` OR `cannot find` |
| `TypeError` | `error TS` OR `Type error` |
| `Lint` | `eslint` (ci) OR `clippy::deny` |
| `LockfileConflict` | `lock file` (ci) OR `lockfile` (ci) OR `Cargo.lock` OR `pnpm-lock` (ci) |
| `TestFailure` | `FAILED` AND (`test ` OR `tests/`) |
| `MissingDependency` | `no such crate` (ci) OR `cannot find crate` (ci) OR `module not found` (ci) OR `package not found` (ci) |

Patterns are evaluated in the order listed; first match wins. New patterns can be added
without touching the ONNX model.

### Unknown Escalation

`FailureClass::Unknown` bypasses Playbook task selection and routes to the most capable
configured model account. The raw log is included as additional context. The model is
expected to reason about the failure class implicitly and produce a fix in a single pass.

### Rust Trait Sketch

```rust
/// Synchronous classify function (Level 1 heuristic).
/// Pure function — no I/O, no allocations beyond the return value.
pub fn classify_heuristic(log: &str) -> Option<FailureClass> {
    let log_lower = log.to_ascii_lowercase();
    if log.contains("error[E") || log.contains("failed to compile") || log.contains("cannot find") {
        return Some(FailureClass::BuildError);
    }
    if log.contains("error TS") || log.contains("Type error") {
        return Some(FailureClass::TypeError);
    }
    if log_lower.contains("eslint") || log.contains("clippy::deny") {
        return Some(FailureClass::Lint);
    }
    if log_lower.contains("lock file") || log_lower.contains("lockfile")
        || log.contains("Cargo.lock") || log_lower.contains("pnpm-lock") {
        return Some(FailureClass::LockfileConflict);
    }
    if log.contains("FAILED") && (log.contains("test ") || log.contains("tests/")) {
        return Some(FailureClass::TestFailure);
    }
    if log_lower.contains("no such crate") || log_lower.contains("cannot find crate")
        || log_lower.contains("module not found") || log_lower.contains("package not found") {
        return Some(FailureClass::MissingDependency);
    }
    None
}

/// Async combined classifier (Level 1 + Level 2).
/// Stored as `Arc<dyn FailureClassifier>` on `WorkerState`;
/// annotated with `#[async_trait]` for dyn compatibility.
#[async_trait]
pub trait FailureClassifier: Send + Sync {
    async fn classify(
        &self,
        log: &str,
    ) -> ClassificationResult;
}

pub struct CascadeFailureClassifier {
    pub onnx: Option<Arc<OnnxClassifier>>,
}

#[async_trait]
impl FailureClassifier for CascadeFailureClassifier {
    async fn classify(&self, log: &str) -> ClassificationResult {
        if let Some(class) = classify_heuristic(log) {
            return ClassificationResult {
                class,
                source: ClassifierSource::Heuristic,
                confidence: 1.0,
            };
        }
        if let Some(ref onnx) = self.onnx {
            let tokens = truncate_to_tokens(log, 2_000);
            if let Ok(dist) = onnx.classify(&tokens).await {
                if dist.top_confidence() >= 0.7 {
                    return ClassificationResult {
                        class:      dist.top_class(),
                        source:     ClassifierSource::Onnx,
                        confidence: dist.top_confidence(),
                    };
                }
            }
        }
        ClassificationResult {
            class:      FailureClass::Unknown,
            source:     ClassifierSource::Onnx,
            confidence: 0.0,
        }
    }
}
```

### `remediation_agent_session` Fields

The classifier result is persisted on every session row for observability and reflexion
learning:

| Column | Type | Values |
|---|---|---|
| `failure_class` | `TEXT` | Enum variant name (snake_case) |
| `classifier_source` | `TEXT` | `heuristic` / `onnx` |
| `classifier_confidence` | `FLOAT` | 0.0 – 1.0 |

---

## Cross-Service Dependency Map

```
RemediationRunJob
  │
  ├─ PolicyResolver ──────────────────► OrgRepository
  │       │                             RemediationPolicyRepository
  │       └─ EffectivePolicy
  │
  ├─ ConsolidationStrategy ───────────► ContainerRuntime (Podman/Docker)
  │       │                             EncryptionService
  │       │                             RemediationCapable (provider)
  │       └─ ConsolidationResult
  │
  ├─ VerificationService (first call) ► RemediationCapable (provider)
  │       └─ CiVerificationResult (green → merging; red → Phase 4)
  │
  ├─ [Phase 4] FailureClassifier ────► OnnxClassifier (optional)
  │       └─ ClassificationResult
  │
  ├─ [Phase 4] RemediationAgentHarness
  │       │    ├─ FailureClassifier
  │       │    ├─ ModelProvider
  │       │    ├─ VerificationService (each iteration)
  │       │    └─ TranscriptStore
  │       └─ AgentOutcome
  │
  └─ VerificationService (TOCTOU guard) ► RemediationCapable (provider)
          └─ CiVerificationResult (green → merge; else → handoff_human)
```

---

## Crate Placement

| Service | Crate | Notes |
|---|---|---|
| `PolicyResolver` | `ampel-core` | Pure domain logic; no Axum/Apalis dependency |
| `ConsolidationStrategy` | `ampel-worker` | Requires Podman/Docker subprocess; worker-only |
| `VerificationService` | `ampel-core` | Shared by both API handlers and worker jobs |
| `RemediationAgentHarness` | `ampel-worker` | Apalis job context; worker-only |
| `FailureClassifier` | `ampel-worker` | ONNX runtime dependency; worker-only |
| `FailureClass`, `CiVerificationResult`, `AgentOutcome` | `ampel-core` | Domain types shared across crates |
| `ConsolidationResult`, `AgentBudget` | `ampel-core` | Domain types shared across crates |

`ampel-api` consumes `PolicyResolver` and `VerificationService` via `AppState`
(preview endpoint, dry-run). `ampel-worker` consumes all five services.
