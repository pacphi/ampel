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
│                              AMPEL PLATFORM                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────────┐   │
│  │   Frontend (UI)  │  │   API Gateway    │  │     Background Worker    │   │
│  │                  │  │    (Axum)        │  │       (Apalis)           │   │
│  │  - Suggestions   │  │                  │  │                          │   │
│  │  - Workflow Edit │◄─┼─► REST Endpoints │◄─┼─► Repo Analysis Jobs     │   │
│  │  - Deploy Config │  │                  │  │    Workflow Generation   │   │
│  └────────┬─────────┘  └────────┬─────────┘  │    Deployment Jobs       │   │
│           │                     │            └────────────┬─────────────┘   │
│           │                     │                         │                 │
│  ─────────┼─────────────────────┼─────────────────────────┼──────────────   │
│           │                     │                         │                 │
│           ▼                     ▼                         ▼                 │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    INTELLIGENCE LAYER (NEW)                          │   │
│  ├──────────────────────────────────────────────────────────────────────┤   │
│  │                                                                      │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐   │   │
│  │  │   Repository    │  │   CI Workflow   │  │    CD Workflow      │   │   │
│  │  │  Intelligence   │  │    Generator    │  │     Generator       │   │   │
│  │  │     Engine      │  │                 │  │                     │   │   │
│  │  │                 │  │  ┌───────────┐  │  │  ┌───────────────┐  │   │   │
│  │  │ - Tech Detect   │  │  │  GitHub   │  │  │  │   Fly.io      │  │   │   │
│  │  │ - Embedding Gen │  │  │  Actions  │  │  │  │   Plugin      │  │   │   │
│  │  │ - Pattern Match │  │  ├───────────┤  │  │  ├───────────────┤  │   │   │
│  │  │ - CI/CD Detect  │  │  │  GitLab   │  │  │  │   AWS         │  │   │   │
│  │  │                 │  │  │  CI/CD    │  │  │  │   Plugin      │  │   │   │
│  │  └────────┬────────┘  │  ├───────────┤  │  │  ├───────────────┤  │   │   │
│  │           │           │  │ Bitbucket │  │  │  │   GCP/Azure   │  │   │   │
│  │           │           │  │ Pipelines │  │  │  │   Plugins     │  │   │   │
│  │           │           │  └───────────┘  │  │  └───────────────┘  │   │   │
│  │           │           └────────┬────────┘  └──────────┬──────────┘   │   │
│  │           │                    │                      │              │   │
│  └───────────┼────────────────────┼──────────────────────┼──────────────┘   │
│              │                    │                      │                  │
│  ────────────┼────────────────────┼──────────────────────┼────────────────  │
│              │                    │                      │                  │
│              ▼                    ▼                      ▼                  │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                         DATA LAYER                                   │   │
│  ├──────────────────────────────────────────────────────────────────────┤   │
│  │                                                                      │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐   │   │
│  │  │   PostgreSQL    │  │  RuVector/      │  │    Redis Cache      │   │   │
│  │  │                 │  │  AgentDB        │  │                     │   │   │
│  │  │ - Users         │  │                 │  │ - Embedding Cache   │   │   │
│  │  │ - Repositories  │  │ - Tech Stack    │  │ - Template Cache    │   │   │
│  │  │ - Credentials   │  │   Embeddings    │  │ - Rate Limit        │   │   │
│  │  │ - Workflows     │  │ - Pattern DB    │  │                     │   │   │
│  │  │ - Deploy Config │  │ - Skills        │  │                     │   │   │
│  │  │                 │  │ - Learning      │  │                     │   │   │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────┘   │   │
│  │                                                                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
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

Detection rules are **externalized as YAML configuration** with JSON Schema validation, enabling easy onboarding of new tech stacks without code changes.

##### Detection Rules JSON Schema

```json
// config/schemas/detection-rules.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "TechStackDetectionRules",
  "description": "Schema for tech stack detection rules",
  "type": "object",
  "properties": {
    "version": { "type": "string", "pattern": "^\\d+\\.\\d+$" },
    "rules": {
      "type": "array",
      "items": { "$ref": "#/definitions/DetectionRule" }
    }
  },
  "required": ["version", "rules"],
  "definitions": {
    "DetectionRule": {
      "type": "object",
      "properties": {
        "id": { "type": "string", "pattern": "^[a-z0-9-]+$" },
        "name": { "type": "string" },
        "description": { "type": "string" },
        "file_patterns": {
          "type": "array",
          "items": { "type": "string" },
          "minItems": 1
        },
        "content_patterns": {
          "type": "array",
          "items": { "type": "string" }
        },
        "weight": { "type": "number", "minimum": 0, "maximum": 1 },
        "detects": { "$ref": "#/definitions/DetectionTarget" },
        "enabled": { "type": "boolean", "default": true }
      },
      "required": ["id", "name", "file_patterns", "weight", "detects"]
    },
    "DetectionTarget": {
      "type": "object",
      "properties": {
        "type": {
          "enum": [
            "language",
            "framework",
            "build_tool",
            "test_framework",
            "ci_workflow",
            "cd_workflow"
          ]
        },
        "value": { "type": "string" }
      },
      "required": ["type", "value"]
    }
  }
}
```

##### Detection Rules Configuration File

```yaml
# config/detection-rules.yaml
version: '1.0'

rules:
  # ============================================
  # LANGUAGES
  # ============================================
  - id: lang-rust
    name: Rust Language
    description: Detect Rust projects via Cargo manifest
    file_patterns:
      - 'Cargo.toml'
      - 'Cargo.lock'
    content_patterns:
      - "\\[package\\]"
    weight: 1.0
    detects:
      type: language
      value: rust

  - id: lang-javascript
    name: JavaScript/Node.js
    description: Detect Node.js projects via package.json
    file_patterns:
      - 'package.json'
    content_patterns:
      - '"dependencies"'
      - '"devDependencies"'
    weight: 1.0
    detects:
      type: language
      value: javascript

  - id: lang-typescript
    name: TypeScript
    description: Detect TypeScript projects
    file_patterns:
      - 'tsconfig.json'
      - 'tsconfig.*.json'
    content_patterns: []
    weight: 0.95
    detects:
      type: language
      value: typescript

  - id: lang-python
    name: Python
    description: Detect Python projects
    file_patterns:
      - 'requirements.txt'
      - 'pyproject.toml'
      - 'setup.py'
      - 'Pipfile'
      - 'poetry.lock'
    content_patterns: []
    weight: 1.0
    detects:
      type: language
      value: python

  - id: lang-go
    name: Go
    description: Detect Go projects
    file_patterns:
      - 'go.mod'
      - 'go.sum'
    content_patterns:
      - '^module '
    weight: 1.0
    detects:
      type: language
      value: go

  - id: lang-java
    name: Java
    description: Detect Java projects
    file_patterns:
      - 'pom.xml'
      - 'build.gradle'
      - 'build.gradle.kts'
    content_patterns: []
    weight: 1.0
    detects:
      type: language
      value: java

  # ============================================
  # FRAMEWORKS
  # ============================================
  - id: framework-react
    name: React Framework
    description: Detect React frontend framework
    file_patterns:
      - 'package.json'
    content_patterns:
      - '"react"\\s*:'
    weight: 0.9
    detects:
      type: framework
      value: react

  - id: framework-vue
    name: Vue.js Framework
    description: Detect Vue.js frontend framework
    file_patterns:
      - 'package.json'
    content_patterns:
      - '"vue"\\s*:'
    weight: 0.9
    detects:
      type: framework
      value: vue

  - id: framework-axum
    name: Axum Framework
    description: Detect Axum Rust web framework
    file_patterns:
      - 'Cargo.toml'
    content_patterns:
      - 'axum\s*='
    weight: 0.9
    detects:
      type: framework
      value: axum

  - id: framework-actix
    name: Actix-Web Framework
    description: Detect Actix-Web Rust framework
    file_patterns:
      - 'Cargo.toml'
    content_patterns:
      - 'actix-web\s*='
    weight: 0.9
    detects:
      type: framework
      value: actix-web

  - id: framework-django
    name: Django Framework
    description: Detect Django Python framework
    file_patterns:
      - 'requirements.txt'
      - 'pyproject.toml'
    content_patterns:
      - 'django'
      - 'Django'
    weight: 0.9
    detects:
      type: framework
      value: django

  - id: framework-fastapi
    name: FastAPI Framework
    description: Detect FastAPI Python framework
    file_patterns:
      - 'requirements.txt'
      - 'pyproject.toml'
    content_patterns:
      - 'fastapi'
    weight: 0.9
    detects:
      type: framework
      value: fastapi

  # ============================================
  # CI WORKFLOWS
  # ============================================
  - id: ci-github-actions
    name: GitHub Actions CI
    description: Detect existing GitHub Actions workflows
    file_patterns:
      - '.github/workflows/*.yml'
      - '.github/workflows/*.yaml'
    content_patterns:
      - "on:\\s*\\[?(push|pull_request)"
    weight: 1.0
    detects:
      type: ci_workflow
      value: github

  - id: ci-gitlab
    name: GitLab CI/CD
    description: Detect existing GitLab CI configuration
    file_patterns:
      - '.gitlab-ci.yml'
    content_patterns:
      - 'stages:'
    weight: 1.0
    detects:
      type: ci_workflow
      value: gitlab

  - id: ci-bitbucket
    name: Bitbucket Pipelines
    description: Detect existing Bitbucket Pipelines
    file_patterns:
      - 'bitbucket-pipelines.yml'
    content_patterns:
      - 'pipelines:'
    weight: 1.0
    detects:
      type: ci_workflow
      value: bitbucket

  # ============================================
  # TEST FRAMEWORKS
  # ============================================
  - id: test-jest
    name: Jest Testing
    description: Detect Jest JavaScript testing framework
    file_patterns:
      - 'package.json'
      - 'jest.config.js'
      - 'jest.config.ts'
    content_patterns:
      - '"jest"'
    weight: 0.85
    detects:
      type: test_framework
      value: jest

  - id: test-vitest
    name: Vitest Testing
    description: Detect Vitest testing framework
    file_patterns:
      - 'package.json'
      - 'vitest.config.ts'
    content_patterns:
      - '"vitest"'
    weight: 0.85
    detects:
      type: test_framework
      value: vitest

  - id: test-pytest
    name: Pytest
    description: Detect pytest Python testing
    file_patterns:
      - 'requirements.txt'
      - 'pyproject.toml'
      - 'pytest.ini'
      - 'conftest.py'
    content_patterns:
      - 'pytest'
    weight: 0.85
    detects:
      type: test_framework
      value: pytest
```

##### Rust Implementation with YAML Loading and Validation

```rust
// crates/ampel-intelligence/src/detection.rs

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use jsonschema::{JSONSchema, Draft};

/// Detected technology stack for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTechStack {
    pub languages: Vec<(Language, f32)>,
    pub frameworks: Vec<DetectedFramework>,
    pub build_tools: Vec<BuildTool>,
    pub test_frameworks: Vec<TestFramework>,
    pub package_managers: Vec<PackageManager>,
    pub existing_workflows: Vec<ExistingWorkflow>,
    pub containerization: Option<ContainerConfig>,
    pub confidence: f32,
}

/// Detection target from YAML config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionTarget {
    #[serde(rename = "type")]
    pub target_type: DetectionTargetType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionTargetType {
    Language,
    Framework,
    BuildTool,
    TestFramework,
    CiWorkflow,
    CdWorkflow,
}

/// File pattern rules for detection (loaded from YAML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub file_patterns: Vec<String>,
    #[serde(default)]
    pub content_patterns: Vec<String>,
    pub weight: f32,
    pub detects: DetectionTarget,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool { true }

/// Detection rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRulesConfig {
    pub version: String,
    pub rules: Vec<DetectionRule>,
}

/// Configuration loader with schema validation
pub struct DetectionConfigLoader {
    schema: JSONSchema,
    config_path: PathBuf,
}

impl DetectionConfigLoader {
    /// Create loader with schema validation
    pub fn new(config_dir: &Path) -> Result<Self, ConfigError> {
        let schema_path = config_dir.join("schemas/detection-rules.schema.json");
        let schema_content = std::fs::read_to_string(&schema_path)
            .map_err(|e| ConfigError::SchemaLoad(e.to_string()))?;
        let schema_value: serde_json::Value = serde_json::from_str(&schema_content)
            .map_err(|e| ConfigError::SchemaLoad(e.to_string()))?;
        let schema = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_value)
            .map_err(|e| ConfigError::SchemaCompile(e.to_string()))?;

        Ok(Self {
            schema,
            config_path: config_dir.join("detection-rules.yaml"),
        })
    }

    /// Load and validate detection rules from YAML
    pub fn load(&self) -> Result<DetectionRulesConfig, ConfigError> {
        let yaml_content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| ConfigError::FileRead(self.config_path.clone(), e.to_string()))?;

        // Parse YAML to JSON for validation
        let config_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)
            .map_err(|e| ConfigError::YamlParse(e.to_string()))?;
        let json_value: serde_json::Value = serde_json::to_value(&config_value)
            .map_err(|e| ConfigError::JsonConvert(e.to_string()))?;

        // Validate against schema
        let validation_result = self.schema.validate(&json_value);
        if let Err(errors) = validation_result {
            let error_messages: Vec<String> = errors
                .map(|e| format!("{} at {}", e, e.instance_path))
                .collect();
            return Err(ConfigError::SchemaValidation(error_messages));
        }

        // Deserialize to config struct
        let config: DetectionRulesConfig = serde_yaml::from_str(&yaml_content)
            .map_err(|e| ConfigError::Deserialize(e.to_string()))?;

        // Validate regex patterns
        for rule in &config.rules {
            for pattern in &rule.content_patterns {
                regex::Regex::new(pattern)
                    .map_err(|e| ConfigError::InvalidRegex(
                        rule.id.clone(), pattern.clone(), e.to_string()
                    ))?;
            }
        }

        tracing::info!(
            "Loaded {} detection rules from {}",
            config.rules.iter().filter(|r| r.enabled).count(),
            self.config_path.display()
        );

        Ok(config)
    }
}

#[async_trait]
pub trait TechStackDetector: Send + Sync {
    async fn detect(&self, repo_path: &Path) -> Result<DetectedTechStack, DetectionError>;
    async fn has_ci_workflow(&self, repo_path: &Path, provider: GitProvider) -> Result<bool, DetectionError>;
    async fn has_cd_workflow(&self, repo_path: &Path, provider: GitProvider) -> Result<bool, DetectionError>;
    async fn reload_rules(&self) -> Result<(), DetectionError>;
}

/// Default detector with externalized config
pub struct DefaultTechStackDetector {
    rules: Arc<RwLock<Vec<DetectionRule>>>,
    config_loader: Arc<DetectionConfigLoader>,
    embedding_service: Arc<dyn EmbeddingService>,
}

impl DefaultTechStackDetector {
    pub fn new(
        config_dir: &Path,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Result<Self, DetectionError> {
        let config_loader = Arc::new(DetectionConfigLoader::new(config_dir)?);
        let config = config_loader.load()?;
        let enabled_rules: Vec<DetectionRule> = config.rules
            .into_iter()
            .filter(|r| r.enabled)
            .collect();

        Ok(Self {
            rules: Arc::new(RwLock::new(enabled_rules)),
            config_loader,
            embedding_service,
        })
    }

    /// Hot-reload rules from YAML config
    pub async fn reload_rules(&self) -> Result<(), DetectionError> {
        let config = self.config_loader.load()?;
        let enabled_rules: Vec<DetectionRule> = config.rules
            .into_iter()
            .filter(|r| r.enabled)
            .collect();
        let mut rules = self.rules.write().await;
        *rules = enabled_rules;
        tracing::info!("Reloaded detection rules: {} active rules", rules.len());
        Ok(())
    }
}
```

#### 3.1.3 Embedding Service

Model selection and configuration is **externalized to YAML** to allow runtime model switching without code changes.

##### Embedding Configuration Schema

```json
// config/schemas/embedding-config.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "EmbeddingServiceConfig",
  "description": "Configuration for embedding model selection",
  "type": "object",
  "properties": {
    "version": { "type": "string" },
    "default_model": { "type": "string" },
    "models": {
      "type": "object",
      "additionalProperties": { "$ref": "#/definitions/ModelConfig" }
    },
    "cache": { "$ref": "#/definitions/CacheConfig" }
  },
  "required": ["version", "default_model", "models"],
  "definitions": {
    "ModelConfig": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "provider": { "enum": ["fastembed", "ort", "candle"] },
        "model_id": { "type": "string" },
        "dimensions": { "type": "integer", "minimum": 1 },
        "max_tokens": { "type": "integer" },
        "use_case": { "type": "string" },
        "quantized": { "type": "boolean", "default": false },
        "show_download_progress": { "type": "boolean", "default": true }
      },
      "required": ["name", "provider", "model_id", "dimensions"]
    },
    "CacheConfig": {
      "type": "object",
      "properties": {
        "enabled": { "type": "boolean", "default": true },
        "max_entries": { "type": "integer", "default": 10000 },
        "ttl_seconds": { "type": "integer", "default": 3600 }
      }
    }
  }
}
```

##### Embedding Configuration File

```yaml
# config/embedding-config.yaml
version: '1.0'

# Default model for general use
default_model: all-minilm-l6-v2

models:
  # Fast, lightweight model for semantic search (recommended default)
  all-minilm-l6-v2:
    name: 'All-MiniLM-L6-v2'
    provider: fastembed
    model_id: 'sentence-transformers/all-MiniLM-L6-v2'
    dimensions: 384
    max_tokens: 256
    use_case: 'General purpose semantic search, fast inference'
    quantized: false
    show_download_progress: true

  # High-quality embeddings (slower, more accurate)
  bge-small-en:
    name: 'BGE Small English'
    provider: fastembed
    model_id: 'BAAI/bge-small-en-v1.5'
    dimensions: 384
    max_tokens: 512
    use_case: 'Higher quality embeddings for similarity matching'
    quantized: false
    show_download_progress: true

  # Code-specific embeddings
  codebert:
    name: 'CodeBERT'
    provider: ort
    model_id: 'microsoft/codebert-base'
    dimensions: 768
    max_tokens: 512
    use_case: 'Code understanding and tech stack analysis'
    quantized: false
    show_download_progress: true

  # Lightweight quantized model for resource-constrained environments
  all-minilm-l6-v2-q:
    name: 'All-MiniLM-L6-v2 Quantized'
    provider: fastembed
    model_id: 'sentence-transformers/all-MiniLM-L6-v2'
    dimensions: 384
    max_tokens: 256
    use_case: 'Low memory footprint, edge deployment'
    quantized: true
    show_download_progress: true

# Embedding cache configuration
cache:
  enabled: true
  max_entries: 10000
  ttl_seconds: 3600 # 1 hour
```

##### Rust Implementation with Config Loading

```rust
// crates/ampel-intelligence/src/embedding.rs

use fastembed::{TextEmbedding, EmbeddingModel, InitOptions};
use std::path::Path;

/// Embedding model configuration from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub provider: EmbeddingProvider,
    pub model_id: String,
    pub dimensions: usize,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub use_case: Option<String>,
    #[serde(default)]
    pub quantized: bool,
    #[serde(default = "default_show_progress")]
    pub show_download_progress: bool,
}

fn default_show_progress() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProvider {
    Fastembed,
    Ort,
    Candle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
}

fn default_cache_enabled() -> bool { true }
fn default_max_entries() -> usize { 10000 }
fn default_ttl() -> u64 { 3600 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingServiceConfig {
    pub version: String,
    pub default_model: String,
    pub models: HashMap<String, ModelConfig>,
    #[serde(default)]
    pub cache: Option<CacheConfig>,
}

/// Configuration loader for embedding service
pub struct EmbeddingConfigLoader {
    config_path: PathBuf,
}

impl EmbeddingConfigLoader {
    pub fn new(config_dir: &Path) -> Self {
        Self {
            config_path: config_dir.join("embedding-config.yaml"),
        }
    }

    pub fn load(&self) -> Result<EmbeddingServiceConfig, ConfigError> {
        let yaml_content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| ConfigError::FileRead(self.config_path.clone(), e.to_string()))?;
        let config: EmbeddingServiceConfig = serde_yaml::from_str(&yaml_content)
            .map_err(|e| ConfigError::Deserialize(e.to_string()))?;

        // Validate default model exists
        if !config.models.contains_key(&config.default_model) {
            return Err(ConfigError::InvalidConfig(format!(
                "Default model '{}' not found in models list",
                config.default_model
            )));
        }

        Ok(config)
    }
}

/// Repository fingerprint as vector embedding
#[derive(Debug, Clone)]
pub struct RepositoryFingerprint {
    pub embedding: Vec<f32>,
    pub tech_stack: DetectedTechStack,
    pub metadata: RepositoryMetadata,
    pub analyzed_at: DateTime<Utc>,
}

#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
    async fn fingerprint(&self, tech_stack: &DetectedTechStack) -> Result<RepositoryFingerprint, EmbeddingError>;
    fn dimensions(&self) -> usize;
    fn model_name(&self) -> &str;
}

/// ONNX-based embedding service with config-driven model selection
pub struct OnnxEmbeddingService {
    model: TextEmbedding,
    config: ModelConfig,
    cache: Option<EmbeddingCache>,
}

impl OnnxEmbeddingService {
    /// Create from configuration directory
    pub fn from_config(config_dir: &Path) -> Result<Self, EmbeddingError> {
        let loader = EmbeddingConfigLoader::new(config_dir);
        let config = loader.load()?;
        Self::with_model_config(
            config.models.get(&config.default_model)
                .ok_or(EmbeddingError::ModelNotFound(config.default_model.clone()))?
                .clone(),
            config.cache,
        )
    }

    /// Create with specific model from config
    pub fn from_config_with_model(config_dir: &Path, model_key: &str) -> Result<Self, EmbeddingError> {
        let loader = EmbeddingConfigLoader::new(config_dir);
        let config = loader.load()?;
        let model_config = config.models.get(model_key)
            .ok_or(EmbeddingError::ModelNotFound(model_key.to_string()))?
            .clone();
        Self::with_model_config(model_config, config.cache)
    }

    fn with_model_config(model_config: ModelConfig, cache_config: Option<CacheConfig>) -> Result<Self, EmbeddingError> {
        let fastembed_model = match model_config.model_id.as_str() {
            "sentence-transformers/all-MiniLM-L6-v2" => EmbeddingModel::AllMiniLML6V2,
            "BAAI/bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
            "BAAI/bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
            _ => return Err(EmbeddingError::UnsupportedModel(model_config.model_id.clone())),
        };

        let model = TextEmbedding::try_new(
            InitOptions::new(fastembed_model)
                .with_show_download_progress(model_config.show_download_progress)
        )?;

        let cache = cache_config.filter(|c| c.enabled).map(|c| {
            EmbeddingCache::new(c.max_entries, Duration::from_secs(c.ttl_seconds))
        });

        Ok(Self { model, config: model_config, cache })
    }
}

#[async_trait]
impl EmbeddingService for OnnxEmbeddingService {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get(text).await {
                return Ok(cached);
            }
        }

        let embeddings = self.model.embed(vec![text], None)?;
        let embedding = embeddings.into_iter().next().unwrap();

        // Store in cache
        if let Some(cache) = &self.cache {
            cache.insert(text.to_string(), embedding.clone()).await;
        }

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let texts: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        Ok(self.model.embed(texts, None)?)
    }

    async fn fingerprint(&self, tech_stack: &DetectedTechStack) -> Result<RepositoryFingerprint, EmbeddingError> {
        let description = format!(
            "Repository using {} with frameworks {} and build tools {}",
            tech_stack.languages.iter().map(|(l, _)| l.to_string()).collect::<Vec<_>>().join(", "),
            tech_stack.frameworks.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", "),
            tech_stack.build_tools.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", "),
        );

        let embedding = self.embed(&description).await?;

        Ok(RepositoryFingerprint {
            embedding,
            tech_stack: tech_stack.clone(),
            metadata: RepositoryMetadata::default(),
            analyzed_at: Utc::now(),
        })
    }

    fn dimensions(&self) -> usize {
        self.config.dimensions
    }

    fn model_name(&self) -> &str {
        &self.config.name
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

CI workflow templates are **externalized as YAML** with JSON Schema validation, enabling customization without code changes.

##### CI Templates Schema

```json
// config/schemas/ci-templates.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CIWorkflowTemplates",
  "description": "Schema for CI workflow template configuration",
  "type": "object",
  "properties": {
    "version": { "type": "string" },
    "templates": {
      "type": "object",
      "additionalProperties": { "$ref": "#/definitions/LanguageTemplate" }
    }
  },
  "required": ["version", "templates"],
  "definitions": {
    "LanguageTemplate": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "file_path": { "type": "string" },
        "base_template": { "type": "string" },
        "env": { "type": "object", "additionalProperties": { "type": "string" } },
        "setup_steps": { "type": "array", "items": { "$ref": "#/definitions/Step" } },
        "build_steps": { "type": "array", "items": { "$ref": "#/definitions/Step" } },
        "test_steps": { "type": "array", "items": { "$ref": "#/definitions/Step" } },
        "lint_steps": { "type": "array", "items": { "$ref": "#/definitions/Step" } },
        "cache_config": { "$ref": "#/definitions/CacheConfig" },
        "security_steps": { "type": "array", "items": { "$ref": "#/definitions/Step" } }
      },
      "required": ["name", "file_path", "build_steps"]
    },
    "Step": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "uses": { "type": "string" },
        "run": { "type": "string" },
        "with": { "type": "object" },
        "env": { "type": "object" },
        "if": { "type": "string" }
      },
      "required": ["name"]
    },
    "CacheConfig": {
      "type": "object",
      "properties": {
        "paths": { "type": "array", "items": { "type": "string" } },
        "key_pattern": { "type": "string" },
        "restore_keys": { "type": "array", "items": { "type": "string" } }
      }
    }
  }
}
```

##### CI Templates Configuration

```yaml
# config/ci-templates.yaml
version: '1.0'

templates:
  # ============================================
  # RUST TEMPLATES
  # ============================================
  rust:
    name: 'Rust CI'
    file_path: '.github/workflows/ci.yml'
    env:
      CARGO_TERM_COLOR: 'always'
      RUST_BACKTRACE: '1'

    setup_steps:
      - name: 'Checkout'
        uses: 'actions/checkout@v6'

      - name: 'Install Rust toolchain'
        uses: 'dtolnay/rust-action@stable'
        with:
          components: 'clippy, rustfmt'

    cache_config:
      paths:
        - '~/.cargo/registry'
        - '~/.cargo/git'
        - 'target'
      key_pattern: "${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}"
      restore_keys:
        - '${{ runner.os }}-cargo-'

    lint_steps:
      - name: 'Check formatting'
        run: 'cargo fmt --all -- --check'

      - name: 'Clippy'
        run: 'cargo clippy --all-targets --all-features -- -D warnings'

    build_steps:
      - name: 'Build'
        run: 'cargo build --all-features'

    test_steps:
      - name: 'Run tests'
        run: 'cargo test --all-features'

    security_steps:
      - name: 'Security audit'
        run: 'cargo audit'

  # ============================================
  # NODE.JS TEMPLATES
  # ============================================
  javascript:
    name: 'Node.js CI'
    file_path: '.github/workflows/ci.yml'
    env:
      CI: 'true'

    setup_steps:
      - name: 'Checkout'
        uses: 'actions/checkout@v6'

      - name: 'Setup Node.js'
        uses: 'actions/setup-node@v4'
        with:
          node-version: '20'
          cache: 'npm'

    cache_config:
      paths:
        - '~/.npm'
        - 'node_modules'
      key_pattern: "${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}"
      restore_keys:
        - '${{ runner.os }}-node-'

    lint_steps:
      - name: 'Lint'
        run: 'npm run lint'

    build_steps:
      - name: 'Install dependencies'
        run: 'npm ci'

      - name: 'Build'
        run: 'npm run build'

    test_steps:
      - name: 'Run tests'
        run: 'npm test'

    security_steps:
      - name: 'Audit dependencies'
        run: 'npm audit --audit-level=high'

  # ============================================
  # PYTHON TEMPLATES
  # ============================================
  python:
    name: 'Python CI'
    file_path: '.github/workflows/ci.yml'

    setup_steps:
      - name: 'Checkout'
        uses: 'actions/checkout@v6'

      - name: 'Setup Python'
        uses: 'actions/setup-python@v5'
        with:
          python-version: '3.12'
          cache: 'pip'

    cache_config:
      paths:
        - '~/.cache/pip'
      key_pattern: "${{ runner.os }}-pip-${{ hashFiles('**/requirements*.txt') }}"
      restore_keys:
        - '${{ runner.os }}-pip-'

    lint_steps:
      - name: 'Lint with ruff'
        run: |
          pip install ruff
          ruff check .

      - name: 'Type check with mypy'
        run: |
          pip install mypy
          mypy .

    build_steps:
      - name: 'Install dependencies'
        run: 'pip install -r requirements.txt'

    test_steps:
      - name: 'Run tests'
        run: |
          pip install pytest pytest-cov
          pytest --cov=.

    security_steps:
      - name: 'Security scan'
        run: |
          pip install safety
          safety check

  # ============================================
  # GO TEMPLATES
  # ============================================
  go:
    name: 'Go CI'
    file_path: '.github/workflows/ci.yml'

    setup_steps:
      - name: 'Checkout'
        uses: 'actions/checkout@v6'

      - name: 'Setup Go'
        uses: 'actions/setup-go@v5'
        with:
          go-version: '1.22'
          cache: true

    lint_steps:
      - name: 'Lint with golangci-lint'
        uses: 'golangci/golangci-lint-action@v4'
        with:
          version: 'latest'

    build_steps:
      - name: 'Build'
        run: 'go build -v ./...'

    test_steps:
      - name: 'Run tests'
        run: 'go test -v -race -coverprofile=coverage.out ./...'

    security_steps:
      - name: 'Security scan'
        run: |
          go install golang.org/x/vuln/cmd/govulncheck@latest
          govulncheck ./...
```

##### Rust Implementation with Template Loading

```rust
// crates/ampel-intelligence/src/ci/github.rs

use super::*;
use handlebars::Handlebars;

/// CI template configuration from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageTemplate {
    pub name: String,
    pub file_path: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub setup_steps: Vec<WorkflowStep>,
    pub build_steps: Vec<WorkflowStep>,
    #[serde(default)]
    pub test_steps: Vec<WorkflowStep>,
    #[serde(default)]
    pub lint_steps: Vec<WorkflowStep>,
    #[serde(default)]
    pub cache_config: Option<CacheConfig>,
    #[serde(default)]
    pub security_steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "with")]
    pub with_params: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "if")]
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CITemplatesConfig {
    pub version: String,
    pub templates: HashMap<String, LanguageTemplate>,
}

/// Template loader with schema validation
pub struct CITemplateLoader {
    config_path: PathBuf,
}

impl CITemplateLoader {
    pub fn new(config_dir: &Path) -> Self {
        Self {
            config_path: config_dir.join("ci-templates.yaml"),
        }
    }

    pub fn load(&self) -> Result<CITemplatesConfig, ConfigError> {
        let yaml_content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| ConfigError::FileRead(self.config_path.clone(), e.to_string()))?;
        let config: CITemplatesConfig = serde_yaml::from_str(&yaml_content)
            .map_err(|e| ConfigError::Deserialize(e.to_string()))?;
        Ok(config)
    }
}

/// GitHub Actions generator with externalized templates
pub struct GitHubActionsGenerator {
    templates: Arc<RwLock<CITemplatesConfig>>,
    template_loader: Arc<CITemplateLoader>,
}

impl GitHubActionsGenerator {
    pub fn new(config_dir: &Path) -> Result<Self, ConfigError> {
        let template_loader = Arc::new(CITemplateLoader::new(config_dir));
        let templates = template_loader.load()?;
        Ok(Self {
            templates: Arc::new(RwLock::new(templates)),
            template_loader,
        })
    }

    /// Hot-reload templates from config
    pub async fn reload_templates(&self) -> Result<(), ConfigError> {
        let config = self.template_loader.load()?;
        let mut templates = self.templates.write().await;
        *templates = config;
        tracing::info!("Reloaded CI templates");
        Ok(())
    }

    fn render_workflow(&self, template: &LanguageTemplate, options: &GenerationOptions) -> String {
        let mut workflow = serde_yaml::Mapping::new();

        // Name and triggers
        workflow.insert("name".into(), template.name.clone().into());
        let triggers = self.build_triggers(options);
        workflow.insert("on".into(), triggers);

        // Environment variables
        if !template.env.is_empty() {
            workflow.insert("env".into(), serde_yaml::to_value(&template.env).unwrap());
        }

        // Jobs
        let jobs = self.build_jobs(template, options);
        workflow.insert("jobs".into(), jobs);

        serde_yaml::to_string(&workflow).unwrap()
    }

    fn build_jobs(&self, template: &LanguageTemplate, options: &GenerationOptions) -> serde_yaml::Value {
        let mut steps = Vec::new();

        // Setup steps
        steps.extend(template.setup_steps.iter().cloned());

        // Cache step (if enabled)
        if options.enable_caching {
            if let Some(cache) = &template.cache_config {
                steps.push(self.build_cache_step(cache));
            }
        }

        // Lint steps
        steps.extend(template.lint_steps.iter().cloned());

        // Build steps
        steps.extend(template.build_steps.iter().cloned());

        // Test steps
        steps.extend(template.test_steps.iter().cloned());

        // Security steps (if enabled)
        if options.enable_security_scan {
            steps.extend(template.security_steps.iter().cloned());
        }

        serde_yaml::to_value(json!({
            "build": {
                "runs-on": "ubuntu-latest",
                "steps": steps
            }
        })).unwrap()
    }
}

#[async_trait]
impl CIWorkflowGenerator for GitHubActionsGenerator {
    async fn generate(
        &self,
        tech_stack: &DetectedTechStack,
        options: &GenerationOptions,
    ) -> Result<GeneratedWorkflow, GenerationError> {
        let templates = self.templates.read().await;
        let language_key = tech_stack.primary_language().to_string().to_lowercase();

        let template = templates.templates.get(&language_key)
            .ok_or(GenerationError::UnsupportedLanguage)?;

        let content = self.render_workflow(template, options);

        Ok(GeneratedWorkflow {
            file_path: template.file_path.clone(),
            content,
            description: format!("CI workflow for {} project", tech_stack.primary_language()),
            provider: GitProvider::GitHub,
            workflow_type: WorkflowType::CI,
        })
    }

    fn validate(&self, workflow: &str) -> Result<(), ValidationError> {
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

Deployment provider credential schemas and configurations are **externalized to YAML** for easy addition of new providers without code changes.

##### Deployment Providers Schema

```json
// config/schemas/deployment-providers.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DeploymentProvidersConfig",
  "description": "Schema for deployment provider configurations",
  "type": "object",
  "properties": {
    "version": { "type": "string" },
    "providers": {
      "type": "object",
      "additionalProperties": { "$ref": "#/definitions/ProviderConfig" }
    }
  },
  "required": ["version", "providers"],
  "definitions": {
    "ProviderConfig": {
      "type": "object",
      "properties": {
        "id": { "type": "string" },
        "name": { "type": "string" },
        "description": { "type": "string" },
        "api_base": { "type": "string", "format": "uri" },
        "docs_url": { "type": "string", "format": "uri" },
        "credential_schema": { "$ref": "#/definitions/CredentialSchema" },
        "deployment_config": { "$ref": "#/definitions/DeploymentDefaults" },
        "regions": { "type": "array", "items": { "$ref": "#/definitions/Region" } },
        "supported_languages": { "type": "array", "items": { "type": "string" } }
      },
      "required": ["id", "name", "credential_schema"]
    },
    "CredentialSchema": {
      "type": "object",
      "properties": {
        "fields": {
          "type": "array",
          "items": { "$ref": "#/definitions/CredentialField" }
        }
      }
    },
    "CredentialField": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "label": { "type": "string" },
        "field_type": { "enum": ["api_token", "secret", "text", "select", "region"] },
        "required": { "type": "boolean" },
        "description": { "type": "string" },
        "validation_pattern": { "type": "string" },
        "options": { "type": "array", "items": { "type": "string" } }
      },
      "required": ["name", "label", "field_type"]
    },
    "DeploymentDefaults": {
      "type": "object",
      "additionalProperties": true
    },
    "Region": {
      "type": "object",
      "properties": {
        "code": { "type": "string" },
        "name": { "type": "string" },
        "location": { "type": "string" }
      },
      "required": ["code", "name"]
    }
  }
}
```

##### Deployment Providers Configuration

```yaml
# config/deployment-providers.yaml
version: '1.0'

providers:
  # ============================================
  # FLY.IO
  # ============================================
  flyio:
    id: 'flyio'
    name: 'Fly.io'
    description: 'Deploy globally distributed applications at the edge'
    api_base: 'https://api.machines.dev'
    docs_url: 'https://fly.io/docs/machines/api/'

    credential_schema:
      fields:
        - name: 'api_token'
          label: 'Fly.io API Token'
          field_type: 'api_token'
          required: true
          description: "Generate with 'fly tokens deploy'"
          validation_pattern: '^fm[12]_[A-Za-z0-9_-]+$'

        - name: 'app_name'
          label: 'Application Name'
          field_type: 'text'
          required: false
          description: 'Leave blank to auto-generate from repo name'
          validation_pattern: '^[a-z0-9-]+$'

        - name: 'region'
          label: 'Primary Region'
          field_type: 'select'
          required: true
          description: 'Deployment region'
          options: [] # Populated from regions list

    regions:
      - code: 'iad'
        name: 'Ashburn, Virginia (US)'
        location: 'US East'
      - code: 'lax'
        name: 'Los Angeles, California (US)'
        location: 'US West'
      - code: 'ord'
        name: 'Chicago, Illinois (US)'
        location: 'US Central'
      - code: 'sjc'
        name: 'San Jose, California (US)'
        location: 'US West'
      - code: 'ams'
        name: 'Amsterdam, Netherlands'
        location: 'Europe'
      - code: 'lhr'
        name: 'London, United Kingdom'
        location: 'Europe'
      - code: 'fra'
        name: 'Frankfurt, Germany'
        location: 'Europe'
      - code: 'sin'
        name: 'Singapore'
        location: 'Asia Pacific'
      - code: 'syd'
        name: 'Sydney, Australia'
        location: 'Australia'
      - code: 'nrt'
        name: 'Tokyo, Japan'
        location: 'Asia Pacific'

    deployment_config:
      language_defaults:
        rust:
          internal_port: 8080
          memory: '256mb'
          cpu_kind: 'shared'
          cpus: 1
        javascript:
          internal_port: 3000
          memory: '256mb'
          cpu_kind: 'shared'
          cpus: 1
        python:
          internal_port: 8000
          memory: '512mb'
          cpu_kind: 'shared'
          cpus: 1
        go:
          internal_port: 8080
          memory: '256mb'
          cpu_kind: 'shared'
          cpus: 1

    supported_languages:
      - 'rust'
      - 'javascript'
      - 'typescript'
      - 'python'
      - 'go'
      - 'java'

  # ============================================
  # AWS ECS
  # ============================================
  aws-ecs:
    id: 'aws-ecs'
    name: 'AWS ECS'
    description: 'Amazon Elastic Container Service'
    api_base: 'https://ecs.amazonaws.com'
    docs_url: 'https://docs.aws.amazon.com/ecs/'

    credential_schema:
      fields:
        - name: 'access_key_id'
          label: 'AWS Access Key ID'
          field_type: 'text'
          required: true
          description: 'AWS access key for programmatic access'
          validation_pattern: '^[A-Z0-9]{20}$'

        - name: 'secret_access_key'
          label: 'AWS Secret Access Key'
          field_type: 'secret'
          required: true
          description: 'Keep this confidential'

        - name: 'region'
          label: 'AWS Region'
          field_type: 'select'
          required: true
          description: 'AWS deployment region'
          options: []

        - name: 'cluster_name'
          label: 'ECS Cluster Name'
          field_type: 'text'
          required: false
          description: 'Leave blank to create new cluster'

    regions:
      - code: 'us-east-1'
        name: 'US East (N. Virginia)'
        location: 'US East'
      - code: 'us-west-2'
        name: 'US West (Oregon)'
        location: 'US West'
      - code: 'eu-west-1'
        name: 'Europe (Ireland)'
        location: 'Europe'
      - code: 'ap-southeast-1'
        name: 'Asia Pacific (Singapore)'
        location: 'Asia Pacific'

    deployment_config:
      task_cpu: '256'
      task_memory: '512'
      launch_type: 'FARGATE'

    supported_languages:
      - 'rust'
      - 'javascript'
      - 'typescript'
      - 'python'
      - 'go'
      - 'java'

  # ============================================
  # GOOGLE CLOUD RUN
  # ============================================
  gcp-cloudrun:
    id: 'gcp-cloudrun'
    name: 'Google Cloud Run'
    description: 'Serverless containers on Google Cloud'
    api_base: 'https://run.googleapis.com'
    docs_url: 'https://cloud.google.com/run/docs'

    credential_schema:
      fields:
        - name: 'service_account_json'
          label: 'Service Account JSON'
          field_type: 'secret'
          required: true
          description: 'Service account key JSON for deployment'

        - name: 'project_id'
          label: 'GCP Project ID'
          field_type: 'text'
          required: true
          description: 'Your Google Cloud project ID'
          validation_pattern: '^[a-z][a-z0-9-]{4,28}[a-z0-9]$'

        - name: 'region'
          label: 'GCP Region'
          field_type: 'select'
          required: true
          description: 'Cloud Run deployment region'
          options: []

    regions:
      - code: 'us-central1'
        name: 'Iowa'
        location: 'US Central'
      - code: 'us-east1'
        name: 'South Carolina'
        location: 'US East'
      - code: 'europe-west1'
        name: 'Belgium'
        location: 'Europe'
      - code: 'asia-east1'
        name: 'Taiwan'
        location: 'Asia Pacific'

    deployment_config:
      cpu: '1'
      memory: '512Mi'
      max_instances: 10
      allow_unauthenticated: true

    supported_languages:
      - 'rust'
      - 'javascript'
      - 'typescript'
      - 'python'
      - 'go'
      - 'java'

  # ============================================
  # DIGITAL OCEAN APP PLATFORM
  # ============================================
  digitalocean:
    id: 'digitalocean'
    name: 'DigitalOcean App Platform'
    description: "Deploy apps quickly with DigitalOcean's PaaS"
    api_base: 'https://api.digitalocean.com/v2'
    docs_url: 'https://docs.digitalocean.com/products/app-platform/'

    credential_schema:
      fields:
        - name: 'api_token'
          label: 'DigitalOcean API Token'
          field_type: 'api_token'
          required: true
          description: 'Personal access token with write access'

        - name: 'region'
          label: 'Deployment Region'
          field_type: 'select'
          required: true
          description: 'App Platform region'
          options: []

    regions:
      - code: 'nyc'
        name: 'New York'
        location: 'US East'
      - code: 'sfo'
        name: 'San Francisco'
        location: 'US West'
      - code: 'ams'
        name: 'Amsterdam'
        location: 'Europe'
      - code: 'sgp'
        name: 'Singapore'
        location: 'Asia Pacific'

    deployment_config:
      instance_size_slug: 'basic-xxs'
      instance_count: 1

    supported_languages:
      - 'javascript'
      - 'typescript'
      - 'python'
      - 'go'
```

##### Rust Implementation with Config Loading

```rust
// crates/ampel-intelligence/src/cd/mod.rs

pub mod flyio;
pub mod aws;
pub mod gcp;
pub mod azure;
pub mod digitalocean;

/// Provider configuration from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub api_base: Option<String>,
    #[serde(default)]
    pub docs_url: Option<String>,
    pub credential_schema: CredentialSchema,
    #[serde(default)]
    pub deployment_config: Option<serde_yaml::Value>,
    #[serde(default)]
    pub regions: Vec<Region>,
    #[serde(default)]
    pub supported_languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentProvidersConfig {
    pub version: String,
    pub providers: HashMap<String, ProviderConfig>,
}

/// Configuration loader for deployment providers
pub struct DeploymentProviderConfigLoader {
    config_path: PathBuf,
}

impl DeploymentProviderConfigLoader {
    pub fn new(config_dir: &Path) -> Self {
        Self {
            config_path: config_dir.join("deployment-providers.yaml"),
        }
    }

    pub fn load(&self) -> Result<DeploymentProvidersConfig, ConfigError> {
        let yaml_content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| ConfigError::FileRead(self.config_path.clone(), e.to_string()))?;
        let config: DeploymentProvidersConfig = serde_yaml::from_str(&yaml_content)
            .map_err(|e| ConfigError::Deserialize(e.to_string()))?;
        Ok(config)
    }
}

/// Deployment provider plugin interface
#[async_trait]
pub trait DeploymentProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn credential_schema(&self) -> &CredentialSchema;
    fn regions(&self) -> &[Region];

    async fn validate_credentials(&self, credentials: &ProviderCredentials) -> Result<bool, DeploymentError>;
    async fn generate_workflow(
        &self,
        tech_stack: &DetectedTechStack,
        config: &DeploymentConfig,
        git_provider: GitProvider,
    ) -> Result<GeneratedWorkflow, DeploymentError>;
    fn generate_config(
        &self,
        tech_stack: &DetectedTechStack,
        config: &DeploymentConfig,
    ) -> Result<Option<GeneratedFile>, DeploymentError>;
    async fn deploy(
        &self,
        credentials: &ProviderCredentials,
        config: &DeploymentConfig,
    ) -> Result<DeploymentResult, DeploymentError>;
}

/// Credential schema loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSchema {
    pub fields: Vec<CredentialField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialField {
    pub name: String,
    pub label: String,
    pub field_type: CredentialFieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub validation_pattern: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialFieldType {
    ApiToken,
    Secret,
    Text,
    Select,
    Region,
}

/// Config-driven provider base implementation
pub struct ConfigDrivenProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl ConfigDrivenProvider {
    pub fn from_config(config: ProviderConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Merge region options into credential schema
    pub fn credential_schema_with_regions(&self) -> CredentialSchema {
        let mut schema = self.config.credential_schema.clone();
        for field in &mut schema.fields {
            if field.field_type == CredentialFieldType::Select && field.options.is_empty() {
                // Populate from regions if field is region-type
                if field.name == "region" {
                    field.options = self.config.regions.iter()
                        .map(|r| r.code.clone())
                        .collect();
                }
            }
        }
        schema
    }
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
      - uses: actions/checkout@v6

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

### 6.6 Quantum-Safe Encryption for Credential Storage

Given the critical importance of protecting PATs, API tokens, and deployment credentials, Ampel adopts **post-quantum cryptography (PQC)** to ensure long-term security against future quantum computing threats. This is essential for a "harvest now, decrypt later" defense strategy.

#### 6.6.1 Recommended Rust PQC Libraries

Based on research into well-maintained, secure, and production-ready options:

| Library                      | Description                                                             | Maintenance                | Security Level |
| ---------------------------- | ----------------------------------------------------------------------- | -------------------------- | -------------- |
| **`aws-lc-rs`**              | Amazon's production-grade cryptographic library with NIST ML-KEM/ML-DSA | AWS-backed, FIPS-validated | ⭐⭐⭐⭐⭐     |
| **`pqcrypto`**               | Pure Rust wrappers for NIST PQC winners                                 | Well-maintained, audited   | ⭐⭐⭐⭐       |
| **`libcrux`**                | Formally verified PQC implementations from Cryspen                      | Formally verified          | ⭐⭐⭐⭐⭐     |
| **`oqs`** (liboqs-rust)      | Open Quantum Safe bindings                                              | Research-backed            | ⭐⭐⭐⭐       |
| **RustCrypto ML-KEM/ML-DSA** | Pure Rust NIST standard implementations                                 | RustCrypto ecosystem       | ⭐⭐⭐⭐       |

#### 6.6.2 Recommended Approach: Hybrid Encryption

For maximum security, we implement **hybrid encryption** combining classical and post-quantum algorithms:

```yaml
# config/encryption-config.yaml
version: '1.0'

credential_encryption:
  # Hybrid approach: combine classical + PQC for defense in depth
  strategy: hybrid

  classical:
    algorithm: AES-256-GCM
    key_derivation: Argon2id
    library: ring # or aws-lc-rs

  post_quantum:
    # NIST-standardized algorithms (August 2024)
    key_encapsulation: ML-KEM-768 # FIPS 203 (formerly Kyber)
    digital_signature: ML-DSA-65 # FIPS 204 (formerly Dilithium)
    library: aws-lc-rs # Production-ready, FIPS-validated

  # Key hierarchy
  key_hierarchy:
    master_key:
      storage: hardware_security_module # AWS CloudHSM or Azure Key Vault
      rotation_days: 365
    data_encryption_keys:
      derivation: HKDF-SHA-384
      rotation_days: 90
    credential_keys:
      per_tenant: true
      rotation_days: 30
```

#### 6.6.3 Rust Implementation

```rust
// crates/ampel-core/src/crypto/hybrid_encryption.rs

use aws_lc_rs::kem::{DecapsulationKey, EncapsulationKey, ML_KEM_768};
use aws_lc_rs::aead::{Aead, LessSafeKey, AES_256_GCM, Nonce};
use aws_lc_rs::hkdf::{Hkdf, HKDF_SHA384};
use zeroize::Zeroizing;

/// Hybrid encryption combining ML-KEM-768 with AES-256-GCM
pub struct HybridEncryptionService {
    /// Classical encryption key (existing AES-256-GCM)
    classical_key: LessSafeKey,
    /// Post-quantum key encapsulation mechanism
    pq_encapsulation_key: EncapsulationKey<ML_KEM_768>,
    pq_decapsulation_key: DecapsulationKey<ML_KEM_768>,
}

impl HybridEncryptionService {
    /// Initialize with hybrid key generation
    pub fn new(master_secret: &[u8]) -> Result<Self, CryptoError> {
        // Derive classical key using Argon2id
        let classical_key = Self::derive_classical_key(master_secret)?;

        // Generate ML-KEM-768 keypair
        let (decap_key, encap_key) = ML_KEM_768
            .generate_key_pair()
            .map_err(|_| CryptoError::KeyGeneration)?;

        Ok(Self {
            classical_key,
            pq_encapsulation_key: encap_key,
            pq_decapsulation_key: decap_key,
        })
    }

    /// Encrypt credential with hybrid encryption
    /// Returns: (pq_ciphertext, classical_ciphertext, nonce)
    pub fn encrypt_credential(&self, plaintext: &[u8]) -> Result<HybridCiphertext, CryptoError> {
        // Step 1: Generate shared secret via ML-KEM encapsulation
        let (pq_ciphertext, shared_secret) = self.pq_encapsulation_key
            .encapsulate()
            .map_err(|_| CryptoError::Encapsulation)?;

        // Step 2: Derive AES key from shared secret using HKDF
        let hkdf = Hkdf::new(HKDF_SHA384, None, shared_secret.as_ref());
        let mut derived_key = Zeroizing::new([0u8; 32]);
        hkdf.expand(&[b"ampel-credential-encryption"], &mut *derived_key)
            .map_err(|_| CryptoError::KeyDerivation)?;

        // Step 3: Encrypt with AES-256-GCM using derived key
        let aes_key = LessSafeKey::new(
            aws_lc_rs::aead::UnboundKey::new(&AES_256_GCM, &*derived_key)
                .map_err(|_| CryptoError::KeyCreation)?
        );

        let nonce = Self::generate_nonce()?;
        let mut ciphertext = plaintext.to_vec();
        aes_key.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce),
            aws_lc_rs::aead::Aad::empty(),
            &mut ciphertext,
        ).map_err(|_| CryptoError::Encryption)?;

        Ok(HybridCiphertext {
            pq_ciphertext: pq_ciphertext.as_ref().to_vec(),
            classical_ciphertext: ciphertext,
            nonce: nonce.to_vec(),
            algorithm: "ML-KEM-768+AES-256-GCM".into(),
        })
    }

    /// Decrypt credential with hybrid decryption
    pub fn decrypt_credential(&self, ciphertext: &HybridCiphertext) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
        // Step 1: Decapsulate to recover shared secret
        let shared_secret = self.pq_decapsulation_key
            .decapsulate(&ciphertext.pq_ciphertext)
            .map_err(|_| CryptoError::Decapsulation)?;

        // Step 2: Derive AES key from shared secret
        let hkdf = Hkdf::new(HKDF_SHA384, None, shared_secret.as_ref());
        let mut derived_key = Zeroizing::new([0u8; 32]);
        hkdf.expand(&[b"ampel-credential-encryption"], &mut *derived_key)
            .map_err(|_| CryptoError::KeyDerivation)?;

        // Step 3: Decrypt with AES-256-GCM
        let aes_key = LessSafeKey::new(
            aws_lc_rs::aead::UnboundKey::new(&AES_256_GCM, &*derived_key)
                .map_err(|_| CryptoError::KeyCreation)?
        );

        let nonce_array: [u8; 12] = ciphertext.nonce.clone().try_into()
            .map_err(|_| CryptoError::InvalidNonce)?;

        let mut plaintext = ciphertext.classical_ciphertext.clone();
        aes_key.open_in_place(
            Nonce::assume_unique_for_key(nonce_array),
            aws_lc_rs::aead::Aad::empty(),
            &mut plaintext,
        ).map_err(|_| CryptoError::Decryption)?;

        // Remove authentication tag
        plaintext.truncate(plaintext.len() - AES_256_GCM.tag_len());

        Ok(Zeroizing::new(plaintext))
    }
}

/// Hybrid ciphertext structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridCiphertext {
    /// ML-KEM encapsulated ciphertext
    pub pq_ciphertext: Vec<u8>,
    /// AES-256-GCM encrypted data with authentication tag
    pub classical_ciphertext: Vec<u8>,
    /// Nonce for AES-GCM
    pub nonce: Vec<u8>,
    /// Algorithm identifier for future versioning
    pub algorithm: String,
}
```

#### 6.6.4 Key Management Best Practices

```yaml
# config/key-management.yaml
version: '1.0'

key_management:
  # Hardware Security Module integration
  hsm:
    enabled: true
    providers:
      - type: aws_cloudhsm
        region: us-east-1
      - type: azure_key_vault
        vault_name: ampel-credentials

  # Key rotation schedule
  rotation:
    master_key:
      frequency_days: 365
      strategy: gradual # Allow time for re-encryption
    data_encryption_key:
      frequency_days: 90
      automatic: true
    per_tenant_keys:
      frequency_days: 30
      automatic: true

  # Key escrow for disaster recovery
  escrow:
    enabled: true
    split_threshold: 3 # Shamir secret sharing
    total_shares: 5
    storage:
      - type: secure_backup
        location: offline_vault
      - type: geo_distributed
        regions: [us-east-1, eu-west-1, ap-southeast-1]

  # Zeroization policy
  memory_protection:
    zeroize_on_drop: true
    mlock_sensitive: true
    guard_pages: true
```

#### 6.6.5 Migration Strategy from Classical to Hybrid Encryption

```rust
// Migration service for transitioning existing credentials
pub struct EncryptionMigrationService {
    legacy_service: Arc<AesEncryptionService>,
    hybrid_service: Arc<HybridEncryptionService>,
}

impl EncryptionMigrationService {
    /// Migrate a single credential to hybrid encryption
    pub async fn migrate_credential(&self, cred_id: Uuid) -> Result<(), MigrationError> {
        // Read with legacy decryption
        let legacy_ciphertext = self.db.get_encrypted_credential(cred_id).await?;
        let plaintext = self.legacy_service.decrypt(&legacy_ciphertext)?;

        // Re-encrypt with hybrid scheme
        let hybrid_ciphertext = self.hybrid_service.encrypt_credential(&plaintext)?;

        // Update with new ciphertext and mark as migrated
        self.db.update_credential_encryption(
            cred_id,
            &hybrid_ciphertext,
            EncryptionVersion::HybridV1,
        ).await?;

        tracing::info!(?cred_id, "Migrated credential to hybrid encryption");
        Ok(())
    }

    /// Background job for gradual migration
    pub async fn run_migration_batch(&self, batch_size: usize) -> MigrationStats {
        let pending = self.db.get_credentials_needing_migration(batch_size).await?;

        let mut stats = MigrationStats::default();
        for cred_id in pending {
            match self.migrate_credential(cred_id).await {
                Ok(()) => stats.success += 1,
                Err(e) => {
                    stats.failures += 1;
                    tracing::error!(?cred_id, ?e, "Migration failed");
                }
            }
        }

        stats
    }
}
```

#### 6.6.6 Security Considerations for PQC Implementation

| Consideration               | Recommendation                                                                                          |
| --------------------------- | ------------------------------------------------------------------------------------------------------- |
| **Library Choice**          | Use `aws-lc-rs` for production (FIPS-validated) or `libcrux` for formally verified implementations      |
| **Key Sizes**               | ML-KEM-768 provides NIST Security Level 3 (equivalent to AES-192); use ML-KEM-1024 for highest security |
| **Hybrid Approach**         | Always combine PQC with classical crypto for defense in depth during transition period                  |
| **Side-Channel Resistance** | Ensure constant-time implementations; `aws-lc-rs` and `libcrux` are designed for this                   |
| **Memory Safety**           | Use `zeroize` crate to clear sensitive data; enable `mlock` for key material                            |
| **NIST Standards**          | Target FIPS 203 (ML-KEM), FIPS 204 (ML-DSA), FIPS 205 (SLH-DSA) compliance                              |

---

## 7. Multi-tenancy Implementation

### 7.1 Data Isolation

```
┌──────────────────────────────────────────────────────────────────┐
│                        TENANT ISOLATION                          │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐             │
│  │  User A     │   │  User B     │   │  User C     │             │
│  │             │   │             │   │             │             │
│  │ Repos: 5    │   │ Repos: 12   │   │ Repos: 3    │             │
│  │ Creds: 2    │   │ Creds: 5    │   │ Creds: 1    │             │
│  │ Analyses: 5 │   │ Analyses: 12│   │ Analyses: 3 │             │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘             │
│         │                 │                 │                    │
│         ▼                 ▼                 ▼                    │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                    PostgreSQL                            │    │
│  │                                                          │    │
│  │   All tables include user_id foreign key                 │    │
│  │   Row-level security via application queries             │    │
│  │                                                          │    │
│  │   SELECT * FROM deployment_credentials                   │    │
│  │   WHERE user_id = $current_user_id  -- Always enforced   │    │
│  │                                                          │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                    RuVector / AgentDB                    │    │
│  │                                                          │    │
│  │   Embeddings tagged with tenant_id in metadata           │    │
│  │   Search queries filtered by tenant_id                   │    │
│  │                                                          │    │
│  │   // Store with tenant context                           │    │
│  │   db.insert(embedding, { tenant_id: user_id, ... })      │    │
│  │                                                          │    │
│  │   // Query with tenant filter                            │    │
│  │   db.search(query, { filter: { tenant_id: user_id } })   │    │
│  │                                                          │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 7.2 Credential Scoping

```
┌────────────────────────────────────────────────────────────────┐
│                    CREDENTIAL HIERARCHY                        │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│   User Account                                                 │
│   └── Default Fly.io Token (account-level, is_default=true)    │
│       │                                                        │
│       ├── Repository: my-api                                   │
│       │   └── (uses account default)                           │
│       │                                                        │
│       ├── Repository: client-app                               │
│       │   └── Custom Fly.io Token (repo-level override)        │
│       │                                                        │
│       └── Repository: internal-tool                            │
│           └── (uses account default)                           │
│                                                                │
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
