# Fleet Remediation — Long-Horizon Phase Orchestrator

A goal-driven, memory-efficient driver that completes **all** phases (0→5) of the Fleet PR
Remediation feature across many sessions, using a hierarchical-mesh swarm. It wraps the
per-phase gate in [`REMEDIATION-PHASE-RUNNER.md`](./REMEDIATION-PHASE-RUNNER.md) in an outer loop so the full ~13-week
scope finishes without any single session holding all the context.

See [`REMEDIATION-IMPLEMENTATION-PLAN.md`](./REMEDIATION-IMPLEMENTATION-PLAN.md) for the phase definitions.

## Why this exists

The scope spans 6 phases / ~13+ weeks — far more than one context window. So state must live
**outside** the conversation. The orchestrator enforces a three-tier memory model:

| Tier | Holds | Mechanism | Survives context reset? |
|---|---|---|---|
| **Durable** | Phase completion, decisions, gotchas | git `gate PASSED` markers + `ruflo memory` namespace `remediation` + beads issues | ✅ yes — the real memory |
| **Per-phase working set** | Only the active phase's plan slice + its gated ADRs + named DDD aggregates | read on demand at phase start | ♻️ rebuilt each phase |
| **Ephemeral / agent-local** | One worker's single task | swarm agents report *compact* results via SendMessage, never file dumps | ❌ discarded after phase |

The load-bearing rule: **never load all of the planning docs.** Each phase reads only
`REMEDIATION-IMPLEMENTATION-PLAN.md`'s section for phase N + the ADRs in that phase's "Gates ADR" line +
the DDD aggregates it names. The durable tier carries everything else as compact summaries.

## Agentic-QE integration

The base gate proves *tests pass*, not that *tests are good* or the *security surface is sound* —
and this feature autonomously merges PRs and runs model-driven edits in a sandbox, so weak tests
mean autonomous wrong merges. A QE fleet (`fleet_init`, called once per phase) layers measured
quality onto the gate. Lightweight checks run every phase; the expensive adversarial passes are
gated to where the risk actually lives:

| AQE capability | Slots into | Runs in | Why |
|---|---|---|---|
| `requirements_validate` | STEP A.5 | every phase | Make the DoD testable before coding |
| `qe-test-architect` / `test_generate_enhanced` | STEP C | every phase | Seed RED with edge/boundary cases |
| `coverage_analyze_sublinear` | STEP D.2 | every phase | Risk-weighted gap = 0 on changed files |
| `qe-mutation-tester` | STEP D.2 | **Phases 2 & 4** | Prove the suite kills bugs (merge/agentic logic) |
| `security_scan_comprehensive` | STEP D.3 | every phase | SAST + secrets + deps |
| `qe-pentest-validator` ("No Exploit, No Report") | STEP D.3 | **Phases 2 & 4** | Sandbox/PAT/egress + prompt-injection/key-leak |
| `qe-chaos-engineer` | STEP D.4 | **Phase 2** | Sandbox crash / CI TOCTOU → clean handoff_human |
| `qe-deployment-advisor` | STEP D.5 | autonomy ramp | Go/no-go per dry_run→consolidate_only→auto_merge |
| `quality_assess` | STEP D.5 | every phase | Aggregate D.1–D.4 into one go/no-go |

QE signals (coverage, mutation score, exploits, flaky tests, chaos verdicts) are persisted to the
durable tier (`namespace: remediation-qe`) — feeding Phase 5b strategy learning and future-phase recall.

## Two modes

- **Mode A — Reviewed (stop after each phase).** The default below. Drives one phase, commits a
  local `gate PASSED` marker, and STOPS for you to inspect before the next phase. Use when you
  want a human checkpoint between phases.
- **Mode B — Autonomous PR/CI (fire-and-forget).** See [Autonomous PR/CI mode](#autonomous-prci-mode-fire-and-forget)
  below. Issue once and walk away: per-phase branch → commits → push → open PR → monitor CI →
  bounded fix-loop until green → squash-merge into the long-running **`develop`** branch → next
  phase. `main` is never touched autonomously; when all phases land, the agent opens one
  `develop`→`main` PR and stops for your decision.

## How to use (Mode A)

1. Ensure infra is up each run: `make docker-up` (the gate's `make test-integration` needs Postgres).
2. Drive it with the `/loop` skill in **self-paced** mode (no interval): paste the prompt block
   below as the loop task. Each firing drives exactly one phase, then stops.
3. The git `gate PASSED` markers make it resumable forever — a new firing reads the markers,
   finds the lowest incomplete phase, runs it, audits, advances. When all 6 are gated it does one
   optimization pass and ends the loop.

**Idempotent:** re-running re-enters a half-done phase via the Step A.4 resume check.
**Gated:** a phase advances only on a real green gate (the `REMEDIATION-PHASE-RUNNER.md` Step 4 checklist).
**Goal-driven:** the loop terminates only when the GOAL state is reached, not after a fixed count.

### `/loop` vs `/goal` — use both, at different scopes

`/loop` and `/goal` operate on orthogonal axes and compose; they are not alternatives:

- **`/loop` (self-paced) = the OUTER driver.** Re-invokes this prompt across *separate, fresh
  contexts*, advancing one phase per firing. This is what makes the long horizon
  memory-efficient — each phase gets a small, clean context.
- **`/goal` = an INNER per-phase stop-guard.** `/goal` checks a condition *before allowing a run
  to stop*. Scope it to the **current phase's gate only**, e.g.:
  > "Do not stop until STEP D `quality_assess` is green AND the `gate PASSED` commit exists for this phase."
  This prevents the agent from declaring a phase done while a gate check is still red or a DoD box
  is unticked.

> ⚠️ **Do NOT set `/goal` to the whole 6-phase scope.** A whole-scope `/goal` ("all phases gated")
> would refuse to stop after each phase and try to grind all six into one ever-growing context —
> defeating the one-phase-per-context memory design. The whole-scope goal belongs to the `/loop`
> driver (it stops firing when STEP E omits the next wakeup), NOT to `/goal`.

| Mechanism | Axis | Scope | Answers |
|---|---|---|---|
| `/loop` (self-paced) | Starts a **new** run | Whole scope (phases 0→5 + optimize) | "Should I fire again?" |
| `/goal` | Keeps **one** run going | Current phase's gate only | "Am I allowed to stop yet?" |

---

## The prompt

```markdown
# Fleet Remediation — Long-Horizon Phase Orchestrator (hierarchical-mesh swarm)

GOAL: every phase 0→5 of the Fleet PR Remediation feature carries a
  `feat(remediation): phase <N> complete — gate PASSED` commit, with CI green and
  a final optimization pass done. You drive ONE phase per run, then STOP.
PLAN: docs/.archives/2026/06/remediation/REMEDIATION-IMPLEMENTATION-PLAN.md
RUNNER: docs/.archives/2026/06/remediation/REMEDIATION-PHASE-RUNNER.md   # per-phase gate spec — obey it verbatim
MEM_NS: remediation                                            # ruflo memory namespace for this scope

## STEP A — Locate current state (read-only, cheap)
1. `git log --oneline | grep "gate PASSED"` → find highest completed phase P.
   Current target phase N = P+1 (or 0 if none). If N > 5 → go to STEP E (optimize).
2. `ruflo memory search -q "remediation phase ${N}" --smart -n ${MEM_NS}` and
   `... -q "remediation gotchas conventions" --smart -n ${MEM_NS}` → recall prior decisions.
3. Read ONLY phase N's section of {PLAN} (Goal, Deliverables, Task Checklist, Definition of Done).
   Read ONLY the ADRs named in phase N's "Gates ADR" line, and ONLY the DDD aggregates phase N names.
   Do NOT read other phases or unrelated docs. This is the memory-efficiency contract.
4. Resume check: grep the repo for phase N's deliverables; build "exists vs. required" list.
   Only the missing/incomplete work is in scope.
5. Validate testability: requirements_validate on phase N's Definition of Done →
   if any DoD box is untestable, flag it and tighten the acceptance criteria BEFORE coding.

## STEP B — Stand up the hierarchical-mesh swarm + QE fleet
`ruflo swarm init --v3-mode`   # hierarchical-mesh topology, 15 agents, hybrid memory + HNSW
# Stand up the QE fleet alongside the swarm (call ONCE; reuse across the phase)
fleet_init({ topology: "hierarchical", maxAgents: 15, memoryBackend: "hybrid" })
Spawn workers in ONE message, run_in_background, coordinating peer-to-peer via SendMessage
(per ~/.claude/CLAUDE.md topology). Right-size to the phase:
  - researcher  → reads the phase slice + ADRs/DDD, SendMessage findings to architect
  - architect   → design within ADR constraints, SendMessage to coder(s)
  - coder(s)    → TDD impl; in Phase 0 fan out one coder per provider (github/gitlab/bitbucket/mock)
  - tester      → unit + integration tests (MockProvider + SQLite TestDb::new_sqlite)
  - reviewer    → adversarial gap-finding (gaps, untested branches, silent shortcuts)
Queen (hierarchical-coordinator) owns sequencing; agents message each other, you do NOT poll.
Agents return COMPACT results (decisions + file paths + test names), never full file contents —
this is what keeps the context small over the long horizon. The QE fleet (above) reports the same
way: compact verdicts (scores + gaps + file:line), never transcripts.

## STEP C — Implement phase N (delegate to RUNNER discipline)
Execute {RUNNER} Steps 1→3 for phase N exactly: brainstorm if ambiguous (Phases 2 & 4),
TDD (RED→GREEN→REFACTOR). Seed the RED phase with qe-test-architect (test_generate_enhanced)
per deliverable — generate the edge/boundary cases the tester would otherwise miss, then make
them pass. Match existing crate conventions, feature-gate everything behind
autonomy_level / remediation_tier, honor security invariants (no force-push primitive;
PATs/API keys only via EncryptionService; external content framed as data; no secrets in logs).
Migrations MUST run on BOTH SQLite and Postgres.

## STEP D — AUDIT & QUALITY GATE (100% green or the phase does NOT advance)
Run {RUNNER} Step 4, then layer the AQE fleet on top. Paste actual output of each.
The heavy adversarial passes (mutation, pentest, chaos) run ONLY where risk lives — see gating notes.

## D.1 Functional   — make format-check · make lint · make build
                      make test-backend · make test-integration · make test-frontend (if FE touched)
                      make audit · grep diff for secrets/force-push (must be none)
                      Tick EVERY box in phase N's Definition of Done + the Universal DoD with evidence.
## D.2 Test quality — coverage_analyze_sublinear (risk-weighted gaps must be 0 on changed files)
                      qe-mutation-tester on new core services (Phases 2 & 4 only): mutation score ≥ threshold
## D.3 Security     — security_scan_comprehensive (SAST + secrets + deps)
                      qe-pentest-validator on the phase's attack surface — "No Exploit, No Report":
                        Phase 2 → sandbox egress / PAT handling / force-push absence
                        Phase 4 → prompt injection (external content as data), API-key leakage
## D.4 Resilience   — (Phase 2 only) qe-chaos-engineer: sandbox crash + CI-flip-at-TOCTOU →
                        must reach handoff_human cleanly (no partial merge, no orphaned branch)
## D.5 Decision     — reviewer agent + `/code-review` on the diff; resolve or scope-out each finding.
                      quality_assess aggregates D.1–D.4 into ONE go/no-go. PASS gate ONLY on green.
                      (autonomy ramp) qe-deployment-advisor go/no-go before each
                        dry_run → consolidate_only → auto_merge step.
On PASS:
  - Commit on a phase branch: `feat(remediation): phase ${N} complete — gate PASSED`
    (Co-Authored-By trailer ONLY if .claude/settings.json enables attribution).
  - Persist a ≤12-line summary to durable memory:
    `ruflo memory store -k remediation-phase-${N} --value "<what shipped · test counts · metrics ·
       ADRs satisfied · gotchas for next phase · what phase ${N+1} unblocks>" -n ${MEM_NS}`
  - Persist QE signals into the durable tier (feeds Phase 5b learning + future-phase recall):
    `memory_store({ namespace: "remediation-qe", persist: true, key: "phase-${N}-signals",
       value: "<coverage % · mutation score · exploits-found · flaky tests · chaos verdicts>" })`
  - Report the summary, then STOP. (Next /loop firing picks up phase N+1 from the git marker.)
On FAIL:
  - Report exactly which checks failed with output. Leave work UNCOMMITTED. Store the blocker:
    `ruflo memory store -k remediation-phase-${N}-blocker --value "<failing check + cause>" -n ${MEM_NS}`
  - STOP. The next firing re-enters phase N at STEP A.4 resume check and continues.

## STEP E — Goal reached (all phases gated): optimization pass
Only when phases 0–5 all show `gate PASSED`:
  - `ruflo analyze boundaries crates/` to find refactor seams across the new code.
  - Swarm pass: dedup, simplify, tighten hot paths; `/simplify` then `/code-review` on the result.
  - Verify full `make test` green; commit `chore(remediation): cross-phase optimization — gate PASSED`.
  - Final memory write: `ruflo memory store -k remediation-COMPLETE --value "<summary>" -n ${MEM_NS}`.
  - Report completion and END THE LOOP (omit the next wakeup).

## INVARIANTS (every run)
- One phase per run. Never start phase N+1 in the same run that completed phase N.
- A phase advances ONLY on a real green gate. No skipped/ignored tests to force a pass.
- Load only the current phase's doc slice — durable memory carries the rest.
- The agent's success claim is never trusted: provider CI / the gate is the verifier.
```

---

## Autonomous PR/CI mode (fire-and-forget)

Issue once and walk away. **`main` is never written autonomously.** The existing long-running
`develop` branch is the base (CI already triggers on `develop` per `.github/workflows/ci.yml`);
each phase ships as its own branch → PR → green CI → squash-merge **into `develop`**. The durable
state marker is the **squash-merge commit on `develop`**, so every run re-derives the next phase
from `develop`'s history (plus any in-flight open PR) — no conversation memory needed. When all
phases land, the agent opens **one PR from `develop` to `main` and STOPS** — you decide whether to
merge the whole feature. Lower blast radius: a bad phase only touches `develop`.

**Prerequisites**
- `gh auth status` authenticated with push + PR + merge rights on the remote.
- `make docker-up` available to the gate; toolchain + `pnpm install` ready.
- Decided policy baked into the prompt: **merge gate = remote CI green only**; **fix budget = 5
  CI-fix iterations per PR, then STOP + handoff** (PR left open for a human).

**Drive it** with `/loop` self-paced for cross-session durability + fresh context per phase:
```
/loop Execute the AUTONOMOUS PR/CI prompt block in @docs/.archives/2026/06/remediation/REMEDIATION-PHASE-ORCHESTRATOR.md — run until GOAL or FIX_BUDGET exhaustion.
```
The in-phase CI wait uses a backgrounded `gh pr checks --watch`, so the agent idles (no token burn)
while CI runs and is re-invoked when checks finish.

```markdown
# Fleet Remediation — Autonomous PR/CI Orchestrator (fire-and-forget)

GOAL: every phase 0→5 is implemented on its own branch, opened as a PR, driven to GREEN CI, and
  SQUASH-MERGED into ${BASE} (the integration branch) carrying
  `feat(remediation): phase <N> complete — gate PASSED`, followed by a final optimization PR.
  When all land, open ONE PR ${BASE} → ${TRUNK} and STOP for human decision. `main` is NEVER
  merged autonomously. Issue once; run UNATTENDED until GOAL or a hard stop.
PLAN:   docs/.archives/2026/06/remediation/REMEDIATION-IMPLEMENTATION-PLAN.md
RUNNER: docs/.archives/2026/06/remediation/REMEDIATION-PHASE-RUNNER.md   # per-phase discipline — obey verbatim
TRUNK:  main                           # protected; only a human merges the final integration PR into it
BASE:   develop                        # long-running integration branch (CI already runs on it) — the autonomous base
MEM_NS: remediation
MERGE_GATE:  remote CI green only      # all required PR checks pass → merge. CI is the merge authority.
FIX_BUDGET:  5                         # CI-fix iterations per PR; on exhaustion STOP + handoff (PR left open)

## STEP A — Locate state (read-only, cheap)
0. Ensure the integration branch exists (idempotent): if `git rev-parse --verify ${BASE}` fails,
   create it from fresh trunk: `git checkout ${TRUNK} && git pull --ff-only &&
   git checkout -b ${BASE} && git push -u origin ${BASE}`. Otherwise `git checkout ${BASE} && git pull --ff-only`.
1. (integration branch now checked out and current)
2. `git log --oneline ${BASE} | grep "gate PASSED"` → highest MERGED phase P → target N = P+1.
   If N > 5 → STEP E (optimization PR).
3. In-flight RESUME check (ordered — covers every interruption window before merge):
   a. Open PR? `gh pr list --state open --head "remediation/phase-${N}"`.
      → If a phase-${N} PR exists → skip to STEP D and resume its CI/fix/merge cycle.
   b. Branch but no PR? `git ls-remote --exit-code --heads origin remediation/phase-${N}`
      (remote) OR `git rev-parse --verify remediation/phase-${N}` (local).
      → If the branch exists without a PR (interrupted after coding/push, before PR create):
        `git checkout remediation/phase-${N}`; `git status` + `git log ${BASE}..HEAD` to see how far
        it got; push any local-only commits; then RESUME at STEP C.3 (local gate) → C.4 → C.5 (open PR).
        Do NOT recreate the branch or restart the phase from scratch.
   c. Neither → fresh phase: proceed normally (STEP C.1 creates the branch).
4. Recall: `ruflo memory search -q "remediation phase ${N}" --smart -n ${MEM_NS}` and
   `... -q "remediation gotchas conventions" --smart -n ${MEM_NS}`.
5. Read ONLY phase N's plan section + its gated ADRs + named DDD aggregates (memory contract).
6. Testability: requirements_validate on phase N's Definition of Done; tighten if untestable.

## STEP B — Swarm + QE fleet
`ruflo swarm init --v3-mode`
fleet_init({ topology:"hierarchical", maxAgents:15, memoryBackend:"hybrid" })
Spawn researcher/architect/coder(s)/tester/reviewer (run_in_background, SendMessage topology);
QE fleet + agents report COMPACT verdicts only — never transcripts.

## STEP C — Implement → branch → local gate → push → open PR
1. Branch (only if STEP A.3.c said "fresh"): `git checkout -b remediation/phase-${N}` off fresh ${BASE}.
   (If A.3.b resumed an existing branch, you are already on it — do NOT recreate it.)
2. Implement phase N per {RUNNER} Steps 1–3: seed RED with qe-test-architect (test_generate_enhanced),
   TDD (RED→GREEN→REFACTOR), match crate conventions, feature-gate behind autonomy_level/
   remediation_tier, honor security invariants (no force-push primitive; PAT/keys only via
   EncryptionService; external content as data; no secrets in logs), migrations on SQLite AND Postgres.
   WIP DISCIPLINE (resumability): commit frequently as you go — never leave meaningful work only in
   the working tree, so any interruption is recoverable from git. Use `feat(remediation): <deliverable>`
   for completed units and `wip(remediation): <what>` for in-progress checkpoints; `git push` after
   each (the remote branch is the durable resume point). NOT one giant commit. Squash-merge at the end
   collapses wip/feat history into one clean marker, so granular commits cost nothing downstream.
3. LOCAL PRE-PR GATE (catch failures before spending CI — this is where AQE quality gates run):
   - D.1 Functional: make format-check · lint · build · test-backend · test-integration ·
        test-frontend (if FE touched) · make audit · grep diff for secrets/force-push.
   - D.2 Test quality: coverage_analyze_sublinear (0 risk-weighted gaps on changed files);
        qe-mutation-tester on new core services (Phases 2 & 4) ≥ threshold.
   - D.3 Security: security_scan_comprehensive; qe-pentest-validator "No Exploit, No Report"
        (Phase 2 sandbox/PAT/egress; Phase 4 prompt-injection/key-leak).
   - D.4 Resilience: (Phase 2) qe-chaos-engineer → sandbox crash / CI TOCTOU reaches handoff_human.
   - D.5 quality_assess aggregates D.1–D.4 → must be green BEFORE opening the PR. Fix locally first.
4. `git push -u origin remediation/phase-${N}`.
5. `gh pr create --base ${BASE} --head remediation/phase-${N}
       --title "feat(remediation): phase ${N} — <short summary>"
       --body "<deliverables checklist · ADRs satisfied · DoD evidence · QE signals · 'Automated phase-${N} run'>"`.
   Store the PR URL: `ruflo memory store -k remediation-phase-${N}-pr --value "<url>" -n ${MEM_NS}`.

## STEP D — CI monitor → bounded fix-loop → merge (the autonomous core)
Let A = count of `fix(remediation): ci attempt` commits already on this branch
   (`git log ${BASE}..HEAD --grep "ci attempt" --oneline | wc -l`) — the durable attempt counter.
1. WAIT FOR CI in the BACKGROUND: `gh pr checks <pr> --watch --fail-fast`.
   Long-running; the harness re-invokes you when it exits. Do NOT busy-poll.
2. On exit, branch on result:
   - ALL REQUIRED CHECKS GREEN →
       `gh pr merge <pr> --squash --delete-branch`
          (squash subject = `feat(remediation): phase ${N} complete — gate PASSED`).
       MERGE_GATE is remote CI only — do NOT re-run the local gate here; CI already passed.
       `git checkout ${BASE} && git pull --ff-only`.
       Persist: `ruflo memory store -k remediation-phase-${N} --value "<≤12-line summary>" -n ${MEM_NS}`
       and `memory_store({ namespace:"remediation-qe", persist:true, key:"phase-${N}-signals",
          value:"<coverage % · mutation · exploits · flaky · chaos>" })`.
       Loop back to STEP A for phase N+1.
   - ANY CHECK RED →
       a. If A ≥ FIX_BUDGET → STOP THE WHOLE LOOP. Leave the PR OPEN. Write handoff:
          `ruflo memory store -k remediation-phase-${N}-handoff
             --value "<failing checks + last failed logs + everything tried over A attempts>" -n ${MEM_NS}`.
          Report the blocker with output and END. Do not merge. Do not advance.
       b. Else (A < FIX_BUDGET) → DIAGNOSE & FIX ON THIS BRANCH ONLY:
          - `gh pr checks <pr>` → failed check names; `gh run view <run-id> --log-failed` → logs.
          - Classify the failure; use the swarm (coder + tester) to fix; re-run the relevant
            local gate slice to confirm locally.
          - Commit `fix(remediation): ci attempt #<A+1> — <root cause>`. `git push`.
          - Go to D.1 (re-watch). A increments automatically via the commit grep.

## STEP E — Optimization PR, then hand off to human (only when N > 5)
1. Run the same branch→local-gate→push→PR→CI→merge cycle on `remediation/optimization`
   (base = ${BASE}): `ruflo analyze boundaries crates/`; swarm dedup/simplify; `/simplify` then
   `/code-review`; squash-merge into ${BASE} with subject
   `chore(remediation): cross-phase optimization — gate PASSED`.
2. FINAL HANDOFF — do NOT merge ${TRUNK}. Open the integration PR for human review:
   `gh pr create --base ${TRUNK} --head ${BASE}
       --title "feat(remediation): Fleet PR Remediation — full feature (phases 0–5)"
       --body "<per-phase summary table · all gate markers · QE signals · ADRs satisfied ·
               'Ready for human review; do NOT auto-merge'>"`.
   Persist `ruflo memory store -k remediation-COMPLETE --value "<integration PR url + summary>" -n ${MEM_NS}`.
   Report the integration PR URL and END THE LOOP. The human decides whether to merge into ${TRUNK}.

## INVARIANTS (every run)
- UNATTENDED: never wait for human input DURING phases. The only stop conditions are GOAL reached
  (→ integration PR opened, human-gated) or FIX_BUDGET exhausted on a phase PR (→ handoff).
- `main`/${TRUNK} is NEVER merged autonomously — only a human merges the final integration PR.
- One phase = one branch = one PR = one squash-merge marker on ${BASE} (the integration branch).
- A phase advances ONLY after its PR is GREEN and merged into ${BASE}. NEVER merge a red or
  required-missing PR.
- NEVER force-push to ${BASE} or ${TRUNK}. The fix-loop pushes only to the phase branch.
- State = ${BASE} merge markers + open-PR list + ruflo memory — re-derived every run, never assumed.
- The agent never self-certifies: remote CI is the merge authority for phase PRs.
```

## Relationship to REMEDIATION-PHASE-RUNNER.md

| Concern | REMEDIATION-PHASE-RUNNER.md (per-phase) | This orchestrator (long-horizon) |
|---|---|---|
| Scope per run | One phase, set manually via `PHASE:` | One phase, **discovered** from git markers |
| Progression | Human increments `PHASE:` and re-runs | `/loop` self-pacing advances automatically on green gate |
| Gate | Step 4 blocking checklist | Delegated **verbatim** to RUNNER Step 4 |
| Memory | Per-run `ruflo memory store` | Three-tier model; durable tier drives resume + recall |
| Parallelism | Optional ruflo agents | Hierarchical-mesh swarm (`--v3-mode`) standard per phase |
| Quality assurance | `make test*` + reviewer + `/code-review` | + AQE fleet: coverage, mutation, SAST/pentest, chaos, `quality_assess` go/no-go |
| Termination | Stops after each phase | Loops until GOAL state, then optimization pass, then ends |

The orchestrator does not replace the runner — it wraps it. The runner remains the source of
truth for what a passing gate means.
