# Fleet PR Remediation Loops

> **Technical Design Document** — Autonomous triage, consolidation, and remediation of open pull requests across every repository under Ampel management, regardless of provider.

## Executive Summary

Ampel today is a **read-and-merge** control plane: it polls repositories across GitHub, GitLab, and Bitbucket, renders PR health as traffic lights, and performs operator-initiated **bulk merges**. This document proposes the next step — turning Ampel into a **read-decide-act** control plane that watches the whole fleet and, on a schedule, *coalesces and remediates* pile-ups of open PRs without a human driving each step.

The trigger condition is simple and matches the request: **when a single repository has more than three open pull requests**, Ampel consolidates the qualifying PRs into one new PR, performs the updates needed to make it pass (resolve conflicts, regenerate lockfiles, optionally hand the branch to a coding agent), and — **only if everything is green** — merges the consolidated PR and closes the others with back-references. This runs for **all repositories under management, on every provider**, and is **off by default**, gated behind a configuration the operator toggles per scope.

This is, deliberately, an exercise in **loop engineering** — the mid-2026 shift from prompting agents by hand to designing the *system that prompts them*. Ampel becomes the **outer loop** (the fleet orchestrator that finds the work, sets the objective, and verifies the result); per-repo coding agents, when enabled, are **inner loops** dispatched into sandboxes. The central design bet — drawn straight from the research — is that **the verifier is the bottleneck, not the model**: the whole feature is only as safe as the green-check it merges on, so the verification layer gets the most engineering attention.

### Design principles

1. **Off by default, opt-in per scope, master kill-switch.** Nothing autonomous happens until an operator turns it on.
2. **Deterministic first, agentic second.** The common case (bot-driven dependency bumps) is handled mechanically with *no LLM*. Agentic remediation is a gated, opt-in escalation tier — not the foundation.
3. **The verifier is external and authoritative.** Ampel never merges on an agent's say-so; it merges on provider CI being green on the *consolidated* ref, re-checked immediately before merge.
4. **Every autonomous action is previewable, reversible-by-reference, and audited.** Dry-run before enable; close-with-comment, never silent; full run history.
5. **Reuse the grain of the existing system.** Apalis cron jobs, SeaORM entities, the `GitProvider` trait, the `AmpelStatus` traffic-light model, the bulk-merge pre-flight verification, and the hierarchical config pattern already in `auto_merge_rule`.

---

## Table of Contents

1. [Research Findings](#1-research-findings)
2. [How It Maps to Loop Engineering](#2-how-it-maps-to-loop-engineering)
3. [System Architecture](#3-system-architecture)
4. [The Remediation Loop (State Machine)](#4-the-remediation-loop-state-machine)
5. [Backend Component Design](#5-backend-component-design)
6. [Provider Trait Gap Analysis](#6-provider-trait-gap-analysis)
7. [Data Models and Schema](#7-data-models-and-schema)
8. [Configuration Model (the Toggles)](#8-configuration-model-the-toggles)
9. [API Design](#9-api-design)
10. [Frontend Architecture](#10-frontend-architecture)
11. [Safety, Guardrails, and Trust](#11-safety-guardrails-and-trust)
12. [Implementation Phases](#12-implementation-phases)
13. [Risk Assessment](#13-risk-assessment)
14. [Relationship to Existing Plans & Future Work](#14-relationship-to-existing-plans--future-work)

---

## 1. Research Findings

### 1.1 Loop engineering (the buzz)

"Loop engineering" was named in June 2026 (Addy Osmani's essay, building on Peter Steinberger and Anthropic's Boris Cherny — *"I don't prompt Claude anymore; I have loops that are running"*). It describes the shift from **prompt engineering** (micromanaging one turn) to **designing the system that prompts an agent autonomously** — finding the work, doing it, verifying it, and remembering what it did, with the human out of the per-step loop.

Key ideas worth importing into this design:

| Concept | Source framing | How we apply it |
|---|---|---|
| **The unit of value is the trajectory, not the response** | A bug on turn 1 is fine if the system detects and fixes it by turn 4 | The loop's success metric is "consolidated PR merged green," not any single intermediate action |
| **The verifier is the bottleneck** | A separate evaluator decides "done"; a loop is only as good as its check | CI green-check on the consolidated ref is the verifier; we invest disproportionately here |
| **Open vs. closed loops** | Closed = no human in the loop; open = human gate | `autonomy_level` config: `consolidate_only` (open) vs `auto_merge` (closed) |
| **Anatomy of a loop** (Osmani): automations, worktrees, skills, connectors, sub-agents, external state | The scaffolding around the model | Apalis cron = automation; sandbox = worktree; provider clients = connectors; coding agent = sub-agent; `remediation_run` table = external state |
| **Artifact / contract / log** | Durable files the loop reads and writes | Artifact = consolidated PR; contract = the policy/criteria; log = `remediation_run` records |
| **Not set-and-forget magic** | Autonomous loops carry real risk; "loopmaxxing" is a failure mode | Default-off, dry-run, shadow mode, budgets, kill-switch |

Lineage cited repeatedly: ReAct (2022) → AutoGPT (2023) → "Ralph loop" bash one-liners (2025) → productized `/goal` and `/loop` completion-condition loops in Claude Code and Codex (2026). The productized `/goal` mechanic — *a small fast model evaluates a completion condition after every turn until it holds* — is exactly the pattern for the **inner** (per-repo agentic) loop.

### 1.2 PR-consolidation prior art (the mechanic)

The "coalesce N PRs into one" operation is well-trodden, almost entirely in the GitHub-only, single-repo, workflow-file form:

- **`github/combine-prs`** (GitHub Action): on cron/dispatch, finds branches by prefix/regex/label, **octopus-merges them into one combined branch + PR**, optionally updates the combined branch from base, and **drops PRs that conflict**. Critical gotcha: the default `GITHUB_TOKEN` *won't re-trigger CI* on the combined branch — you need a PAT or App token. (Ampel already authenticates via PATs, so this is handled.)
- **Dependabot grouped updates** (and **cross-ecosystem grouping**, GA mid-2025): native grouping into fewer PRs; Dependabot **auto-closes superseded PRs** when a combined branch merges.
- **Renovate grouping** (`packageRules`, `group:`): branch-name + PR-title act as a cache key; closing a grouped PR has subtle "immortal PR" recreation semantics we must coordinate with.
- **Community workflows** (Hrvey, Typeform, `gh combine-prs` CLI): all confirm the same pain points — **adjacent-line merge conflicts** (especially `package.json`, `*.lock`, `build.gradle`) and **wasted CI** re-running every PR after each rebase.

**What's missing in all prior art — and what Ampel uniquely can offer:** a **provider-agnostic, centrally-orchestrated** version that runs as a *control plane* over a whole portfolio (not a `.github/workflows/*.yml` copied into each repo), with a real verification gate, autonomous merge, source-PR closure with references, and an optional agentic remediation tier for the cases that mechanical merge can't fix.

---

## 2. How It Maps to Loop Engineering

The feature is two nested loops.

```
OUTER LOOP  (Ampel worker — the fleet orchestrator; runs on a schedule)
  perceive:  poll fleet → which repos have > N open PRs and an enabled policy?
  select:    for each qualifying repo, choose the PRs to coalesce (criteria)
  dispatch:  spawn a bounded number of per-repo RemediationRunJobs
  record:    persist run state; back off on rate limits; repeat next cycle

      INNER LOOP  (per-repo RemediationRunJob — the state machine)
        consolidate:  octopus-merge selected branches into a fresh branch
        remediate:    TIER 1 mechanical (lockfile regen, base update)
                      TIER 2 agentic (sandbox + coding agent /goal loop)   [opt-in]
        verify:       CI green on the consolidated ref?  (the bottleneck)
        finalize:     if green AND closed-loop → merge, close sources w/ refs
                      else → leave PR open, label, comment, notify (open-loop)
```

The outer loop is *deterministic orchestration*. The inner loop's tiers escalate only as needed. The agent (when used) is a **sub-agent inside a sandbox worktree**, and it is **never the verifier** — provider CI is. This keeps the autonomy bounded and the trust model legible.

---

## 3. System Architecture

```
                         ┌─────────────────────────────────────────────┐
                         │                Frontend (React 19)           │
                         │  Remediation page · Policy editor · Run      │
                         │  timeline · Dry-run preview · Audit log      │
                         └───────────────┬─────────────────────────────┘
                                         │  REST + SSE (/api/remediation/*)
                         ┌───────────────▼─────────────────────────────┐
                         │            ampel-api (Axum)                  │
                         │  remediation handlers: policies, runs,       │
                         │  preview, trigger, approve, cancel, events   │
                         └───────────────┬─────────────────────────────┘
                                         │
            ┌────────────────────────────┼────────────────────────────────┐
            │                            │                                 │
   ┌────────▼─────────┐        ┌─────────▼──────────┐           ┌──────────▼─────────┐
   │   ampel-core     │        │     ampel-db       │           │   ampel-worker     │
   │ RemediationSvc   │        │  remediation_*     │           │  (Apalis cron)     │
   │ Consolidation    │◄──────►│  policy / run /    │◄─────────► │ RemediationSweep   │
   │ Verification     │        │  run_pr / agent    │           │  (OUTER loop)      │
   │ PolicyResolver   │        │  entities + migr.  │           │ RemediationRunJob  │
   └────────┬─────────┘        └────────────────────┘           │  (INNER loop)      │
            │                                                    └─────────┬──────────┘
            │ uses                                                         │ dispatches
   ┌────────▼──────────────────────────────────┐            ┌─────────────▼────────────┐
   │            ampel-providers                 │            │   Remediation Sandbox    │
   │  GitProvider (read)  +  RemediationCapable │            │  ephemeral worktree:     │
   │  create_branch / create_pr / close_pr /    │            │  shallow clone (scoped   │
   │  comment / status-for-ref / update_branch  │            │  PAT) → octopus merge →  │
   │  GitHub · GitLab · Bitbucket · Mock        │            │  lockfile regen → push;  │
   └────────────────────────────────────────────┘            │  optional coding agent   │
                                                              └──────────────────────────┘
```

New crates are avoided; the feature slots into the existing five (`ampel-api`, `ampel-core`, `ampel-db`, `ampel-providers`, `ampel-worker`). The only genuinely new runtime surface is the **Remediation Sandbox** — an ephemeral, isolated execution environment the worker uses for clone-based merges and (optionally) for the coding agent.

---

## 4. The Remediation Loop (State Machine)

Each per-repo run is a persisted state machine (`remediation_run.state`). Persisting every transition makes the loop **idempotent and resumable** across worker restarts — the "external state" pillar.

```
pending
  └─► selecting            (apply criteria; if ≤ threshold or none qualify → no_op)
        └─► consolidating   (create branch off default; octopus-merge sources)
              ├─ conflict (mechanically unresolvable, agentic off) ─► handoff_human
              └─► remediating
                    ├─ tier1: regenerate lockfiles, update from base
                    ├─ tier2 (opt-in): sandbox + agent /goal loop until green/budget
                    └─► verifying        (poll CI on consolidated ref)
                          ├─ green ──────► (closed-loop?) ─► merging ─► closing_sources ─► completed
                          │                 └ (open-loop) ─► awaiting_approval ─► (approve) ─► merging …
                          ├─ red, budget left, tier2 ─► remediating  (loop back)
                          └─ red, exhausted ─► handoff_human
        states: handoff_human · failed · cancelled · no_op · completed
```

- **`closing_sources`** runs *only after* a successful merge. Each source PR is closed with a comment: *"Superseded by `#<consolidated>` — changes incorporated and merged."* IDs recorded in `closed_pr_ids` for auditability and reversal.
- **`awaiting_approval`** is the open-loop human gate: the consolidated PR is built, verified green, labeled `ampel/ready`, and a reviewer is pinged; merge happens on approval via the API.
- **Re-entrancy:** the consolidated branch name is deterministic (`ampel/remediation/<run-short-id>`), and an advisory lock / `state IN (active…)` check guarantees **one active run per repository**, so a re-trigger reconciles rather than double-consolidating.

---

## 5. Backend Component Design

### 5.1 `ampel-worker` — the outer loop (`RemediationSweepJob`)

A new Apalis `CronStream` job, registered alongside the existing `poll-repository`, `cleanup`, `metrics-collection`, and `health-score` monitors in `crates/ampel-worker/src/main.rs`. Default schedule: every 15 minutes (configurable per policy; respects each policy's own cron).

```rust
// crates/ampel-worker/src/jobs/remediation_sweep.rs  (new)
pub struct RemediationSweepJob;

impl RemediationSweepJob {
    pub async fn execute(&self, st: &WorkerState) -> anyhow::Result<()> {
        // 1. Resolve enabled policies (user/org/team/repo scope), respecting each schedule.
        // 2. For each in-scope repo, count OPEN PRs (DB; already kept fresh by poll job).
        // 3. Keep repos where open_pr_count > policy.min_open_prs AND no active run AND due.
        // 4. Bound concurrency (policy.max_concurrent_repos); enqueue RemediationRunJob per repo.
        Ok(())
    }
}
```

This reuses the exact ergonomics of `PollRepositoryJob::find_repos_to_poll` (ordering, `limit(50)`, due-filtering) so it inherits the same rate-limit-friendly batching.

### 5.2 `RemediationRunJob` — the inner loop driver

A separate Apalis job (or a step inside the sweep, executed with bounded `tokio` concurrency). It instantiates `RemediationService` and drives the state machine for **one** repository, persisting each transition.

### 5.3 `ampel-core::services::RemediationService`

Orchestrates a single run. Pure orchestration; all I/O via providers and the sandbox.

- `select_prs(policy, repo) -> Vec<Pr>` — applies criteria (§8).
- `consolidate(repo, prs) -> Consolidation` — calls the sandbox to build the branch.
- `remediate(consolidation, policy) -> Remediated` — tier 1 then (if enabled) tier 2.
- `verify(repo, ref) -> Verdict` — delegates to `VerificationService`.
- `finalize(verdict, policy) -> Outcome` — merge + close-sources, or open-loop handoff.

### 5.4 `ConsolidationStrategy` (the mechanical merge engine)

Runs inside the sandbox. Steps, in order:
1. Shallow-clone the repo with a scoped PAT; create `ampel/remediation/<id>` off the default branch.
2. **Octopus-merge** each selected source branch in turn (`git merge --no-ff`), recording which merged cleanly and which conflicted.
3. For **known conflict classes**, prefer *regeneration over line-merge*:
   - `package-lock.json` / `pnpm-lock.yaml` / `yarn.lock` → take union of `package.json` changes, then `npm install` / `pnpm install --lockfile-only` / `yarn install --mode update-lockfile`.
   - `Cargo.lock` → `cargo update --workspace` (or `cargo generate-lockfile`).
   - `go.sum` / `go.mod` → `go mod tidy`. `poetry.lock` → `poetry lock --no-update`. `Gemfile.lock` → `bundle lock`.
   - The right command per repo can be inferred from the repo fingerprint that the planned [CI/CD Intelligence engine](#14-relationship-to-existing-plans--future-work) already produces.
4. Push the branch; open the consolidated PR via the provider; body lists every source PR (`Closes #…` / provider-equivalent) and the per-source merge disposition.
5. Conflicts that *aren't* a known class and aren't agentically resolved are left out and reported (`run_pr.disposition = skipped_conflict`), mirroring `github/combine-prs` behavior.

### 5.5 `VerificationService` — the verifier (the bottleneck)

The most safety-critical component. A near-pure function over provider data that answers **"is this ref truly mergeable and green?"** It must:
- Aggregate **all** CI checks for the *consolidated ref* (not the individual PRs) and normalize them into the existing **`AmpelStatus`** traffic-light model (`crates/ampel-core/src/models/ampel_status.rs`) — green/yellow/red is literally the verifier's output, and it's the product's core metaphor.
- Honor **required checks / branch protection** (a green non-required check is not sufficient; a missing required check is red).
- Confirm **mergeability** (no conflicts with base; not draft; not blocked by requested changes) — reusing the **pre-flight verification** logic the bulk-merge handler already performs.
- **Re-verify immediately before merge** (TOCTOU guard) — state can drift between `verifying` and `merging`.

Only an unambiguous **green** with all required checks complete permits an autonomous merge. Anything else routes to handoff. This single rule is the backbone of the trust model.

### 5.6 `PolicyResolver`

Resolves the **effective** policy for a repo by walking the scope hierarchy (repo → team → org → user default), exactly mirroring how `auto_merge_rule` and `user_settings` already layer per-repo over per-user config.

### 5.7 Remediation Sandbox & the agentic tier

- **Isolation:** ephemeral container/worktree, destroyed after each run. Egress allow-list: the relevant provider host + package registries only. PAT injected as a short-lived, scoped credential; never written to disk in plaintext beyond the git credential helper's lifetime.
- **Tier 2 (agentic), opt-in:** when mechanical merge or CI fails and `remediation_tier = agentic`, hand the worktree to a coding agent behind a swappable `RemediationAgent` trait. The default implementation runs an agent in **headless `/goal` mode** with the completion condition *"the project's CI/build/test command exits 0"* and the failing logs + diff as context — the inner loop-engineering pattern. Hard budgets: max iterations, max wall-clock, max tokens/cost (`remediation_agent_session`). **The agent cannot self-certify** — after it claims success, control returns to `verifying`, and provider CI is the authority.

---

## 6. Provider Trait Gap Analysis

The current `GitProvider` trait (`crates/ampel-providers/src/traits.rs`) is **read + merge** only:

> `validate_credentials`, `get_user`, `list/get_repositories`, `list/get_pull_requests`, `get_ci_checks`, `get_reviews`, **`merge_pull_request`**, `get_rate_limit`.

It is missing every **write** primitive this feature needs. Proposed: add a **`RemediationCapable` supertrait** (separate from `GitProvider` so read-only PATs and providers with partial support are handled gracefully via capability flags).

```rust
#[async_trait]
pub trait RemediationCapable: GitProvider {
    async fn get_default_branch_sha(&self, c:&Creds, owner:&str, repo:&str) -> R<String>;
    async fn create_branch(&self, c:&Creds, owner:&str, repo:&str, name:&str, sha:&str) -> R<()>;
    async fn update_branch_from_base(&self, c:&Creds, owner:&str, repo:&str, pr:i32) -> R<()>; // GitHub "Update branch"; others: clone-push
    async fn create_pull_request(&self, c:&Creds, owner:&str, repo:&str, req:&NewPr) -> R<ProviderPullRequest>;
    async fn update_pull_request(&self, c:&Creds, owner:&str, repo:&str, n:i32, patch:&PrPatch) -> R<()>; // body/labels/title
    async fn close_pull_request(&self, c:&Creds, owner:&str, repo:&str, n:i32, comment:Option<&str>) -> R<()>;
    async fn create_comment(&self, c:&Creds, owner:&str, repo:&str, n:i32, body:&str) -> R<()>;
    async fn add_labels(&self, c:&Creds, owner:&str, repo:&str, n:i32, labels:&[String]) -> R<()>;
    async fn get_status_for_ref(&self, c:&Creds, owner:&str, repo:&str, r:&str) -> R<Vec<ProviderCICheck>>; // CI on an arbitrary ref, not just a PR
    async fn delete_branch(&self, c:&Creds, owner:&str, repo:&str, name:&str) -> R<()>; // cleanup, opt-in
    fn capabilities(&self) -> RemediationCaps; // which of the above are supported
}
```

Per-provider notes:
- **GitHub:** all primitives map cleanly to REST (Git refs API, Pulls API, Checks/Statuses, "Update branch"). PAT auth already re-triggers CI (unlike the default Actions token).
- **GitLab:** "merge requests," not "PRs"; `get_status_for_ref` → pipelines/commit statuses; "Update branch" → `/rebase`. Terminology mapping lives in the provider impl, invisible to core.
- **Bitbucket:** thinner API; some operations (arbitrary-ref status) require the commit-status endpoint and more clone-side work; `capabilities()` reflects gaps so the sandbox clone-push path is used as fallback.
- **Mock:** extend `mock.rs` to simulate the full write surface for deterministic worker tests (the project already tests workers against the mock provider).

This gap analysis is itself a deliverable: it's the minimum provider work that *any* version of this feature requires.

---

## 7. Data Models and Schema

Four new tables, following the existing SeaORM entity + timestamped-migration conventions and mirroring the `merge_operation` / `merge_operation_item` and `auto_merge_rule` shapes.

```
remediation_policy                       -- the configuration / toggle (§8)
├── id (UUID, pk)
├── scope (String: "user"|"org"|"team"|"repository")
├── scope_id (UUID)                      -- the user/org/team/repo it applies to
├── enabled (bool)                       -- the master on/off for this scope
├── min_open_prs (i32, default 3)        -- trigger when open count > this
├── schedule_cron (String)               -- per-policy cadence
├── autonomy_level (String)              -- off | dry_run | consolidate_only | auto_merge
├── remediation_tier (String)            -- mechanical_only | agentic
├── pr_selection (JSON)                  -- filters: authors, labels, drafts, age (§8)
├── merge_strategy (String)              -- merge | squash | rebase
├── require_all_checks (bool)
├── require_human_approval (bool)        -- forces open-loop even when green
├── max_concurrent_repos (i32)
├── delete_source_branches (bool)
├── agent_budget (JSON)                  -- max_iterations, max_seconds, max_cost
├── notify (JSON)                        -- channels for autonomous actions
├── created_at / updated_at

remediation_run                          -- one execution per repo per cycle (external state / log)
├── id (UUID, pk)
├── policy_id (UUID, fk)
├── repository_id (UUID, fk)
├── triggered_by (String: "schedule"|"manual"|"preview")
├── state (String)                       -- §4 state machine
├── consolidated_branch (String, null)
├── consolidated_pr_id / number / url (null)
├── source_pr_count (i32)
├── strategy_used (String, null)
├── ci_status (String, null)             -- green | yellow | red  (AmpelStatus)
├── conflict_summary (JSON, null)
├── remediation_tier_used (String, null)
├── agent_session_id (UUID, null)
├── merged (bool) / merged_sha (String, null)
├── closed_pr_ids (JSON)                 -- for audit + reversal
├── attempts (i32) / error (String, null)
├── started_at / finished_at (null)

remediation_run_pr                       -- per-source-PR disposition (mirrors merge_operation_item)
├── id (UUID, pk)
├── remediation_run_id (UUID, fk)
├── pull_request_id (UUID, fk)
├── disposition (String)                 -- consolidated | closed_with_ref | skipped_conflict | left_open
├── reason (String, null)

remediation_agent_session                -- only when tier 2 used
├── id (UUID, pk)
├── remediation_run_id (UUID, fk)
├── iterations (i32) / tokens (i64) / cost_usd (decimal)
├── outcome (String)                     -- passed | budget_exhausted | aborted
├── transcript_ref (String, null)        -- pointer to stored transcript
```

Indexes: `remediation_run(repository_id, state)` (active-run lookup / locking), `remediation_run(policy_id, started_at)` (history), `remediation_policy(scope, scope_id)` (resolution). Migration filename follows the `mYYYYMMDD_NNNNNN_remediation_loops.rs` convention.

---

## 8. Configuration Model (the Toggles)

This is the "configuration the user can toggle on and off." Config is **hierarchical and inherited**: a repo inherits its team's policy, which inherits the org's, which inherits the user default — each level can override or opt out. Resolution is handled by `PolicyResolver` (§5.6), reusing the established override pattern.

| Setting | Type | Default | Purpose |
|---|---|---|---|
| `enabled` | toggle | **off** | Master on/off for the scope |
| `min_open_prs` | int | **3** | Fire when open PR count **> 3** |
| `schedule_cron` | cron | `*/15 * * * *` | How often the outer loop checks this scope |
| `autonomy_level` | enum | `dry_run` | `off` / `dry_run` (log only) / `consolidate_only` (open-loop, stop at green PR) / `auto_merge` (closed-loop) |
| `remediation_tier` | enum | `mechanical_only` | `mechanical_only` vs `agentic` (opt-in sandbox + coding agent) |
| `pr_selection.authors` | list | `[]` (all) | e.g. restrict to `dependabot`, `renovate` |
| `pr_selection.include_labels` | list | `[]` | Only PRs with these labels |
| `pr_selection.exclude_labels` | list | `["do-not-merge","wip"]` | Never touch these |
| `pr_selection.exclude_drafts` | bool | `true` | Skip draft PRs |
| `pr_selection.min_age_hours` | int | `0` | Avoid coalescing brand-new PRs |
| `pr_selection.require_no_changes_requested` | bool | `true` | Skip PRs a human flagged |
| `merge_strategy` | enum | `squash` | Merge style for the consolidated PR |
| `require_all_checks` | bool | `true` | All required checks must be green |
| `require_human_approval` | bool | `false` | Force open-loop even when green |
| `max_concurrent_repos` | int | `5` | Blast-radius cap |
| `delete_source_branches` | bool | `false` | Clean up after close |
| `agent_budget` | object | `{iters:6, secs:900, cost:2.00}` | Tier-2 ceilings |

Two non-obvious defaults that matter for safety: the system ships **`dry_run`** (not `off`, not `auto_merge`) so the very first thing an operator sees after flipping `enabled` is *what it would have done*, and `auto_merge` requires an explicit, deliberate change.

---

## 9. API Design

New handlers under `crates/ampel-api/src/handlers/remediation.rs`, registered in `routes/mod.rs` under `/api/remediation`, all authenticated and ownership-scoped exactly like the existing repository/PR routes.

```
# Policy (the toggles)
GET    /api/remediation/policy                       # effective resolved policy for current scope
GET    /api/remediation/policies                     # list scoped policies
POST   /api/remediation/policies                     # create a scoped override
PATCH  /api/remediation/policies/{id}                # edit
DELETE /api/remediation/policies/{id}
POST   /api/remediation/policies/{id}/toggle         # the on/off switch  { enabled: bool }

# Planning & execution
POST   /api/remediation/repositories/{repo_id}/preview   # DRY-RUN: plan only, zero writes
POST   /api/remediation/repositories/{repo_id}/run       # manual trigger
GET    /api/remediation/runs                          # history (filter by repo/state/date)
GET    /api/remediation/runs/{id}                     # detail: per-PR dispositions, CI matrix, conflict report, agent session
GET    /api/remediation/runs/{id}/events              # SSE live progress (reuses bulk-merge progress pattern)
POST   /api/remediation/runs/{id}/approve             # open-loop human gate → triggers merge
POST   /api/remediation/runs/{id}/cancel

# Fleet view
GET    /api/remediation/fleet                         # every managed repo: open count, eligibility, policy state, last/next run
```

The **`/preview`** endpoint is a first-class trust feature, not an afterthought: it runs `select_prs` and a *clone-only* dry consolidation (no push, no PR, no merge) and returns the exact plan — which PRs would be coalesced, predicted conflicts, and what the closed-loop would do. The UI runs this fleet-wide before an operator ever enables `auto_merge`.

---

## 10. Frontend Architecture

Stack is fixed by the repo: **React 19 + TypeScript, Vite, TanStack Query, shadcn/ui, Tailwind**, with the established `pages/` + `api/` + `components/` + `hooks/` + `types/` layout and a heavy **i18n** system (27 languages, RTL) that new strings must respect.

### 10.1 New surface

- **`pages/Remediation.tsx`** + a nav entry. Tabs: **Fleet**, **Policies**, **Runs**, **Audit**.
- **`api/remediation.ts`** — typed client (mirrors `api/merge.ts`).
- **`hooks/`** — `useRemediationPolicies`, `useRemediationRuns`, `useFleetRemediation`, `useRemediationRunEvents` (SSE).
- **`components/remediation/`** — the pieces below. Reuse `components/merge/` progress patterns.
- **`types/remediation.ts`** — shared DTOs.

### 10.2 Key views

**Fleet overview.** A table/grid of every managed repo: open-PR count, an eligibility badge (`> 3 ✓` / `not yet`), policy state (`On` / `Inherited` / `Off`), last-run **traffic light** (on-brand reuse of `AmpelStatus`), and next scheduled run. A prominent **"Preview across fleet"** button runs `/preview` for all eligible repos and shows what *would* happen — the single most important trust affordance before turning on autonomy.

**Policy editor.** The master **toggle** plus the §8 form, with scope selector (org/team/repo) and a clear "inherited from …" indicator. The autonomy control is a four-stop selector — *Off · Dry-run · Consolidate only · Auto-merge* — with inline copy describing the blast radius of each, and an explicit confirm step to reach Auto-merge.

**Run timeline (detail).** A vertical state-machine timeline (§4), the consolidated PR link, a list of source PRs each with a disposition badge (`Consolidated` / `Closed → #123` / `Left open: conflict`), a **CI check matrix** (the verifier's view, traffic-lit), the conflict report, and — if tier 2 ran — the agent session (iterations, tokens, cost, transcript link). Live via the `events` SSE stream, reusing the bulk-merge progress UI.

**Audit log.** Append-only list of every autonomous merge/close: repo, run, actor (`Ampel`), timestamp, and reversible references (`closed_pr_ids`, `merged_sha`). Filterable, exportable.

### 10.3 Interaction safeguards in the UI

- Enabling `auto_merge` is a two-step confirm that *requires* a successful fleet preview first.
- A persistent **"Pause all remediation"** kill-switch in the page header (calls `toggle` on the top-scope policy).
- Open-loop runs surface an **Approve / Reject** action on the run detail and the fleet view.

---

## 11. Safety, Guardrails, and Trust

The research is blunt that autonomous loops are "not set-and-forget magic," and "loopmaxxing" is a real failure mode. Guardrails are therefore load-bearing, not garnish.

1. **Default off; opt-in per scope; master kill-switch.** Ships `enabled=false`, `autonomy_level=dry_run`.
2. **Shadow / dry-run warm-up.** First runs log intended actions only. `/preview` is mandatory in the UI before `auto_merge`.
3. **Hard trigger gate + criteria.** `open_count > 3` *and* the §8 filters (exclude drafts, `do-not-merge`/`wip` labels, changes-requested, min age, optional bot-only).
4. **One active run per repo.** Advisory lock + state check; deterministic branch name → re-trigger reconciles, never double-consolidates.
5. **The verifier is external and re-checked.** Merge only on unambiguous green with all *required* checks on the *consolidated* ref, **re-verified immediately before merge** (TOCTOU). Honor branch protection and provider merge restrictions. An agent's "done" is never sufficient.
6. **Close after merge, with references, reversibly.** Sources are closed only post-merge, each with a "superseded by #X" comment; IDs recorded. Source branches deleted only if configured.
7. **Conflicts are conservative.** Unknown, mechanically-unresolvable conflicts are left out and reported — never force-merged. Agentic resolution is opt-in and still gated by the external verifier.
8. **Blast-radius caps.** `max_concurrent_repos`, per-cycle repo limits (like the existing poll job's `limit(50)`), and rate-limit back-off via the trait's existing `get_rate_limit`.
9. **Agentic-tier containment.** Sandboxed worktree, egress allow-list, tool allow-list, no force-push to protected branches, strict iteration/time/cost budgets, transcript retained.
10. **Secrets.** PATs remain AES-256-GCM encrypted at rest (existing `EncryptionService`); the sandbox receives short-lived scoped credentials and is destroyed afterward.
11. **Notify every autonomous action.** Hook the in-progress Slack/email notification workers so a human always learns of a merge/close.
12. **Bot coordination.** Closing Dependabot/Renovate PRs interacts with their recreation semantics (Renovate "immortal PRs"; Dependabot auto-closing superseded PRs). The close comment + label and, optionally, writing ignore/group config prevents churn.

---

## 12. Implementation Phases

Sequenced so value lands early and autonomy is the *last* thing switched on — each phase is independently shippable.

**Phase 0 — Provider write primitives (≈2–3 wks).** Add `RemediationCapable` supertrait + GitHub/GitLab/Bitbucket impls + `capabilities()`; extend `mock.rs`; unit + worker tests. *No product behavior yet — pure capability.*

**Phase 1 — Data model, policy CRUD, dry-run (≈2–3 wks).** New entities/migrations; `PolicyResolver`; policy API + the toggle; **`/preview`** (read-only planning, zero writes); Fleet overview + Policy editor UI. Operators can *see* what would happen. No writes to any repo.

**Phase 2 — Mechanical consolidation + verification + closed-loop (≈3–4 wks).** `ConsolidationStrategy` (octopus merge + lockfile regen), the **sandbox**, `VerificationService`, the outer/inner Apalis jobs. Ship behind **shadow mode** first, then enable `consolidate_only`, then `auto_merge` for the bot-PR case. This delivers the headline outcome for the dominant scenario.

**Phase 3 — Observability & UX (≈2 wks).** Run timeline + live SSE events, audit log, Slack/email notifications, Prometheus metrics (runs, merges, conflicts, agent cost) into the existing Grafana stack.

**Phase 4 — Agentic remediation tier (≈3–4 wks, opt-in).** `RemediationAgent` trait + default headless `/goal`-loop implementation in the sandbox; budgets; `remediation_agent_session`; agent transcript UI. Strictly gated; verifier remains external.

**Phase 5 — Cross-provider hardening & learning (ongoing).** Bitbucket clone-push fallbacks, per-repo strategy learning (which consolidation/conflict tactics succeed), and integration with the planned repo-fingerprint intelligence.

---

## 13. Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| Autonomous merge of a bad change | **High** | External re-verified green-check; required-checks only; default off; dry-run/shadow; approval gate option; notify-on-action |
| Merge-conflict resolution corrupts code | High | Regenerate (don't line-merge) lockfiles; skip+report unknown conflicts; agentic resolution still gated by CI |
| "Loopmaxxing" — runs thrash, waste CI/cost | Medium | Concurrency caps, schedule cadence, one-run-per-repo lock, agent budgets, rate-limit back-off |
| Provider API divergence (esp. Bitbucket) | Medium | `capabilities()` + clone-push fallback; mock-provider parity tests |
| CI not re-triggering on consolidated branch | Medium | Ampel uses PATs (not the default Actions token); verify CI actually started before trusting "green" |
| Bot PR recreation churn (Renovate/Dependabot) | Low–Med | Close-with-comment + label; optional ignore/group config; rely on Dependabot's superseded auto-close |
| Sandbox compromise / secret leakage | High | Isolation, egress allow-list, short-lived scoped creds, ephemeral teardown, existing encryption at rest |
| Operator over-trust | Medium | Mandatory fleet preview before `auto_merge`; explicit confirm; visible kill-switch; audit log |

---

## 14. Relationship to Existing Plans & Future Work

This complements — does not overlap — the existing **`docs/planning/CICD_AUTOMATION_INTELLIGENCE.md`**, which *generates* CI/CD workflow files via repo fingerprinting, embeddings, and local LLM inference. That feature decides **what CI a repo should have**; this feature **operates on PRs and CI at runtime**. They compose:

- The intelligence engine's **repo fingerprint** tells `ConsolidationStrategy` which package manager / lockfile-regen command to use, and which build/test command becomes the agent's `/goal` completion condition.
- The vector-DB / reflexion-memory layer planned there is a natural home for **Phase 5 strategy learning** — remembering which remediation tactics succeed for which repo shapes, so the loop compounds (the "skills" pillar of loop engineering).
- Both reuse **Apalis** for scheduling and the same **multi-account / encrypted-credential** model.

Future extensions: merge-queue-style serialization across dependent repos; "stacked" consolidation (group by ecosystem, like Dependabot cross-ecosystem grouping); policy templates ("aggressive dependency hygiene" vs "conservative"); and a portfolio-level SLA view ("time-to-green" per repo) feeding the existing health-score and analytics features.

---

*Prepared as a design proposal for the Ampel project. Mechanics and conventions referenced (the `GitProvider` trait, Apalis cron worker, `auto_merge_rule` / `merge_operation` entities, `AmpelStatus`, bulk-merge pre-flight verification, hierarchical settings) reflect the current `main` of `pacphi/ampel`.*
