# Phased Technical Plan

## Ampel Upgrade Intelligence

**Total estimated duration:** 52–72 weeks across 8 phases  
**Team size assumed:** 2–4 engineers

---

## Phase 1 — Ecosystem Discovery Foundation (Weeks 1–8)

**Goal:** Auto-detect all ecosystems present in managed repos; build the manifest registry and initial repo fingerprinting pipeline.

**Why first:** Discovery is the prerequisite for everything. No upgrade plan can be generated without knowing what ecosystems exist.

### Task Group 1.1 — Manifest Registry (Weeks 1–3)

- [ ] **T1.1.1** Add `repo_manifests` SeaORM entity and migration  
  `{ id, repository_id, path, ecosystem, manager, manifest_file, last_scanned_at }`
- [ ] **T1.1.2** Implement `EcosystemDetector` trait in new `ampel-upgrades` crate  
  `fn detect(repo_path: &Path) -> Vec<DetectedManifest>`
- [ ] **T1.1.3** Implement file-presence detectors for P0 ecosystems:  
  Maven (`pom.xml`), Gradle (`build.gradle*`, `gradle-wrapper.properties`), npm (`package.json`, `.nvmrc`), Docker (`Dockerfile*`, `docker-compose*.yml`), GitHub Actions (`.github/workflows/*.yml`)
- [ ] **T1.1.4** Implement file-presence detectors for P1 ecosystems:  
  Python (`requirements.txt`, `pyproject.toml`, `Pipfile`, `.python-version`), Go (`go.mod`), Rust (`Cargo.toml`), Helm (`Chart.yaml`, `values.yaml`), CF manifest (`manifest.yml`)
- [ ] **T1.1.5** Implement `scan_ecosystems` Apalis background job  
  Triggered on: repository add, weekly schedule
- [ ] **T1.1.6** REST endpoint: `GET /api/repositories/{id}/manifests`
- [ ] **T1.1.7** Frontend: ecosystem badges on repository cards

### Task Group 1.2 — Repo Fingerprinting (Weeks 4–6)

- [ ] **T1.2.1** Integrate `ruvector-core` as Cargo dependency in `ampel-upgrades`  
  Features: `storage`, `hnsw`, `parallel`, `simd`
- [ ] **T1.2.2** Integrate `jina-embeddings-v2-base-code` via ONNX runtime  
  768-dim embeddings, 8192-token context window
- [ ] **T1.2.3** Implement tree-sitter-based semantic chunker  
  Language-specific grammars: Java, Kotlin, JavaScript/TypeScript, Python, Go, Rust
- [ ] **T1.2.4** Implement `RepoFingerprinter`:  
  tree-sitter chunk → embed → aggregate to repo-level 768-dim vector
- [ ] **T1.2.5** Add `repo_fingerprints` HNSW index (ruvector-core)  
  Payload: `{ repo_id, ecosystem[], framework[], libyears_estimate, ci_present }`
- [ ] **T1.2.6** Implement git-webhook-triggered incremental re-embedding  
  Only re-embed changed files (detected via `git diff --name-only`)
- [ ] **T1.2.7** `find_similar_repos(repo_id, k=10) -> Vec<(RepoId, f32)>` — HNSW k-NN query

### Task Group 1.3 — Staleness Scoring (Weeks 6–8)

- [ ] **T1.3.1** Implement libyears staleness score per dependency  
  `staleness = (today - release_date_of_installed_version).years`
- [ ] **T1.3.2** Live version resolution: query Maven Central, npmjs, PyPI, crates.io, pkg.go.dev
- [ ] **T1.3.3** Composite staleness dashboard: fleet-level heatmap by repo × ecosystem
- [ ] **T1.3.4** Add staleness fields to `repo_fingerprints` HNSW payload

**Phase 1 milestone:** Fleet dashboard shows all repos with ecosystem tags, staleness scores, and similar-repo recommendations.

---

## Phase 2 — Upgrade Planning + PR Creation (Weeks 9–18)

**Goal:** Generate deterministic upgrade plans and create PRs across all three existing providers.

**Why second:** Planning requires discovery (Phase 1). PR creation is the core value delivery.

### Task Group 2.1 — Upgrade Plan Data Model (Weeks 9–11)

- [ ] **T2.1.1** Add `upgrade_plans` SeaORM entity and migration  
  `{ id, repository_id, manifest_id, plan_type, ecosystem, dependency, from_version, to_version, semver_change, risk_tier, status, provider_pr_id, group_id, plan_data }`
- [ ] **T2.1.2** Add `upgrade_pr_groups` entity and migration
- [ ] **T2.1.3** Implement `VersionBumpPlanner`:  
  live registry query → semver comparison → plan generation
- [ ] **T2.1.4** Implement `RiskScorer`:  
  patch=low, minor=medium, major=high, CVSS>7=security
- [ ] **T2.1.5** `generate_plans` Apalis job (daily schedule)
- [ ] **T2.1.6** REST endpoints: `GET /api/upgrade-plans`, `POST /api/upgrade-plans/generate`

### Task Group 2.2 — Provider Write Operations (Weeks 10–13)

- [ ] **T2.2.1** Add `create_branch()` to `GitProvider` trait  
  `async fn create_branch(credentials, owner, repo, branch, from) -> ProviderResult<()>`
- [ ] **T2.2.2** Add `commit_files()` to `GitProvider` trait  
  `async fn commit_files(credentials, owner, repo, branch, files: &[FileChange], message) -> ProviderResult<CommitResult>`
- [ ] **T2.2.3** Add `create_pull_request()` to `GitProvider` trait  
  `async fn create_pull_request(credentials, owner, repo, request: &CreatePRRequest) -> ProviderResult<ProviderPullRequest>`
- [ ] **T2.2.4** Implement all three methods in `github.rs`
- [ ] **T2.2.5** Implement all three methods in `gitlab.rs`
- [ ] **T2.2.6** Implement all three methods in `bitbucket.rs`
- [ ] **T2.2.7** Integration tests for each provider (using mock provider)

### Task Group 2.3 — Manifest Patcher + Lockfile Regeneration (Weeks 12–15)

- [ ] **T2.3.1** Implement `ManifestPatcher` for Maven (`pom.xml` version string update)
- [ ] **T2.3.2** Implement `ManifestPatcher` for Gradle (`gradle-wrapper.properties`, `build.gradle`)
- [ ] **T2.3.3** Implement `ManifestPatcher` for npm (`package.json` version range update)
- [ ] **T2.3.4** Implement `ManifestPatcher` for Go (`go.mod` version update)
- [ ] **T2.3.5** Implement `ManifestPatcher` for Docker (`Dockerfile` FROM line, digest update)
- [ ] **T2.3.6** Implement `ManifestPatcher` for GitHub Actions (`uses:` version pin update)
- [ ] **T2.3.7** Implement `LockfileRegenerator` — subprocess executor per ecosystem:  
  Maven: `./mvnw dependency:resolve`, npm: `npm install --package-lock-only`, Go: `go mod tidy`
- [ ] **T2.3.8** Sandbox execution environment for lockfile regeneration (no network during generation)

### Task Group 2.4 — PR Creation + Grouping (Weeks 14–18)

- [ ] **T2.4.1** Implement `PrCreator` — orchestrates patch → commit → PR
- [ ] **T2.4.2** Implement `AtomicGrouping` strategy: one PR per dependency update
- [ ] **T2.4.3** Implement `ByEcosystemGrouping` strategy: one PR per ecosystem per repo
- [ ] **T2.4.4** Implement `BySemverTierGrouping` strategy: one PR per tier (patches together, minors together)
- [ ] **T2.4.5** PR description generation: version delta, changelog excerpt, readiness score, ecosystem badges
- [ ] **T2.4.6** Detect existing Renovate/Dependabot PRs via `list_pull_requests()`; surface in dashboard to avoid duplication
- [ ] **T2.4.7** REST endpoint: `POST /api/repositories/{id}/upgrade-prs`
- [ ] **T2.4.8** Frontend: "Create upgrade PRs" action on repository detail page

**Phase 2 milestone:** Reproduce today's Maven wrapper upgrade across all 4 `cf-toolsuite` repos automatically from the UI, end-to-end.

---

## Phase 3 — Validation Pipeline + Auto-Merge (Weeks 19–26)

**Goal:** Gate PRs on CI status; expose auto-merge policy; implement safety guardrails.

### Task Group 3.1 — CI Validation Gate (Weeks 19–21)

- [ ] **T3.1.1** Implement `CIPoller` Apalis job: poll `get_ci_checks()` every 5 minutes for open upgrade PRs
- [ ] **T3.1.2** Track required vs. advisory checks per repo (configurable)
- [ ] **T3.1.3** Implement CI check flakiness detector:  
  if check fails > 5% of runs on un-modified branches → demote from required to advisory
- [ ] **T3.1.4** Implement webhook receiver endpoint: `POST /api/webhooks/{provider}`  
  Normalizes provider-specific CI payloads to canonical `CIEvent` struct (eliminates polling latency)
- [ ] **T3.1.5** Connect webhook receiver to upgrade PR status updates

### Task Group 3.2 — Auto-Merge Policy Engine (Weeks 21–24)

- [ ] **T3.2.1** Extend `auto_merge_rule` DB schema:  
  Add `automerge_patch`, `automerge_minor`, `automerge_major`, `automerge_security`, `minimum_release_age_days`, `require_supply_chain_pass`, `pr_hourly_limit`, `pr_concurrent_limit`, `automerge_schedule`, `apply_to_ecosystems`, `apply_to_paths`, `merge_strategy`
- [ ] **T3.2.2** Expose `auto_merge_rule` via REST API (CRUD)
- [ ] **T3.2.3** Implement `PolicyEngine`: evaluate auto_merge_rule conditions per PR
- [ ] **T3.2.4** Implement minimum_release_age check: `release_date + min_age_days < now()`
- [ ] **T3.2.5** Implement platform-native auto-merge delegation:  
  GitHub: `enablePullRequestAutoMerge` GraphQL mutation  
  GitLab: `merge_when_pipeline_succeeds` API  
  Bitbucket: auto-merge on CI pass via status webhook
- [ ] **T3.2.6** Implement `automerge_eligible` Apalis job (check every 15 minutes)
- [ ] **T3.2.7** Per-org policy inheritance: org policy → repo override chain

### Task Group 3.3 — Supply Chain Gates (Weeks 23–26)

- [ ] **T3.3.1** Integrate OSV.dev API: check new dependency versions for known vulnerabilities
- [ ] **T3.3.2** Block auto-merge if new version introduces CVE not present in current version
- [ ] **T3.3.3** Implement `minimum_release_age_days` enforcement (default: patch=3, minor=7, major=0)
- [ ] **T3.3.4** Frontend: auto-merge policy configuration UI (per-repo and per-org)
- [ ] **T3.3.5** Frontend: per-PR merge readiness widget (checklist: CI, age, supply chain)

**Phase 3 milestone:** Patch updates auto-merge safely across fleet with zero human interaction. Priya can leave Friday confident that safe updates will merge over the weekend.

---

## Phase 4 — Self-Learning Foundation (Weeks 27–34)

**Goal:** Record every upgrade outcome as a SONA trajectory; build fleet merge confidence; integrate Brain Server.

### Task Group 4.1 — Trajectory Recording (Weeks 27–29)

- [ ] **T4.1.1** Integrate `ruvector-sona` Cargo dependency in `ampel-intelligence` crate
- [ ] **T4.1.2** Implement `UpgradeTrajectory` struct and conversion from PR outcome data
- [ ] **T4.1.3** Implement `TrajectoryRecorder`: captures outcome when PR is merged/closed/reverted
- [ ] **T4.1.4** Implement `RewardCalculator`: assigns reward value per outcome  
  CI pass=+10, security resolved=+2/vuln, test failures=-5, revert=-10
- [ ] **T4.1.5** Add `upgrade_trajectories` DB table for persistent replay buffer

### Task Group 4.2 — Merge Confidence (Weeks 28–31)

- [ ] **T4.2.1** Add `merge_confidence_cache` DB table:  
  `{ library, from_version, to_version, fleet_pass_rate, n_observations, updated_at }`
- [ ] **T4.2.2** `update_merge_confidence` job: recompute pass rate after each new outcome
- [ ] **T4.2.3** Include merge_confidence score in PR descriptions
- [ ] **T4.2.4** Frontend: merge confidence badge on upgrade plan cards
- [ ] **T4.2.5** Integrate merge_confidence as a factor in readiness score

### Task Group 4.3 — Brain Server Integration (Weeks 30–34)

- [ ] **T4.3.1** Implement `BrainServerClient` in `ampel-intelligence`:  
  REST client wrapping Brain Server's `/v1/memories`, `/v1/memories/search`, `/v1/train/enhanced`
- [ ] **T4.3.2** `brain_share` integration: after each upgrade outcome, contribute to Brain Server  
  Content: ecosystem, dependency, version_delta, outcome, anonymized_repo_profile
- [ ] **T4.3.3** `brain_search` integration: at pre-flight time, query Brain Server for incompatibility patterns
- [ ] **T4.3.4** `brain_search` integration: at plan-generation time, retrieve merge confidence precedents
- [ ] **T4.3.5** Brain Server configuration in Ampel settings (URL, auth token)
- [ ] **T4.3.6** Fallback behavior when Brain Server is unavailable (graceful degradation, no blocking)

**Phase 4 milestone:** Fleet merge confidence scores appear on upgrade PRs. Pre-flight intelligence is powered by Brain Server. Priya sees "Fleet confidence: 0.94 based on 234 observations" on every upgrade PR.

---

## Phase 5 — Adaptive Triage + Wave Scheduling (Weeks 35–42)

**Goal:** Implement SONA-powered contextual bandit recipe selection and upgrade wave scheduling with circuit breakers.

### Task Group 5.1 — Upgrade Readiness Score (Weeks 35–37)

- [ ] **T5.1.1** Implement `ReadinessScorer` combining:  
  libyears, test_coverage_ratio, ci_reliability_90d, similar_repo_success_rate, merge_confidence, pre_flight_flags
- [ ] **T5.1.2** Store readiness score in `upgrade_plans` table; update on each plan generation
- [ ] **T5.1.3** Frontend: readiness score bar with breakdown tooltip

### Task Group 5.2 — Contextual Bandit Recipe Selection (Weeks 36–39)

- [ ] **T5.2.1** Implement `ContextualBandit` (Neural Thompson Sampling) in `ampel-intelligence`:  
  Context = repo fingerprint embedding; Action = recipe selection; Reward = CI outcome
- [ ] **T5.2.2** Initialize bandit priors from existing merge_confidence data (warm start)
- [ ] **T5.2.3** Implement explore-exploit risk gating:  
  critical repos: ε=0.02; sandbox repos: ε=0.20
- [ ] **T5.2.4** Posterior update: after each upgrade outcome, update Beta(successes, failures) per recipe
- [ ] **T5.2.5** Recipe selection audit log: explain why each recipe was selected

### Task Group 5.3 — Wave Scheduler (Weeks 38–42)

- [ ] **T5.3.1** Implement `WaveScheduler`: assign repos to canary/wave1/wave2/wave3 based on readiness
- [ ] **T5.3.2** Campaign data model: `upgrade_campaigns` table  
  `{ id, name, target_dependency, target_version, wave_config, status, created_at }`
- [ ] **T5.3.3** Campaign progress tracking: per-repo status per wave
- [ ] **T5.3.4** Circuit breaker: halt wave progression if failure rate > configurable threshold (default 20%)
- [ ] **T5.3.5** Auto-promotion: advance to next wave after observation window + zero production incidents
- [ ] **T5.3.6** Frontend: campaign dashboard with wave progress visualization
- [ ] **T5.3.7** Campaign REST API: create, start, pause, resume, cancel

**Phase 5 milestone:** Priya can launch a "migrate all Spring Boot repos to 3.4" campaign, set it, and come back a week later to see results — with circuit breakers having protected production automatically.

---

## Phase 6 — Code Transformation + OpenRewrite (Weeks 43–52)

**Goal:** Add ruvllm local LLM for patch generation and OpenRewrite for Java/Kotlin AST-level code migration.

### Task Group 6.1 — ruvllm Integration (Weeks 43–46)

- [ ] **T6.1.1** Add `ruvllm` Cargo dependency; feature-gate per hardware (`inference-metal` for Mac, `inference-cuda` for NVIDIA)
- [ ] **T6.1.2** Implement `RuvllmClient` in `ampel-intelligence`: local inference via ruvllm REST API
- [ ] **T6.1.3** Implement `FewShotTransformationRetriever`: retrieve similar past transformations from Brain Server as LLM context
- [ ] **T6.1.4** Implement `PatchGenerator`: ruvllm + few-shot context → code transformation patch
- [ ] **T6.1.5** Hot-swap LoRA adapter selection: coder (code changes), architect (build file changes), researcher (research mode)
- [ ] **T6.1.6** RAG-augmented prompting: inject retrieved transformation examples into prompt
- [ ] **T6.1.7** MicroLoRA per-repo adaptation: fine-tune to repo's coding style in <1ms on first interaction

### Task Group 6.2 — OpenRewrite Integration (Weeks 45–50)

- [ ] **T6.2.1** Implement `OpenRewriteRunner`: subprocess executor for OpenRewrite CLI  
  `./gradlew rewrite:run -Drewrite.activeRecipes=...` or `./mvnw rewrite:run -Drewrite.activeRecipes=...`
- [ ] **T6.2.2** Recipe catalog: map from (ecosystem, from_version, to_version) → applicable OpenRewrite recipes
- [ ] **T6.2.3** `CodeMigrationPlan` generation: when major version detected, attach applicable recipe list
- [ ] **T6.2.4** Diff capture: capture OpenRewrite output as `FileChange[]` for PR commit
- [ ] **T6.2.5** Integrate spring-m11n skill recipes (from `agentic-incubator/java-spring-modernization-marketplace`) into catalog
- [ ] **T6.2.6** Pre-flight recipe applicability check: dry-run OpenRewrite to count affected files before committing

### Task Group 6.3 — Ecosystem-Specific Transformers (Weeks 48–52)

- [ ] **T6.3.1** `cargo fix --edition` runner for Rust edition upgrades
- [ ] **T6.3.2** `go fix` runner for Go API migrations
- [ ] **T6.3.3** `npm-check-updates` runner for Node.js range updates
- [ ] **T6.3.4** `python -m lib2to3` runner for Python 2→3 migrations
- [ ] **T6.3.5** `kubectl-convert` runner for Kubernetes API version migrations

**Phase 6 milestone:** The system can handle Spring Boot 2→3 migrations end-to-end with no manual code editing required for the 80% case.

---

## Phase 7 — Provider Expansion (Weeks 53–60)

**Goal:** Add Azure DevOps and Gitea/Forgejo provider adapters; implement Gerrit (optional).

### Task Group 7.1 — Azure DevOps Adapter (Weeks 53–57)

- [ ] **T7.1.1** Implement `AzureDevOpsProvider` implementing `GitProvider` trait
- [ ] **T7.1.2** Azure DevOps API: REST + SOAP, `azure-devops-node-api` equivalent in Rust
- [ ] **T7.1.3** Auth: Personal Access Token (Basic auth with base64 encoding)
- [ ] **T7.1.4** Implement: `validate_credentials`, `get_user`, `list_repositories`, `list_pull_requests`
- [ ] **T7.1.5** Implement: `create_branch`, `commit_files`, `create_pull_request`, `merge_pull_request`
- [ ] **T7.1.6** Implement: `get_ci_checks` (Azure Pipelines build status)
- [ ] **T7.1.7** Register in `ProviderFactory`; add to provider enum; update UI

### Task Group 7.2 — Gitea/Forgejo Adapter (Weeks 55–58)

- [ ] **T7.2.1** Implement `GiteaProvider` implementing `GitProvider` trait (covers both Gitea and Forgejo, same API)
- [ ] **T7.2.2** Gitea REST v1 API (OpenAPI-compliant); configurable instance URL
- [ ] **T7.2.3** Implement all 13 required trait methods
- [ ] **T7.2.4** Register in `ProviderFactory`

### Task Group 7.3 — Bitbucket Data Center (Weeks 57–60)

- [ ] **T7.3.1** Implement `BitbucketDataCenterProvider` (distinct API from Bitbucket Cloud)  
  REST v1.0, different endpoint shapes, PAT auth vs. app passwords
- [ ] **T7.3.2** Key difference: `merge_pull_request` uses different payload schema
- [ ] **T7.3.3** Register in `ProviderFactory`

**Phase 7 milestone:** Marcus (enterprise persona) can manage repos on GitHub Enterprise, GitLab self-hosted, Azure DevOps, and Gitea from a single Ampel instance.

---

## Phase 8 — Federated Intelligence + Polyglot Expansion (Weeks 61–72)

**Goal:** Federated LoRA learning across deployments; expand ecosystem coverage to 20+ languages.

### Task Group 8.1 — Federated LoRA (Weeks 61–66)

- [ ] **T8.1.1** Configure Brain Server federation: enterprise coordinator + per-org instances
- [ ] **T8.1.2** Implement LoRA weight delta submission (`brain_lora_submit`) after SONA training cycles
- [ ] **T8.1.3** Implement consensus weight pull (`brain_lora_latest`) to update local model
- [ ] **T8.1.4** Differential privacy enforcement: ε=1.0 noise on all shared embeddings
- [ ] **T8.1.5** Federation UI: connectivity status, weight sync history, privacy settings

### Task Group 8.2 — Automated Recipe Discovery (Weeks 63–68)

- [ ] **T8.2.1** Implement failure cluster analyzer: SONA crystallizes repeated failure patterns
- [ ] **T8.2.2** When cluster size ≥ 10 trajectories: auto-generate recipe proposal
- [ ] **T8.2.3** Recipe proposal workflow: draft page in Brain Server Brainpedia
- [ ] **T8.2.4** Human review + evidence workflow: `brain_page_evidence` + promotion to Canonical
- [ ] **T8.2.5** Auto-deploy canonical recipes to recipe catalog

### Task Group 8.3 — Additional Ecosystem Detectors (Weeks 64–70)

- [ ] **T8.3.1** Detectors: .NET/NuGet (`.csproj`, `NuGet.Config`)
- [ ] **T8.3.2** Detectors: Ruby/Bundler (`Gemfile`, `.ruby-version`)
- [ ] **T8.3.3** Detectors: Terraform HCL (`*.tf` provider blocks)
- [ ] **T8.3.4** Detectors: Elixir/Mix (`mix.exs`)
- [ ] **T8.3.5** Detectors: Scala/SBT (`build.sbt`)
- [ ] **T8.3.6** Detectors: Swift/SPM (`Package.swift`)
- [ ] **T8.3.7** Detectors: Ansible (`requirements.yml`)
- [ ] **T8.3.8** Detectors: Bazel (`WORKSPACE`, `BUILD`)
- [ ] **T8.3.9** Manifest patchers for each new ecosystem
- [ ] **T8.3.10** Lockfile regenerators for each new ecosystem

### Task Group 8.4 — Natural Language Fleet Queries (Weeks 68–72)

- [ ] **T8.4.1** MCP tool set: expose fleet intelligence via Claude Code MCP
- [ ] **T8.4.2** `fleet_query(natural_language) -> FleetInsight`: Brain Server + fleet data → conversational response
- [ ] **T8.4.3** Campaign creation from natural language: "upgrade all Python repos to 3.13" → generates campaign
- [ ] **T8.4.4** SBOM generation post-merge (SPDX / CycloneDX) for compliance

**Phase 8 milestone:** Aiko can ask "which repos need urgent attention?" in natural language and get an actionable answer backed by real vector search over the fleet's code state.

---

## Dependency Graph

```
Phase 1 (Discovery)
    └── Phase 2 (Planning + PR Creation)
            ├── Phase 3 (Validation + Auto-merge)
            │       └── Phase 4 (Self-Learning)
            │               └── Phase 5 (Adaptive Triage)
            │                       └── Phase 6 (Code Transformation)
            └── Phase 7 (Provider Expansion) ← can parallel with Phase 3+
Phase 8 (Federated + Polyglot) ← requires Phase 4 + Phase 6 + Phase 7
```

---

## Key Technical Dependencies

| Dependency | Phase | Source |
|-----------|-------|--------|
| `ruvector-core` | Phase 1 | `ruvnet/ruvector` (Cargo) |
| `ruvector-sona` | Phase 4 | `ruvnet/ruvector` (Cargo) |
| `ruvllm` | Phase 6 | `ruvnet/ruvector` (Cargo) |
| Brain Server MCP | Phase 4 | `ruvnet/ruvector` (Docker) |
| tree-sitter (Rust bindings) | Phase 1 | `tree-sitter` crate |
| ONNX runtime | Phase 1 | `ort` Rust crate |
| jina-embeddings-v2-base-code | Phase 1 | HuggingFace Hub ONNX |
| OpenRewrite CLI | Phase 6 | Maven/Gradle plugin |
| OSV.dev API | Phase 3 | HTTP (no auth required) |
| EPSS API | Phase 5 | https://api.first.org/data/v1/epss |

---

## Risk Register

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| ruvllm/ruvector API instability (active development) | High | Medium | Pin to specific git commit; monitor CHANGELOG |
| Lockfile regeneration requires build toolchain in sandbox | High | High | Docker-in-Docker or nix sandbox for lockfile regen |
| Provider rate limits at fleet scale | Medium | High | Respect `prHourlyLimit`; use webhook polling over API polling |
| ONNX model inference speed on CPU | Medium | Low | GPU optional; acceptable latency for background jobs |
| Brain Server cost at scale (Cloud Run) | Low | Medium | Self-hosted option; batch training cycles |
| OpenRewrite fails on non-standard project layouts | Medium | Medium | Pre-flight applicability check before attempting |
