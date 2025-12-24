# CI/CD Testing Guide

This document covers continuous integration testing configuration, pipeline stages, and optimizing test runs in CI environments.

## Table of Contents

- [Overview](#overview)
- [GitHub Actions Workflow](#github-actions-workflow)
- [Pipeline Stages](#pipeline-stages)
- [cargo-nextest Profiles](#cargo-nextest-profiles)
- [Environment Variables](#environment-variables)
- [Test Artifacts](#test-artifacts)
- [Local CI Emulation](#local-ci-emulation)
- [Troubleshooting](#troubleshooting)
- [Performance Optimization](#performance-optimization)

## Overview

Tests run automatically on:

- Every pull request to `main` or `develop`
- Every push to `main` or `develop`

**Workflow file:** `.github/workflows/ci.yml`

### CI Strategy

1. **Parallel execution**: Backend and frontend tests run concurrently
2. **Dual database testing**: SQLite for fast unit tests, PostgreSQL for integration tests
3. **Smart caching**: sccache and dependency caching reduce build time by 30-50%
4. **Coverage on PRs**: Coverage reports generated only for pull requests

## GitHub Actions Workflow

### Trigger Events

```yaml
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
```

### Job Dependencies

```
┌─────────────────┐     ┌─────────────────────┐
│  backend-lint   │     │   frontend-lint     │
└────────┬────────┘     └──────────┬──────────┘
         │                         │
         ▼                         ▼
┌─────────────────┐     ┌─────────────────────┐
│backend-unit-test│     │   frontend-test     │
└────────┬────────┘     └──────────┬──────────┘
         │                         │
         ▼                         │
┌─────────────────────────┐        │
│backend-integration-test │        │
└────────┬────────────────┘        │
         │                         │
         ▼                         ▼
┌─────────────────────────────────────────────┐
│              backend-build                   │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
              ┌─────────────────┐
              │  docker-build   │
              └─────────────────┘
```

## Pipeline Stages

### 1. Backend Validation (Parallel)

```yaml
backend-lint:
  - cargo fmt --check
  - cargo clippy --all-targets --all-features -- -D warnings

backend-security:
  - cargo audit
  - cargo deny check
```

### 2. Backend Tests (Parallel, after lint)

#### Unit Tests (SQLite)

```yaml
backend-unit-test:
  database: sqlite::memory:
  runner: cargo nextest
  threads: 4
  retries: 2
  timeout: 10 minutes
```

Fast feedback with in-memory SQLite. Tests that require PostgreSQL features are automatically skipped.

#### Integration Tests (PostgreSQL)

```yaml
backend-integration-test:
  service: postgres:16-alpine
  database: postgres://ampel:ampel@localhost:5432/ampel_test
  runner: cargo nextest
  threads: 2
  retries: 3
  timeout: 15 minutes
```

Comprehensive testing with real PostgreSQL. Each test gets an isolated database instance.

### 3. Frontend Tests (Parallel, after lint)

```yaml
frontend-lint:
  - pnpm run lint
  - pnpm run type-check

frontend-test:
  runner: vitest
  coverage: true
  artifacts: coverage report (7 days)
```

### 4. Build (After tests pass)

```yaml
backend-build:
  - cargo build --release

frontend-build:
  - pnpm run build

docker-build:
  - docker build (push events only)
```

### 5. Coverage (PRs only)

```yaml
backend-coverage:
  tool: cargo-llvm-cov
  database: PostgreSQL (required)
  upload: Codecov
```

## cargo-nextest Profiles

Configuration in `.config/nextest.toml`:

### fast (Local Development)

```toml
[profile.fast]
test-threads = 8
retries = 0
fail-fast = true
slow-timeout = { period = "30s", terminate-after = 2 }
```

- **When**: Quick feedback during coding
- **Speed**: Fastest
- **Command**: `cargo nextest run --profile fast`

### default (Standard Testing)

```toml
[profile.default]
test-threads = 4
retries = 2
fail-fast = false
slow-timeout = { period = "60s", terminate-after = 3 }
```

- **When**: Pre-commit checks
- **Speed**: Moderate
- **Command**: `cargo nextest run`

### ci (GitHub Actions)

```toml
[profile.ci]
test-threads = 4
retries = 3
fail-fast = false
slow-timeout = { period = "120s", terminate-after = 3 }
```

- **When**: Automatically in CI
- **Speed**: Optimized for reliability
- **Command**: `cargo nextest run --profile ci`

### coverage (Test Coverage)

```toml
[profile.coverage]
test-threads = 1
retries = 0
```

- **When**: Generating coverage reports
- **Speed**: Slower (single-threaded for accurate coverage)
- **Command**: `cargo nextest run --profile coverage`

## Environment Variables

### CI Environment

```yaml
# Rust build
CARGO_TERM_COLOR: always
RUST_BACKTRACE: 1
RUSTFLAGS: '-D warnings'
RUST_VERSION: '1.91.1'

# Test configuration
DATABASE_URL: postgres://ampel:ampel@localhost:5432/ampel_test
TEST_DATABASE_TYPE: postgres
JWT_SECRET: test-jwt-secret-for-ci-minimum-32-chars
ENCRYPTION_KEY: dGVzdC1lbmNyeXB0aW9uLWtleS0zMi1ieXRlcw==
RUST_LOG: info
CI: true

# Performance
SCCACHE_GHA_ENABLED: 'true'
RUSTC_WRAPPER: 'sccache'
```

### Required for All Tests

```bash
JWT_SECRET=test-jwt-secret-for-ci-minimum-32-chars
ENCRYPTION_KEY=dGVzdC1lbmNyeXB0aW9uLWtleS0zMi1ieXRlcw==
```

### Database Selection

```bash
# SQLite (fast unit tests)
DATABASE_URL=sqlite::memory:
TEST_DATABASE_TYPE=sqlite

# PostgreSQL (integration tests)
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test
TEST_DATABASE_TYPE=postgres
```

## Test Artifacts

Uploaded on test completion with 7-day retention:

| Artifact                           | Contents                  |
| ---------------------------------- | ------------------------- |
| `backend-unit-test-results`        | nextest JUnit output      |
| `backend-integration-test-results` | nextest JUnit output      |
| `frontend-coverage`                | Vitest coverage report    |
| `backend-coverage-report`          | Tarpaulin coverage report |

### Downloading Artifacts

```bash
# Using GitHub CLI
gh run download <run-id> -n backend-unit-test-results
```

## Local CI Emulation

### Run Same Tests as CI

```bash
# Backend unit tests (SQLite)
DATABASE_URL=sqlite::memory: cargo nextest run --profile ci

# Backend integration tests (PostgreSQL)
docker compose up -d postgres
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test \
  cargo nextest run --profile ci

# Frontend tests
cd frontend && pnpm test -- --run --coverage
```

### Full CI Check

```bash
# All checks in one command
make ci
```

### Before Committing Checklist

```bash
# 1. Run fast unit tests
cargo nextest run --profile fast

# 2. Run linter
make lint-backend

# 3. Check formatting
make format-check-backend
```

### Before Submitting PR Checklist

```bash
# 1. Start PostgreSQL
docker compose up -d postgres

# 2. Run full test suite
make test-backend

# 3. Generate coverage
cargo llvm-cov --all-features --workspace --html --output-dir coverage

# 4. Run all CI checks
make ci
```

## Troubleshooting

### Tests Pass Locally but Fail in CI

**Likely causes:**

1. Using different databases (SQLite vs PostgreSQL)
2. Different environment variables
3. Timing issues / race conditions

**Solutions:**

```bash
# Test with same environment as CI
DATABASE_URL=sqlite::memory: cargo nextest run --profile ci

# Then test with PostgreSQL
DATABASE_URL=postgres://... cargo nextest run --profile ci
```

### Slow Test Performance

**Check which tests are slow:**

```bash
cargo nextest run --profile ci
# Look for "SLOW" markers in output
```

**Speed up tests:**

```bash
# Use more threads
cargo nextest run --test-threads 8

# Use fast profile
cargo nextest run --profile fast
```

### Database Connection Errors

**PostgreSQL not running:**

```bash
# Start PostgreSQL
docker compose up -d postgres

# Wait for it to be ready
until pg_isready -h localhost -p 5432 -U ampel; do sleep 1; done
```

**SQLite permission errors:**

```bash
# Use in-memory SQLite
DATABASE_URL=sqlite::memory: cargo nextest run
```

### Flaky Tests

**Run with retries:**

```bash
# 2 retries (default profile)
cargo nextest run

# 3 retries (CI profile)
cargo nextest run --profile ci

# Custom retries
cargo nextest run --retries 5
```

### Viewing CI Results

```bash
# View recent workflow runs
gh run list

# Watch current workflow
gh run watch

# View specific run
gh run view <run-id>

# View CI status for current branch
gh pr checks
```

## Performance Optimization

### Build Caching

CI uses sccache for faster builds:

```yaml
SCCACHE_GHA_ENABLED: 'true'
RUSTC_WRAPPER: 'sccache'
```

This reduces rebuild time by 30-50%.

### Dependency Caching

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: 'ci'
```

### Parallel Test Execution

- Backend unit and integration tests run in parallel
- Frontend tests run in parallel with backend
- Up to 4 test threads per job

### Smart Test Selection (Future)

Consider implementing:

- Run only tests affected by changed files
- Skip unchanged crates
- Incremental test runs

## Common Workflows

### Quick Validation During Development

```bash
cargo nextest run --profile fast
```

Speed: ~2 seconds for 100 tests

### Pre-commit Validation

```bash
make ci-backend
```

Runs lint, format check, and tests

### Full Validation Before PR

```bash
docker compose up -d postgres
make test-backend
cargo tarpaulin --all-features --workspace
```

Speed: ~30-60 seconds

### CI Emulation

```bash
cargo nextest run --profile ci
```

Same configuration as GitHub Actions

## Summary

| Use Case        | Command                                     | Speed  |
| --------------- | ------------------------------------------- | ------ |
| Quick check     | `cargo nextest run --profile fast`          | 2s     |
| Pre-commit      | `make test-backend`                         | 5s     |
| Full validation | `docker compose up -d && cargo nextest run` | 30-60s |
| Coverage        | `cargo llvm-cov --all-features --workspace` | 30-60s |
| CI emulation    | `cargo nextest run --profile ci`            | 30-60s |

## References

- [cargo-nextest Documentation](https://nexte.st/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [sccache](https://github.com/mozilla/sccache)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [Codecov](https://codecov.io/)
