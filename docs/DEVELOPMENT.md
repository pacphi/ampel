# Development Guide

This guide covers setting up and running Ampel for local development.

## Prerequisites

### Required

- **Rust** 1.91+ - Install via [rustup](https://rustup.rs/)
- **Node.js** 20+ - Install via [nvm](https://github.com/nvm-sh/nvm) or [official installer](https://nodejs.org/)
- **pnpm** 10+ - Install via `corepack enable && corepack prepare pnpm@latest --activate`
- **PostgreSQL** 16+ - Via Docker or local installation

### Optional

- **Docker** - For containerized development
- **Redis** - For background job queues (optional for basic development)
- **Make** - For unified command interface

## Initial Setup

### 1. Clone and Configure

```bash
git clone https://github.com/pacphi/ampel.git
cd ampel

# Copy environment template
cp .env.example .env
```

### 2. Configure Environment

Edit `.env` with your settings:

```bash
# Required
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel
JWT_SECRET=your-secret-key-minimum-32-characters-long

# Generate encryption key
ENCRYPTION_KEY=$(openssl rand -base64 32)

# Optional: OAuth providers (add your credentials)
GITHUB_CLIENT_ID=your-github-client-id
GITHUB_CLIENT_SECRET=your-github-client-secret
```

### 3. Install Dependencies

```bash
# Using Make (recommended)
make install

# Or manually:
cd frontend && pnpm install
```

### 4. Start PostgreSQL

**Option A: Docker (Recommended)**

```bash
docker run -d --name ampel-postgres \
  -e POSTGRES_USER=ampel \
  -e POSTGRES_PASSWORD=ampel \
  -e POSTGRES_DB=ampel \
  -p 5432:5432 \
  postgres:16-alpine
```

**Option B: Local PostgreSQL**

```bash
createuser -s ampel
createdb -O ampel ampel
```

## Using Make Commands

Ampel provides a unified Makefile for all common operations:

```bash
make help              # Show all available commands

# Build
make build             # Build everything (debug)
make build-release     # Build everything (release)
make clean             # Clean all artifacts

# Development
make dev-api           # Start API server
make dev-worker        # Start background worker
make dev-frontend      # Start frontend dev server

# Testing
make test              # Run all tests
make test-backend      # Run backend tests only
make test-frontend     # Run frontend tests only

# Code Quality
make lint              # Run all linters
make format            # Format all code
make format-check      # Check formatting

# Docker
make docker-build      # Build Docker images
make docker-up         # Start Docker services
make docker-down       # Stop Docker services
```

## Backend Development

### Building

```bash
# Using Make
make build-backend

# Or directly
cargo build

# Release build
cargo build --release
```

### Running the API Server

```bash
# Using Make
make dev-api

# With hot reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x 'run --bin ampel-api'

# Or run directly
cargo run --bin ampel-api
```

The API server starts at `http://localhost:8080`.

### Running the Background Worker

```bash
# Using Make
make dev-worker

# Or directly
cargo run --bin ampel-worker
```

### Running Tests

```bash
# Using Make
make test-backend

# Or directly
cargo test --all-features

# Specific crate
cargo test -p ampel-core

# With output
cargo test -- --nocapture
```

### Code Quality

```bash
# Using Make
make lint-backend    # Run clippy
make format-backend  # Format with rustfmt

# Or directly
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
make audit-backend
# or: cargo audit
```

### Database Migrations

Migrations run automatically on API startup. For manual control:

```bash
# Using sea-orm-cli
cargo install sea-orm-cli

# Generate migration
sea-orm-cli migrate generate create_table_name

# Run migrations
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down
```

## Frontend Development

### Setup

```bash
cd frontend
pnpm install
```

### Development Server

```bash
# Using Make (from root)
make dev-frontend

# Or directly
cd frontend && pnpm run dev
```

The frontend starts at `http://localhost:5173` with hot reload.

### Available Scripts

```bash
# Development server
pnpm run dev

# Type checking
pnpm run type-check

# Linting
pnpm run lint

# Run tests
pnpm test

# Run tests in watch mode
pnpm test -- --watch

# Production build
pnpm run build

# Preview production build
pnpm run preview
```

### Adding UI Components

We use [shadcn/ui](https://ui.shadcn.com/). To add components:

```bash
pnpm dlx shadcn-ui@latest add button
pnpm dlx shadcn-ui@latest add card
```

## Building Container Images

### Using Make

```bash
make docker-build  # Build all images
```

### Manually

```bash
# API Image
docker build -t ampel-api:dev -f docker/Dockerfile.api .

# Worker Image
docker build -t ampel-worker:dev -f docker/Dockerfile.worker .

# Frontend Image
docker build -t ampel-frontend:dev \
  --build-arg VITE_API_URL=http://localhost:8080/api \
  -f docker/Dockerfile.frontend .
```

### All Images (Docker Compose)

```bash
cd docker
docker compose build
```

## Project Architecture

### Backend Crates

| Crate             | Purpose                                                |
| ----------------- | ------------------------------------------------------ |
| `ampel-api`       | REST API server (Axum), HTTP handlers, middleware      |
| `ampel-core`      | Business logic, domain models, services                |
| `ampel-db`        | Database layer (SeaORM), entities, migrations, queries |
| `ampel-providers` | Git provider integrations (GitHub, GitLab, Bitbucket)  |
| `ampel-worker`    | Background job processing (Apalis)                     |

### Frontend Structure

```text
frontend/src/
├── api/           # API client functions
├── components/    # React components
│   ├── ui/        # shadcn/ui components
│   ├── layout/    # Layout components
│   └── dashboard/ # Dashboard-specific components
├── hooks/         # Custom React hooks
├── lib/           # Utilities
├── pages/         # Page components
└── types/         # TypeScript types
```

### Makefile Structure

```text
ampel/
├── Makefile              # Root Makefile (delegates to others)
├── crates/Makefile       # Backend-specific targets
└── frontend/Makefile     # Frontend-specific targets
```

## IDE Setup

### VS Code

Recommended extensions:

- rust-analyzer
- ESLint
- Tailwind CSS IntelliSense
- Prettier

Settings (`.vscode/settings.json`):

```json
{
  "editor.formatOnSave": true,
  "rust-analyzer.checkOnSave.command": "clippy",
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[typescriptreact]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

### JetBrains IDEs

- Install Rust plugin
- Enable ESLint integration
- Configure Prettier for TypeScript

## Common Issues

### Database Connection Errors

```bash
# Check PostgreSQL is running
pg_isready -h localhost -p 5432

# Check connection string
psql $DATABASE_URL -c "SELECT 1"
```

### Port Already in Use

```bash
# Find and kill process on port 8080
lsof -i :8080
kill -9 <PID>
```

### Rust Build Errors

```bash
# Clean and rebuild
make clean
make build

# Or directly
cargo clean
cargo build
```

### Frontend Dependency Issues

```bash
cd frontend
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

## Environment Variables Reference

| Variable                  | Required | Default   | Description                             |
| ------------------------- | -------- | --------- | --------------------------------------- |
| `DATABASE_URL`            | Yes      | -         | PostgreSQL connection string            |
| `JWT_SECRET`              | Yes      | -         | JWT signing secret (min 32 chars)       |
| `ENCRYPTION_KEY`          | Yes      | -         | Base64 32-byte key for token encryption |
| `HOST`                    | No       | `0.0.0.0` | API server bind address                 |
| `PORT`                    | No       | `8080`    | API server port                         |
| `RUST_LOG`                | No       | `info`    | Log level                               |
| `CORS_ORIGINS`            | No       | -         | Comma-separated allowed origins         |
| `GITHUB_CLIENT_ID`        | No       | -         | GitHub OAuth app client ID              |
| `GITHUB_CLIENT_SECRET`    | No       | -         | GitHub OAuth app secret                 |
| `GITLAB_CLIENT_ID`        | No       | -         | GitLab OAuth app client ID              |
| `GITLAB_CLIENT_SECRET`    | No       | -         | GitLab OAuth app secret                 |
| `BITBUCKET_CLIENT_ID`     | No       | -         | Bitbucket OAuth app key                 |
| `BITBUCKET_CLIENT_SECRET` | No       | -         | Bitbucket OAuth app secret              |
