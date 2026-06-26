# ADR-003: Sandbox Isolation via Rootless Podman/Docker

**Status**: Accepted  
**Date**: 2026-06-24  
**Deciders**: Architecture Team  
**Technical Story**: Fleet PR Remediation requires an isolated execution environment for
cloning repositories, performing sequential merges, regenerating lockfiles, and — in the
agentic tier — running a coding agent with access to source code and CI logs.

---

## Context

### Problem Statement

The `ConsolidationStrategy` must: shallow-clone a repository using a scoped PAT,
merge 3–10 branches sequentially, run ecosystem-specific lockfile regeneration commands
(`npm install`, `cargo update`, `go mod tidy`, etc.), push the consolidated branch, and
— in Phase 4 — hand the working tree to a coding agent that may invoke additional
build/test commands.

Running these operations directly on the worker host creates four distinct risk
categories:

1. **Credential exposure** — A PAT injected as an env var leaks into child processes,
   `/proc/self/environ` reads by other processes on the host, and subprocess output
   captured in logs.
2. **Egress leakage** — Lockfile regen commands (`npm install`, `pip install`, `cargo
   fetch`) resolve packages from the internet. Without network containment, a compromised
   dependency or a prompt-injected agent command could exfiltrate secrets to arbitrary
   hosts.
3. **Privilege escalation** — Running `sudo`-capable commands or exploiting SUID binaries
   affects the worker host and all other jobs on that worker.
4. **State contamination** — Failed or interrupted runs leave partial checkouts, stale
   lock files, and corrupted git state that bleed into subsequent runs sharing the same
   working directory.

The agentic tier (Phase 4) makes the risk surface materially larger: a coding agent
can be prompted via adversarial CI logs to execute arbitrary shell commands. Any sandbox
that relies on process-level isolation alone is insufficient when the agent's tool surface
includes `run_command`.

### Technical Context

- Workers run on Fly.io (Linux/amd64) in production; developers use macOS.
- The `RemediationRunJob` in `ampel-worker` is an Apalis job; it spawns child processes
  via `tokio::process::Command`.
- Podman is available rootless on Fly.io and on macOS via Podman Desktop. Rootless
  Docker is available on Linux; Docker Desktop on macOS.
- PATs are decrypted in-process by `EncryptionService`; they must not be written to
  disk in plaintext.
- Octopus merges require subprocess `git` commands (ADR-005); `git` must be present
  in the sandbox image.

---

## Decision

**Each `RemediationRunJob` spawns an ephemeral rootless Podman (preferred) or rootless
Docker container. The container is the sole execution surface for all clone, merge,
lockfile, push, and agent operations. It is destroyed — not paused — after the run
completes or is cancelled.**

A `ContainerRuntime` enum resolved at worker startup selects between Podman and Docker.
The runtime is detected from `AMPEL_SANDBOX_RUNTIME` env var (`podman` | `docker`) or
auto-detected from PATH.

### Container Invocation

```bash
podman run --rm \
  --read-only \
  --tmpfs /workspace \
  --tmpfs /tmp \
  --env-file /run/secrets/ampel-remediation-<run-id>  \  # tmpfs-backed, erased after
  --security-opt no-new-privileges \
  --network ampel-egress \
  --user 1000:1000 \
  ampel/sandbox:<tag> \
  /entrypoint.sh <run-id>
```

The env-file is written to a `tmpfs` mount on the host, not to disk, and is securely
erased (zeroed + unlinked) after the container exits.

### Network Egress Allowlist

A dedicated container network `ampel-egress` is created at worker startup with
iptables/nftables rules permitting only:

- Provider API: `api.github.com`, `gitlab.com`, `api.bitbucket.org`
- npm: `registry.npmjs.org`
- Cargo: `crates.io`, `static.crates.io`
- Go: `proxy.golang.org`, `sum.golang.org`
- PyPI: `pypi.org`, `files.pythonhosted.org`
- RubyGems: `rubygems.org`, `gems.ruby-lang.org`
- Maven Central: `repo1.maven.org` (future)

All other egress is `REJECT`ed.

### Sandbox Image (`docker/sandbox/Dockerfile`)

```dockerfile
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y \
    git curl build-essential python3 python3-pip ruby bundler ca-certificates
# Node 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash && apt-get install -y nodejs
RUN npm install -g pnpm yarn
# Rust
RUN curl https://sh.rustup.rs | sh -s -- -y --no-modify-path
# Go
RUN curl -LO https://go.dev/dl/go1.23.linux-amd64.tar.gz && \
    tar -C /usr/local -xzf go*.tar.gz && rm go*.tar.gz
# Poetry
RUN pip3 install poetry
RUN useradd -u 1000 -m sandbox
USER sandbox
WORKDIR /workspace
```

### macOS Development Fallback

On developer machines, if neither `podman` nor `docker` is on PATH, the worker logs a
warning and falls back to git-worktree + process isolation (adequate for Phase 2
mechanical consolidation; insufficient for Phase 4 agentic tier). The worker startup
health check emits a warning metric when running in fallback mode.

---

## Alternatives Considered

### Option A: Git worktree + process isolation only (Rejected)

**Approach**: `git worktree add` creates an ephemeral working tree; PAT injected via
`git config credential.helper`; lockfile commands run as child processes on the host.

**Pros**: Zero container runtime dependency; sub-millisecond startup; cross-platform.

**Cons**:
- ❌ No network egress control — lockfile commands can reach any host
- ❌ No filesystem isolation outside the worktree
- ❌ Insufficient for Phase 4 agentic tier — prompted agent can execute arbitrary
  commands on the host
- ❌ Credential leakage via `/proc/self/environ`

**Verdict**: REJECTED — adequate for Phase 2 but fails Phase 4. Retained as dev fallback
only.

### Option B: Rootless Podman/Docker per run (ACCEPTED)

**Pros**:
- ✅ Strong filesystem and process isolation
- ✅ Network egress enforced at container network layer
- ✅ Credential injected via tmpfs, never written to disk
- ✅ Rootless — no root escalation on worker host
- ✅ ~0.5–2 s startup (pre-pulled image)
- ✅ Works on Fly.io (Linux) and macOS dev (Podman Desktop)
- ✅ Sufficient for Phase 4 agentic tier

**Cons**:
- ⚠️ Worker machines must have Podman or Docker installed
- ⚠️ Sandbox image must be built, tested, and published on each release
- ⚠️ 0.5–2 s startup overhead per run (negligible vs minutes-long runs)

**Verdict**: ACCEPTED.

### Option C: nsjail / Linux namespaces (Rejected)

**Approach**: Linux namespace sandboxing (user/network/mount/PID/IPC) via `nsjail` or
`unshare`, with a seccomp filter.

**Pros**: Kernel-native, minimal overhead (~10 ms), maximum security.

**Cons**:
- ❌ Linux-only — macOS development requires a separate fallback
- ❌ `nsjail` is not in standard Ubuntu repos; requires building from source
- ❌ Seccomp filter must enumerate every syscall used by `cargo`, `npm`, `go`, etc.;
  high maintenance burden

**Verdict**: REJECTED — Linux-only constraint and custom tooling burden outweigh benefits.

---

## Trade-off Analysis

| Aspect | Option A (worktree) | Option B (Podman) ⭐ | Option C (nsjail) |
|--------|--------------------|--------------------|-------------------|
| **Egress control** | ❌ None | ✅ Container network | ✅ Network namespace |
| **Filesystem isolation** | ⚠️ Partial | ✅ Full | ✅ Full |
| **Credential safety** | ❌ Host env | ✅ tmpfs env-file | ✅ Namespace |
| **Agentic tier safe** | ❌ No | ✅ Yes | ✅ Yes |
| **Startup overhead** | < 1 ms | 0.5–2 s | ~10 ms |
| **macOS dev support** | ✅ Native | ✅ Podman Desktop | ❌ No |
| **Root required** | No | No (rootless) | No (capabilities) |
| **Operational complexity** | Low | Medium | High |

---

## Consequences

### Positive

- Egress is enforced at the network level — lockfile commands cannot call arbitrary hosts
- Credentials are never written to disk in plaintext
- Container destruction is guaranteed on run completion or worker crash
- Agentic tier (Phase 4) can safely run agent `run_command` calls within the allowlist
- Rootless operation means worker compromise does not yield host root

### Negative

- Worker machines must have Podman or Docker; adds an infrastructure prerequisite
- Sandbox image must be maintained, built, and pushed on each release
- 0.5–2 s container startup added to every run
- macOS developers without a container runtime get reduced-isolation behaviour (logged)

### Neutral

- Sandbox image version is pinned per Ampel release; out-of-date images are known risk
- Phase 2 (mechanical) and Phase 4 (agentic) both use the same container image

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Container runtime unavailable on worker | High | Health check at worker startup; fail with clear error and metric |
| Egress allowlist too narrow (blocks package registry) | Medium | Configurable `SANDBOX_EGRESS_ALLOWLIST` env var; default covers all major ecosystems |
| Image pull failure at run time | Medium | Pre-pull at worker startup; fail run, not worker |
| Prompt injection causes container breakout | Low | Rootless + default seccomp profile; kernel exploit required |
| macOS dev gets no isolation | Low | Clearly documented; production always uses container runtime |
| tmpfs env-file not erased on worker crash | Low | Systemd unit / Fly.io machine lifecycle cleans `/run/secrets/` on restart |

---

## Related ADRs

- ADR-002: `RemediationCapable` supertrait — write primitives called from worker after
  container push completes
- ADR-005: Octopus merge via subprocess git — subprocess `git` runs inside this container
- ADR-007: `ModelProvider` trait — agent-kind providers receive the worktree path from
  within this container in Phase 4
- ADR-010: CI verification TOCTOU guard — verification runs from the worker process;
  container responsibility ends at `git push`
