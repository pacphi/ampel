# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ampel is a unified PR management dashboard that consolidates pull requests from GitHub, GitLab,
and Bitbucket into a single interface using a traffic light system (green=ready to merge,
yellow=in progress, red=blocked).

## Tech Stack

- **Backend**: Rust 1.95+ (Axum 0.8, SeaORM, Apalis for background jobs, Tokio async runtime)
- **Frontend**: React 19 + TypeScript, Vite, TanStack Query, shadcn/ui, Tailwind CSS
- **Database**: PostgreSQL 16, Redis 7 for caching
- **Package Manager**: pnpm 11.1.3
- **Rust Version**: 1.95.0 (pinned in CI and Docker images)

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

## Agentic QE v3 (project config)

This repo is initialized with **Agentic QE v3**. Generic AQE/ruflo operating guidance
(MCP tool usage, QE agent routing, critical policies like `npm test -- --run`) lives in
`~/.claude/CLAUDE.md` — only project-specific config is recorded here.

- **Enabled domains**: test-generation, test-execution, coverage-analysis,
  learning-optimization, quality-assessment, security-compliance (+1 more)
- **Max concurrent agents**: 8
- **Background workers**: pattern-consolidator, routing-accuracy-monitor, coverage-gap-scanner
- **MCP server**: configured in `.claude/mcp.json`; V3 QE agents in `.claude/agents/v3/`
- **Local data**: memory `.agentic-qe/data/memory.db`, patterns `.agentic-qe/data/qe-patterns.db`,
  HNSW index `.agentic-qe/data/hnsw/index.bin`, config `.agentic-qe/config.yaml`

_Generated by AQE v3 init - 2026-01-17_
