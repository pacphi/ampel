The bounded context document has been written to:

`/Users/cphillipson/Development/active/ai/ampel/docs/architecture/ddd/bounded-context-fleet-remediation.md`

The document covers all requested sections:

**1. Bounded Context Definition**
Defines Fleet Remediation as the context that owns the decision to consolidate, remediate, and merge PRs autonomously. Maps each concern to its crate and source path. States the trigger condition (> 3 open PRs) and the language the context speaks.

**2. Context Map**
Shows the three-context landscape with ASCII diagram: PR Dashboard (upstream, conformist read path) → Fleet Remediation (downstream) → CICD Automation Intelligence (planned peer). Integration patterns are tabulated: conformist for reads, anti-corruption layer for writes, open-host/published-language for the intelligence integration. Explains why the conformist choice is deliberate on the read path.

**3. Ubiquitous Language**
Table of 13 terms with precise definitions: Run, Policy, Consolidation, Remediation, Verification, Handoff, Disposition, Playbook, ModelProvider, Sandbox, FailureClass, Budget, EgressClass, AirGapped.

**4. Core Domain Model**
Rust structs and enums grounded in the actual codebase: `RemediationRun` and `RemediationPolicy` aggregate roots, value objects (`RunState`, `AutonomyLevel`, `RemediationTier`, `Disposition`, `VerificationStatus`, `FailureClass`, `AgentBudget`, `PolicyScope`, `PrSelectionCriteria`), and domain entities (`RunPr`, `AgentSession`).

**5. Anti-Corruption Layer**
Three components: `RemediationCapable` supertrait (full Rust trait definition with all write primitives, per-provider notes for GitHub/GitLab/Bitbucket/Mock), `PolicyResolver` (scope-hierarchy translation), and `RemediationPrView` (read-only projection preventing tight coupling to PR model evolution).

**6. Integration with CICD Automation Intelligence**
Two seams: `FingerprintSource` trait + `RepoFingerprint` published language for lockfile-regen command selection, and `RemediationOutcomeSignal` for reflexion/vector-DB strategy learning. Both are defined as planned interfaces with a local-table fallback until the intelligence context ships.

**7. Context Boundaries**
Explicit exclusion table covering 9 concerns: provider auth, user management, org/team structure, repository registration and polling, PR data ingestion, AmpelStatus computation, notification dispatch, frontend auth middleware, and CI/CD workflow generation.

**8. Application Services and Domain Events**
Service table (7 services with crate locations and responsibilities) and domain event table (8 events with triggers and consumers).

**9. Invariants and Business Rules**
10 numbered rules enforced by `RemediationService`: one active Run per repo, merge only on unambiguous green, sources closed only after merge, policy must be enabled, air-gapped repos barred from external providers, deterministic branch names, DryRun produces zero writes, agent cannot self-certify, excluded labels always honored, budget axes are hard ceilings.

**10. Crate Placement Guide**
Complete table mapping every new artifact to its crate and file path within the existing five-crate layout — no new crates introduced.
