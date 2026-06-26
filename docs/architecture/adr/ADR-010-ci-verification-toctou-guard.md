# ADR-010: CI Verification TOCTOU Guard and Required-Checks Enforcement

**Status**: Accepted  
**Date**: 2026-06-24  
**Deciders**: Architecture Team  
**Technical Story**: The `VerificationService` is the most safety-critical component in
the remediation loop — it decides whether a consolidated PR is safe to merge
autonomously. Two correctness concerns must be addressed: a time-of-check/time-of-use
(TOCTOU) race between verification and merge, and correct enforcement of required vs
optional CI checks.

---

## Context

### Problem Statement

**TOCTOU race**: The run enters `verifying` state, calls `VerificationService`, gets a
green result, and transitions to `merging` state. In the time between `verifying` and
the actual `merge_pull_request()` API call — which can be several seconds if the worker
is under load — a previously-green CI check can flip red (flaky test re-run, a
concurrent force-push to the base branch, a new PR check added by branch protection).
An autonomous merge on a stale green result is a safety violation.

**Required-checks enforcement**: Provider branch protection rules designate some CI
checks as "required" (must pass before merge) and others as optional. A non-required
check that is green does not satisfy a required check that is missing or red. The
`VerificationService` must distinguish between the two and treat a missing required
check as red, not yellow.

### Technical Context

- The `merging` state transition issues `RemediationCapable::merge_pull_request()` — a
  write operation that cannot be rolled back.
- Provider APIs expose required checks via branch protection APIs (GitHub:
  `GET /repos/{owner}/{repo}/branches/{branch}/protection`; GitLab: `merge_requests_events`
  with `only_allow_merge_if_all_status_checks_passed`; Bitbucket: restrictions API).
- The existing `AmpelStatus` model (`green` / `yellow` / `red`) is the output of
  verification; it is already used on the PR dashboard as the traffic light.
- Mergeability is a distinct concern from CI: a PR can be CI-green but blocked by
  a draft flag, a "changes requested" review, or a base-branch merge conflict.

---

## Decision

**Re-verify immediately before merge (double-check), and require all required checks to
be present and green. Missing required checks are red, not yellow. Non-required checks
do not block merge.**

### `CiVerificationResult`

```rust
pub struct CiVerificationResult {
    pub ref_sha: String,
    pub checks: Vec<NormalizedCiCheck>,
    pub all_required_green: bool,
    pub mergeable: bool,
    pub ampel_status: AmpelStatus,
}

pub struct NormalizedCiCheck {
    pub context: String,       // check name
    pub status: CheckStatus,   // Pending | Running | Green | Red | Skipped
    pub required: bool,        // from branch protection
    pub url: Option<String>,   // link to CI run
}
```

### `is_safe_to_merge` predicate

```rust
pub fn is_safe_to_merge(result: &CiVerificationResult) -> bool {
    result.ampel_status == AmpelStatus::Green
        && result.all_required_green
        && result.mergeable
}

// AmpelStatus is computed as:
// - Red  if any required check is Red or Missing
// - Red  if PR is not mergeable (draft | changes_requested | base_conflict)
// - Yellow if any required check is Pending or Running
// - Green if all required checks are present and Green, and PR is mergeable
```

### Double-check Protocol in `merging` State

```rust
// In RemediationService::do_merge()
let pre_merge_result = verification_service
    .verify(&repo, &run.consolidated_ref_sha)
    .await?;

counter!("ampel_remediation_pre_merge_verification_total",
    "result" => if is_safe_to_merge(&pre_merge_result) { "green" } else { "blocked" }
).increment(1);

if !is_safe_to_merge(&pre_merge_result) {
    return self.advance(run.id, RunState::HandoffHuman, /* reason */ ).await;
}

// Only after passing the re-verify gate:
provider.merge_pull_request(&creds, &repo, run.consolidated_pr_number, strategy).await?
```

### Required-Checks API

A new `get_required_checks(owner, repo, branch)` method is added to `GitProvider`
(not `RemediationCapable`) because required-check discovery is a read operation:

```rust
async fn get_required_checks(
    &self,
    creds: &ProviderCredentials,
    owner: &str,
    repo: &str,
    branch: &str,
) -> ProviderResult<Vec<String>>;  // list of required context names
```

---

## Alternatives Considered

### Option A: Trust cached CI status from last poll (Rejected)

**Approach**: The `RepositoryPollJob` already fetches CI check statuses; use the cached
value from the last poll when transitioning to `merging`.

**Pros**: Zero extra API calls at merge time.

**Cons**:
- ❌ TOCTOU risk — the cached status can be minutes old; a flapping check can flip red
  after the cache was written
- ❌ The poll interval is typically 5–15 minutes; this window is unacceptably large
  for an autonomous merge decision

**Verdict**: REJECTED — unacceptable TOCTOU exposure for an irreversible write operation.

### Option B: Re-verify immediately before merge (ACCEPTED)

**Approach**: Call `VerificationService::verify()` inside the `merging` state handler,
immediately before the `merge_pull_request()` call.

**Pros**:
- ✅ Eliminates the practical TOCTOU window (1–5 API round-trips, seconds)
- ✅ Required-checks enforcement is applied at both `verifying` and `merging` states
- ✅ A residual sub-second race window is acknowledged but acceptable (equivalent to
  what any CI-gated merge button does in a web UI)
- ✅ Cost: 2–3 extra provider API calls per run; negligible vs the run's total latency

**Cons**:
- ⚠️ Adds 2–3 extra provider API calls per successful run

**Verdict**: ACCEPTED.

### Option C: Optimistic lock + single verify (Rejected)

**Approach**: Use an ETag or SHA comparison: store the ref SHA at `verifying` time;
re-fetch the SHA at `merging` time; if they differ, abort.

**Pros**: Fewer API calls than Option B if SHA is stable.

**Cons**:
- ❌ A SHA match does not guarantee CI checks have not changed (e.g., a new required
  check added to branch protection after the SHA was pinned)
- ❌ Equivalent safety to Option B only if required-checks cannot change between states;
  this assumption is not safe

**Verdict**: REJECTED — not equivalent to full re-verification.

---

## Trade-off Analysis

| Aspect | Option A (cached) | Option B (re-verify) ⭐ | Option C (SHA lock) |
|--------|------------------|------------------------|---------------------|
| **TOCTOU protection** | ❌ None | ✅ Practical elimination | ⚠️ Partial |
| **Required-checks enforcement** | ⚠️ Best-effort | ✅ Full | ⚠️ Partial |
| **Extra API calls** | 0 | 2–3 | 1 |
| **Implementation complexity** | Low | Medium | Medium |
| **Safety for auto_merge** | ❌ Inadequate | ✅ Adequate | ⚠️ Marginally adequate |

---

## Consequences

### Positive

- Autonomous merges are gated on a fresh CI state check; flapping checks cause a
  `handoff_human`, not a bad merge
- Required checks are enforced consistently; non-required passing checks do not
  substitute for required missing ones
- The `ampel_remediation_pre_merge_verification_total{result}` metric surfaces how
  often runs are blocked at the final gate (aids threshold tuning)

### Negative

- Each successful run incurs 2–3 extra provider API calls for the pre-merge check;
  at the fleet scale, this is a measurable increase in rate-limit consumption
- Flapping checks cause `handoff_human` rather than eventually succeeding; mitigation
  is a `verification_retry_count` config in the Playbook

### Neutral

- A sub-second TOCTOU window between the pre-merge verify and the actual merge API
  call remains; this is accepted as equivalent to human-driven merge UX

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Provider branch-protection API unavailable at merge time | Medium | Treat unavailability as red; route to `handoff_human`; retry not attempted for merge |
| Flapping check causes repeated `handoff_human` notifications | Medium | Playbook-configurable `verification_retry_count`; exponential backoff on re-entry |
| Required-checks API differs significantly per provider | Low | Per-provider `get_required_checks()` impl; `MockProvider` must simulate branch protection |

---

## Related ADRs

- ADR-004: State machine persistence — `merging` state transition wraps the re-verify
  and merge call in a single DB transaction context
- ADR-002: `RemediationCapable` supertrait — `merge_pull_request()` is the write call
  guarded by this re-verify gate
- ADR-007: `ModelProvider` trait — after agent-kind provider completes, control returns
  to `verifying` state; the same TOCTOU guard applies before the subsequent merge
