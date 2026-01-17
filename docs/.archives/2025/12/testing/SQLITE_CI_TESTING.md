# SQLite CI Testing Strategy

## Overview

This document describes the testing strategy for using SQLite in CI/CD pipelines while maintaining PostgreSQL for production.

## Architecture

### Test Database Isolation

Each test runs with a completely isolated SQLite database:

- **Backend (Rust)**: Unique in-memory SQLite database per test
- **Frontend (TypeScript)**: Isolated test environment with mocked API calls
- **Integration Tests**: File-based SQLite for cross-component testing

### Directory Structure

```
crates/ampel-db/
├── tests/
│   ├── common/
│   │   ├── mod.rs           # Test utilities and helpers
│   │   └── fixtures.rs      # Test data fixtures
│   ├── integration/
│   │   ├── mod.rs           # Integration tests entry point
│   │   └── provider_account_queries.rs
│   └── ...
│
frontend/
├── tests/
│   ├── setup.ts             # Global test setup
│   └── fixtures/            # Test data fixtures
├── vitest.config.ts         # Vitest configuration
└── src/**/*.test.tsx        # Component tests
```

## Backend Testing (Rust + Cargo)

### Test Database Creation

```rust
use ampel_db::tests::common::TestDb;

#[tokio::test]
async fn test_example() {
    // Create isolated test database
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");

    let db = test_db.connection();

    // Run your test
    // ...

    // Cleanup (automatic via Drop, but can be explicit)
    test_db.cleanup().await;
}
```

### Key Features

1. **Automatic Isolation**: Each test gets a unique SQLite database
2. **Migration Support**: Runs SeaORM migrations automatically
3. **Parallel Execution**: Tests run in parallel without conflicts
4. **CI Detection**: Optimizes for CI environments
5. **Cleanup**: Automatic cleanup via Drop trait

### Running Tests

```bash
# Run all tests (parallel by default)
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_find_by_user

# Run with specific thread count
cargo test -- --test-threads=4
```

## Frontend Testing (Vitest)

### Configuration

The `vitest.config.ts` provides:

- **Parallel Execution**: Up to 4 threads
- **Test Isolation**: Each test runs in isolated environment
- **Coverage**: V8 coverage provider with 80% target
- **Mock Reset**: Automatic mock cleanup between tests

### Running Tests

```bash
# Run tests
pnpm test

# Run with coverage
pnpm test -- --coverage

# Run in watch mode
pnpm test -- --watch

# Run specific test
pnpm test src/components/Dashboard.test.tsx
```

## CI/CD Configuration

### GitHub Actions

The CI pipeline is configured to:

1. **Backend Tests**:
   - Use SQLite for fast, isolated testing
   - Run with `--all-features` flag
   - Parallel execution enabled by default

2. **Frontend Tests**:
   - Use Vitest with jsdom environment
   - Generate coverage reports
   - Upload artifacts for review

### Environment Variables

```yaml
# Backend (Rust)
DATABASE_URL: sqlite::memory:
JWT_SECRET: test-jwt-secret-for-ci-minimum-32-chars
ENCRYPTION_KEY: dGVzdC1lbmNyeXB0aW9uLWtleS0zMi1ieXRlcw==

# Frontend (Vitest)
CI: true
NODE_ENV: test
```

## Test Utilities

### Common Test Helpers

```rust
// Create test database
let test_db = TestDb::new().await?;
test_db.run_migrations().await?;

// Create test user
let user = create_test_user(db, "test@example.com", "testuser").await?;

// Create test provider account
let account = create_test_provider_account(
    db,
    user.id,
    "github",
    "Work Account",
    true // is_default
).await?;
```

### Fixture Builders

```rust
use ampel_db::tests::common::fixtures::{UserFixture, ProviderAccountFixture};

// Fluent fixture API
let user = UserFixture::new("test@example.com", "testuser")
    .with_full_name("Test User")
    .unverified()
    .create(db)
    .await?;

let account = ProviderAccountFixture::new(user.id, "github", "Work")
    .as_default()
    .with_scopes(r#"["repo","read:user"]"#)
    .create(db)
    .await?;
```

## Best Practices

### 1. Test Isolation

Always use `TestDb::new()` for each test:

```rust
#[tokio::test]
async fn test_example() {
    let test_db = TestDb::new().await.unwrap();
    test_db.run_migrations().await.unwrap();
    // Test logic
}
```

### 2. Cleanup

Tests automatically cleanup, but explicit cleanup is supported:

```rust
#[tokio::test]
async fn test_with_explicit_cleanup() {
    let test_db = TestDb::new().await.unwrap();
    test_db.run_migrations().await.unwrap();

    // Test logic

    test_db.cleanup().await; // Explicit cleanup
}
```

### 3. Parallel Testing

Rust tests run in parallel by default. Ensure tests are independent:

```rust
// ✅ Good - isolated database
#[tokio::test]
async fn test_isolated() {
    let test_db = TestDb::new().await.unwrap();
    // Each test has its own DB
}

// ❌ Bad - shared state
static SHARED_DB: OnceCell<DatabaseConnection> = OnceCell::new();
```

### 4. Real Database Testing

Always use real database queries, not mocks:

```rust
// ✅ Good - real database
let accounts = ProviderAccountQueries::find_by_user(db, user_id).await?;

// ❌ Bad - mocked (for integration tests)
// let mock_accounts = vec![...];
```

### 5. CI-Specific Optimizations

The test framework auto-detects CI environments:

```rust
// Automatically uses optimal settings for CI
let test_db = TestDb::new().await?;

// CI detection
fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}
```

## Performance Considerations

### SQLite vs PostgreSQL

- **SQLite**: 5-10x faster for test execution
- **In-Memory**: No disk I/O overhead
- **Parallel**: Each test has isolated database
- **CI Optimized**: Reduced resource usage

### Expected Performance

- **Unit Tests**: < 100ms per test
- **Integration Tests**: < 500ms per test
- **Full Suite**: < 5 minutes

## Migration Strategy

### Development

1. Develop against PostgreSQL locally
2. Run tests with SQLite for speed
3. Periodic integration tests against PostgreSQL

### CI/CD

1. Fast feedback: SQLite tests (< 5 minutes)
2. Nightly: Full PostgreSQL integration tests
3. Pre-merge: SQLite + critical PostgreSQL tests

## Troubleshooting

### Common Issues

1. **Migration Failures**
   - Ensure migrations are compatible with both SQLite and PostgreSQL
   - Test with both databases locally

2. **Test Flakiness**
   - Check for shared state between tests
   - Verify proper cleanup

3. **Performance Issues**
   - Review test parallelization settings
   - Check for unnecessary database operations

### Debugging

```bash
# Run tests with verbose output
cargo test -- --nocapture --test-threads=1

# Run specific test with logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Check test execution time
cargo test -- --nocapture --test-threads=1 --show-output
```

## Future Enhancements

1. **Snapshot Testing**: Add database state snapshot utilities
2. **Performance Tracking**: Track test execution time over time
3. **Fixture Library**: Expand fixture builders for all entities
4. **Test Data Seeding**: Bulk test data generation utilities
5. **PostgreSQL Parity**: Automated compatibility testing

## References

- [SeaORM Testing](https://www.sea-ql.org/SeaORM/docs/write-test/testing/)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Vitest Configuration](https://vitest.dev/config/)
- [GitHub Actions Best Practices](https://docs.github.com/en/actions/learn-github-actions/best-practices-for-using-github-actions)
