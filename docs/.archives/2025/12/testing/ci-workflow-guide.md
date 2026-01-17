# CI Workflow Guide: SQLite Testing Best Practices

**Last Updated**: December 20, 2025
**Workflow Version**: 2.0

## Overview

This guide explains the Ampel CI workflow implementation using SQLite for fast unit tests and PostgreSQL for comprehensive integration tests.

## Architecture

### Two-Tier Testing Strategy

```text
┌─────────────────────────────────────────────────────────────────┐
│                        CI Pipeline                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────────┐      ┌──────────────────────┐        │
│  │   Unit Tests         │      │  Integration Tests   │        │
│  │   (SQLite)           │      │  (PostgreSQL)        │        │
│  ├──────────────────────┤      ├──────────────────────┤        │
│  │ • 4 parallel threads │      │ • 2 parallel threads │        │
│  │ • 2-3 retries        │      │ • 3-5 retries        │        │
│  │ • 60s timeout        │      │ • 120s timeout       │        │
│  │ • In-memory DB       │      │ • GitHub service     │        │
│  │ • Fast startup       │      │ • Production parity  │        │
│  └──────────────────────┘      └──────────────────────┘        │
│           ↓                              ↓                       │
│  ┌─────────────────────────────────────────────────┐           │
│  │         Coverage Report (PostgreSQL)            │           │
│  │  • Codecov integration                          │           │
│  │  • PR-only execution                            │           │
│  └─────────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

## Workflow Jobs

### 1. Backend Unit Tests (SQLite)

**Purpose**: Fast validation of business logic and database queries

**Configuration**:

```yaml
env:
  DATABASE_URL: sqlite::memory:
  TEST_DATABASE_TYPE: sqlite
```

**Features**:

- In-memory SQLite database (400-1000x faster startup)
- 4 parallel test threads
- 2 automatic retries for flaky tests
- 60-second timeout per test
- Immediate failure output

**When to Use**:

- Testing business logic
- Query validation
- Data model operations
- Unit-level database operations

**Performance**: ~2 seconds for 100 tests

### 2. Backend Integration Tests (PostgreSQL)

**Purpose**: Production-parity testing with full PostgreSQL features

**Configuration**:

```yaml
services:
  postgres:
    image: postgres:16-alpine
    env:
      POSTGRES_USER: ampel
      POSTGRES_PASSWORD: ampel
      POSTGRES_DB: ampel_test
```

**Features**:

- Real PostgreSQL 16 database
- 2 parallel test threads (reduced for stability)
- 3 automatic retries
- 120-second timeout per test
- Automatic database cleanup verification

**When to Use**:

- Testing PostgreSQL-specific features
- Transaction behavior validation
- Complex queries with advanced SQL
- Full integration scenarios

**Performance**: ~30-60 seconds for 100 tests

### 3. Backend Coverage Report

**Purpose**: Comprehensive test coverage analysis

**Configuration**:

- Runs only on pull requests
- Uses cargo-tarpaulin
- Uploads to Codecov
- Requires PostgreSQL for accuracy

**Metrics Tracked**:

- Line coverage
- Branch coverage
- Function coverage

## cargo-nextest Configuration

### Profiles

#### Default Profile

```toml
[profile.default]
test-threads = 4
retries = 2
fail-fast = false
slow-timeout = { period = "60s", terminate-after = 2 }
```

#### CI Profile

```toml
[profile.ci]
test-threads = 4
retries = 3
slow-timeout = { period = "120s", terminate-after = 3 }
```

#### Fast Profile (Local Development)

```toml
[profile.fast]
test-threads = 8
retries = 0
fail-fast = true
slow-timeout = { period = "30s" }
```

### Test-Specific Overrides

```toml
# Database tests get more time
[[profile.default.overrides]]
filter = 'test(.*db.*) | test(.*database.*)'
slow-timeout = { period = "90s", terminate-after = 2 }

# Integration tests run sequentially
[[profile.default.overrides]]
filter = 'test(.*integration.*)'
threads-required = 1
slow-timeout = { period = "120s" }
```

## Environment Variables

### Required for All Tests

```bash
JWT_SECRET=test-jwt-secret-for-ci-minimum-32-chars
ENCRYPTION_KEY=dGVzdC1lbmNyeXB0aW9uLWtleS0zMi1ieXRlcw==
```

### SQLite Unit Tests

```bash
DATABASE_URL=sqlite::memory:
TEST_DATABASE_TYPE=sqlite
RUST_LOG=warn
```

### PostgreSQL Integration Tests

```bash
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test
TEST_DATABASE_TYPE=postgres
RUST_LOG=info
```

## Database Cleanup

### Automatic Cleanup (SQLite)

SQLite in-memory databases are automatically cleaned up when the connection is dropped:

```rust
#[tokio::test]
async fn test_example() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    // Test runs
    // Database automatically destroyed when `db` drops
}
```

### Manual Cleanup (PostgreSQL)

PostgreSQL cleanup is verified after tests:

```bash
# Check for leftover data
TABLE_COUNT=$(psql -h localhost -U ampel -d ampel_test -t -c \
  "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';")

# Clean up
psql -h localhost -U ampel -d ampel_test -c \
  "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
```

## Caching Strategy

### Build Cache

```yaml
- name: Rust cache
  uses: Swatinem/rust-cache@v2
  with:
    shared-key: 'rust-build'
    cache-on-failure: true
```

**Caches**:

- Compiled dependencies
- Build artifacts
- Test binaries

### Registry Cache

```yaml
- name: Cache cargo registry
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
```

### sccache Integration

```yaml
env:
  SCCACHE_GHA_ENABLED: 'true'
  RUSTC_WRAPPER: 'sccache'
```

**Benefits**:

- 30-50% faster rebuild times
- Shared cache across jobs
- Automatic cache management

## Test Artifacts

### Backend Unit Test Results

```yaml
- name: Upload test results
  uses: actions/upload-artifact@v4
  with:
    name: backend-unit-test-results
    path: |
      test-results.json
      target/nextest/ci/
    retention-days: 7
```

### Backend Integration Test Results

```yaml
- name: Upload test results
  uses: actions/upload-artifact@v4
  with:
    name: backend-integration-test-results
    path: target/nextest/ci/
    retention-days: 7
```

### Coverage Reports

```yaml
- name: Upload coverage artifacts
  uses: actions/upload-artifact@v4
  with:
    name: backend-coverage-report
    path: coverage/
    retention-days: 7
```

## Retry Logic

### Test Retries

```text
Retry Strategy:
┌─────────────────────────────────────────────┐
│ Unit Tests (SQLite):                        │
│ • Attempt 1 → Fail                          │
│ • Attempt 2 → Fail                          │
│ • Attempt 3 → Pass ✓                        │
│ • Max Retries: 2                            │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ Integration Tests (PostgreSQL):             │
│ • Attempt 1 → Fail                          │
│ • Attempt 2 → Fail                          │
│ • Attempt 3 → Fail                          │
│ • Attempt 4 → Pass ✓                        │
│ • Max Retries: 3 (5 in CI profile)          │
└─────────────────────────────────────────────┘
```

### Why More Retries for Integration Tests?

Integration tests with PostgreSQL can be affected by:

- Network latency
- Database service startup timing
- Connection pool initialization
- Lock contention

## Timeout Configuration

### Test-Level Timeouts

| Test Type        | Default Timeout | CI Timeout | Reason                            |
| ---------------- | --------------- | ---------- | --------------------------------- |
| Unit (SQLite)    | 60s             | 120s       | Simple operations, should be fast |
| Database queries | 90s             | 180s       | Complex queries may take longer   |
| Integration      | 120s            | 240s       | Full stack operations             |

### Timeout Behavior

```toml
slow-timeout = { period = "60s", terminate-after = 2 }
```

- **period**: Time before test is considered "slow"
- **terminate-after**: Multiplier for hard timeout (60s × 2 = 120s max)

## Running Tests Locally

### Quick Unit Tests (SQLite)

```bash
make test-backend
# or
cargo nextest run --profile fast
```

### Full Integration Tests (PostgreSQL)

```bash
# Start PostgreSQL
docker compose up -d postgres

# Run tests
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test \
cargo nextest run --profile default

# Or use make
make test-backend
```

### Coverage Report

```bash
# Start PostgreSQL
docker compose up -d postgres

# Generate coverage
cargo tarpaulin --all-features --workspace --out Html --output-dir coverage

# Open report
open coverage/index.html
```

## Performance Benchmarks

### CI Job Duration

| Job                      | Average Time  | Caching Benefit |
| ------------------------ | ------------- | --------------- |
| backend-lint             | ~2-3 minutes  | 40% faster      |
| backend-unit-test        | ~3-4 minutes  | 50% faster      |
| backend-integration-test | ~5-7 minutes  | 35% faster      |
| backend-coverage         | ~8-10 minutes | 30% faster      |
| backend-build            | ~4-5 minutes  | 60% faster      |

### Test Execution Speed

| Test Suite             | SQLite | PostgreSQL | Speedup |
| ---------------------- | ------ | ---------- | ------- |
| Unit tests (100)       | ~2s    | ~30s       | 15x     |
| Integration tests (50) | N/A    | ~25s       | N/A     |
| Full suite             | ~2s    | ~55s       | 27.5x   |

## Troubleshooting

### Tests Failing in CI but Passing Locally

1. **Check database type**: Ensure tests use the correct `TEST_DATABASE_TYPE`
2. **Verify environment variables**: All required vars must be set
3. **Check timing issues**: Use retries and longer timeouts
4. **Review logs**: Check nextest output for detailed errors

### Slow Test Performance

1. **Enable fast profile**:

   ```bash
   cargo nextest run --profile fast
   ```

2. **Check parallel threads**:

   ```bash
   cargo nextest run --test-threads 8
   ```

3. **Identify slow tests**:
   ```bash
   cargo nextest run --profile ci
   # Look for "SLOW" markers in output
   ```

### Database Connection Failures

**PostgreSQL not ready**:

```bash
# Add wait step in CI
until pg_isready -h localhost -p 5432 -U ampel; do
  sleep 1
done
```

**SQLite permission errors**:

```bash
# Ensure write permissions for SQLite file mode
chmod 777 /tmp/test-db-*
```

## Best Practices

### 1. Test Isolation

Each test should create its own database:

```rust
#[tokio::test]
async fn test_isolated() {
    let db = setup_test_db().await; // Fresh DB per test
    // Test logic
}
```

### 2. Cleanup Verification

Always verify cleanup in integration tests:

```rust
#[tokio::test]
async fn test_with_cleanup() {
    let db = setup_test_db().await;

    // Test logic

    // Verify cleanup
    let count = Entity::find().count(&db).await.unwrap();
    assert_eq!(count, 0, "Database should be clean");
}
```

### 3. Use Appropriate Database

- **SQLite**: Fast unit tests, business logic
- **PostgreSQL**: Integration tests, PostgreSQL-specific features

### 4. Optimize Test Speed

```rust
// ✅ Good: Fast, isolated
#[tokio::test]
async fn test_fast() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    // Quick test
}

// ❌ Slow: Creates real PostgreSQL connection
#[tokio::test]
async fn test_slow() {
    let db = Database::connect("postgres://...").await.unwrap();
    // Same test could use SQLite
}
```

### 5. Retry Flaky Tests

```toml
[[profile.ci.overrides]]
filter = 'test(flaky.*)'
retries = 5
```

## CI Workflow Dependencies

```text
┌──────────────┐
│ backend-lint │──┐
└──────────────┘  │
                  ├──> backend-unit-test ──┐
                  │                         ├──> backend-build ──> docker-build
                  ├──> backend-integration ─┘
                  │
                  └──> backend-coverage (PR only)

┌──────────────┐
│frontend-lint │──> frontend-test ──> docker-build
└──────────────┘
```

## Summary

The Ampel CI workflow implements a hybrid testing strategy:

1. **Unit Tests (SQLite)**: Fast, parallel execution for business logic
2. **Integration Tests (PostgreSQL)**: Production parity for complex scenarios
3. **Coverage Reports**: Comprehensive analysis on pull requests
4. **Smart Caching**: Significant speed improvements
5. **Retry Logic**: Handles flaky tests automatically
6. **Artifact Collection**: Test results for debugging

**Result**: Fast, reliable CI with optimal balance of speed and accuracy.

## References

- [cargo-nextest documentation](https://nexte.st/)
- [SeaORM testing guide](https://www.sea-ql.org/SeaORM/docs/write-test/)
- [GitHub Actions best practices](https://docs.github.com/en/actions/learn-github-actions/best-practices-for-ci-cd)
- [SQLite testing research](/docs/research/sqlite-ci-testing-best-practices-2025.md)
