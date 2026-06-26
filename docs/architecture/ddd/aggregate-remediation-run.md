DDD aggregate design document for `RemediationRun` written to the path above.

The document covers all requested sections:

1. Aggregate boundary — `RemediationRun` root owns `RemediationRunPr[]` child entities; all mutations flow through the root.

2. Full state machine — 9 active/transitional states plus 5 terminal states (`completed`, `handoff_human`, `failed`, `cancelled`, `no_op`), with an ASCII flow diagram.

3. State transition reference — every edge documented with guard conditions, action performed, output recorded, and failure path. Includes TOCTOU re-verification on both the `merging` and `verifying→merging` transitions.

4. Key fields table — all 20+ fields with types and descriptions matching the requested field list.

5. Child entity — `RemediationRunPr` with the `Disposition` enum (`Consolidated`, `ClosedWithRef`, `SkippedConflict`, `LeftOpen`).

6. Commands — full `RemediationRunCommand` enum covering all 22 commands in the spec.

7. Invariants — 6 invariants documented: one-run-per-repo lock (DB partial unique index + advisory lock), deterministic branch name formula, `closing_sources` requires `merged=true`, `handoff_human` disposition completeness, attempts ceiling, and TOCTOU SHA check.

8. Rust types — `RunState` enum with `is_terminal()`/`is_active()` helpers; `TriggeredBy`, `RemediationTier`, `CiStatus`, `MergeStrategy`, `Disposition` enums; `RemediationRun` struct; `RemediationRunPr` struct; `RemediationRunError` (thiserror-based).

9. SeaORM entity sketches for `remediation_runs` and `remediation_run_prs` tables, following existing project entity conventions.

10. Repository lock protocol — SQL partial unique index DDL + advisory lock pseudocode.

11. Integration points table — `RepositoryPollJob`, `RemediationRunJob`, `GitProvider`, SSE endpoint, sandbox container, AI model provider, `RemediationPolicy`.

12. Open questions for implementors — partial consolidation ratio, approval timeout, branch protection bypass, concurrent org-level ceilings, and `RecordMerge` idempotency.
