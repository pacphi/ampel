# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ampel is a unified PR management dashboard that consolidates pull requests from GitHub, GitLab,
and Bitbucket into a single interface using a traffic light system (green=ready to merge,
yellow=in progress, red=blocked).

## Tech Stack

- **Backend**: Rust 1.91+ (Axum, SeaORM, Apalis for background jobs, Tokio async runtime)
- **Frontend**: React 19 + TypeScript, Vite, TanStack Query, shadcn/ui, Tailwind CSS
- **Database**: PostgreSQL 16, Redis 7 for caching
- **Package Manager**: pnpm 10.24.0
- **Rust Version**: 1.91.1 (pinned in CI and Docker images)

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
make test               # Run all tests
make test-backend       # cargo test --all-features
make test-frontend      # vitest --run

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

### Key Patterns

- **State Management**: TanStack Query for server state caching
- **Forms**: React Hook Form with Zod validation
- **Auth**: JWT (15-min access tokens, 7-day refresh in httpOnly cookies)
- **Token Storage**: Provider OAuth tokens encrypted with AES-256-GCM
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

## Environment Setup

```bash
cp .env.example .env    # Configure environment variables
make install            # Install all dependencies
```
