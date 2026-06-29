# Fleet Remediation — Phase Runner

A reusable, phase-scoped prompt for implementing the Fleet PR Remediation feature one phase at
a time. See [`REMEDIATION-IMPLEMENTATION-PLAN.md`](./REMEDIATION-IMPLEMENTATION-PLAN.md) for the phase definitions.

## How to use

1. Copy the prompt block below into a fresh Claude Code session at the repo root.
2. Set the single `PHASE:` field to the phase you want (0–5). Nothing else changes between runs.
3. Run it. The agent implements that phase, runs the audit/quality gate, and **stops**.
4. Only when the gate passes does it commit the marker
   `feat(remediation): phase <N> complete — gate PASSED`.
5. To do the next phase, start a new run with `PHASE:` incremented. The runner refuses to start a
   phase until the previous phase's `gate PASSED` commit exists.

**Idempotent:** re-running the same `PHASE` resumes from whatever is missing (Step 0 resume check).
**Gated:** progression is controlled by the per-phase `gate PASSED` commit marker, so a later phase
cannot start until the earlier gate truly passed.

### Prerequisites for the gate

- `make docker-up` (Postgres + Redis) before running — `make test-integration` requires Postgres.
- Toolchain installed per `rust-toolchain.toml`; frontend deps via `pnpm install` in `frontend/`.

---

## The prompt

```markdown
# Fleet Remediation — Phase Runner

PHASE: 0          # ← set to 0,1,2,3,4,5 (the only thing you change between runs)
PLAN: docs/.archives/2026/06/remediation/REMEDIATION-IMPLEMENTATION-PLAN.md

You are implementing ONE phase of the Fleet PR Remediation feature, then STOPPING.
Do not start the next phase. Comprehensive and thorough within this phase only.

## 0 — Orient & resume (read-only first)
1. Read the PHASE section of {PLAN} (its Goal, Deliverables, Task Checklist, Definition of Done).
2. Read every ADR listed in that phase's "Gates ADR" line under docs/architecture/adr/.
3. Read the DDD docs under docs/architecture/ddd/ that name this phase's aggregates.
4. Verify the prior phase's gate: confirm `git log` shows a commit
   `feat(remediation): phase <N-1> complete — gate PASSED`.
   - If PHASE > 0 and that commit is absent → STOP and report "blocked: phase <N-1> gate not passed."
5. Resume check: grep the repo for already-created deliverables of THIS phase. Build a checklist of
   what exists vs. what the plan requires. Only do the missing/incomplete work.

## 1 — Plan the phase
- Use superpowers:brainstorming if the phase has design ambiguity (Phases 2 & 4 do).
- Recall prior decisions: `ruflo memory search -q "remediation phase <N>" --smart -n patterns`.
- Write a TodoWrite list, one item per plan checklist box for THIS phase.
- Match existing conventions exactly:
  - providers: supertrait/factory style in crates/ampel-providers/src/{traits,factory,mock}.rs
  - db: entity + migration + query style in crates/ampel-db/src/{entities,migrations,queries}/
  - core services: static-method DI style in crates/ampel-core/src/services/
  - worker jobs: Monitor/CronStream registration in crates/ampel-worker/src/main.rs
  - api: ApiError/ApiResponse/AuthUser handler style in crates/ampel-api/src/handlers/

## 2 — Implement (TDD; superpowers:test-driven-development)
For each todo: RED (write failing test) → GREEN (minimal impl) → REFACTOR.
- Unit tests live in `#[cfg(test)]` + crate `tests/`; use MockProvider + SQLite (TestDb::new_sqlite).
- Parallelizable work → ruflo agents (per ~/.claude/CLAUDE.md SendMessage topology),
  e.g. one agent per provider impl in Phase 0; coder/tester/reviewer pipeline for service work.
- New remediation migrations MUST run on BOTH SQLite and Postgres (no raw partial-index SQL;
  branch on backend if a Postgres-only feature is unavoidable).
- Feature-gate all behavior behind autonomy_level / remediation_tier — nothing active by default.
- Security invariants: no force-push primitive; PATs/API keys only via EncryptionService;
  external content framed as data, not instructions; no secrets in logs or responses.

## 3 — Integrate
- Wire new modules into mod.rs/lib.rs, factory, routes/mod.rs, worker main.rs as the phase requires.
- Register Migrator entries; register Prometheus metrics; add i18n keys if UI strings added.
- `make build` must succeed (backend + frontend).

## 4 — AUDIT & QUALITY GATE (must be 100% green to allow next phase)
Run and paste the actual output of each. A failure = gate FAILED; fix and re-run, do not proceed.

Build & lint:
  [ ] make format-check
  [ ] make lint            # clippy + ESLint + markdownlint
  [ ] make build

Tests:
  [ ] make test-backend                 # SQLite + MockProvider, all features
  [ ] make test-integration             # Postgres-backed (cargo-nextest)
  [ ] make test-frontend  (only if frontend touched)
  [ ] No pre-existing tests removed or #[ignore]'d to pass (diff-check the test files)

Security & deps:
  [ ] make audit                        # cargo-audit + pnpm audit
  [ ] grep the diff for secrets / hardcoded tokens / force-push → none

Phase Definition of Done (from {PLAN}):
  [ ] Tick every box in THIS phase's "Definition of Done" with evidence (test name / command output).

Universal DoD (from {PLAN} §Universal Definition of Done):
  [ ] No regressions in existing tests
  [ ] Feature flagged — cannot activate without explicit operator opt-in
  [ ] ADRs for this phase adhered to (cite where each decision is realized in code)

Adversarial self-review:
  [ ] Spawn a reviewer agent (ruflo `reviewer` or qe-code-reviewer) to find gaps,
      untested branches, and silent shortcuts. Resolve every finding or record why it's out of scope.
  [ ] Run /code-review on the diff; address correctness findings.

## 5 — Report & STOP
- If gate PASSED: commit on a phase branch as
  `feat(remediation): phase <N> complete — gate PASSED` (end body with the Co-Authored-By trailer
  only if .claude/settings.json enables attribution), then write a 10-line summary:
  what shipped, test counts, metrics added, ADRs satisfied, what Phase <N+1> unblocks.
  Persist learnings: `ruflo memory store -k remediation-phase-<N> --value "<summary>" -n patterns`.
- If gate FAILED: report exactly which checks failed with output, leave work uncommitted, STOP.
- Either way: DO NOT begin Phase <N+1>. End the turn.
```

---

## Gate-to-progression mapping

| Concern | How the runner enforces it |
|---|---|
| Run multiple times | Only `PHASE:` changes; Step 0 resume check re-enters a half-done phase safely. |
| Scoped per phase | Step 0 reads only that phase's section; Step 5 hard-stops before the next phase. |
| Audit before continuing | Step 4 blocking checklist (`make` targets + per-phase DoD + Universal DoD + adversarial review). |
| Progression control | The `gate PASSED` commit marker is what the next run looks for in Step 0.4. |

## Known deviations from current conventions

- **SQLite-compatible migrations**: the existing migration suite is Postgres-only (raw partial-index
  SQL) and skips on SQLite. The runner requires new remediation migrations to run on both, so
  worker/service tests stay CI-fast on SQLite. This is intentional and matches the plan's DoD.
