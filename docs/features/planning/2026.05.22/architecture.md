# Technical Architecture

## Ampel Upgrade Intelligence

---

## 1. System Overview

```
┌────────────────────────────────────────────────────────────────────┐
│                        Ampel Upgrade Intelligence                   │
│                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐ │
│  │  React UI    │  │  REST API    │  │  Background Workers       │ │
│  │  (frontend/) │  │  (ampel-api) │  │  (ampel-worker / Apalis)  │ │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────────┘ │
│         │                 │                       │                  │
│         └─────────────────┼───────────────────────┘                 │
│                           │                                          │
│  ┌────────────────────────┼──────────────────────────────────────┐  │
│  │                  Core Business Logic                           │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │  │
│  │  │ ampel-core   │  │ ampel-upgrades│  │ ampel-intelligence   │ │  │
│  │  │ (domain)     │  │ (discovery,  │  │ (SONA, embeddings,   │ │  │
│  │  │              │  │  planning,   │  │  bandit, brain)      │ │  │
│  │  │              │  │  execution)  │  │                      │ │  │
│  │  └──────────────┘  └──────────────┘  └──────────────────────┘ │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                           │                                          │
│  ┌────────────────────────┼──────────────────────────────────────┐  │
│  │                  Data & Provider Layer                         │  │
│  │  ┌──────────────┐  ┌──────────────────────────────────────┐  │  │
│  │  │  ampel-db    │  │         ampel-providers               │  │  │
│  │  │  (SeaORM,    │  │  GitHub │ GitLab │ Bitbucket │ Azure  │  │  │
│  │  │   PostgreSQL)│  │         GitProvider trait             │  │  │
│  │  └──────────────┘  └──────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
         │                          │
         ▼                          ▼
┌────────────────┐       ┌──────────────────────┐
│  RuVector      │       │  Git Providers        │
│  Stack         │       │  GitHub / GitLab /    │
│                │       │  Bitbucket / Azure /  │
│  ruvector-core │       │  Gitea                │
│  ruvector-sona │       └──────────────────────┘
│  ruvllm        │
│  Brain Server  │
└────────────────┘
```

---

## 2. Crate Structure

### Existing Ampel Crates (unchanged or additive only)

| Crate | Role | Changes |
|-------|------|---------|
| `ampel-api` | REST API handlers, routes, middleware | +new handler modules |
| `ampel-core` | Domain models, services | +upgrade domain models |
| `ampel-db` | SeaORM entities, migrations | +9 new migration files |
| `ampel-providers` | GitHub/GitLab/Bitbucket trait + impls | +3 new trait methods; +2 new providers |
| `ampel-worker` | Apalis background jobs | +5 new job types |

### New Crates

**`ampel-upgrades`** — Upgrade lifecycle orchestration:
```
crates/ampel-upgrades/src/
├── discovery/
│   ├── mod.rs               # EcosystemDetector trait
│   └── detectors/           # Per-ecosystem file-presence detectors
├── planning/
│   ├── version_bump.rs      # Live registry version resolution
│   ├── code_migration.rs    # OpenRewrite recipe catalog
│   └── risk_scorer.rs       # patch/minor/major risk classification
├── execution/
│   ├── manifest_patcher.rs  # Deterministic manifest file rewriting
│   ├── lockfile_regen.rs    # Subprocess lockfile regeneration
│   └── pr_creator.rs        # Uses ampel-providers write operations
├── grouping/
│   └── strategies.rs        # atomic, by_ecosystem, by_semver_tier
└── validation/
    └── gates.rs             # CI status polling, OSV supply chain
```

**`ampel-intelligence`** — Self-learning and vector intelligence:
```
crates/ampel-intelligence/src/
├── fingerprinting/
│   ├── repo_fingerprinter.rs    # Tree-sitter → embedding → HNSW upsert
│   └── ecosystem_embedder.rs   # Per-ecosystem manifest embedding
├── triage/
│   ├── readiness_scorer.rs     # Multi-factor upgrade readiness score
│   ├── bandit.rs               # Neural Thompson Sampling recipe selection
│   └── wave_scheduler.rs       # Cohort assignment + circuit breaker
├── learning/
│   ├── trajectory_recorder.rs  # SONA trajectory from PR outcomes
│   ├── replay_buffer.rs        # Prioritized experience replay
│   └── brain_client.rs         # Brain Server REST/MCP client
├── preflight/
│   ├── incompatibility_store.rs # Vector store for known incompatibilities
│   └── pattern_detector.rs     # Tree-sitter + embedding pre-flight
└── generation/
    └── ruvllm_client.rs        # ruvllm REST client for patch generation
```

---

## 3. Database Schema Extensions

### New Tables (SeaORM migrations, additive)

```sql
-- Detected manifests per repository
CREATE TABLE repo_manifests (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id   UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    path            TEXT NOT NULL,          -- "services/api/pom.xml"
    ecosystem       TEXT NOT NULL,          -- "maven", "npm", "cargo", "docker"
    manager         TEXT NOT NULL,          -- "maven-wrapper", "npm", "cargo"
    last_scanned_at TIMESTAMPTZ,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(repository_id, path)
);

-- Upgrade plans (one row per planned upgrade)
CREATE TABLE upgrade_plans (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id   UUID NOT NULL REFERENCES repositories(id),
    manifest_id     UUID REFERENCES repo_manifests(id),
    plan_type       TEXT NOT NULL,          -- "version_bump" | "code_migration"
    ecosystem       TEXT NOT NULL,
    dependency      TEXT,                   -- null for code_migration
    from_version    TEXT,
    to_version      TEXT,
    semver_change   TEXT,                   -- "patch"|"minor"|"major"|"digest"|"security"
    risk_tier       TEXT NOT NULL,          -- "low"|"medium"|"high"|"security"
    readiness_score REAL,                   -- 0.0-1.0
    status          TEXT DEFAULT 'pending', -- pending|pr_open|merged|closed|failed
    provider_pr_id  TEXT,
    group_id        UUID REFERENCES upgrade_pr_groups(id),
    plan_data       JSONB,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- PR groups (N plans → 1 PR)
CREATE TABLE upgrade_pr_groups (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id   UUID NOT NULL REFERENCES repositories(id),
    group_strategy  TEXT NOT NULL,          -- "atomic"|"by_ecosystem"|"by_semver_tier"
    provider_pr_id  TEXT,
    provider_pr_url TEXT,
    status          TEXT DEFAULT 'open',
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Fleet-level merge confidence cache
CREATE TABLE merge_confidence_cache (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    library         TEXT NOT NULL,
    from_version    TEXT NOT NULL,
    to_version      TEXT NOT NULL,
    fleet_pass_rate REAL NOT NULL,
    n_observations  INTEGER NOT NULL DEFAULT 0,
    updated_at      TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(library, from_version, to_version)
);

-- Upgrade outcome trajectories (SONA replay buffer)
CREATE TABLE upgrade_trajectories (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    upgrade_plan_id     UUID REFERENCES upgrade_plans(id),
    repo_fingerprint    JSONB,              -- repo state at time of upgrade
    action_data         JSONB,              -- recipe, grouping, timing
    reward              REAL,               -- computed reward value
    ci_outcome          TEXT,               -- "pass"|"fail"|"timeout"
    reviewer_edits      INTEGER DEFAULT 0,
    time_to_merge_hours REAL,
    reverted            BOOLEAN DEFAULT false,
    created_at          TIMESTAMPTZ DEFAULT NOW()
);

-- Upgrade campaigns (wave scheduling)
CREATE TABLE upgrade_campaigns (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    target_ecosystem TEXT,
    target_library  TEXT,
    target_version  TEXT,
    wave_config     JSONB,                  -- wave thresholds, circuit breaker %
    status          TEXT DEFAULT 'planned', -- planned|active|paused|complete|cancelled
    created_by      UUID REFERENCES users(id),
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Extended auto_merge_rule (additive columns)
ALTER TABLE auto_merge_rule ADD COLUMN IF NOT EXISTS
    automerge_patch          BOOLEAN DEFAULT true,
    automerge_minor          BOOLEAN DEFAULT false,
    automerge_major          BOOLEAN DEFAULT false,
    automerge_security       BOOLEAN DEFAULT true,
    minimum_release_age_days INTEGER DEFAULT 3,
    require_supply_chain_pass BOOLEAN DEFAULT true,
    pr_hourly_limit          INTEGER DEFAULT 2,
    pr_concurrent_limit      INTEGER DEFAULT 10,
    automerge_schedule       TEXT,
    apply_to_ecosystems      JSONB,
    apply_to_paths           JSONB,
    merge_strategy           TEXT DEFAULT 'squash';

-- Upgrade policies (per-repo or per-org overrides)
CREATE TABLE upgrade_policies (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scope_type  TEXT NOT NULL,              -- "repo"|"org"|"global"
    scope_id    UUID,
    policy_data JSONB NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    updated_at  TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 4. RuVector Integration Points

### Embedding Index (ruvector-core)

```rust
// ampel-intelligence/src/fingerprinting/repo_fingerprinter.rs
use ruvector_core::{VectorDB, DbOptions, VectorEntry, SearchQuery};

pub struct RepoFingerprintIndex {
    db: VectorDB,  // ruvector-core HNSW index
}

impl RepoFingerprintIndex {
    pub fn upsert_repo(&mut self, repo_id: Uuid, embedding: Vec<f32>, metadata: RepoMetadata) {
        let entry = VectorEntry {
            id: repo_id.to_string(),
            vector: embedding,
            metadata: serde_json::to_value(metadata).unwrap(),
        };
        self.db.insert(entry).expect("HNSW upsert");
    }

    pub fn find_similar(&self, repo_id: Uuid, k: usize) -> Vec<(Uuid, f32)> {
        let query_vec = self.db.get(repo_id.to_string()).unwrap().vector;
        let results = self.db.search(SearchQuery { vector: query_vec, k, ..Default::default() }).unwrap();
        results.into_iter().map(|r| (r.id.parse().unwrap(), r.score)).collect()
    }
}
```

### SONA Learning Loop (ruvector-sona)

```rust
// ampel-intelligence/src/learning/trajectory_recorder.rs
use sona::{SonaEngine, SonaConfig};

pub struct UpgradeLearner {
    engine: SonaEngine,
}

impl UpgradeLearner {
    pub fn record_outcome(&self, trajectory: &UpgradeTrajectory) {
        let mut builder = self.engine.begin_trajectory(trajectory.repo_embedding.clone());
        
        // Add decision step
        builder.add_step(
            trajectory.action_embedding.clone(),
            vec![],          // no intermediate observations
            trajectory.reward,
        );
        
        // Close trajectory — SONA learns from this
        self.engine.end_trajectory(builder, trajectory.reward);
    }
    
    pub fn apply_learning(&self, input: &[f32], output: &mut Vec<f32>) {
        // MicroLoRA applies learned patterns in <1ms
        self.engine.apply_micro_lora(input, output);
    }
}
```

### Brain Server Client

```rust
// ampel-intelligence/src/learning/brain_client.rs
pub struct BrainServerClient {
    base_url: String,
    client: reqwest::Client,
}

impl BrainServerClient {
    pub async fn share_outcome(&self, outcome: &UpgradeOutcome) -> Result<()> {
        // POST /v1/memories — contribute to shared Brain Server
        self.client.post(format!("{}/v1/memories", self.base_url))
            .json(&BrainMemory {
                category: "upgrade_outcome".into(),
                title: format!("{} {} → {}", outcome.library, outcome.from, outcome.to),
                content: outcome.to_description(),
                tags: outcome.tags(),
            })
            .send().await?;
        Ok(())
    }
    
    pub async fn search_incompatibilities(&self, query: &str) -> Result<Vec<IncompatibilityPattern>> {
        // GET /v1/memories/search — retrieve known incompatibilities
        let resp: BrainSearchResponse = self.client
            .get(format!("{}/v1/memories/search", self.base_url))
            .query(&[("q", query), ("category", "incompatibility"), ("limit", "5")])
            .send().await?.json().await?;
        Ok(resp.into_patterns())
    }
}
```

---

## 5. GitProvider Trait Extensions

```rust
// crates/ampel-providers/src/traits.rs — new methods added
#[async_trait]
pub trait GitProvider: Send + Sync {
    // EXISTING methods (unchanged):
    async fn validate_credentials(&self, credentials: &ProviderCredentials) -> ProviderResult<TokenValidation>;
    async fn list_repositories(&self, ...) -> ProviderResult<Vec<DiscoveredRepository>>;
    async fn list_pull_requests(&self, ...) -> ProviderResult<Vec<ProviderPullRequest>>;
    async fn get_ci_checks(&self, ...) -> ProviderResult<Vec<ProviderCICheck>>;
    async fn merge_pull_request(&self, ...) -> ProviderResult<MergeResult>;
    // ... (all 10 existing methods)
    
    // NEW methods (Phase 2):
    async fn create_branch(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        branch: &str,
        from_ref: &str,
    ) -> ProviderResult<()>;

    async fn commit_files(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        branch: &str,
        files: &[FileChange],
        message: &str,
    ) -> ProviderResult<CommitResult>;

    async fn create_pull_request(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        request: &CreatePRRequest,
    ) -> ProviderResult<ProviderPullRequest>;
}

pub struct FileChange {
    pub path: String,
    pub content: Vec<u8>,
    pub encoding: FileEncoding,  // Text | Binary | Base64
}

pub struct CreatePRRequest {
    pub title: String,
    pub body: String,
    pub head_branch: String,
    pub base_branch: String,
    pub draft: bool,
    pub labels: Vec<String>,
}
```

---

## 6. Background Job Schedule

| Job | Apalis Cron | Purpose |
|-----|-------------|---------|
| `scan_ecosystems` | `0 2 * * 0` (weekly) | Detect/update manifests for all repos |
| `generate_plans` | `0 6 * * *` (daily 6am) | Live registry query → upgrade plans |
| `execute_upgrades` | `0 8 * * *` (daily 8am) | Create PRs for pending plans within rate limits |
| `poll_pr_validation` | `*/5 * * * *` (every 5min) | Check CI status on open upgrade PRs |
| `automerge_eligible` | `*/15 * * * *` (every 15min) | Evaluate auto_merge_rule conditions |
| `update_merge_confidence` | `0 * * * *` (hourly) | Recompute fleet pass rates |
| `sona_training` | `0 */6 * * *` (every 6h) | Brain Server enhanced training cycle |
| `drift_check` | `0 2 * * *` (daily 2am) | Detect embedding drift in fleet |
| `staleness_report` | `0 9 * * 1` (Monday 9am) | Generate weekly staleness digest |

---

## 7. Vector Index Specification

| Collection | Dimensions | Quantization | M | ef | Payload fields |
|-----------|-----------|-------------|---|-----|----------------|
| `code_chunks` | 768 | int8 scalar (4x) | 16 | 100 | repo_id, path, lang, chunk_hash, line_start |
| `repo_fingerprints` | 768 | float32 | 32 | 200 | repo_id, ecosystems, frameworks, libyears, ci_present |
| `upgrade_outcomes` | 768 | float32 | 16 | 100 | recipe_id, repo_profile_hash, outcome, n_obs |
| `known_incompatibilities` | 768 | float32 | 16 | 100 | pattern_desc, affected_versions, resolution |

**Embedding model:** `jina-embeddings-v2-base-code` (768-dim, 8192-token, Apache 2.0)  
**Runtime:** ONNX via `ort` Rust crate  
**Index build:** Incremental upsert on git webhook (content hash as ID, idempotent)  
**Full reindex:** Scheduled monthly or on model version update

---

## 8. Security Model

All security properties from the existing Ampel deployment are preserved and extended:

| Property | Mechanism | Applies To |
|----------|-----------|------------|
| PAT encryption at rest | AES-256-GCM | All provider credentials |
| Auth | JWT (access 15min / refresh 7d) | All API endpoints |
| Password hashing | Argon2 | User accounts |
| Transport | TLS (enforced) | All provider API calls |
| Code privacy | Embeddings only, no raw code in Brain Server | Federated intelligence |
| Differential privacy | ε=1.0 noise on shared embeddings | Federated LoRA |
| Audit trail | All upgrade actions logged | Compliance |
| Input validation | At all API boundaries | Injection prevention |
| Rate limiting | prHourlyLimit, prConcurrentLimit | PR creation |
| Sandbox execution | Isolated subprocess for lockfile regen | Code execution safety |

---

## 9. Deployment Architecture

### Minimal (single developer, < 100 repos)

```yaml
# docker-compose.yml
services:
  ampel-api:    image: ampel/api:latest
  ampel-worker: image: ampel/worker:latest
  postgres:     image: postgres:16
  # ruvector-core embedded in ampel-intelligence crate
  # No separate Brain Server needed at this scale
```

### Team (100–1K repos)

```yaml
services:
  ampel-api:      image: ampel/api:latest
  ampel-worker:   image: ampel/worker:latest
  postgres:       image: postgres:16
  brain-server:   image: ruvnet/mcp-brain-server:latest
                  environment:
                    SONA_ENABLED: "true"
                    LORA_FEDERATION: "true"
                    RVF_DP_ENABLED: "true"
```

### Enterprise (1K–10K repos)

```
Cloud Run (auto-scaling):
  ampel-api          (min 2 instances)
  ampel-worker       (min 2 instances)
  brain-server       (min 1 instance, 4 CPU / 4 GiB)

Cloud SQL PostgreSQL 16 (HA, read replica)
Cloud Storage (Brain Server REDB backup)
```
