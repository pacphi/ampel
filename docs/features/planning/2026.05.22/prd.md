# Product Requirements Document
## Ampel Upgrade Intelligence

**Version:** 1.0  
**Date:** 2026-05-22  
**Status:** Draft for Review

---

## 1. Executive Summary

Software teams managing repositories across multiple Git providers face a fragmented, manual, and non-learning dependency upgrade process. Renovate and Dependabot automate version bumping for individual repos but provide no fleet-level intelligence, no polyglot code migration capability, no cross-provider management, and no mechanism for learning from upgrade outcomes over time.

**Ampel Upgrade Intelligence** extends the existing Ampel PR management platform into a self-improving, multi-provider, polyglot upgrade orchestration system. It auto-discovers ecosystems across a managed fleet, generates deterministic upgrade plans, creates and groups PRs across any Git provider, validates via CI gates, and continuously learns from every outcome — making each subsequent upgrade faster, safer, and more accurate.

The system is **self-hosted**, **privacy-preserving**, and **learning-first**: it improves without requiring manual tuning, retraining jobs, or vendor data sharing.

---

## 2. Problem Statement

### 2.1 Current Pain Points

**For individual developers managing multiple repos:**
- Context-switching across GitHub, GitLab, and Bitbucket to track pending upgrades
- No visibility into which repos are most at risk from a given CVE
- Wasted CI cycles on upgrades that were predictably incompatible
- No learning from previous upgrade outcomes — every upgrade starts from scratch

**For platform/DevOps teams managing dozens of repos:**
- No fleet-level view of dependency staleness across ecosystems
- Duplicate effort: the same Spring Boot 3→4 migration done independently in 15 repos
- No automated detection of which repos are upgrade-ready vs. which need code migration
- Manual PR creation for routine patch/minor updates

**For enterprise teams managing hundreds to thousands of repos:**
- Security exposure from untracked CVEs across a large fleet
- No mechanism for controlled rollout (canary → waves → full fleet)
- No audit trail of who upgraded what, when, with what outcome
- Inability to benefit from upgrade patterns learned across the organization

### 2.2 Gap in Existing Tools

| Tool | What It Does | What It Misses |
|------|-------------|----------------|
| Renovate | Version-bump PRs for 90+ ecosystems | GitHub-centric SaaS; no fleet intelligence; no learning |
| Dependabot | Version-bump PRs on GitHub | GitHub only; no cross-repo learning; no code migration |
| OpenRewrite | AST-level Java code transformation | No PR creation; no scheduling; no multi-repo orchestration |
| Moderne | Fleet-scale OpenRewrite | Enterprise SaaS pricing; no self-hosted multi-provider story |
| Snyk | CVE-driven upgrades | Vulnerability-only; not a general upgrade scheduler |

**No existing tool** provides: self-hosted + multi-provider + polyglot ecosystems + code migration + learning from outcomes + privacy-preserving fleet intelligence.

---

## 3. Goals and Non-Goals

### 3.1 Goals

**G1 — Polyglot auto-discovery:** Automatically detect all ecosystems present in a managed repo (Java, Kotlin, Node.js, Python, Go, Rust, Ruby, .NET, Docker, Kubernetes, GitHub Actions, and more) without user configuration.

**G2 — Deterministic upgrade planning:** Generate upgrade plans that are reproducible, idempotent, and verifiable — producing the same diff given the same inputs. Always target live-resolved latest versions, not static pinned tables.

**G3 — Multi-provider PR orchestration:** Create, group, and manage upgrade PRs across GitHub, GitLab, Bitbucket, Azure DevOps, and Gitea/Forgejo using a unified control plane.

**G4 — CI-gated validation:** Block auto-merge until all required CI checks pass, minimum release age has elapsed, and supply chain checks are clear.

**G5 — Self-learning triage:** Every upgrade outcome (CI pass/fail, revert, time-to-merge, reviewer edits) is a training signal. The system's upgrade readiness scores, recipe selection, and wave scheduling improve continuously without manual retraining.

**G6 — Privacy-preserving fleet intelligence:** Multiple deployments share upgrade pattern intelligence via federated LoRA (ruvector Brain Server) without raw code or proprietary data leaving any deployment.

**G7 — Fleet management at scale:** Support fleets from 10 to 10,000+ repositories with appropriate wave scheduling, circuit breakers, and rate limiting.

### 3.2 Non-Goals

- Replacing Renovate or Dependabot for teams already satisfied with single-provider setups
- Providing a hosted/SaaS offering (self-hosted is the primary deployment model)
- Full IDE integration (browser/dashboard-first)
- Generating application feature code (upgrade automation only, not general code generation)
- Supporting binary/compiled artifact upgrades (source code and manifests only)

---

## 4. User Personas

### Persona A — Thiago, Solo Developer / OSS Maintainer
**Context:** Maintains 8–15 open-source repositories across GitHub and GitLab, mixing Java and TypeScript projects. Spends 2–3 hours per week manually checking and applying dependency updates.

**Needs:**
- Single dashboard to see all pending upgrades across providers
- Auto-merge for safe patch updates so he doesn't have to review them
- Alert on CVEs that affect his repos, with one-click fix

**Frustrations:**
- Renovate works great on GitHub but not on his GitLab repos
- Every CVE alert requires manually checking which repos are affected
- Has been burned by auto-merged minor updates that broke things silently

---

### Persona B — Priya, Platform Engineer
**Context:** Owns the developer platform at a 200-person startup. Manages a fleet of 80–150 microservices across GitHub, with a mix of Spring Boot, FastAPI, and Go services. Responsible for keeping the fleet secure and current.

**Needs:**
- Fleet-level view: "show me everything more than 6 months out of date"
- Controlled rollout: upgrade the canary services first, then waves
- Automated grouping: batch all Spring Boot patch updates into one PR per repo
- Audit trail: "show me all upgrade activity from last quarter"

**Frustrations:**
- Renovate creates 20+ PRs per repo per month, overwhelming her team
- No way to know which upgrade will succeed before wasting CI time
- Has to manually coordinate "upgrade spring boot across all 80 services" campaigns

---

### Persona C — Marcus, VP of Engineering / Enterprise
**Context:** Oversees a portfolio of 500–2000 repositories at a financial services firm. Operates across GitHub Enterprise, GitLab self-hosted, and Azure DevOps. Compliance requires evidence of security patch application within SLA.

**Needs:**
- Portfolio-level risk dashboard: CVE exposure, staleness heatmap, SLA compliance
- Policy enforcement: "all critical CVEs patched within 7 days across all repos"
- Separation of concerns: dev teams approve their own merges; platform team sets policy
- Federated deployment: intelligence shared across business units without cross-contamination

**Frustrations:**
- No single tool spans GitHub Enterprise, GitLab, and Azure DevOps
- Audit reports require manually aggregating data from multiple systems
- Security team and dev teams use different tools with no integration

---

### Persona D — Aiko, Developer Experience Lead
**Context:** Owns the internal developer platform at a tech company. Wants to reduce upgrade toil across 300 repos. Interested in AI-assisted tooling but wary of vendor lock-in and data privacy concerns.

**Needs:**
- Self-hosted AI capabilities: no code sent to external APIs
- Learning from internal patterns: "what upgrade recipes have worked for our specific tech stack?"
- Natural language queries: "which repos need the most urgent attention today?"
- Transparency: understand why the system recommended a particular approach

**Frustrations:**
- GitHub Copilot sends code to Microsoft; privacy policy is unclear
- Existing AI tools don't learn from her org's specific patterns
- Upgrading Terraform modules requires different skills than upgrading Java deps — no single tool handles both

---

## 5. Functional Requirements

### 5.1 Ecosystem Auto-Discovery

| ID | Requirement | Priority |
|----|-------------|----------|
| F1.1 | Detect Maven, Gradle, npm, Python, Go, Rust, Docker, Helm, GitHub Actions ecosystems via file-presence heuristics | P0 |
| F1.2 | Scan full repo tree (not just root) to support monorepos | P0 |
| F1.3 | Update manifest registry incrementally on git push events | P1 |
| F1.4 | Support additional ecosystems: .NET, Ruby, Terraform, Ansible, Elixir, Swift, Scala, Dart, Bazel | P1 |
| F1.5 | Detect deprecated API usage patterns via tree-sitter structural queries | P1 |
| F1.6 | Generate repo fingerprint embedding (768-dim) for cross-repo similarity | P2 |

### 5.2 Upgrade Planning

| ID | Requirement | Priority |
|----|-------------|----------|
| F2.1 | Resolve live latest version from registry (Maven Central, npmjs, crates.io, PyPI, etc.) at plan-creation time | P0 |
| F2.2 | Distinguish version-bump plans from code-migration plans in the data model | P0 |
| F2.3 | Assign risk tier per upgrade: patch (low), minor (medium), major (high), security (critical) | P0 |
| F2.4 | Compute upgrade readiness score per repo before attempting | P1 |
| F2.5 | Pre-flight incompatibility detection via semantic search against known-incompatibilities store | P1 |
| F2.6 | Generate libyears staleness score per repo | P2 |
| F2.7 | Integrate EPSS exploit prediction score for security prioritization | P2 |

### 5.3 PR Creation and Management

| ID | Requirement | Priority |
|----|-------------|----------|
| F3.1 | Create branches, commit manifest changes, and open PRs on GitHub, GitLab, and Bitbucket | P0 |
| F3.2 | Regenerate lockfiles (mvnw, npm install, go mod tidy, cargo update, etc.) as part of upgrade | P0 |
| F3.3 | Support grouping strategies: atomic, by-ecosystem, by-semver-tier | P1 |
| F3.4 | Detect and surface existing Renovate/Dependabot PRs; avoid duplication | P1 |
| F3.5 | Add Azure DevOps, Gitea/Forgejo provider adapters | P1 |
| F3.6 | Support digest-pinning for Docker/OCI image references | P2 |
| F3.7 | Generate PR descriptions that include: version delta, changelog excerpt, readiness score, fleet confidence | P2 |

### 5.4 Validation Pipeline

| ID | Requirement | Priority |
|----|-------------|----------|
| F4.1 | Poll CI status checks via provider API; gate auto-merge on all required checks passing | P0 |
| F4.2 | Enforce minimum release age (configurable; default 3 days patch, 7 days minor) | P0 |
| F4.3 | Supply chain check: verify new package version against OSV vulnerability database | P1 |
| F4.4 | Track CI check flakiness rate; demote checks with >5% false-failure rate from required set | P2 |
| F4.5 | Integrate Socket.dev for npm supply chain risk (malicious package detection) | P2 |

### 5.5 Auto-Merge

| ID | Requirement | Priority |
|----|-------------|----------|
| F5.1 | Expose auto_merge_rule configuration via REST API (CRUD) | P0 |
| F5.2 | Graduated default policy: patch auto-merge eligible; minor opt-in; major always manual | P0 |
| F5.3 | Delegate final merge action to provider's native auto-merge (GitHub, GitLab MWPS) | P0 |
| F5.4 | Per-repo and per-org policy inheritance (org policy → repo override) | P1 |
| F5.5 | Rate limiting: configurable prHourlyLimit and prConcurrentLimit | P1 |
| F5.6 | Schedule-based auto-merge window (e.g., "only merge during off-hours") | P2 |

### 5.6 Fleet Management and Wave Scheduling

| ID | Requirement | Priority |
|----|-------------|----------|
| F6.1 | Assign repos to upgrade waves based on readiness score | P1 |
| F6.2 | Circuit breaker: halt remaining waves if failure rate exceeds threshold (default 20%) | P1 |
| F6.3 | Dependency-order wave scheduling: topological sort of org dependency graph | P2 |
| F6.4 | Campaign view: fleet-level progress dashboard for a given upgrade across all repos | P1 |
| F6.5 | Canary repo designation: always upgrade designated repos first | P2 |

### 5.7 Self-Learning (RuVector/SONA Integration)

| ID | Requirement | Priority |
|----|-------------|----------|
| F7.1 | Record SONA trajectory for every upgrade attempt (state, action, reward) | P1 |
| F7.2 | Store upgrade outcomes in prioritized experience replay buffer | P1 |
| F7.3 | Compute and update fleet-level merge confidence scores per (library, version_bump) pair | P1 |
| F7.4 | Use Neural Thompson Sampling contextual bandit for recipe selection | P2 |
| F7.5 | Connect to Brain Server MCP for fleet-wide shared intelligence | P2 |
| F7.6 | Support federated LoRA: contribute weight deltas to shared model while preserving privacy | P3 |
| F7.7 | Automated recipe discovery: SONA crystallizes failure clusters into proposed new recipes | P3 |

### 5.8 Code Transformation (ruvllm)

| ID | Requirement | Priority |
|----|-------------|----------|
| F8.1 | Integrate ruvllm for local LLM patch generation (no cloud API requirement) | P2 |
| F8.2 | Use task-specific LoRA adapters (coder, researcher, architect) for different transformation contexts | P2 |
| F8.3 | RAG-augmented transformation: retrieve similar past transformations as few-shot context | P2 |
| F8.4 | Integrate OpenRewrite for Java/Kotlin AST-level code migration recipes | P2 |
| F8.5 | cargo fix, go fix, npm-check-updates integration for respective ecosystems | P3 |

---

## 6. Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NF1 | Pre-flight check latency | < 500ms per repo |
| NF2 | Repo fingerprint embedding update | < 30s per commit (incremental) |
| NF3 | HNSW search latency (1M vectors) | < 10ms p99 |
| NF4 | PR creation per provider | < 5s |
| NF5 | Fleet scan (1000 repos) | < 30 minutes |
| NF6 | Self-hosted deployment requirement | Single Docker Compose or Kubernetes manifest |
| NF7 | Data privacy | No raw code sent to external services; differential privacy ε=1.0 on shared embeddings |
| NF8 | Audit trail | All upgrade actions logged with actor, timestamp, outcome |
| NF9 | Idempotency | Running the planner twice produces the same set of planned PRs |
| NF10 | Provider credential encryption | AES-256-GCM (matching Ampel's existing standard) |

---

## 7. Success Metrics

### 7.1 Adoption Metrics
- **Time-to-first-PR**: < 30 minutes from onboarding to first upgrade PR created
- **Fleet coverage**: % of repos with at least one successful upgrade within 30 days of onboarding
- **Ecosystem breadth**: number of distinct ecosystems handled per deployment (target: ≥ 5 by month 3)

### 7.2 Quality Metrics
- **Upgrade success rate**: % of auto-generated PRs that pass CI on first attempt (target: ≥ 85%)
- **Pre-flight accuracy**: % of pre-flight incompatibility flags that correctly predict CI failure (target: ≥ 80%)
- **Revert rate**: % of auto-merged upgrades that are subsequently reverted (target: < 1%)

### 7.3 Learning Metrics
- **Merge confidence accuracy**: correlation between fleet merge confidence score and actual CI pass rate (target: r ≥ 0.80)
- **Recipe improvement rate**: improvement in upgrade success rate over the first 6 months as SONA accumulates data (target: +15% from month 1 to month 6)
- **Time-to-triage improvement**: reduction in time from CVE publication to upgrade PR open for affected repos (target: < 4 hours for critical CVEs)

### 7.4 Scale Metrics
- **Fleet size at which wave scheduling engages**: ≥ 20 repos
- **Supported providers at GA**: GitHub, GitLab, Bitbucket (Phase 1); Azure DevOps, Gitea/Forgejo (Phase 2)
- **Supported ecosystems at GA**: 10 (Phase 1); 20+ (Phase 3)

---

## 8. Constraints and Assumptions

### 8.1 Constraints
- Must run self-hosted; no mandatory cloud dependency
- Must not transmit raw source code to external services
- Must preserve Ampel's existing security model (AES-256-GCM token encryption, JWT auth, argon2 password hashing)
- Initial implementation in Rust to maintain consistency with Ampel and ruvector codebases
- Must support the existing Ampel database schema as the foundation (additive migrations only)

### 8.2 Assumptions
- Users have existing Ampel deployment (or are onboarding fresh)
- Git provider PATs are available with read + write repository permissions
- CI/CD pipelines exist in target repos (absence of CI is surfaced in readiness score, not a blocker)
- Ampel's existing provider adapters (GitHub, GitLab, Bitbucket) are extended, not replaced

---

## 9. Out of Scope for v1.0

- Mobile application
- Slack/Teams notification integration (infrastructure exists; not exposed in v1)
- Public hosted service
- Gerrit support
- Bitbucket Data Center (distinct API from Bitbucket Cloud)
- Automatic PR merging for major version upgrades
- SBOM generation (planned for v1.1)
- WASM upgrade node publishing to Brain Server (planned for v1.2)
