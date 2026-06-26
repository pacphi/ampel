# ADR-005: Octopus Merge Implementation via Subprocess Git

**Status**: Accepted  
**Date**: 2026-06-24  
**Deciders**: Architecture Team  
**Technical Story**: `ConsolidationStrategy` must merge N selected branches (typically
3–10) into a single consolidated branch. This requires multi-branch merge capability
that neither `git2-rs` nor `gitoxide` provides in stable form.

---

## Context

### Problem Statement

The core consolidation operation merges a set of selected source branches into a fresh
branch off the default branch, resolves known conflict classes mechanically (lockfile
regeneration), and pushes the result. For N > 2 branches, this is an octopus-style
merge.

**`git2-rs` (libgit2 bindings)**: `git_merge_trees` and `git_merge_commits` are
pairwise (two-way) only. `git_merge_base_octopus()` computes an octopus merge base but
there is no high-level API for executing an N-way merge — the caller must implement the
recursive merge loop manually.

**`gitoxide` (`gix`)**: Actively developed pure-Rust git library. As of June 2026, the
merge implementation is experimental and N-way merge is not documented as stable.

The `git` CLI has handled octopus merges since 2005 and is available in the sandbox
container (ADR-003). Sequential single-branch merges (`git merge --no-ff` in a loop)
produce the same result as a true octopus merge while enabling per-branch outcome
tracking — which is required for recording `MergeDisposition` per source PR.

### Technical Context

- Consolidation runs inside a rootless Podman container (ADR-003); the container image
  includes a full `git` installation and all lockfile CLIs.
- `tokio::process::Command` provides async subprocess execution with stdout/stderr
  capture for structured error reporting.
- Merge order: oldest PR first (by `created_at`) to minimise conflicts.
- Per-PR conflict disposition must be recorded (`MergeDisposition`); a sequential loop
  makes per-branch outcome tracking natural.
- Lockfile conflict classes require running regeneration commands between merges; this
  is easier to orchestrate in a sequential loop than in a single octopus invocation.

---

## Decision

**Use sequential `tokio::process::Command` `git merge --no-ff` calls (one per source
branch) inside the Podman container. This is not a true octopus merge but produces an
equivalent result while enabling per-branch conflict detection, lockfile regeneration
between merges, and structured disposition recording.**

### Merge Loop

```rust
pub async fn run(&self, ctx: &ConsolidationContext) -> Result<ConsolidationResult> {
    let env = [("GIT_TERMINAL_PROMPT", "0"), ("GIT_ASKPASS", "echo")];

    git(["clone", "--depth", "50", &ctx.repo_url, "/workspace"], &env).await?;
    git(["checkout", "-b", &ctx.branch, &format!("origin/{}", ctx.default_branch)], &env).await?;

    let mut dispositions = Vec::new();

    for pr in ctx.source_prs.iter().sorted_by_key(|p| p.created_at) {
        git(["fetch", "origin", &pr.head_ref], &env).await?;

        match git(["merge", "--no-ff", "FETCH_HEAD"], &env).await {
            Ok(_) => {
                dispositions.push((pr.id, MergeDisposition::Consolidated));
            }
            Err(conflict) => {
                match classify_lockfile_conflict(&conflict.conflicted_files) {
                    Some(class) => {
                        // Known conflict class: regenerate lockfile
                        run_regen_command(class, &env).await?;
                        git(["add", "."], &env).await?;
                        git(["commit", "--no-edit"], &env).await?;
                        dispositions.push((pr.id, MergeDisposition::Consolidated));
                    }
                    None => {
                        git(["merge", "--abort"], &env).await?;
                        dispositions.push((pr.id, MergeDisposition::SkippedConflict {
                            reason: conflict.summary(),
                        }));
                    }
                }
            }
        }
    }

    git(["push", "origin", &ctx.branch], &env).await?;
    Ok(ConsolidationResult { branch: ctx.branch.clone(), dispositions })
}
```

### Lockfile Regen Command Map

| Conflict file pattern | Regen command |
|----------------------|---------------|
| `package-lock.json` | `npm install` |
| `pnpm-lock.yaml` | `pnpm install --lockfile-only` |
| `yarn.lock` | `yarn install --mode update-lockfile` |
| `Cargo.lock` | `cargo update --workspace` |
| `go.sum` / `go.mod` | `go mod tidy` |
| `poetry.lock` | `poetry lock --no-update` |
| `Gemfile.lock` | `bundle lock` |

Unknown conflict files → `MergeDisposition::SkippedConflict`; the source PR is left
open with a `reason` explaining the conflict.

---

## Alternatives Considered

### Option A: `git2-rs` (libgit2) (Rejected)

**Pros**: In-process; already a dependency in `ampel-providers`.

**Cons**:
- ❌ No octopus (N-way) merge API in libgit2
- ❌ A manual recursive merge loop would be equivalent complexity to subprocess calls
  without the maturity of the git CLI's conflict resolution
- ❌ Lockfile regen commands must still be run as subprocesses; cannot avoid subprocess
  calls regardless

**Verdict**: REJECTED — fundamental API limitation.

### Option B: Sequential subprocess `git merge` (ACCEPTED)

**Pros**:
- ✅ Battle-tested in the git CLI
- ✅ Per-branch outcome tracking is natural
- ✅ Lockfile regen runs between merges
- ✅ `git` is already in the sandbox container image
- ✅ `tokio::process::Command` provides async, non-blocking execution

**Cons**:
- ⚠️ ~50–200 ms subprocess overhead per merge; negligible for 3–10 branches

**Verdict**: ACCEPTED.

### Option C: `gitoxide` (`gix`) (Rejected)

**Pros**: Pure Rust; in-process; no subprocess overhead.

**Cons**:
- ❌ Merge support is experimental as of June 2026; N-way merges not stable
- ❌ Adopting an unstable API creates maintenance risk for a safety-critical operation

**Verdict**: REJECTED — not production-ready. Re-evaluate for Phase 5.

---

## Trade-off Analysis

| Aspect | Option A (git2-rs) | Option B (subprocess git) ⭐ | Option C (gitoxide) |
|--------|-------------------|-----------------------------|--------------------|
| **Octopus support** | ❌ None | ✅ Full (sequential) | ❌ Experimental |
| **Per-branch tracking** | ⚠️ Complex | ✅ Natural | ⚠️ Unknown |
| **Subprocess avoidance** | ⚠️ Partial | ❌ Required | ✅ In-process |
| **Maturity** | ✅ Stable | ✅ Stable (git CLI) | ❌ Unstable |
| **Overhead per merge** | Low | ~50–200 ms | Low |
| **Lockfile regen between merges** | ⚠️ Complex | ✅ Natural | ⚠️ Unknown |

---

## Consequences

### Positive

- Sequential merge loop enables per-branch conflict detection and disposition recording
- Lockfile regen commands run between merges, resolving the dominant conflict class
- git CLI behaviour is well-documented and stable

### Negative

- Subprocess invocations add latency (~50–200 ms per branch); acceptable for a
  multi-minute workflow on 3–10 branches
- Must parse git exit codes and stderr for conflict detection

### Neutral

- The sandbox container image must include git and all ecosystem CLIs; maintained as
  part of the standard release pipeline

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| git CLI output format changes break conflict detection | Low | Pin git version in sandbox image; integration tests with real git operations |
| Subprocess hangs waiting for input | Medium | `GIT_TERMINAL_PROMPT=0` + `GIT_ASKPASS=echo`; per-subprocess timeout |
| Lockfile regen command not available in container | Medium | Integration test suite verifies all regen commands run successfully |
| Shallow clone too shallow to reach merge base | Medium | `--depth 50` is configurable via `AMPEL_CLONE_DEPTH` env var |

---

## Related ADRs

- ADR-003: Sandbox isolation — subprocess git runs inside the Podman container
- ADR-004: State machine persistence — each merge result recorded as a `RemediationRunPr`
  disposition in the state transition transaction
- ADR-002: `RemediationCapable` supertrait — `create_pull_request()` called after all
  branches are merged and pushed
