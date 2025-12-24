# CI/CD Workflow Automation Intelligence

> **Technical Design Document for Intelligent CI/CD Workflow Generation**

## Executive Summary

This document presents a comprehensive technical plan for adding intelligent CI/CD workflow automation to Ampel. The feature leverages vector databases (AgentDB/RuVector), local LLM inference via ONNX, and repository analysis to automatically detect missing CI/CD workflows and generate provider-specific automation tailored to each repository's tech stack.

### Key Capabilities

1. **Repository Intelligence Engine**: Analyzes repositories to detect languages, frameworks, build tools, and test frameworks using embeddings and pattern matching
2. **CI Workflow Generator**: Generates CI workflows for GitHub Actions, GitLab CI/CD, and Bitbucket Pipelines based on detected tech stack
3. **CD Workflow Generator**: Generates deployment workflows with a plugin architecture supporting Fly.io initially, with extensibility for AWS, GCP, Azure, Digital Ocean, and Alibaba Cloud
4. **Multi-tenant Security**: Secure credential storage with per-account defaults and per-repository overrides

---

## Table of Contents

1. [Research Findings](#1-research-findings)
2. [System Architecture](#2-system-architecture)
3. [Component Design](#3-component-design)
4. [Data Models and Database Schema](#4-data-models-and-database-schema)
5. [API Design](#5-api-design)
6. [Security Considerations](#6-security-considerations)
7. [Multi-tenancy Implementation](#7-multi-tenancy-implementation)
8. [Implementation Phases](#8-implementation-phases)
9. [Technology Stack Decisions](#9-technology-stack-decisions)
10. [Risk Assessment](#10-risk-assessment)
11. [Future Extensibility](#11-future-extensibility)

---

## 1. Research Findings

### 1.1 AgentDB

**Source**: [AgentDB](https://agentdb.ruv.io/) | [GitHub Issue #829](https://github.com/ruvnet/claude-flow/issues/829)

AgentDB is a high-performance vector database designed for AI agents with the following capabilities:

- **29 MCP Tools**: 5 core vector DB + 5 core AgentDB + 9 frontier memory + 10 learning system
- **Performance**: 150x-12,500x faster than traditional solutions with <100µs search latency
- **Indexing**: HNSW indexing with quantization support
- **Learning System**: Supports Q-Learning, SARSA, DQN, Policy Gradient, Actor-Critic, PPO, Decision Transformer, MCTS
- **Reflexion Memory**: Self-critique and skill library with semantic search
- **Causal Reasoning**: Pattern discovery and causal edge storage

**Key APIs for Integration**:

- `agentdb_init`: Initialize database with schema
- `agentdb_insert` / `agentdb_insert_batch`: Store embeddings with metadata
- `agentdb_search`: Semantic k-NN search with filters
- `reflexion_store` / `reflexion_retrieve`: Learning from experience
- `skill_create` / `skill_search`: Reusable skill patterns

### 1.2 RuVector

**Source**: [RuVector GitHub](https://github.com/ruvnet/ruvector)

RuVector is a distributed vector database in Rust combining multiple capabilities:

- **Vector Search**: HNSW indexing with 61µs p50 latency, 16,400 QPS
- **Graph Queries**: Neo4j-compatible Cypher syntax for complex relationships
- **Self-Learning**: GNN layers that improve search through usage patterns
- **Distributed**: Raft consensus, multi-master replication, auto-sharding
- **39 Attention Mechanisms**: Including FlashAttention, RoPE, hyperbolic embeddings
- **Compression**: 2-32x reduction with adaptive tiered compression
- **SIMD**: AVX-512/NEON acceleration

**Integration Options**:

- Direct Rust library usage (ideal for Ampel)
- Node.js bindings via napi-rs
- HTTP/gRPC server
- PostgreSQL extension (pgvector-compatible)

### 1.3 ONNX Runtime for Rust

**Key Libraries**:

| Crate                                                    | Description                      | Use Case                  |
| -------------------------------------------------------- | -------------------------------- | ------------------------- |
| [fastembed-rs](https://github.com/Anush008/fastembed-rs) | Production-ready text embeddings | Repository fingerprinting |
| [ort](https://ort.pyke.io/)                              | ONNX Runtime bindings            | General ONNX inference    |
| [candle](https://github.com/huggingface/candle)          | Hugging Face ML framework        | Custom model inference    |

**Recommended Models**:

- `all-MiniLM-L6-v2`: Fast (384-dim) embeddings for semantic search
- `BAAI/bge-small-en-v1.5`: High-quality embeddings (default in fastembed)
- `CodeBERT` / `CodeT5`: Code-specific embeddings for tech stack analysis

### 1.4 CI Provider Workflow Patterns

| Provider                | File Location             | Syntax | Key Features                               |
| ----------------------- | ------------------------- | ------ | ------------------------------------------ |
| **GitHub Actions**      | `.github/workflows/*.yml` | YAML   | Matrix builds, reusable workflows, caching |
| **GitLab CI/CD**        | `.gitlab-ci.yml`          | YAML   | Stages, includes, extends, anchors         |
| **Bitbucket Pipelines** | `bitbucket-pipelines.yml` | YAML   | Steps, parallel, caches, services          |

**Common Patterns by Language**:

- **Rust**: `cargo check`, `cargo test`, `cargo clippy`, `cargo build --release`
- **Node.js**: `npm ci`, `npm test`, `npm run build`
- **Python**: `pip install`, `pytest`, `mypy`, `ruff`
- **Go**: `go build`, `go test`, `golangci-lint`

### 1.5 Deployment Provider APIs

**Fly.io** (Initial Target):

- **API**: `https://api.machines.dev` (public) or `http://_api.internal:4280` (internal)
- **Auth**: Bearer token via `fly tokens deploy`
- **Resources**: Apps, Machines, Volumes, Tokens
- **Rate Limits**: 1 req/s standard, 5 req/s for GET

**Future Targets**:
| Provider | API Type | Key Service |
|----------|----------|-------------|
| AWS | REST/SDK | ECS, Lambda, Elastic Beanstalk |
| Google Cloud | REST/gRPC | Cloud Run, GKE, App Engine |
| Azure | REST/SDK | App Service, AKS, Functions |
| Digital Ocean | REST | App Platform, Kubernetes |
| Alibaba Cloud | REST/SDK | Container Service, Function Compute |

---

## 2. System Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              AMPEL PLATFORM                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────────┐  │
│  │   Frontend (UI)  │  │   API Gateway    │  │     Background Worker    │  │
│  │                  │  │    (Axum)        │  │       (Apalis)           │  │
│  │  - Suggestions   │  │                  │  │                          │  │
│  │  - Workflow Edit │◄─┼─► REST Endpoints │◄─┼─► Repo Analysis Jobs     │  │
│  │  - Deploy Config │  │                  │  │    Workflow Generation   │  │
│  └────────┬─────────┘  └────────┬─────────┘  │    Deployment Jobs       │  │
│           │                     │             └────────────┬─────────────┘  │
│           │                     │                          │                │
│  ─────────┼─────────────────────┼──────────────────────────┼────────────── │
│           │                     │                          │                │
│           ▼                     ▼                          ▼                │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                    INTELLIGENCE LAYER (NEW)                           │  │
│  ├──────────────────────────────────────────────────────────────────────┤  │
│  │                                                                       │  │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐   │  │
│  │  │   Repository    │  │   CI Workflow   │  │    CD Workflow      │   │  │
│  │  │  Intelligence   │  │    Generator    │  │     Generator       │   │  │
│  │  │     Engine      │  │                 │  │                     │   │  │
│  │  │                 │  │  ┌───────────┐  │  │  ┌───────────────┐  │   │  │
│  │  │ - Tech Detect   │  │  │  GitHub   │  │  │  │   Fly.io      │  │   │  │
│  │  │ - Embedding Gen │  │  │  Actions  │  │  │  │   Plugin      │  │   │  │
│  │  │ - Pattern Match │  │  ├───────────┤  │  │  ├───────────────┤  │   │  │
│  │  │ - CI/CD Detect  │  │  │  GitLab   │  │  │  │   AWS         │  │   │  │
│  │  │                 │  │  │  CI/CD    │  │  │  │   Plugin      │  │   │  │
│  │  └────────┬────────┘  │  ├───────────┤  │  │  ├───────────────┤  │   │  │
│  │           │           │  │ Bitbucket │  │  │  │   GCP/Azure   │  │   │  │
│  │           │           │  │ Pipelines │  │  │  │   Plugins     │  │   │  │
│  │           │           │  └───────────┘  │  │  └───────────────┘  │   │  │
│  │           │           └────────┬────────┘  └──────────┬──────────┘   │  │
│  │           │                    │                      │              │  │
│  └───────────┼────────────────────┼──────────────────────┼──────────────┘  │
│              │                    │                      │                 │
│  ────────────┼────────────────────┼──────────────────────┼────────────────│
│              │                    │                      │                 │
│              ▼                    ▼                      ▼                 │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                         DATA LAYER                                    │  │
│  ├──────────────────────────────────────────────────────────────────────┤  │
│  │                                                                       │  │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐   │  │
│  │  │   PostgreSQL    │  │  RuVector/      │  │    Redis Cache      │   │  │
│  │  │                 │  │  AgentDB        │  │                     │   │  │
│  │  │ - Users         │  │                 │  │ - Embedding Cache   │   │  │
│  │  │ - Repositories  │  │ - Tech Stack    │  │ - Template Cache    │   │  │
│  │  │ - Credentials   │  │   Embeddings    │  │ - Rate Limit        │   │  │
│  │  │ - Workflows     │  │ - Pattern DB    │  │                     │   │  │
│  │  │ - Deploy Config │  │ - Skills        │  │                     │   │  │
│  │  │                 │  │ - Learning      │  │                     │   │  │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────┘   │  │
│  │                                                                       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Repository │     │   Analysis   │     │  Suggestion  │     │  Generation  │
│  Registered │────▶│    Job       │────▶│   Created    │────▶│   Workflow   │
└─────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ 1. Clone/    │
                    │    Fetch     │
                    │              │
                    │ 2. Detect    │
                    │    Files     │
                    │              │
                    │ 3. Generate  │
                    │    Embedding │
                    │              │
                    │ 4. Match     │
                    │    Patterns  │
                    │              │
                    │ 5. Check     │
                    │    CI/CD     │
                    └──────────────┘
```

---

## 3. Component Design

### 3.1 Repository Intelligence Engine

The intelligence engine analyzes repositories to detect tech stacks and missing workflows.

#### 3.1.1 New Crate: `ampel-intelligence`

```rust
// crates/ampel-intelligence/src/lib.rs

pub mod analysis;
pub mod detection;
pub mod embedding;
pub mod patterns;

// Re-exports
pub use analysis::{RepositoryAnalysis, AnalysisResult};
pub use detection::{TechStackDetector, DetectedTechStack};
pub use embedding::{EmbeddingService, RepositoryFingerprint};
pub use patterns::{PatternMatcher, WorkflowPattern};
```

#### 3.1.2 Tech Stack Detection

```rust
// crates/ampel-intelligence/src/detection.rs

use std::collections::HashMap;

/// Detected technology stack for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTechStack {
    /// Primary programming languages with confidence scores
    pub languages: Vec<(Language, f32)>,

    /// Detected frameworks (e.g., React, Axum, Django)
    pub frameworks: Vec<DetectedFramework>,

    /// Build tools (e.g., Cargo, npm, Gradle)
    pub build_tools: Vec<BuildTool>,

    /// Test frameworks (e.g., Jest, pytest, cargo-test)
    pub test_frameworks: Vec<TestFramework>,

    /// Package managers (e.g., npm, pip, cargo)
    pub package_managers: Vec<PackageManager>,

    /// Detected CI/CD workflows
    pub existing_workflows: Vec<ExistingWorkflow>,

    /// Docker/containerization detected
    pub containerization: Option<ContainerConfig>,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// File pattern rules for detection
#[derive(Debug, Clone)]
pub struct DetectionRule {
    /// File patterns to match (glob)
    pub file_patterns: Vec<String>,

    /// Content patterns to search (regex)
    pub content_patterns: Vec<String>,

    /// Weight for this rule
    pub weight: f32,

    /// What this rule detects
    pub detects: DetectionTarget,
}

#[async_trait]
pub trait TechStackDetector: Send + Sync {
    /// Analyze repository and detect tech stack
    async fn detect(&self, repo_path: &Path) -> Result<DetectedTechStack, DetectionError>;

    /// Check if CI workflow exists
    async fn has_ci_workflow(&self, repo_path: &Path, provider: GitProvider)
        -> Result<bool, DetectionError>;

    /// Check if CD workflow exists
    async fn has_cd_workflow(&self, repo_path: &Path, provider: GitProvider)
        -> Result<bool, DetectionError>;
}

/// Default detector implementation
pub struct DefaultTechStackDetector {
    rules: Vec<DetectionRule>,
    embedding_service: Arc<dyn EmbeddingService>,
}

impl DefaultTechStackDetector {
    pub fn new(embedding_service: Arc<dyn EmbeddingService>) -> Self {
        Self {
            rules: Self::default_rules(),
            embedding_service,
        }
    }

    fn default_rules() -> Vec<DetectionRule> {
        vec![
            // Rust
            DetectionRule {
                file_patterns: vec!["Cargo.toml".into(), "Cargo.lock".into()],
                content_patterns: vec![r"\[package\]".into()],
                weight: 1.0,
                detects: DetectionTarget::Language(Language::Rust),
            },
            // Node.js / JavaScript
            DetectionRule {
                file_patterns: vec!["package.json".into()],
                content_patterns: vec![r#""dependencies""#.into()],
                weight: 1.0,
                detects: DetectionTarget::Language(Language::JavaScript),
            },
            // Python
            DetectionRule {
                file_patterns: vec![
                    "requirements.txt".into(),
                    "pyproject.toml".into(),
                    "setup.py".into(),
                ],
                content_patterns: vec![],
                weight: 1.0,
                detects: DetectionTarget::Language(Language::Python),
            },
            // React Framework
            DetectionRule {
                file_patterns: vec!["package.json".into()],
                content_patterns: vec![r#""react""#.into()],
                weight: 0.9,
                detects: DetectionTarget::Framework(Framework::React),
            },
            // Axum Framework
            DetectionRule {
                file_patterns: vec!["Cargo.toml".into()],
                content_patterns: vec![r#"axum\s*="#.into()],
                weight: 0.9,
                detects: DetectionTarget::Framework(Framework::Axum),
            },
            // GitHub Actions CI
            DetectionRule {
                file_patterns: vec![".github/workflows/*.yml".into()],
                content_patterns: vec![r"on:\s*\[?(push|pull_request)".into()],
                weight: 1.0,
                detects: DetectionTarget::CIWorkflow(GitProvider::GitHub),
            },
            // GitLab CI
            DetectionRule {
                file_patterns: vec![".gitlab-ci.yml".into()],
                content_patterns: vec![r"stages:".into()],
                weight: 1.0,
                detects: DetectionTarget::CIWorkflow(GitProvider::GitLab),
            },
            // Bitbucket Pipelines
            DetectionRule {
                file_patterns: vec!["bitbucket-pipelines.yml".into()],
                content_patterns: vec![r"pipelines:".into()],
                weight: 1.0,
                detects: DetectionTarget::CIWorkflow(GitProvider::Bitbucket),
            },
            // ... additional rules for frameworks, test tools, etc.
        ]
    }
}
```

#### 3.1.3 Embedding Service

```rust
// crates/ampel-intelligence/src/embedding.rs

use fastembed::{TextEmbedding, EmbeddingModel, InitOptions};

/// Repository fingerprint as vector embedding
#[derive(Debug, Clone)]
pub struct RepositoryFingerprint {
    /// Embedding vector (typically 384 dimensions for MiniLM)
    pub embedding: Vec<f32>,

    /// Detected tech stack summary
    pub tech_stack: DetectedTechStack,

    /// Repository metadata
    pub metadata: RepositoryMetadata,

    /// Timestamp of analysis
    pub analyzed_at: DateTime<Utc>,
}

#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate embedding for text
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Generate embedding batch
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// Generate repository fingerprint
    async fn fingerprint(&self, tech_stack: &DetectedTechStack)
        -> Result<RepositoryFingerprint, EmbeddingError>;
}

/// ONNX-based embedding service using fastembed-rs
pub struct OnnxEmbeddingService {
    model: TextEmbedding,
}

impl OnnxEmbeddingService {
    pub fn new() -> Result<Self, EmbeddingError> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true)
        )?;

        Ok(Self { model })
    }

    pub fn with_model(model: EmbeddingModel) -> Result<Self, EmbeddingError> {
        let model = TextEmbedding::try_new(
            InitOptions::new(model)
                .with_show_download_progress(true)
        )?;

        Ok(Self { model })
    }
}

#[async_trait]
impl EmbeddingService for OnnxEmbeddingService {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let embeddings = self.model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap())
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let texts: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        Ok(self.model.embed(texts, None)?)
    }

    async fn fingerprint(&self, tech_stack: &DetectedTechStack)
        -> Result<RepositoryFingerprint, EmbeddingError>
    {
        // Create descriptive text from tech stack
        let description = format!(
            "Repository using {} with frameworks {} and build tools {}",
            tech_stack.languages.iter()
                .map(|(l, _)| l.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            tech_stack.frameworks.iter()
                .map(|f| f.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            tech_stack.build_tools.iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );

        let embedding = self.embed(&description).await?;

        Ok(RepositoryFingerprint {
            embedding,
            tech_stack: tech_stack.clone(),
            metadata: RepositoryMetadata::default(),
            analyzed_at: Utc::now(),
        })
    }
}
```

#### 3.1.4 Vector Storage Integration

```rust
// crates/ampel-intelligence/src/storage.rs

use ruvector_core::{VectorDB, HnswConfig, SearchResult};

/// Vector storage for repository fingerprints
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Store repository fingerprint
    async fn store_fingerprint(
        &self,
        repo_id: Uuid,
        fingerprint: &RepositoryFingerprint,
    ) -> Result<(), StorageError>;

    /// Search similar repositories
    async fn search_similar(
        &self,
        embedding: &[f32],
        k: usize,
        tenant_id: Option<Uuid>,
    ) -> Result<Vec<SimilarRepository>, StorageError>;

    /// Get fingerprint by repository ID
    async fn get_fingerprint(
        &self,
        repo_id: Uuid,
    ) -> Result<Option<RepositoryFingerprint>, StorageError>;

    /// Delete fingerprint
    async fn delete_fingerprint(
        &self,
        repo_id: Uuid,
    ) -> Result<(), StorageError>;
}

/// RuVector-based implementation
pub struct RuVectorStorage {
    db: VectorDB,
    collection: String,
}

impl RuVectorStorage {
    pub async fn new(config: RuVectorConfig) -> Result<Self, StorageError> {
        let db = VectorDB::connect(&config.connection_string).await?;

        // Create collection with HNSW index
        db.create_collection(
            &config.collection_name,
            HnswConfig {
                dimensions: 384, // MiniLM embedding size
                m: 16,
                ef_construction: 200,
                distance_metric: DistanceMetric::Cosine,
            },
        ).await?;

        Ok(Self {
            db,
            collection: config.collection_name,
        })
    }
}

#[async_trait]
impl VectorStorage for RuVectorStorage {
    async fn store_fingerprint(
        &self,
        repo_id: Uuid,
        fingerprint: &RepositoryFingerprint,
    ) -> Result<(), StorageError> {
        let metadata = serde_json::to_value(&fingerprint.tech_stack)?;

        self.db.insert(
            &self.collection,
            repo_id.to_string(),
            &fingerprint.embedding,
            metadata,
        ).await?;

        Ok(())
    }

    async fn search_similar(
        &self,
        embedding: &[f32],
        k: usize,
        tenant_id: Option<Uuid>,
    ) -> Result<Vec<SimilarRepository>, StorageError> {
        let filter = tenant_id.map(|id| {
            json!({ "tenant_id": id.to_string() })
        });

        let results = self.db.search(
            &self.collection,
            embedding,
            k,
            filter,
        ).await?;

        Ok(results.into_iter().map(|r| SimilarRepository {
            repo_id: Uuid::parse_str(&r.id).unwrap(),
            similarity: r.score,
            tech_stack: serde_json::from_value(r.metadata).unwrap(),
        }).collect())
    }

    // ... other implementations
}
```

### 3.2 CI Workflow Generator

#### 3.2.1 Provider-Agnostic Workflow Trait

```rust
// crates/ampel-intelligence/src/ci/mod.rs

pub mod github;
pub mod gitlab;
pub mod bitbucket;
pub mod templates;

/// Generated workflow file
#[derive(Debug, Clone)]
pub struct GeneratedWorkflow {
    /// File path relative to repo root
    pub file_path: String,

    /// Workflow content (YAML)
    pub content: String,

    /// Human-readable description
    pub description: String,

    /// Provider this workflow is for
    pub provider: GitProvider,

    /// Workflow type
    pub workflow_type: WorkflowType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowType {
    CI,
    CD,
    Combined,
}

/// CI workflow generator trait
#[async_trait]
pub trait CIWorkflowGenerator: Send + Sync {
    /// Generate CI workflow for detected tech stack
    async fn generate(
        &self,
        tech_stack: &DetectedTechStack,
        options: &GenerationOptions,
    ) -> Result<GeneratedWorkflow, GenerationError>;

    /// Validate generated workflow syntax
    fn validate(&self, workflow: &str) -> Result<(), ValidationError>;

    /// Get provider type
    fn provider(&self) -> GitProvider;
}

/// Generation options
#[derive(Debug, Clone, Default)]
pub struct GenerationOptions {
    /// Include caching configuration
    pub enable_caching: bool,

    /// Include matrix builds
    pub enable_matrix: bool,

    /// Include security scanning
    pub enable_security_scan: bool,

    /// Include code coverage
    pub enable_coverage: bool,

    /// Custom steps to include
    pub custom_steps: Vec<CustomStep>,

    /// Branch patterns to trigger on
    pub trigger_branches: Vec<String>,
}
```

#### 3.2.2 GitHub Actions Generator

```rust
// crates/ampel-intelligence/src/ci/github.rs

use super::*;

pub struct GitHubActionsGenerator {
    templates: TemplateEngine,
}

impl GitHubActionsGenerator {
    pub fn new() -> Self {
        Self {
            templates: TemplateEngine::new(),
        }
    }

    fn generate_rust_workflow(&self, options: &GenerationOptions) -> String {
        let cache_section = if options.enable_caching {
            r#"
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-
"#
        } else {
            ""
        };

        let security_section = if options.enable_security_scan {
            r#"
      - name: Security audit
        run: cargo audit
"#
        } else {
            ""
        };

        format!(r#"name: CI

on:
  push:
    branches: [{branches}]
  pull_request:
    branches: [{branches}]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        with:
          components: clippy, rustfmt
{cache}
      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Build
        run: cargo build --all-features

      - name: Run tests
        run: cargo test --all-features
{security}
"#,
            branches = options.trigger_branches.join(", "),
            cache = cache_section,
            security = security_section,
        )
    }

    fn generate_nodejs_workflow(&self, options: &GenerationOptions) -> String {
        // Similar implementation for Node.js projects
        // ...
    }
}

#[async_trait]
impl CIWorkflowGenerator for GitHubActionsGenerator {
    async fn generate(
        &self,
        tech_stack: &DetectedTechStack,
        options: &GenerationOptions,
    ) -> Result<GeneratedWorkflow, GenerationError> {
        let content = match tech_stack.primary_language() {
            Language::Rust => self.generate_rust_workflow(options),
            Language::JavaScript | Language::TypeScript => {
                self.generate_nodejs_workflow(options)
            }
            Language::Python => self.generate_python_workflow(options),
            Language::Go => self.generate_go_workflow(options),
            _ => return Err(GenerationError::UnsupportedLanguage),
        };

        Ok(GeneratedWorkflow {
            file_path: ".github/workflows/ci.yml".into(),
            content,
            description: format!(
                "CI workflow for {} project",
                tech_stack.primary_language()
            ),
            provider: GitProvider::GitHub,
            workflow_type: WorkflowType::CI,
        })
    }

    fn validate(&self, workflow: &str) -> Result<(), ValidationError> {
        // Parse YAML and validate structure
        let _parsed: serde_yaml::Value = serde_yaml::from_str(workflow)?;
        Ok(())
    }

    fn provider(&self) -> GitProvider {
        GitProvider::GitHub
    }
}
```

### 3.3 CD Workflow Generator with Plugin Architecture

#### 3.3.1 Deployment Provider Trait

```rust
// crates/ampel-intelligence/src/cd/mod.rs

pub mod flyio;
pub mod aws;
pub mod gcp;
pub mod azure;
pub mod digitalocean;

/// Deployment provider plugin interface
#[async_trait]
pub trait DeploymentProvider: Send + Sync {
    /// Provider identifier
    fn provider_id(&self) -> &'static str;

    /// Human-readable name
    fn display_name(&self) -> &'static str;

    /// Required credentials schema
    fn credential_schema(&self) -> CredentialSchema;

    /// Validate credentials
    async fn validate_credentials(
        &self,
        credentials: &ProviderCredentials,
    ) -> Result<bool, DeploymentError>;

    /// Generate deployment workflow
    async fn generate_workflow(
        &self,
        tech_stack: &DetectedTechStack,
        config: &DeploymentConfig,
        git_provider: GitProvider,
    ) -> Result<GeneratedWorkflow, DeploymentError>;

    /// Generate configuration file (e.g., fly.toml, app.yaml)
    fn generate_config(
        &self,
        tech_stack: &DetectedTechStack,
        config: &DeploymentConfig,
    ) -> Result<Option<GeneratedFile>, DeploymentError>;

    /// Execute deployment (for direct API deployments)
    async fn deploy(
        &self,
        credentials: &ProviderCredentials,
        config: &DeploymentConfig,
    ) -> Result<DeploymentResult, DeploymentError>;
}

/// Credential schema for UI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSchema {
    pub fields: Vec<CredentialField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialField {
    pub name: String,
    pub label: String,
    pub field_type: CredentialFieldType,
    pub required: bool,
    pub description: String,
    pub validation_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialFieldType {
    ApiToken,
    Secret,
    Text,
    Select(Vec<String>),
    Region,
}
```

#### 3.3.2 Fly.io Provider Implementation

```rust
// crates/ampel-intelligence/src/cd/flyio.rs

use super::*;
use reqwest::Client;

pub struct FlyioProvider {
    client: Client,
    api_base: String,
}

impl FlyioProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_base: "https://api.machines.dev".into(),
        }
    }
}

#[async_trait]
impl DeploymentProvider for FlyioProvider {
    fn provider_id(&self) -> &'static str {
        "flyio"
    }

    fn display_name(&self) -> &'static str {
        "Fly.io"
    }

    fn credential_schema(&self) -> CredentialSchema {
        CredentialSchema {
            fields: vec![
                CredentialField {
                    name: "api_token".into(),
                    label: "Fly.io API Token".into(),
                    field_type: CredentialFieldType::ApiToken,
                    required: true,
                    description: "Generate with 'fly tokens deploy'".into(),
                    validation_pattern: Some(r"^fm[12]_[A-Za-z0-9_-]+$".into()),
                },
                CredentialField {
                    name: "app_name".into(),
                    label: "Application Name".into(),
                    field_type: CredentialFieldType::Text,
                    required: false,
                    description: "Leave blank to auto-generate from repo name".into(),
                    validation_pattern: Some(r"^[a-z0-9-]+$".into()),
                },
                CredentialField {
                    name: "region".into(),
                    label: "Primary Region".into(),
                    field_type: CredentialFieldType::Select(vec![
                        "iad".into(), "lax".into(), "ord".into(),
                        "sjc".into(), "ams".into(), "lhr".into(),
                    ]),
                    required: true,
                    description: "Deployment region".into(),
                    validation_pattern: None,
                },
            ],
        }
    }

    async fn validate_credentials(
        &self,
        credentials: &ProviderCredentials,
    ) -> Result<bool, DeploymentError> {
        let token = credentials.get_secret("api_token")?;

        let response = self.client
            .get(format!("{}/v1/apps", self.api_base))
            .bearer_auth(&token)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn generate_workflow(
        &self,
        tech_stack: &DetectedTechStack,
        config: &DeploymentConfig,
        git_provider: GitProvider,
    ) -> Result<GeneratedWorkflow, DeploymentError> {
        match git_provider {
            GitProvider::GitHub => {
                self.generate_github_workflow(tech_stack, config)
            }
            GitProvider::GitLab => {
                self.generate_gitlab_workflow(tech_stack, config)
            }
            GitProvider::Bitbucket => {
                self.generate_bitbucket_workflow(tech_stack, config)
            }
        }
    }

    fn generate_config(
        &self,
        tech_stack: &DetectedTechStack,
        config: &DeploymentConfig,
    ) -> Result<Option<GeneratedFile>, DeploymentError> {
        let app_name = config.app_name.as_deref()
            .unwrap_or("my-app");
        let region = config.region.as_deref()
            .unwrap_or("iad");

        let (internal_port, memory) = match tech_stack.primary_language() {
            Language::Rust => (8080, "256mb"),
            Language::JavaScript => (3000, "256mb"),
            Language::Python => (8000, "512mb"),
            Language::Go => (8080, "256mb"),
            _ => (8080, "256mb"),
        };

        let fly_toml = format!(r#"# Fly.io configuration
# Generated by Ampel CI/CD Intelligence

app = "{app_name}"
primary_region = "{region}"

[build]

[http_service]
  internal_port = {internal_port}
  force_https = true
  auto_stop_machines = "stop"
  auto_start_machines = true
  min_machines_running = 0

[[vm]]
  memory = "{memory}"
  cpu_kind = "shared"
  cpus = 1
"#);

        Ok(Some(GeneratedFile {
            path: "fly.toml".into(),
            content: fly_toml,
        }))
    }

    async fn deploy(
        &self,
        credentials: &ProviderCredentials,
        config: &DeploymentConfig,
    ) -> Result<DeploymentResult, DeploymentError> {
        // Direct API deployment using Machines API
        let token = credentials.get_secret("api_token")?;

        // Create/update app
        let app_name = config.app_name.as_ref()
            .ok_or(DeploymentError::MissingConfig("app_name"))?;

        // Implementation details for machine creation/update
        // ...

        Ok(DeploymentResult {
            success: true,
            url: Some(format!("https://{}.fly.dev", app_name)),
            deployment_id: Some(Uuid::new_v4().to_string()),
            logs: vec![],
        })
    }
}

impl FlyioProvider {
    fn generate_github_workflow(
        &self,
        _tech_stack: &DetectedTechStack,
        config: &DeploymentConfig,
    ) -> Result<GeneratedWorkflow, DeploymentError> {
        let app_name = config.app_name.as_deref()
            .unwrap_or("${{ github.event.repository.name }}");

        let workflow = format!(r#"name: Deploy to Fly.io

on:
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  deploy:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Fly.io CLI
        uses: superfly/flyctl-actions/setup-flyctl@master

      - name: Deploy to Fly.io
        run: flyctl deploy --remote-only
        env:
          FLY_API_TOKEN: ${{{{ secrets.FLY_API_TOKEN }}}}
          FLY_APP: {app_name}
"#);

        Ok(GeneratedWorkflow {
            file_path: ".github/workflows/deploy.yml".into(),
            content: workflow,
            description: "Deploy to Fly.io on push to main".into(),
            provider: GitProvider::GitHub,
            workflow_type: WorkflowType::CD,
        })
    }

    fn generate_gitlab_workflow(
        &self,
        _tech_stack: &DetectedTechStack,
        _config: &DeploymentConfig,
    ) -> Result<GeneratedWorkflow, DeploymentError> {
        let workflow = r#"deploy:
  stage: deploy
  image: flyio/flyctl:latest
  only:
    - main
  script:
    - flyctl deploy --remote-only
  variables:
    FLY_API_TOKEN: $FLY_API_TOKEN
"#;

        Ok(GeneratedWorkflow {
            file_path: ".gitlab-ci.yml".into(),
            content: workflow.into(),
            description: "Deploy to Fly.io via GitLab CI".into(),
            provider: GitProvider::GitLab,
            workflow_type: WorkflowType::CD,
        })
    }
}
```

#### 3.3.3 Plugin Registry

```rust
// crates/ampel-intelligence/src/cd/registry.rs

use std::collections::HashMap;
use std::sync::Arc;

/// Registry for deployment provider plugins
pub struct DeploymentProviderRegistry {
    providers: HashMap<String, Arc<dyn DeploymentProvider>>,
}

impl DeploymentProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
        };

        // Register built-in providers
        registry.register(Arc::new(FlyioProvider::new()));
        // Future: registry.register(Arc::new(AwsProvider::new()));
        // Future: registry.register(Arc::new(GcpProvider::new()));

        registry
    }

    pub fn register(&mut self, provider: Arc<dyn DeploymentProvider>) {
        self.providers.insert(
            provider.provider_id().to_string(),
            provider,
        );
    }

    pub fn get(&self, provider_id: &str) -> Option<Arc<dyn DeploymentProvider>> {
        self.providers.get(provider_id).cloned()
    }

    pub fn list(&self) -> Vec<ProviderInfo> {
        self.providers.values()
            .map(|p| ProviderInfo {
                id: p.provider_id().to_string(),
                name: p.display_name().to_string(),
                credential_schema: p.credential_schema(),
            })
            .collect()
    }
}
```

---

## 4. Data Models and Database Schema

### 4.1 New Database Tables

```sql
-- Migration: Add CI/CD automation tables

-- Repository analysis results
CREATE TABLE repository_analyses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,

    -- Detected tech stack (JSON)
    tech_stack JSONB NOT NULL,

    -- Embedding vector for similarity search
    embedding_id VARCHAR(255), -- Reference to RuVector

    -- CI/CD status
    has_ci_workflow BOOLEAN NOT NULL DEFAULT FALSE,
    has_cd_workflow BOOLEAN NOT NULL DEFAULT FALSE,
    ci_provider VARCHAR(50), -- github, gitlab, bitbucket
    cd_provider VARCHAR(50), -- flyio, aws, gcp, etc.

    -- Analysis metadata
    confidence_score REAL NOT NULL DEFAULT 0.0,
    file_count INTEGER NOT NULL DEFAULT 0,
    analyzed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Tenant isolation
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_repo_analysis UNIQUE (repository_id)
);

CREATE INDEX idx_repo_analyses_user ON repository_analyses(user_id);
CREATE INDEX idx_repo_analyses_has_ci ON repository_analyses(has_ci_workflow);
CREATE INDEX idx_repo_analyses_has_cd ON repository_analyses(has_cd_workflow);

-- CI/CD suggestions for repositories
CREATE TABLE workflow_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    analysis_id UUID NOT NULL REFERENCES repository_analyses(id) ON DELETE CASCADE,

    -- Suggestion details
    suggestion_type VARCHAR(10) NOT NULL CHECK (suggestion_type IN ('ci', 'cd')),
    title VARCHAR(255) NOT NULL,
    description TEXT,

    -- Generated workflow
    workflow_content TEXT NOT NULL,
    file_path VARCHAR(255) NOT NULL,
    git_provider VARCHAR(50) NOT NULL,

    -- For CD suggestions
    deployment_provider VARCHAR(50),
    config_content TEXT,
    config_path VARCHAR(255),

    -- User interaction
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'rejected', 'applied')),
    user_feedback TEXT,

    -- Tenant isolation
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workflow_suggestions_repo ON workflow_suggestions(repository_id);
CREATE INDEX idx_workflow_suggestions_status ON workflow_suggestions(status);
CREATE INDEX idx_workflow_suggestions_user ON workflow_suggestions(user_id);

-- Deployment credentials per account/repository
CREATE TABLE deployment_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Provider identification
    provider_id VARCHAR(50) NOT NULL, -- flyio, aws, gcp, etc.
    label VARCHAR(255) NOT NULL, -- User-friendly name

    -- Scope
    scope VARCHAR(20) NOT NULL DEFAULT 'account'
        CHECK (scope IN ('account', 'repository')),
    repository_id UUID REFERENCES repositories(id) ON DELETE CASCADE,

    -- Whether this is the default for the account
    is_default BOOLEAN NOT NULL DEFAULT FALSE,

    -- Encrypted credentials
    credentials_encrypted BYTEA NOT NULL,

    -- Validation status
    last_validated_at TIMESTAMPTZ,
    validation_status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (validation_status IN ('pending', 'valid', 'invalid', 'expired')),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Only one default per provider per user
    CONSTRAINT unique_default_per_provider
        UNIQUE NULLS NOT DISTINCT (user_id, provider_id, is_default)
        WHERE is_default = TRUE AND scope = 'account'
);

CREATE INDEX idx_deploy_creds_user ON deployment_credentials(user_id);
CREATE INDEX idx_deploy_creds_provider ON deployment_credentials(provider_id);
CREATE INDEX idx_deploy_creds_repo ON deployment_credentials(repository_id);

-- Deployment history
CREATE TABLE deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    credential_id UUID REFERENCES deployment_credentials(id) ON DELETE SET NULL,

    -- Deployment details
    provider_id VARCHAR(50) NOT NULL,
    deployment_type VARCHAR(20) NOT NULL CHECK (deployment_type IN ('manual', 'workflow')),

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'running', 'success', 'failed', 'cancelled')),

    -- Result
    deployment_url VARCHAR(500),
    provider_deployment_id VARCHAR(255),
    error_message TEXT,
    logs JSONB,

    -- Timing
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,

    -- Tenant isolation
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_deployments_repo ON deployments(repository_id);
CREATE INDEX idx_deployments_user ON deployments(user_id);
CREATE INDEX idx_deployments_status ON deployments(status);
```

### 4.2 SeaORM Entities

```rust
// crates/ampel-db/src/entities/repository_analysis.rs

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "repository_analyses")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub repository_id: Uuid,
    pub tech_stack: JsonValue,
    pub embedding_id: Option<String>,
    pub has_ci_workflow: bool,
    pub has_cd_workflow: bool,
    pub ci_provider: Option<String>,
    pub cd_provider: Option<String>,
    pub confidence_score: f32,
    pub file_count: i32,
    pub analyzed_at: DateTimeUtc,
    pub user_id: Uuid,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

// ... Relations and ActiveModelBehavior

// crates/ampel-db/src/entities/deployment_credential.rs

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "deployment_credentials")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider_id: String,
    pub label: String,
    pub scope: String,
    pub repository_id: Option<Uuid>,
    pub is_default: bool,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub credentials_encrypted: Vec<u8>,
    pub last_validated_at: Option<DateTimeUtc>,
    pub validation_status: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

---

## 5. API Design

### 5.1 REST Endpoints

```yaml
# OpenAPI 3.0 specification excerpt

paths:
  # Repository Analysis
  /api/v1/repositories/{repo_id}/analysis:
    get:
      summary: Get repository analysis
      description: Get tech stack analysis and CI/CD status for a repository
      tags: [Analysis]
      parameters:
        - name: repo_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        200:
          description: Analysis result
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/RepositoryAnalysis'

    post:
      summary: Trigger repository analysis
      description: Queue a new analysis job for the repository
      tags: [Analysis]
      parameters:
        - name: repo_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                force:
                  type: boolean
                  description: Re-analyze even if recent analysis exists
      responses:
        202:
          description: Analysis job queued
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/JobStatus'

  # Workflow Suggestions
  /api/v1/repositories/{repo_id}/suggestions:
    get:
      summary: Get workflow suggestions
      description: Get CI/CD workflow suggestions for a repository
      tags: [Suggestions]
      parameters:
        - name: repo_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
        - name: type
          in: query
          schema:
            type: string
            enum: [ci, cd, all]
      responses:
        200:
          description: List of suggestions
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/WorkflowSuggestion'

  /api/v1/suggestions/{suggestion_id}/apply:
    post:
      summary: Apply workflow suggestion
      description: Create a PR with the suggested workflow
      tags: [Suggestions]
      parameters:
        - name: suggestion_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                branch_name:
                  type: string
                  description: Branch name for the PR
                commit_message:
                  type: string
                  description: Custom commit message
      responses:
        200:
          description: PR created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApplyResult'

  # Deployment Credentials
  /api/v1/deployment-credentials:
    get:
      summary: List deployment credentials
      description: List all deployment credentials for the current user
      tags: [Credentials]
      parameters:
        - name: provider_id
          in: query
          schema:
            type: string
      responses:
        200:
          description: List of credentials (tokens masked)
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/DeploymentCredential'

    post:
      summary: Add deployment credential
      description: Add a new deployment provider credential
      tags: [Credentials]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateDeploymentCredential'
      responses:
        201:
          description: Credential created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DeploymentCredential'

  /api/v1/deployment-credentials/{cred_id}:
    put:
      summary: Update deployment credential
      tags: [Credentials]
    delete:
      summary: Delete deployment credential
      tags: [Credentials]

  /api/v1/deployment-credentials/{cred_id}/validate:
    post:
      summary: Validate credential
      description: Test if the credential is valid with the provider
      tags: [Credentials]
      responses:
        200:
          description: Validation result
          content:
            application/json:
              schema:
                type: object
                properties:
                  valid:
                    type: boolean
                  message:
                    type: string

  # Deployment Providers
  /api/v1/deployment-providers:
    get:
      summary: List available deployment providers
      description: Get list of supported deployment providers and their schemas
      tags: [Providers]
      responses:
        200:
          description: List of providers
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/DeploymentProvider'

components:
  schemas:
    RepositoryAnalysis:
      type: object
      properties:
        id:
          type: string
          format: uuid
        repository_id:
          type: string
          format: uuid
        tech_stack:
          $ref: '#/components/schemas/TechStack'
        has_ci_workflow:
          type: boolean
        has_cd_workflow:
          type: boolean
        ci_provider:
          type: string
          enum: [github, gitlab, bitbucket]
        confidence_score:
          type: number
          format: float
        analyzed_at:
          type: string
          format: date-time

    TechStack:
      type: object
      properties:
        languages:
          type: array
          items:
            type: object
            properties:
              language:
                type: string
              confidence:
                type: number
        frameworks:
          type: array
          items:
            type: string
        build_tools:
          type: array
          items:
            type: string
        test_frameworks:
          type: array
          items:
            type: string

    WorkflowSuggestion:
      type: object
      properties:
        id:
          type: string
          format: uuid
        type:
          type: string
          enum: [ci, cd]
        title:
          type: string
        description:
          type: string
        workflow_content:
          type: string
        file_path:
          type: string
        deployment_provider:
          type: string
        status:
          type: string
          enum: [pending, accepted, rejected, applied]

    CreateDeploymentCredential:
      type: object
      required:
        - provider_id
        - label
        - credentials
      properties:
        provider_id:
          type: string
        label:
          type: string
        scope:
          type: string
          enum: [account, repository]
        repository_id:
          type: string
          format: uuid
        is_default:
          type: boolean
        credentials:
          type: object
          additionalProperties:
            type: string
```

### 5.2 Axum Handler Implementation

```rust
// crates/ampel-api/src/routes/analysis.rs

use axum::{
    extract::{Path, State, Query},
    Json,
    response::IntoResponse,
};
use uuid::Uuid;

pub async fn get_analysis(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user owns repository
    let repo = state.repo_service
        .get_repository(repo_id, user.id)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Get analysis
    let analysis = state.intelligence_service
        .get_analysis(repo_id)
        .await?;

    Ok(Json(analysis))
}

pub async fn trigger_analysis(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<TriggerAnalysisRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user owns repository
    let repo = state.repo_service
        .get_repository(repo_id, user.id)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Queue analysis job
    let job_id = state.job_queue
        .enqueue(AnalysisJob {
            repository_id: repo_id,
            user_id: user.id,
            force: request.force.unwrap_or(false),
        })
        .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(JobStatus { job_id, status: "queued".into() })
    ))
}

pub async fn get_suggestions(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    Query(params): Query<SuggestionParams>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user owns repository
    let _ = state.repo_service
        .get_repository(repo_id, user.id)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Get suggestions
    let suggestions = state.intelligence_service
        .get_suggestions(repo_id, params.suggestion_type)
        .await?;

    Ok(Json(suggestions))
}

pub async fn apply_suggestion(
    State(state): State<AppState>,
    Path(suggestion_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<ApplySuggestionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Get suggestion and verify ownership
    let suggestion = state.intelligence_service
        .get_suggestion(suggestion_id, user.id)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Create PR with the workflow
    let result = state.intelligence_service
        .apply_suggestion(suggestion, request, user.id)
        .await?;

    Ok(Json(result))
}
```

---

## 6. Security Considerations

### 6.1 Credential Storage

All deployment credentials are encrypted at rest using the existing AES-256-GCM encryption service:

```rust
// Credential encryption flow
impl DeploymentCredentialService {
    pub async fn store_credential(
        &self,
        user_id: Uuid,
        provider_id: &str,
        credentials: HashMap<String, String>,
        scope: CredentialScope,
    ) -> Result<DeploymentCredential, ServiceError> {
        // Serialize credentials
        let credentials_json = serde_json::to_string(&credentials)?;

        // Encrypt using existing encryption service
        let encrypted = self.encryption_service.encrypt(&credentials_json)?;

        // Store in database
        let model = deployment_credential::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            provider_id: Set(provider_id.to_string()),
            credentials_encrypted: Set(encrypted),
            // ... other fields
        };

        let result = model.insert(&self.db).await?;
        Ok(result.into())
    }

    pub async fn get_decrypted_credential(
        &self,
        cred_id: Uuid,
        user_id: Uuid, // Authorization check
    ) -> Result<HashMap<String, String>, ServiceError> {
        // Fetch and verify ownership
        let cred = self.get_credential(cred_id, user_id).await?;

        // Decrypt
        let decrypted_json = self.encryption_service
            .decrypt(&cred.credentials_encrypted)?;

        let credentials: HashMap<String, String> =
            serde_json::from_str(&decrypted_json)?;

        Ok(credentials)
    }
}
```

### 6.2 Tenant Isolation

All queries enforce tenant isolation:

```rust
// All queries include user_id filter
pub async fn get_suggestions_for_user(
    db: &DatabaseConnection,
    repository_id: Uuid,
    user_id: Uuid, // REQUIRED - enforces tenant isolation
) -> Result<Vec<workflow_suggestion::Model>, DbErr> {
    workflow_suggestion::Entity::find()
        .filter(workflow_suggestion::Column::RepositoryId.eq(repository_id))
        .filter(workflow_suggestion::Column::UserId.eq(user_id)) // Critical!
        .all(db)
        .await
}
```

### 6.3 Workflow Security

Generated workflows are validated to prevent injection:

```rust
// Workflow validation
impl WorkflowValidator {
    pub fn validate(&self, workflow: &str) -> Result<(), ValidationError> {
        // Parse YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(workflow)?;

        // Check for dangerous patterns
        self.check_no_shell_injection(&parsed)?;
        self.check_no_credential_exposure(&parsed)?;
        self.check_no_external_script_execution(&parsed)?;

        Ok(())
    }

    fn check_no_shell_injection(&self, yaml: &Value) -> Result<(), ValidationError> {
        // Ensure no untrusted input in shell commands
        // Pattern: run: echo ${{ github.event.pull_request.title }}
        let dangerous_patterns = [
            r"\$\{\{\s*github\.event\.(issue|pull_request)\.(title|body)",
            r"\$\{\{\s*github\.event\.comment\.body",
        ];

        let yaml_str = serde_yaml::to_string(yaml)?;
        for pattern in dangerous_patterns {
            if Regex::new(pattern)?.is_match(&yaml_str) {
                return Err(ValidationError::PotentialInjection(pattern.into()));
            }
        }

        Ok(())
    }
}
```

### 6.4 API Token Scoping

Deployment tokens are validated for minimum required scopes:

```rust
impl FlyioProvider {
    async fn validate_credentials(
        &self,
        credentials: &ProviderCredentials,
    ) -> Result<TokenValidation, DeploymentError> {
        let token = credentials.get_secret("api_token")?;

        // Validate token format
        if !token.starts_with("fm1_") && !token.starts_with("fm2_") {
            return Err(DeploymentError::InvalidTokenFormat);
        }

        // Test API access
        let response = self.client
            .get(format!("{}/v1/apps", self.api_base))
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            return Ok(TokenValidation {
                valid: false,
                message: Some("Invalid or expired token".into()),
                scopes: vec![],
            });
        }

        Ok(TokenValidation {
            valid: true,
            message: None,
            scopes: vec!["deploy".into()], // Fly tokens are deploy-scoped
        })
    }
}
```

### 6.5 Audit Logging

All credential and deployment operations are logged:

```rust
// Audit events
pub enum AuditEvent {
    CredentialCreated { provider_id: String, scope: String },
    CredentialDeleted { provider_id: String },
    CredentialAccessed { provider_id: String, purpose: String },
    DeploymentInitiated { repository_id: Uuid, provider_id: String },
    DeploymentCompleted { deployment_id: Uuid, status: String },
    WorkflowApplied { suggestion_id: Uuid, repository_id: Uuid },
}

impl AuditService {
    pub async fn log(&self, user_id: Uuid, event: AuditEvent) -> Result<(), AuditError> {
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            user_id,
            event_type: event.event_type(),
            event_data: serde_json::to_value(&event)?,
            ip_address: self.request_context.ip_address(),
            user_agent: self.request_context.user_agent(),
            created_at: Utc::now(),
        };

        entry.insert(&self.db).await?;
        Ok(())
    }
}
```

---

## 7. Multi-tenancy Implementation

### 7.1 Data Isolation

```
┌──────────────────────────────────────────────────────────────────┐
│                        TENANT ISOLATION                           │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐            │
│  │  User A     │   │  User B     │   │  User C     │            │
│  │             │   │             │   │             │            │
│  │ Repos: 5    │   │ Repos: 12   │   │ Repos: 3    │            │
│  │ Creds: 2    │   │ Creds: 5    │   │ Creds: 1    │            │
│  │ Analyses: 5 │   │ Analyses: 12│   │ Analyses: 3 │            │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘            │
│         │                 │                 │                    │
│         ▼                 ▼                 ▼                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    PostgreSQL                             │   │
│  │                                                           │   │
│  │   All tables include user_id foreign key                 │   │
│  │   Row-level security via application queries              │   │
│  │                                                           │   │
│  │   SELECT * FROM deployment_credentials                    │   │
│  │   WHERE user_id = $current_user_id  -- Always enforced   │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    RuVector / AgentDB                     │   │
│  │                                                           │   │
│  │   Embeddings tagged with tenant_id in metadata           │   │
│  │   Search queries filtered by tenant_id                    │   │
│  │                                                           │   │
│  │   // Store with tenant context                            │   │
│  │   db.insert(embedding, { tenant_id: user_id, ... })      │   │
│  │                                                           │   │
│  │   // Query with tenant filter                             │   │
│  │   db.search(query, { filter: { tenant_id: user_id } })   │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

### 7.2 Credential Scoping

```
┌────────────────────────────────────────────────────────────────┐
│                    CREDENTIAL HIERARCHY                         │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│   User Account                                                  │
│   └── Default Fly.io Token (account-level, is_default=true)   │
│       │                                                        │
│       ├── Repository: my-api                                   │
│       │   └── (uses account default)                          │
│       │                                                        │
│       ├── Repository: client-app                               │
│       │   └── Custom Fly.io Token (repo-level override)       │
│       │                                                        │
│       └── Repository: internal-tool                            │
│           └── (uses account default)                          │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

```rust
// Credential resolution logic
impl DeploymentCredentialService {
    /// Get the appropriate credential for a repository
    pub async fn resolve_credential(
        &self,
        user_id: Uuid,
        repository_id: Uuid,
        provider_id: &str,
    ) -> Result<Option<DeploymentCredential>, ServiceError> {
        // 1. Check for repository-specific credential
        let repo_cred = self.find_by_repo(
            user_id,
            repository_id,
            provider_id
        ).await?;

        if repo_cred.is_some() {
            return Ok(repo_cred);
        }

        // 2. Fall back to account default
        let default_cred = self.find_default(
            user_id,
            provider_id
        ).await?;

        Ok(default_cred)
    }
}
```

---

## 8. Implementation Phases

### Phase 1: Foundation (4-6 weeks)

**Goals**: Core infrastructure and repository analysis

- [ ] Create `ampel-intelligence` crate
- [ ] Implement tech stack detection rules
- [ ] Integrate fastembed-rs for embeddings
- [ ] Set up RuVector/AgentDB integration
- [ ] Database migrations for new tables
- [ ] Basic API endpoints for analysis

**Deliverables**:

- Repository analysis working for Rust, Node.js, Python, Go
- Tech stack embeddings stored in vector database
- CI/CD workflow detection for all three providers

### Phase 2: CI Generation (3-4 weeks)

**Goals**: Generate CI workflows for detected tech stacks

- [ ] Implement GitHub Actions generator
- [ ] Implement GitLab CI generator
- [ ] Implement Bitbucket Pipelines generator
- [ ] Template engine for workflow customization
- [ ] Suggestion API endpoints
- [ ] Frontend UI for suggestions

**Deliverables**:

- CI workflow suggestions visible in dashboard
- Users can preview and customize workflows
- Apply workflow via PR creation

### Phase 3: CD Generation - Fly.io (3-4 weeks)

**Goals**: Deployment workflow generation and credential management

- [ ] Deployment provider trait and registry
- [ ] Fly.io provider implementation
- [ ] Credential storage with encryption
- [ ] Account/repository credential scoping
- [ ] Deployment workflow generation
- [ ] Credential management UI

**Deliverables**:

- Users can add Fly.io API tokens
- CD workflows generated for Fly.io
- Account default with per-repo override

### Phase 4: Production Hardening (2-3 weeks)

**Goals**: Security, performance, and reliability

- [ ] Security audit of credential handling
- [ ] Rate limiting for analysis jobs
- [ ] Caching for embeddings
- [ ] Background job monitoring
- [ ] Audit logging
- [ ] Error handling and recovery

**Deliverables**:

- Production-ready security posture
- Monitoring and alerting
- Documentation complete

### Phase 5: Provider Extensions (Ongoing)

**Goals**: Additional deployment providers

- [ ] AWS provider (ECS, Lambda)
- [ ] Google Cloud provider (Cloud Run)
- [ ] Azure provider (App Service)
- [ ] Digital Ocean provider
- [ ] Provider marketplace/plugin system

**Deliverables**:

- Multiple deployment targets available
- Plugin documentation for contributors

---

## 9. Technology Stack Decisions

### 9.1 Vector Database: RuVector with AgentDB MCP

**Decision**: Use RuVector as the primary vector store with AgentDB MCP tools for enhanced learning

**Rationale**:

- Native Rust integration with Ampel backend
- HNSW indexing with <100µs latency
- Built-in tenant isolation via metadata filtering
- Learning capabilities for improving suggestions over time
- Optional AgentDB MCP for reflexion patterns

### 9.2 Embedding Model: fastembed-rs with all-MiniLM-L6-v2

**Decision**: Use fastembed-rs with MiniLM for text embeddings

**Rationale**:

- Production-ready Rust library
- Fast inference (~5ms per text)
- 384-dimensional embeddings (good balance of quality/size)
- ONNX backend for portability
- MIT licensed

### 9.3 Template Engine: Built-in with YAML generation

**Decision**: Custom template engine using Rust string formatting

**Rationale**:

- Full control over generated YAML
- Type-safe template parameters
- No external template language dependency
- Easy to validate output

### 9.4 Background Jobs: Existing Apalis infrastructure

**Decision**: Extend existing Apalis job queue for analysis jobs

**Rationale**:

- Already integrated in Ampel
- PostgreSQL persistence
- Retry handling
- No new infrastructure needed

---

## 10. Risk Assessment

### 10.1 Technical Risks

| Risk                                     | Impact   | Likelihood | Mitigation                                  |
| ---------------------------------------- | -------- | ---------- | ------------------------------------------- |
| Embedding model too large for deployment | High     | Medium     | Use quantized models, lazy loading          |
| Vector search latency under load         | Medium   | Low        | HNSW tuning, caching, sharding              |
| Generated workflows have errors          | High     | Medium     | Extensive testing, syntax validation        |
| Provider API changes                     | Medium   | Medium     | Abstraction layer, version pinning          |
| Credential leakage                       | Critical | Low        | Encryption, audit logging, minimal exposure |

### 10.2 Security Risks

| Risk                              | Impact   | Mitigation                              |
| --------------------------------- | -------- | --------------------------------------- |
| SQL injection in tech stack data  | High     | Parameterized queries, input validation |
| Workflow injection via PR titles  | High     | Sanitize all user input in workflows    |
| Token theft from memory           | Critical | Minimize decrypted token lifetime       |
| Unauthorized access to embeddings | Medium   | Tenant filtering at all query points    |

### 10.3 Operational Risks

| Risk                          | Impact | Mitigation                               |
| ----------------------------- | ------ | ---------------------------------------- |
| Analysis jobs overload system | Medium | Rate limiting, job prioritization        |
| Stale analysis data           | Low    | Periodic re-analysis, cache invalidation |
| Provider rate limiting        | Medium | Respect limits, exponential backoff      |

---

## 11. Future Extensibility

### 11.1 Additional Deployment Providers

The plugin architecture enables easy addition of new providers:

```rust
// Example: Adding AWS ECS provider
pub struct AwsEcsProvider {
    client: aws_sdk_ecs::Client,
}

#[async_trait]
impl DeploymentProvider for AwsEcsProvider {
    fn provider_id(&self) -> &'static str { "aws-ecs" }
    fn display_name(&self) -> &'static str { "AWS ECS" }

    fn credential_schema(&self) -> CredentialSchema {
        CredentialSchema {
            fields: vec![
                CredentialField {
                    name: "access_key_id".into(),
                    label: "AWS Access Key ID".into(),
                    field_type: CredentialFieldType::Text,
                    required: true,
                    // ...
                },
                CredentialField {
                    name: "secret_access_key".into(),
                    label: "AWS Secret Access Key".into(),
                    field_type: CredentialFieldType::Secret,
                    required: true,
                    // ...
                },
                CredentialField {
                    name: "region".into(),
                    label: "AWS Region".into(),
                    field_type: CredentialFieldType::Select(vec![
                        "us-east-1".into(),
                        "us-west-2".into(),
                        "eu-west-1".into(),
                        // ...
                    ]),
                    required: true,
                    // ...
                },
            ],
        }
    }

    // ... implementation
}
```

### 11.2 Learning and Improvement

Using AgentDB's reflexion patterns to improve suggestions:

```rust
// Store successful workflow applications
async fn record_workflow_success(
    &self,
    suggestion: &WorkflowSuggestion,
    outcome: WorkflowOutcome,
) -> Result<(), ServiceError> {
    // Store episode for reflexion learning
    self.agentdb.reflexion_store(ReflexionEpisode {
        session_id: format!("workflow-{}", suggestion.id),
        task: format!("Generate {} workflow for {}",
            suggestion.suggestion_type,
            suggestion.tech_stack_summary()),
        input: serde_json::to_string(&suggestion.tech_stack)?,
        output: suggestion.workflow_content.clone(),
        reward: if outcome.success { 1.0 } else { 0.0 },
        success: outcome.success,
        critique: outcome.user_feedback.clone(),
        latency_ms: Some(outcome.generation_time_ms),
        tokens: None,
    }).await?;

    Ok(())
}

// Use past experiences to improve generation
async fn generate_with_learning(
    &self,
    tech_stack: &DetectedTechStack,
) -> Result<GeneratedWorkflow, ServiceError> {
    // Retrieve similar successful generations
    let similar = self.agentdb.reflexion_retrieve(ReflexionQuery {
        task: format!("Generate workflow for {}", tech_stack.summary()),
        k: 5,
        only_successes: Some(true),
        min_reward: Some(0.8),
    }).await?;

    // Use successful patterns to guide generation
    let patterns = similar.iter()
        .map(|ep| serde_json::from_str::<TechStack>(&ep.input).ok())
        .flatten()
        .collect();

    self.generator.generate_with_context(tech_stack, patterns)
}
```

### 11.3 Advanced Features Roadmap

1. **Smart Scheduling**: Learn optimal deployment times from success patterns
2. **Canary Deployments**: Progressive rollout support for CD
3. **Rollback Automation**: Detect failures and auto-rollback
4. **Cost Optimization**: Suggest resource configurations based on usage
5. **Multi-Environment**: Support dev/staging/prod deployment chains
6. **Secrets Management**: Integration with Vault, AWS Secrets Manager
7. **Compliance Scanning**: Ensure workflows meet security policies

---

## Appendix A: File Structure

```
crates/
├── ampel-intelligence/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── analysis/
│       │   ├── mod.rs
│       │   ├── analyzer.rs
│       │   └── job.rs
│       ├── detection/
│       │   ├── mod.rs
│       │   ├── detector.rs
│       │   ├── rules.rs
│       │   └── languages.rs
│       ├── embedding/
│       │   ├── mod.rs
│       │   ├── service.rs
│       │   └── models.rs
│       ├── storage/
│       │   ├── mod.rs
│       │   ├── vector.rs
│       │   └── agentdb.rs
│       ├── ci/
│       │   ├── mod.rs
│       │   ├── github.rs
│       │   ├── gitlab.rs
│       │   ├── bitbucket.rs
│       │   └── templates/
│       ├── cd/
│       │   ├── mod.rs
│       │   ├── registry.rs
│       │   ├── flyio.rs
│       │   ├── aws.rs
│       │   └── ...
│       └── security/
│           ├── mod.rs
│           ├── validation.rs
│           └── audit.rs
│
├── ampel-db/
│   └── src/
│       ├── entities/
│       │   ├── repository_analysis.rs    # NEW
│       │   ├── workflow_suggestion.rs     # NEW
│       │   ├── deployment_credential.rs  # NEW
│       │   └── deployment.rs              # NEW
│       ├── migrations/
│       │   └── m20250124_cicd_automation.rs  # NEW
│       └── queries/
│           ├── analysis_queries.rs        # NEW
│           └── credential_queries.rs      # NEW
│
└── ampel-api/
    └── src/
        └── routes/
            ├── analysis.rs    # NEW
            ├── suggestions.rs # NEW
            └── credentials.rs # NEW

frontend/src/
├── components/
│   └── dashboard/
│       ├── CiCdSuggestions.tsx    # NEW
│       ├── WorkflowPreview.tsx    # NEW
│       └── CredentialManager.tsx  # NEW
├── pages/
│   └── settings/
│       └── DeploymentSettings.tsx # NEW
└── api/
    ├── analysis.ts    # NEW
    ├── suggestions.ts # NEW
    └── credentials.ts # NEW
```

---

## Appendix B: References

- [AgentDB Documentation](https://agentdb.ruv.io/)
- [RuVector GitHub](https://github.com/ruvnet/ruvector)
- [fastembed-rs](https://github.com/Anush008/fastembed-rs)
- [ONNX Runtime for Rust (ort)](https://ort.pyke.io/)
- [Fly.io Machines API](https://fly.io/docs/machines/api/)
- [GitHub Actions Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [GitLab CI/CD YAML Reference](https://docs.gitlab.com/ci/yaml/)
- [Bitbucket Pipelines Configuration](https://support.atlassian.com/bitbucket-cloud/docs/bitbucket-pipelines-configuration-reference/)

---

_Document Version: 1.0_
_Created: 2025-12-24_
_Author: Claude Code / Ampel Team_
