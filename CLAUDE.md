# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ampel is a unified PR management dashboard that consolidates pull requests from GitHub, GitLab,
and Bitbucket into a single interface using a traffic light system (green=ready to merge,
yellow=in progress, red=blocked).

## Tech Stack

- **Backend**: Rust 1.92+ (Axum 0.8, SeaORM, Apalis for background jobs, Tokio async runtime)
- **Frontend**: React 19 + TypeScript, Vite, TanStack Query, shadcn/ui, Tailwind CSS
- **Database**: PostgreSQL 16, Redis 7 for caching
- **Package Manager**: pnpm 10.24.0
- **Rust Version**: 1.92.0 (pinned in CI and Docker images)

## Commands

All commands run via the root Makefile:

```bash
# Development (run in separate terminals)
make dev-api            # API server on :8080
make dev-worker         # Background job worker
make dev-frontend       # Frontend dev server on :5173

# Build
make build              # Build all (debug)
make build-release      # Build all (release)

# Testing
make test               # Run all tests (backend + frontend)
make test-backend       # Backend tests: cargo test --all-features
make test-frontend      # Frontend tests: vitest --run

# Code Quality
make lint               # Run all linters
make lint-backend       # cargo clippy --all-targets --all-features -- -D warnings
make lint-frontend      # pnpm run lint (ESLint)
make lint-fix           # Auto-fix all lint issues
make format             # Format all code
make format-check       # Check formatting without changes

# Docker
make docker-up          # Start all services
make docker-down        # Stop services
```

**See [docs/TESTING.md](docs/TESTING.md) for comprehensive testing documentation.**

## Architecture

### Crate Structure (Rust Backend)

```text
crates/
├── ampel-api/          # REST API (Axum handlers, routes, middleware)
├── ampel-core/         # Business logic and domain models
├── ampel-db/           # SeaORM entities, migrations, queries
├── ampel-providers/    # Git provider abstractions (GitHub, GitLab, Bitbucket)
└── ampel-worker/       # Background job processing (Apalis)
```

### Provider Abstraction

Git providers implement a trait-based abstraction in `ampel-providers`:

```rust
#[async_trait]
pub trait GitProvider: Send + Sync {
    async fn list_repositories(&self) -> Result<Vec<Repository>, ProviderError>;
    async fn list_pull_requests(&self, repo_id: &str) -> Result<Vec<PullRequest>, ProviderError>;
}
```

### Frontend Structure

```text
frontend/src/
├── api/                # Axios-based API client functions
├── components/
│   ├── ui/             # shadcn/ui components
│   ├── layout/         # Layout wrappers
│   └── dashboard/      # PR dashboard components
├── hooks/              # Custom React hooks
├── pages/              # Route page components
└── types/              # TypeScript interfaces
```

### Nginx Configuration (Dual Environment Setup)

The project uses separate nginx configurations for development and production:

- **`docker/nginx.dev.conf`**: Permissive CSP for local Docker development
  - Allows `localhost:*` and `127.0.0.1:*` connections
  - Includes `'unsafe-eval'` for development tools
  - Used by `docker-compose.yml` via `NGINX_CONFIG` build arg
  - Fixes Firefox blocking issues when frontend (port 3000) calls API (port 8080)

- **`docker/nginx.prod.conf`**: Strict CSP for Fly.io production deployment
  - Only allows connections to production API domains
  - Removes unsafe directives
  - Used by `.github/workflows/deploy.yml` for Fly.io deployments

**Important**: Never deploy `nginx.dev.conf` to production. The GitHub Actions workflow automatically uses `nginx.prod.conf` for all Fly.io deployments.

### Key Patterns

- **State Management**: TanStack Query for server state caching
- **Forms**: React Hook Form with Zod validation
- **Auth**: JWT (15-min access tokens, 7-day refresh in httpOnly cookies)
- **Token Storage**: Provider PAT tokens encrypted with AES-256-GCM
- **Password Hashing**: Argon2id
- **Background Jobs**: Apalis with PostgreSQL persistence

### Database Models

Core entity relationships:

- Users → Organizations (many-to-many) → Teams → Repositories → PullRequests
- PullRequests have: ci_statuses, reviews, ampel_status (green/yellow/red)

## Entry Points

- API Server: `crates/ampel-api/src/main.rs`
- Worker: `crates/ampel-worker/src/main.rs`
- Frontend: `frontend/src/main.tsx`
- API Docs: `/api/docs` (Swagger UI via utoipa)

## Testing

### Quick Reference

```bash
make test               # Run all tests
make test-backend       # Backend only (cargo test --all-features)
make test-frontend      # Frontend only (vitest --run)
```

### Test Organization

- **Backend Unit Tests**: In `#[cfg(test)]` modules alongside source code
- **Backend Integration Tests**: In `crates/*/tests/` directories
- **Frontend Tests**: Co-located with components or in `__tests__/` directories

### Database Testing

Backend integration tests support both PostgreSQL and SQLite:

- **PostgreSQL** (CI default): Full feature testing with migrations
- **SQLite** (Fast local): Quick unit tests, auto-skips migration-dependent tests

```bash
# Use PostgreSQL for tests
export DATABASE_URL="postgres://ampel:ampel@localhost:5432/ampel_test"
export TEST_DATABASE_TYPE=postgres
cargo test --all-features

# Use SQLite for tests (default if no DATABASE_URL)
export DATABASE_URL="sqlite::memory:"
cargo test --all-features
```

### Coverage Goals

- **Target**: 80% code coverage
- **Focus**: Critical paths (auth, data validation, business logic)
- **CI**: Automatic coverage reports on pull requests

**For detailed testing guide, see [docs/TESTING.md](docs/TESTING.md)**

## Environment Setup

```bash
cp .env.example .env    # Configure environment variables
make install            # Install all dependencies
```

---

# Claude Code Configuration - Claude Flow V3

## 🚨 AUTOMATIC SWARM ORCHESTRATION

**When starting work on complex tasks, Claude Code MUST automatically:**

1. **Initialize the swarm** using CLI tools via Bash
2. **Spawn concurrent agents** using Claude Code's Task tool
3. **Coordinate via hooks** and memory

### 🚨 CRITICAL: CLI + Task Tool in SAME Message

**When user says "spawn swarm" or requests complex work, Claude Code MUST in ONE message:**

1. Call CLI tools via Bash to initialize coordination
2. **IMMEDIATELY** call Task tool to spawn REAL working agents
3. Both CLI and Task calls must be in the SAME response

**CLI coordinates, Task tool agents do the actual work!**

### 🤖 INTELLIGENT 3-TIER MODEL ROUTING (ADR-026)

**The routing system has 3 tiers for optimal cost/performance:**

| Tier  | Handler       | Latency | Cost          | Use Cases                                                |
| ----- | ------------- | ------- | ------------- | -------------------------------------------------------- |
| **1** | Agent Booster | <1ms    | $0            | Simple transforms (var→const, add-types, remove-console) |
| **2** | Haiku         | ~500ms  | $0.0002       | Simple tasks, bug fixes, low complexity                  |
| **3** | Sonnet/Opus   | 2-5s    | $0.003-$0.015 | Architecture, security, complex reasoning                |

**Before spawning agents, get routing recommendation:**

```bash
npx @claude-flow/cli@latest hooks pre-task --description "[task description]"
```

**When you see these recommendations:**

1. `[AGENT_BOOSTER_AVAILABLE]` → Skip LLM entirely, use Edit tool directly
   - Intent types: `var-to-const`, `add-types`, `add-error-handling`, `async-await`, `add-logging`, `remove-console`

2. `[TASK_MODEL_RECOMMENDATION] Use model="X"` → Use that model in Task tool:

```javascript
Task({
  prompt: '...',
  subagent_type: 'coder',
  model: 'haiku', // ← USE THE RECOMMENDED MODEL (haiku/sonnet/opus)
});
```

**Benefits:** 75% cost reduction, 352x faster for Tier 1 tasks

---

### 🛡️ Anti-Drift Config (PREFERRED)

**Use this to prevent agent drift:**

```bash
# Small teams (6-8 agents) - use hierarchical for tight control
npx @claude-flow/cli@latest swarm init --topology hierarchical --max-agents 8 --strategy specialized

# Large teams (10-15 agents) - use hierarchical-mesh for V3 queen + peer communication
npx @claude-flow/cli@latest swarm init --topology hierarchical-mesh --max-agents 15 --strategy specialized
```

**Valid Topologies:**

- `hierarchical` - Queen controls workers directly (anti-drift for small teams)
- `hierarchical-mesh` - V3 queen + peer communication (recommended for 10+ agents)
- `mesh` - Fully connected peer network
- `ring` - Circular communication pattern
- `star` - Central coordinator with spokes
- `hybrid` - Dynamic topology switching

**Anti-Drift Guidelines:**

- **hierarchical**: Coordinator catches divergence
- **max-agents 6-8**: Smaller team = less drift
- **specialized**: Clear roles, no overlap
- **consensus**: raft (leader maintains state)

---

### 🔄 Auto-Start Swarm Protocol (Background Execution)

When the user requests a complex task, **spawn agents in background and WAIT for completion:**

```javascript
// STEP 1: Initialize swarm coordination (anti-drift config)
Bash(
  'npx @claude-flow/cli@latest swarm init --topology hierarchical --max-agents 8 --strategy specialized'
);

// STEP 2: Spawn ALL agents IN BACKGROUND in a SINGLE message
// Use run_in_background: true so agents work concurrently
Task({
  prompt: 'Research requirements, analyze codebase patterns, store findings in memory',
  subagent_type: 'researcher',
  description: 'Research phase',
  run_in_background: true, // ← CRITICAL: Run in background
});
Task({
  prompt: 'Design architecture based on research. Document decisions.',
  subagent_type: 'system-architect',
  description: 'Architecture phase',
  run_in_background: true,
});
Task({
  prompt: 'Implement the solution following the design. Write clean code.',
  subagent_type: 'coder',
  description: 'Implementation phase',
  run_in_background: true,
});
Task({
  prompt: 'Write comprehensive tests for the implementation.',
  subagent_type: 'tester',
  description: 'Testing phase',
  run_in_background: true,
});
Task({
  prompt: 'Review code quality, security, and best practices.',
  subagent_type: 'reviewer',
  description: 'Review phase',
  run_in_background: true,
});

// STEP 3: WAIT - Tell user agents are working, then STOP
// Say: "I've spawned 5 agents to work on this in parallel. They'll report back when done."
// DO NOT check status repeatedly. Just wait for user or agent responses.
```

### ⏸️ CRITICAL: Spawn and Wait Pattern

**After spawning background agents:**

1. **TELL USER** - "I've spawned X agents working in parallel on: [list tasks]"
2. **STOP** - Do not continue with more tool calls
3. **WAIT** - Let the background agents complete their work
4. **RESPOND** - When agents return results, review and synthesize

**Example response after spawning:**

```
I've launched 5 concurrent agents to work on this:
- 🔍 Researcher: Analyzing requirements and codebase
- 🏗️ Architect: Designing the implementation approach
- 💻 Coder: Implementing the solution
- 🧪 Tester: Writing tests
- 👀 Reviewer: Code review and security check

They're working in parallel. I'll synthesize their results when they complete.
```

### 🚫 DO NOT:

- Continuously check swarm status
- Poll TaskOutput repeatedly
- Add more tool calls after spawning
- Ask "should I check on the agents?"

### ✅ DO:

- Spawn all agents in ONE message
- Tell user what's happening
- Wait for agent results to arrive
- Synthesize results when they return

## 🧠 AUTO-LEARNING PROTOCOL

### Before Starting Any Task

```bash
# 1. Search memory for relevant patterns from past successes
Bash("npx @claude-flow/cli@latest memory search --query '[task keywords]' --namespace patterns")

# 2. Check if similar task was done before
Bash("npx @claude-flow/cli@latest memory search --query '[task type]' --namespace tasks")

# 3. Load learned optimizations
Bash("npx @claude-flow/cli@latest hooks route --task '[task description]'")
```

### After Completing Any Task Successfully

```bash
# 1. Store successful pattern for future reference
Bash("npx @claude-flow/cli@latest memory store --namespace patterns --key '[pattern-name]' --value '[what worked]'")

# 2. Train neural patterns on the successful approach
Bash("npx @claude-flow/cli@latest hooks post-edit --file '[main-file]' --train-neural true")

# 3. Record task completion with metrics
Bash("npx @claude-flow/cli@latest hooks post-task --task-id '[id]' --success true --store-results true")

# 4. Trigger optimization worker if performance-related
Bash("npx @claude-flow/cli@latest hooks worker dispatch --trigger optimize")
```

### Continuous Improvement Triggers

| Trigger                | Worker     | When to Use                |
| ---------------------- | ---------- | -------------------------- |
| After major refactor   | `optimize` | Performance optimization   |
| After adding features  | `testgaps` | Find missing test coverage |
| After security changes | `audit`    | Security analysis          |
| After API changes      | `document` | Update documentation       |
| Every 5+ file changes  | `map`      | Update codebase map        |
| Complex debugging      | `deepdive` | Deep code analysis         |

### Memory-Enhanced Development

**ALWAYS check memory before:**

- Starting a new feature (search for similar implementations)
- Debugging an issue (search for past solutions)
- Refactoring code (search for learned patterns)
- Performance work (search for optimization strategies)

**ALWAYS store in memory after:**

- Solving a tricky bug (store the solution pattern)
- Completing a feature (store the approach)
- Finding a performance fix (store the optimization)
- Discovering a security issue (store the vulnerability pattern)

### 📋 Agent Routing (Anti-Drift)

| Code | Task        | Agents                                          |
| ---- | ----------- | ----------------------------------------------- |
| 1    | Bug Fix     | coordinator, researcher, coder, tester          |
| 3    | Feature     | coordinator, architect, coder, tester, reviewer |
| 5    | Refactor    | coordinator, architect, coder, reviewer         |
| 7    | Performance | coordinator, perf-engineer, coder               |
| 9    | Security    | coordinator, security-architect, auditor        |
| 11   | Docs        | researcher, api-docs                            |

**Codes 1-9: hierarchical/specialized (anti-drift). Code 11: mesh/balanced**

### 🎯 Task Complexity Detection

**AUTO-INVOKE SWARM when task involves:**

- Multiple files (3+)
- New feature implementation
- Refactoring across modules
- API changes with tests
- Security-related changes
- Performance optimization
- Database schema changes

**SKIP SWARM for:**

- Single file edits
- Simple bug fixes (1-2 lines)
- Documentation updates
- Configuration changes
- Quick questions/exploration

## 🚨 CRITICAL: CONCURRENT EXECUTION & FILE MANAGEMENT

**ABSOLUTE RULES**:

1. ALL operations MUST be concurrent/parallel in a single message
2. **NEVER save working files, text/mds and tests to the root folder**
3. ALWAYS organize files in appropriate subdirectories
4. **USE CLAUDE CODE'S TASK TOOL** for spawning agents concurrently, not just MCP

### ⚡ GOLDEN RULE: "1 MESSAGE = ALL RELATED OPERATIONS"

**MANDATORY PATTERNS:**

- **TodoWrite**: ALWAYS batch ALL todos in ONE call (5-10+ todos minimum)
- **Task tool (Claude Code)**: ALWAYS spawn ALL agents in ONE message with full instructions
- **File operations**: ALWAYS batch ALL reads/writes/edits in ONE message
- **Bash commands**: ALWAYS batch ALL terminal operations in ONE message
- **Memory operations**: ALWAYS batch ALL memory store/retrieve in ONE message

### 📁 File Organization Rules

**NEVER save to root folder. Use these directories:**

- `/src` - Source code files
- `/tests` - Test files
- `/docs` - Documentation and markdown files
- `/config` - Configuration files
- `/scripts` - Utility scripts
- `/examples` - Example code

## Project Config (Anti-Drift Defaults)

- **Topology**: hierarchical (prevents drift)
- **Max Agents**: 8 (smaller = less drift)
- **Strategy**: specialized (clear roles)
- **Consensus**: raft
- **Memory**: hybrid
- **HNSW**: Enabled
- **Neural**: Enabled

## 🚀 V3 CLI Commands (26 Commands, 140+ Subcommands)

### Core Commands

| Command     | Subcommands | Description                                                              |
| ----------- | ----------- | ------------------------------------------------------------------------ |
| `init`      | 4           | Project initialization with wizard, presets, skills, hooks               |
| `agent`     | 8           | Agent lifecycle (spawn, list, status, stop, metrics, pool, health, logs) |
| `swarm`     | 6           | Multi-agent swarm coordination and orchestration                         |
| `memory`    | 11          | AgentDB memory with vector search (150x-12,500x faster)                  |
| `mcp`       | 9           | MCP server management and tool execution                                 |
| `task`      | 6           | Task creation, assignment, and lifecycle                                 |
| `session`   | 7           | Session state management and persistence                                 |
| `config`    | 7           | Configuration management and provider setup                              |
| `status`    | 3           | System status monitoring with watch mode                                 |
| `workflow`  | 6           | Workflow execution and template management                               |
| `hooks`     | 17          | Self-learning hooks + 12 background workers                              |
| `hive-mind` | 6           | Queen-led Byzantine fault-tolerant consensus                             |

### Advanced Commands

| Command       | Subcommands | Description                                                                   |
| ------------- | ----------- | ----------------------------------------------------------------------------- |
| `daemon`      | 5           | Background worker daemon (start, stop, status, trigger, enable)               |
| `neural`      | 5           | Neural pattern training (train, status, patterns, predict, optimize)          |
| `security`    | 6           | Security scanning (scan, audit, cve, threats, validate, report)               |
| `performance` | 5           | Performance profiling (benchmark, profile, metrics, optimize, report)         |
| `providers`   | 5           | AI providers (list, add, remove, test, configure)                             |
| `plugins`     | 5           | Plugin management (list, install, uninstall, enable, disable)                 |
| `deployment`  | 5           | Deployment management (deploy, rollback, status, environments, release)       |
| `embeddings`  | 4           | Vector embeddings (embed, batch, search, init) - 75x faster with agentic-flow |
| `claims`      | 4           | Claims-based authorization (check, grant, revoke, list)                       |
| `migrate`     | 5           | V2 to V3 migration with rollback support                                      |
| `doctor`      | 1           | System diagnostics with health checks                                         |
| `completions` | 4           | Shell completions (bash, zsh, fish, powershell)                               |

### Quick CLI Examples

```bash
# Initialize project
npx @claude-flow/cli@latest init --wizard

# Start daemon with background workers
npx @claude-flow/cli@latest daemon start

# Spawn an agent
npx @claude-flow/cli@latest agent spawn -t coder --name my-coder

# Initialize swarm
npx @claude-flow/cli@latest swarm init --v3-mode

# Search memory (HNSW-indexed)
npx @claude-flow/cli@latest memory search --query "authentication patterns"

# System diagnostics
npx @claude-flow/cli@latest doctor --fix

# Security scan
npx @claude-flow/cli@latest security scan --depth full

# Performance benchmark
npx @claude-flow/cli@latest performance benchmark --suite all
```

## 🚀 Available Agents (60+ Types)

### Core Development

`coder`, `reviewer`, `tester`, `planner`, `researcher`

### V3 Specialized Agents

`security-architect`, `security-auditor`, `memory-specialist`, `performance-engineer`

### 🔐 @claude-flow/security

CVE remediation, input validation, path security:

- `InputValidator` - Zod validation
- `PathValidator` - Traversal prevention
- `SafeExecutor` - Injection protection

### Swarm Coordination

`hierarchical-coordinator`, `mesh-coordinator`, `adaptive-coordinator`, `collective-intelligence-coordinator`, `swarm-memory-manager`

### Consensus & Distributed

`byzantine-coordinator`, `raft-manager`, `gossip-coordinator`, `consensus-builder`, `crdt-synchronizer`, `quorum-manager`, `security-manager`

### Performance & Optimization

`perf-analyzer`, `performance-benchmarker`, `task-orchestrator`, `memory-coordinator`, `smart-agent`

### GitHub & Repository

`github-modes`, `pr-manager`, `code-review-swarm`, `issue-tracker`, `release-manager`, `workflow-automation`, `project-board-sync`, `repo-architect`, `multi-repo-swarm`

### SPARC Methodology

`sparc-coord`, `sparc-coder`, `specification`, `pseudocode`, `architecture`, `refinement`

### Specialized Development

`backend-dev`, `mobile-dev`, `ml-developer`, `cicd-engineer`, `api-docs`, `system-architect`, `code-analyzer`, `base-template-generator`

### Testing & Validation

`tdd-london-swarm`, `production-validator`

## 🪝 V3 Hooks System (27 Hooks + 12 Workers)

### All Available Hooks

| Hook               | Description                              | Key Options                                 |
| ------------------ | ---------------------------------------- | ------------------------------------------- |
| `pre-edit`         | Get context before editing files         | `--file`, `--operation`                     |
| `post-edit`        | Record editing outcome for learning      | `--file`, `--success`, `--train-neural`     |
| `pre-command`      | Assess risk before commands              | `--command`, `--validate-safety`            |
| `post-command`     | Record command execution outcome         | `--command`, `--track-metrics`              |
| `pre-task`         | Record task start, get agent suggestions | `--description`, `--coordinate-swarm`       |
| `post-task`        | Record task completion for learning      | `--task-id`, `--success`, `--store-results` |
| `session-start`    | Start/restore session (v2 compat)        | `--session-id`, `--auto-configure`          |
| `session-end`      | End session and persist state            | `--generate-summary`, `--export-metrics`    |
| `session-restore`  | Restore a previous session               | `--session-id`, `--latest`                  |
| `route`            | Route task to optimal agent              | `--task`, `--context`, `--top-k`            |
| `route-task`       | (v2 compat) Alias for route              | `--task`, `--auto-swarm`                    |
| `explain`          | Explain routing decision                 | `--topic`, `--detailed`                     |
| `pretrain`         | Bootstrap intelligence from repo         | `--model-type`, `--epochs`                  |
| `build-agents`     | Generate optimized agent configs         | `--agent-types`, `--focus`                  |
| `metrics`          | View learning metrics dashboard          | `--v3-dashboard`, `--format`                |
| `transfer`         | Transfer patterns via IPFS registry      | `store`, `from-project`                     |
| `list`             | List all registered hooks                | `--format`                                  |
| `intelligence`     | RuVector intelligence system             | `trajectory-*`, `pattern-*`, `stats`        |
| `worker`           | Background worker management             | `list`, `dispatch`, `status`, `detect`      |
| `progress`         | Check V3 implementation progress         | `--detailed`, `--format`                    |
| `statusline`       | Generate dynamic statusline              | `--json`, `--compact`, `--no-color`         |
| `coverage-route`   | Route based on test coverage gaps        | `--task`, `--path`                          |
| `coverage-suggest` | Suggest coverage improvements            | `--path`                                    |
| `coverage-gaps`    | List coverage gaps with priorities       | `--format`, `--limit`                       |
| `pre-bash`         | (v2 compat) Alias for pre-command        | Same as pre-command                         |
| `post-bash`        | (v2 compat) Alias for post-command       | Same as post-command                        |

### 12 Background Workers

| Worker        | Priority | Description                |
| ------------- | -------- | -------------------------- |
| `ultralearn`  | normal   | Deep knowledge acquisition |
| `optimize`    | high     | Performance optimization   |
| `consolidate` | low      | Memory consolidation       |
| `predict`     | normal   | Predictive preloading      |
| `audit`       | critical | Security analysis          |
| `map`         | normal   | Codebase mapping           |
| `preload`     | low      | Resource preloading        |
| `deepdive`    | normal   | Deep code analysis         |
| `document`    | normal   | Auto-documentation         |
| `refactor`    | normal   | Refactoring suggestions    |
| `benchmark`   | normal   | Performance benchmarking   |
| `testgaps`    | normal   | Test coverage analysis     |

### Essential Hook Commands

```bash
# Core hooks
npx @claude-flow/cli@latest hooks pre-task --description "[task]"
npx @claude-flow/cli@latest hooks post-task --task-id "[id]" --success true
npx @claude-flow/cli@latest hooks post-edit --file "[file]" --train-neural true

# Session management
npx @claude-flow/cli@latest hooks session-start --session-id "[id]"
npx @claude-flow/cli@latest hooks session-end --export-metrics true
npx @claude-flow/cli@latest hooks session-restore --session-id "[id]"

# Intelligence routing
npx @claude-flow/cli@latest hooks route --task "[task]"
npx @claude-flow/cli@latest hooks explain --topic "[topic]"

# Neural learning
npx @claude-flow/cli@latest hooks pretrain --model-type moe --epochs 10
npx @claude-flow/cli@latest hooks build-agents --agent-types coder,tester

# Background workers
npx @claude-flow/cli@latest hooks worker list
npx @claude-flow/cli@latest hooks worker dispatch --trigger audit
npx @claude-flow/cli@latest hooks worker status

# Coverage-aware routing
npx @claude-flow/cli@latest hooks coverage-gaps --format table
npx @claude-flow/cli@latest hooks coverage-route --task "[task]"

# Statusline (for Claude Code integration)
npx @claude-flow/cli@latest hooks statusline
npx @claude-flow/cli@latest hooks statusline --json
```

## 🔄 Migration (V2 to V3)

```bash
# Check migration status
npx @claude-flow/cli@latest migrate status

# Run migration with backup
npx @claude-flow/cli@latest migrate run --backup

# Rollback if needed
npx @claude-flow/cli@latest migrate rollback

# Validate migration
npx @claude-flow/cli@latest migrate validate
```

## 🧠 Intelligence System (RuVector)

V3 includes the RuVector Intelligence System:

- **SONA**: Self-Optimizing Neural Architecture (<0.05ms adaptation)
- **MoE**: Mixture of Experts for specialized routing
- **HNSW**: 150x-12,500x faster pattern search
- **EWC++**: Elastic Weight Consolidation (prevents forgetting)
- **Flash Attention**: 2.49x-7.47x speedup

The 4-step intelligence pipeline:

1. **RETRIEVE** - Fetch relevant patterns via HNSW
2. **JUDGE** - Evaluate with verdicts (success/failure)
3. **DISTILL** - Extract key learnings via LoRA
4. **CONSOLIDATE** - Prevent catastrophic forgetting via EWC++

## 📦 Embeddings Package (v3.0.0-alpha.12)

Features:

- **sql.js**: Cross-platform SQLite persistent cache (WASM, no native compilation)
- **Document chunking**: Configurable overlap and size
- **Normalization**: L2, L1, min-max, z-score
- **Hyperbolic embeddings**: Poincaré ball model for hierarchical data
- **75x faster**: With agentic-flow ONNX integration
- **Neural substrate**: Integration with RuVector

## 🐝 Hive-Mind Consensus

### Topologies

- `hierarchical` - Queen controls workers directly
- `mesh` - Fully connected peer network
- `hierarchical-mesh` - Hybrid (recommended)
- `adaptive` - Dynamic based on load

### Consensus Strategies

- `byzantine` - BFT (tolerates f < n/3 faulty)
- `raft` - Leader-based (tolerates f < n/2)
- `gossip` - Epidemic for eventual consistency
- `crdt` - Conflict-free replicated data types
- `quorum` - Configurable quorum-based

## V3 Performance Targets

| Metric           | Target                   |
| ---------------- | ------------------------ |
| Flash Attention  | 2.49x-7.47x speedup      |
| HNSW Search      | 150x-12,500x faster      |
| Memory Reduction | 50-75% with quantization |
| MCP Response     | <100ms                   |
| CLI Startup      | <500ms                   |
| SONA Adaptation  | <0.05ms                  |

## 📊 Performance Optimization Protocol

### Automatic Performance Tracking

```bash
# After any significant operation, track metrics
Bash("npx @claude-flow/cli@latest hooks post-command --command '[operation]' --track-metrics true")

# Periodically run benchmarks (every major feature)
Bash("npx @claude-flow/cli@latest performance benchmark --suite all")

# Analyze bottlenecks when performance degrades
Bash("npx @claude-flow/cli@latest performance profile --target '[component]'")
```

### Session Persistence (Cross-Conversation Learning)

```bash
# At session start - restore previous context
Bash("npx @claude-flow/cli@latest session restore --latest")

# At session end - persist learned patterns
Bash("npx @claude-flow/cli@latest hooks session-end --generate-summary true --persist-state true --export-metrics true")
```

### Neural Pattern Training

```bash
# Train on successful code patterns
Bash("npx @claude-flow/cli@latest neural train --pattern-type coordination --epochs 10")

# Predict optimal approach for new tasks
Bash("npx @claude-flow/cli@latest neural predict --input '[task description]'")

# View learned patterns
Bash("npx @claude-flow/cli@latest neural patterns --list")
```

## 🔧 Environment Variables

```bash
# Configuration
CLAUDE_FLOW_CONFIG=./claude-flow.config.json
CLAUDE_FLOW_LOG_LEVEL=info

# Provider API Keys
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
GOOGLE_API_KEY=...

# MCP Server
CLAUDE_FLOW_MCP_PORT=3000
CLAUDE_FLOW_MCP_HOST=localhost
CLAUDE_FLOW_MCP_TRANSPORT=stdio

# Memory
CLAUDE_FLOW_MEMORY_BACKEND=hybrid
CLAUDE_FLOW_MEMORY_PATH=./data/memory
```

## 🔍 Doctor Health Checks

Run `npx @claude-flow/cli@latest doctor` to check:

- Node.js version (20+)
- npm version (9+)
- Git installation
- Config file validity
- Daemon status
- Memory database
- API keys
- MCP servers
- Disk space
- TypeScript installation

## 🚀 Quick Setup

```bash
# Add MCP servers (auto-detects MCP mode when stdin is piped)
claude mcp add claude-flow -- npx -y @claude-flow/cli@latest
claude mcp add ruv-swarm -- npx -y ruv-swarm mcp start  # Optional
claude mcp add flow-nexus -- npx -y flow-nexus@latest mcp start  # Optional

# Start daemon
npx @claude-flow/cli@latest daemon start

# Run doctor
npx @claude-flow/cli@latest doctor --fix
```

## 🎯 Claude Code vs CLI Tools

### Claude Code Handles ALL EXECUTION:

- **Task tool**: Spawn and run agents concurrently
- File operations (Read, Write, Edit, MultiEdit, Glob, Grep)
- Code generation and programming
- Bash commands and system operations
- TodoWrite and task management
- Git operations

### CLI Tools Handle Coordination (via Bash):

- **Swarm init**: `npx @claude-flow/cli@latest swarm init --topology <type>`
- **Swarm status**: `npx @claude-flow/cli@latest swarm status`
- **Agent spawn**: `npx @claude-flow/cli@latest agent spawn -t <type> --name <name>`
- **Memory store**: `npx @claude-flow/cli@latest memory store --key "mykey" --value "myvalue" --namespace patterns`
- **Memory search**: `npx @claude-flow/cli@latest memory search --query "search terms"`
- **Memory list**: `npx @claude-flow/cli@latest memory list --namespace patterns`
- **Memory retrieve**: `npx @claude-flow/cli@latest memory retrieve --key "mykey" --namespace patterns`
- **Hooks**: `npx @claude-flow/cli@latest hooks <hook-name> [options]`

## 📝 Memory Commands Reference (IMPORTANT)

### Store Data (ALL options shown)

```bash
# REQUIRED: --key and --value
# OPTIONAL: --namespace (default: "default"), --ttl, --tags
npx @claude-flow/cli@latest memory store --key "pattern-auth" --value "JWT with refresh tokens" --namespace patterns
npx @claude-flow/cli@latest memory store --key "bug-fix-123" --value "Fixed null check" --namespace solutions --tags "bugfix,auth"
```

### Search Data (semantic vector search)

```bash
# REQUIRED: --query (full flag, not -q)
# OPTIONAL: --namespace, --limit, --threshold
npx @claude-flow/cli@latest memory search --query "authentication patterns"
npx @claude-flow/cli@latest memory search --query "error handling" --namespace patterns --limit 5
```

### List Entries

```bash
# OPTIONAL: --namespace, --limit
npx @claude-flow/cli@latest memory list
npx @claude-flow/cli@latest memory list --namespace patterns --limit 10
```

### Retrieve Specific Entry

```bash
# REQUIRED: --key
# OPTIONAL: --namespace (default: "default")
npx @claude-flow/cli@latest memory retrieve --key "pattern-auth"
npx @claude-flow/cli@latest memory retrieve --key "pattern-auth" --namespace patterns
```

### Initialize Memory Database

```bash
npx @claude-flow/cli@latest memory init --force --verbose
```

**KEY**: CLI coordinates the strategy via Bash, Claude Code's Task tool executes with real agents.

## 📚 Full Capabilities Reference

For a comprehensive overview of all Claude Flow V3 features, agents, commands, and integrations, see:

**`.claude-flow/CAPABILITIES.md`** - Complete reference generated during init

This includes:

- All 60+ agent types with routing recommendations
- All 26 CLI commands with 140+ subcommands
- All 27 hooks + 12 background workers
- RuVector intelligence system details
- Hive-Mind consensus mechanisms
- Integration ecosystem (agentic-flow, agentdb, ruv-swarm, flow-nexus, agentic-jujutsu)
- Performance targets and status

## Support

- Documentation: https://github.com/ruvnet/claude-flow
- Issues: https://github.com/ruvnet/claude-flow/issues

---

Remember: **Claude Flow CLI coordinates, Claude Code Task tool creates!**

# important-instruction-reminders

Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (\*.md) or README files. Only create documentation files if explicitly requested by the User.
Never save working files, text/mds and tests to the root folder.

## 🚨 SWARM EXECUTION RULES (CRITICAL)

1. **SPAWN IN BACKGROUND**: Use `run_in_background: true` for all agent Task calls
2. **SPAWN ALL AT ONCE**: Put ALL agent Task calls in ONE message for parallel execution
3. **TELL USER**: After spawning, list what each agent is doing (use emojis for clarity)
4. **STOP AND WAIT**: After spawning, STOP - do NOT add more tool calls or check status
5. **NO POLLING**: Never poll TaskOutput or check swarm status - trust agents to return
6. **SYNTHESIZE**: When agent results arrive, review ALL results before proceeding
7. **NO CONFIRMATION**: Don't ask "should I check?" - just wait for results

Example spawn message:

```
"I've launched 4 agents in background:
- 🔍 Researcher: [task]
- 💻 Coder: [task]
- 🧪 Tester: [task]
- 👀 Reviewer: [task]
Working in parallel - I'll synthesize when they complete."
```

---

## Agentic QE v3

This project uses **Agentic QE v3** - a Domain-Driven Quality Engineering platform with 12 bounded contexts, ReasoningBank learning, and HNSW vector search.

### Quick Reference

```bash
# Run tests
npm test -- --run

# Check quality
npx @agentic-qe/v3 quality assess

# Generate tests
npx @agentic-qe/v3 test generate <file>

# Coverage analysis
npx @agentic-qe/v3 coverage <path>
```

### MCP Server

The AQE v3 MCP server is configured in `.claude/mcp.json`. Available tools:

| Tool                          | Description                |
| ----------------------------- | -------------------------- |
| `fleet_init`                  | Initialize QE fleet        |
| `task_submit`                 | Submit QE tasks            |
| `test_generate_enhanced`      | AI-powered test generation |
| `coverage_analyze_sublinear`  | O(log n) coverage analysis |
| `quality_assess`              | Quality gate evaluation    |
| `security_scan_comprehensive` | SAST/DAST scanning         |

### 12 DDD Bounded Contexts

| Domain                  | Purpose                       |
| ----------------------- | ----------------------------- |
| test-generation         | AI-powered test creation      |
| test-execution          | Parallel execution with retry |
| coverage-analysis       | Sublinear gap detection       |
| quality-assessment      | Quality gates                 |
| defect-intelligence     | Defect prediction             |
| requirements-validation | BDD scenarios                 |
| code-intelligence       | Knowledge graph               |
| security-compliance     | SAST/DAST                     |
| contract-testing        | API contracts                 |
| visual-accessibility    | Visual regression             |
| chaos-resilience        | Chaos engineering             |
| learning-optimization   | Cross-domain learning         |

### Configuration

- **Enabled Domains**: test-generation, test-execution, coverage-analysis, learning-optimization, quality-assessment, security-compliance (+1 more)
- **Learning**: Enabled (transformer embeddings)
- **Max Concurrent Agents**: 8
- **Background Workers**: pattern-consolidator, routing-accuracy-monitor, coverage-gap-scanner

### V3 QE Agents

V3 QE agents are installed in `.claude/agents/v3/`. Use with Claude Code's Task tool:

```
# Example: Generate tests
Task("Generate unit tests", "v3-qe-test-generator")

# Example: Analyze coverage
Task("Find coverage gaps", "v3-qe-coverage-specialist")
```

### Data Storage

- **Memory Backend**: `.agentic-qe/data/memory.db` (SQLite)
- **Pattern Storage**: `.agentic-qe/data/qe-patterns.db` (ReasoningBank)
- **HNSW Index**: `.agentic-qe/data/hnsw/index.bin`
- **Configuration**: `.agentic-qe/config.yaml`

### Best Practices

1. **Test Execution**: Always use `npm test -- --run` (not `npm test` which runs in watch mode)
2. **Coverage Targets**: Aim for 80%+ coverage on critical paths
3. **Quality Gates**: Run `quality_assess` before merging PRs
4. **Pattern Learning**: AQE learns from successful test patterns - consistent naming helps

### Troubleshooting

If MCP tools aren't working:

```bash
# Verify MCP server is installed globally
npm install -g @agentic-qe/v3
aqe-v3-mcp --help

# Check configuration
cat .claude/mcp.json

# Reinitialize if needed
aqe-v3 init --auto
```

---

_Generated by AQE v3 init - 2026-01-17T04:48:43.670Z_
