# CI Testing Quick Start Guide

Quick reference for running tests locally and understanding the CI workflow.

## TL;DR

```bash
# Fast unit tests (SQLite)
cargo nextest run --profile fast

# Full tests with PostgreSQL
docker compose up -d postgres
cargo nextest run

# Coverage report
cargo tarpaulin --all-features --workspace
```

## Prerequisites

```bash
# Install cargo-nextest (required)
cargo install cargo-nextest --locked

# Install cargo-tarpaulin (optional, for coverage)
cargo install cargo-tarpaulin --locked
```

## Running Tests

### 1. Unit Tests (SQLite - Fast)

**Recommended for**: Quick validation during development

```bash
# Using make (uses standard cargo test)
make test-backend

# Using cargo-nextest (faster, parallel)
DATABASE_URL=sqlite::memory: cargo nextest run --profile fast

# Or use the default profile
cargo nextest run
```

**Speed**: ~2 seconds for 100 tests

### 2. Integration Tests (PostgreSQL)

**Recommended for**: Pre-commit validation, comprehensive testing

```bash
# Start PostgreSQL
docker compose up -d postgres

# Run tests
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test \
  cargo nextest run --profile default

# Or use make (handles DATABASE_URL automatically if .env is set)
make test-backend
```

**Speed**: ~30-60 seconds for 100 tests

### 3. Coverage Report

**Recommended for**: Before submitting PRs

```bash
# Start PostgreSQL
docker compose up -d postgres

# Generate coverage (HTML + Codecov XML)
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test \
  cargo tarpaulin --all-features --workspace --out Html --output-dir coverage

# Open HTML report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

## cargo-nextest Profiles

### fast (Local Development)

- **When**: Quick feedback during coding
- **Speed**: Fastest
- **Threads**: 8
- **Retries**: 0 (fail immediately)
- **Command**: `cargo nextest run --profile fast`

### default (Standard Testing)

- **When**: Pre-commit checks
- **Speed**: Moderate
- **Threads**: 4
- **Retries**: 2
- **Command**: `cargo nextest run`

### ci (GitHub Actions)

- **When**: Automatically in CI
- **Speed**: Optimized for reliability
- **Threads**: 4
- **Retries**: 3
- **Command**: `cargo nextest run --profile ci`

### coverage (Test Coverage)

- **When**: Generating coverage reports
- **Speed**: Slower (single-threaded)
- **Threads**: 1
- **Command**: `cargo nextest run --profile coverage`

## Environment Variables

### Required for All Tests

```bash
# Set in .env or export
JWT_SECRET=test-jwt-secret-for-ci-minimum-32-chars
ENCRYPTION_KEY=dGVzdC1lbmNyeXB0aW9uLWtleS0zMi1ieXRlcw==
```

### Optional but Recommended

```bash
# Use SQLite for faster tests
DATABASE_URL=sqlite::memory:
TEST_DATABASE_TYPE=sqlite

# Or use PostgreSQL for full testing
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test
TEST_DATABASE_TYPE=postgres

# Reduce log noise
RUST_LOG=warn

# Enable colored output
CARGO_TERM_COLOR=always
```

## Common Workflows

### Before Committing

```bash
# 1. Run fast unit tests
cargo nextest run --profile fast

# 2. Run linter
make lint-backend

# 3. Check formatting
make format-check-backend

# All in one (using make)
make ci-backend
```

### Before Submitting PR

```bash
# 1. Start PostgreSQL
docker compose up -d postgres

# 2. Run full test suite
make test-backend

# 3. Generate coverage
cargo tarpaulin --all-features --workspace

# 4. Run all CI checks
make ci
```

### Debugging Failed Tests

```bash
# Run specific test with output
cargo nextest run --profile fast test_name -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo nextest run --profile fast test_name

# Run single test file
cargo nextest run --profile fast --test provider_account_queries_test
```

## CI Workflow Jobs

### What Runs When

**On every push/PR**:

1. backend-lint (linting + formatting)
2. backend-security (audit)
3. backend-unit-test (SQLite)
4. backend-integration-test (PostgreSQL)
5. backend-build (release)
6. frontend-lint
7. frontend-test
8. docker-build (push only)

**On PRs only**:

- backend-coverage (test coverage report)

### Viewing CI Results

```bash
# View recent workflow runs
make gh-runs

# Watch current workflow
make gh-watch

# View CI status
make gh-status
```

## Troubleshooting

### Tests Pass Locally but Fail in CI

**Likely causes**:

1. Using different databases (SQLite vs PostgreSQL)
2. Different environment variables
3. Timing issues

**Solutions**:

```bash
# Test with same environment as CI
DATABASE_URL=sqlite::memory: cargo nextest run --profile ci

# Then test with PostgreSQL
DATABASE_URL=postgres://... cargo nextest run --profile ci
```

### Slow Test Performance

**Check which tests are slow**:

```bash
cargo nextest run --profile ci
# Look for "SLOW" markers in output
```

**Speed up tests**:

```bash
# Use more threads
cargo nextest run --test-threads 8

# Use fast profile
cargo nextest run --profile fast
```

### Database Connection Errors

**PostgreSQL not running**:

```bash
# Start PostgreSQL
docker compose up -d postgres

# Wait for it to be ready
until pg_isready -h localhost -p 5432 -U ampel; do sleep 1; done
```

**SQLite permission errors**:

```bash
# Use in-memory SQLite
DATABASE_URL=sqlite::memory: cargo nextest run
```

### Flaky Tests

**Run with retries**:

```bash
# 2 retries (default profile)
cargo nextest run

# 3 retries (CI profile)
cargo nextest run --profile ci

# Custom retries
cargo nextest run --retries 5
```

## Performance Tips

### Speed Up Test Runs

1. **Use SQLite for unit tests**:

   ```bash
   DATABASE_URL=sqlite::memory: cargo nextest run --profile fast
   ```

2. **Increase parallel threads**:

   ```bash
   cargo nextest run --test-threads 8
   ```

3. **Cache builds**:

   ```bash
   # sccache (if installed)
   export RUSTC_WRAPPER=sccache
   ```

4. **Use fast profile**:
   ```bash
   cargo nextest run --profile fast
   ```

### Optimize CI Time

1. Tests run in parallel (SQLite unit + PostgreSQL integration)
2. Smart caching reduces build time by 30-50%
3. Coverage only runs on PRs
4. Artifacts saved for 7 days

## Useful Commands

```bash
# List all tests
cargo nextest list

# Run specific test
cargo nextest run test_find_by_user

# Run tests matching pattern
cargo nextest run 'test_*_account*'

# Show test times
cargo nextest run --profile ci --verbose

# Generate JUnit report
cargo nextest run --profile ci
# Report saved to: target/nextest/ci/junit.xml

# Clean test artifacts
cargo clean
rm -rf target/nextest/
```

## Quick Links

- **Full Guide**: [docs/testing/ci-workflow-guide.md](./ci-workflow-guide.md)
- **Implementation Summary**: [docs/testing/ci-workflow-implementation-summary.md](./ci-workflow-implementation-summary.md)
- **Research**: [docs/research/sqlite-ci-testing-best-practices-2025.md](../research/sqlite-ci-testing-best-practices-2025.md)
- **Nextest Config**: [.config/nextest.toml](../../.config/nextest.toml)
- **CI Workflow**: [.github/workflows/ci.yml](../../.github/workflows/ci.yml)

## Summary

| Use Case        | Command                                      | Speed  |
| --------------- | -------------------------------------------- | ------ |
| Quick check     | `cargo nextest run --profile fast`           | 2s     |
| Pre-commit      | `make test-backend`                          | 5s     |
| Full validation | `docker compose up -d && cargo nextest run`  | 30-60s |
| Coverage        | `cargo tarpaulin --all-features --workspace` | 2-3min |
| CI emulation    | `cargo nextest run --profile ci`             | 30-60s |

**Default recommendation**: Use `cargo nextest run --profile fast` for development, run full `make ci-backend` before committing.
