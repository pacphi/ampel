# Backend Testing Guide

This document covers all aspects of testing the Rust backend, including unit tests, integration tests, database testing, and test utilities.

## Table of Contents

- [Overview](#overview)
- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Database Testing](#database-testing)
- [Test Utilities](#test-utilities)
- [Writing Tests](#writing-tests)
- [Best Practices](#best-practices)
- [Debugging](#debugging)
- [Coverage](#coverage)

## Overview

The backend uses Rust's built-in testing framework with additional tooling:

- **Test Runner**: `cargo test` (standard) or `cargo-nextest` (faster, parallel)
- **Coverage**: `cargo-tarpaulin`
- **Database**: PostgreSQL for integration tests, SQLite for fast unit tests

### Testing Philosophy

1. **Test Organization**: Unit tests live alongside code in `#[cfg(test)]` modules, integration tests in separate `tests/` directories ([Rust Book - Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html))
2. **Isolation**: Each integration test runs in a completely isolated database instance
3. **Fast Feedback**: SQLite for quick unit tests, PostgreSQL for comprehensive integration tests
4. **Real Data**: Use actual database queries, not mocks, for integration tests

## Test Organization

### Directory Structure

```text
crates/
├── ampel-api/
│   ├── src/
│   │   └── **/*.rs          # Unit tests in #[cfg(test)] modules
│   └── tests/               # Integration tests
├── ampel-core/
│   └── src/
│       └── **/*.rs          # Unit tests in #[cfg(test)] modules
├── ampel-db/
│   ├── src/
│   │   └── **/*.rs          # Unit tests in #[cfg(test)] modules
│   └── tests/
│       ├── common/          # Shared test utilities
│       │   ├── mod.rs       # TestDb helper for isolated databases
│       │   └── fixtures.rs  # Test data builders
│       └── integration/     # Integration tests
│           ├── mod.rs
│           ├── provider_account_queries.rs
│           ├── user_queries.rs
│           ├── repo_queries.rs
│           └── pr_queries.rs
├── ampel-providers/
│   ├── src/
│   │   └── **/*.rs          # Unit tests in #[cfg(test)] modules
│   └── tests/
│       ├── github_tests.rs
│       └── mock_provider_tests.rs
└── ampel-worker/
    ├── src/
    │   └── **/*.rs          # Unit tests in #[cfg(test)] modules
    └── tests/               # Worker integration tests
```

### Key Patterns

- **Unit Tests**: Live in `#[cfg(test)]` modules within the same file as the code they test
- **Integration Tests**: Separate `tests/` directory at crate root ([Rust by Example - Integration Testing](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html))
- **Test Helpers**: `tests/common/` module for shared utilities (not auto-discovered as tests)

## Running Tests

### Quick Reference

```bash
# Run all backend tests
make test-backend               # cargo test --all-features

# Using cargo-nextest (faster)
cargo nextest run --profile fast
```

### All Tests

```bash
cargo test --all-features
```

### Specific Crate

```bash
cargo test -p ampel-db --all-features
cargo test -p ampel-providers --all-features
cargo test -p ampel-api --all-features
```

### Specific Test

```bash
# By test name
cargo test test_find_by_user

# By module path
cargo test provider_account_queries::test_find_by_user

# Integration tests only
cargo test --test integration
```

### With Output

```bash
cargo test -- --nocapture
```

## Database Testing

### Dual-Database Strategy

Ampel supports both PostgreSQL and SQLite for testing:

| Database   | Use Case                   | Speed  | Features                |
| ---------- | -------------------------- | ------ | ----------------------- |
| PostgreSQL | Integration tests, CI      | Slower | Full feature support    |
| SQLite     | Unit tests, fast local dev | Faster | Limited (no migrations) |

### PostgreSQL (Integration Tests)

Integration tests require PostgreSQL when:

- `TEST_DATABASE_TYPE=postgres` is set, OR
- `DATABASE_URL` starts with `postgres://`

```bash
# Set environment variables
export DATABASE_URL="postgres://ampel:ampel@localhost:5432/ampel_test"
export TEST_DATABASE_TYPE=postgres

# Run tests
cargo test --all-features
```

### SQLite (Unit Tests)

```bash
# Tests automatically use SQLite when PostgreSQL is not configured
export DATABASE_URL="sqlite::memory:"
export TEST_DATABASE_TYPE=sqlite

# Run tests
cargo test --all-features
```

**Note:** Some integration tests require PostgreSQL and will be automatically skipped in SQLite mode because they use PostgreSQL-specific features (foreign keys in migrations, partial unique indexes).

### Running with Docker

```bash
# Option 1: docker compose
docker compose up -d postgres
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test cargo test

# Option 2: standalone container
docker run -d --name ampel-test-postgres \
  -e POSTGRES_USER=ampel \
  -e POSTGRES_PASSWORD=ampel \
  -e POSTGRES_DB=ampel_test \
  -p 5432:5432 \
  postgres:16-alpine

# Wait and test
sleep 5
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test cargo test
```

### Environment Variables

| Variable             | Purpose                              | Example                                            |
| -------------------- | ------------------------------------ | -------------------------------------------------- |
| `TEST_DATABASE_TYPE` | Primary way to select backend in CI  | `postgres` or `sqlite`                             |
| `DATABASE_URL`       | Connection string                    | `postgres://ampel:ampel@localhost:5432/ampel_test` |
| `TEST_DATABASE_URL`  | Base URL for creating test databases | `postgres://ampel:ampel@localhost:5432`            |
| `JWT_SECRET`         | Required for auth tests              | `test-jwt-secret-for-ci-minimum-32-chars`          |
| `ENCRYPTION_KEY`     | Required for encryption tests        | `dGVzdC1lbmNyeXB0aW9uLWtleS0zMi1ieXRlcw==`         |

## Test Utilities

### TestDb - Isolated Database Per Test

The `TestDb` struct provides isolated database instances for each test:

```rust
use crate::common::TestDb;

#[tokio::test]
async fn my_test() {
    // Skip if migrations not supported (SQLite)
    if TestDb::skip_if_sqlite() {
        return;
    }

    // Create isolated database (auto-generates unique name)
    let test_db = TestDb::new().await.expect("Failed to create test DB");

    // Run migrations
    test_db.run_migrations().await.expect("Failed to run migrations");

    // Get connection
    let db = test_db.connection();

    // ... your test code ...

    // Clean up (drops database)
    test_db.cleanup().await;
}
```

**Key Methods:**

| Method                     | Description                                        |
| -------------------------- | -------------------------------------------------- |
| `TestDb::new()`            | Creates PostgreSQL or SQLite based on environment  |
| `TestDb::new_postgres()`   | Explicitly create PostgreSQL test DB               |
| `TestDb::new_sqlite()`     | Explicitly create SQLite test DB                   |
| `TestDb::skip_if_sqlite()` | Skip tests that require PostgreSQL features        |
| `test_db.run_migrations()` | Apply all migrations                               |
| `test_db.cleanup()`        | Drop database (PostgreSQL) or delete file (SQLite) |

### Fixtures - Test Data Builders

Use builder pattern for consistent test data:

```rust
use crate::common::fixtures::{UserFixture, ProviderAccountFixture};

#[tokio::test]
async fn test_with_fixtures() {
    let test_db = TestDb::new().await.unwrap();
    test_db.run_migrations().await.unwrap();
    let db = test_db.connection();

    // Create user with builder pattern
    let user = UserFixture::new("user@example.com", "Test User")
        .with_avatar_url("https://example.com/avatar.png")
        .create(db)
        .await
        .unwrap();

    // Create provider account
    let account = ProviderAccountFixture::new(user.id, "github", "Work Account")
        .set_default()
        .with_scopes(r#"["repo", "read:user"]"#)
        .create(db)
        .await
        .unwrap();

    // Test assertions
    assert_eq!(account.user_id, user.id);
    assert!(account.is_default);

    test_db.cleanup().await;
}
```

**Quick Helpers:**

```rust
use crate::common::fixtures::{create_test_user, create_test_provider_account};

let user = create_test_user(db, "test@example.com", "testuser").await.unwrap();
let account = create_test_provider_account(db, user.id, "github", "Work", true).await.unwrap();
```

## Writing Tests

### Unit Tests

Located in `#[cfg(test)]` modules within source files:

```rust
// In src/models/user.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_email_validation() {
        assert!(User::is_valid_email("test@example.com"));
        assert!(!User::is_valid_email("invalid-email"));
    }
}
```

**When to use:**

- Testing pure functions
- Testing struct methods
- Testing business logic without database

### Integration Tests

Located in `tests/` directory:

````rust
// In crates/ampel-db/tests/integration/provider_account_queries.rs

//! Integration tests for provider account queries
//!
//! Prerequisites:
//! - PostgreSQL database (tests auto-skip in SQLite mode)
//! - Environment variables: DATABASE_URL, JWT_SECRET, ENCRYPTION_KEY
//!
//! Run these tests:
//! ```bash
//! cargo test -p ampel-db --test integration
//! ```

use ampel_db::queries::ProviderAccountQueries;
use super::common::{TestDb, fixtures::create_test_user};

#[tokio::test]
async fn test_find_by_user() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .expect("Failed to create user");

    let accounts = ProviderAccountQueries::find_by_user(db, user.id)
        .await
        .expect("Failed to find accounts");

    assert_eq!(accounts.len(), 0);

    test_db.cleanup().await;
}
````

**When to use:**

- Testing database queries
- Testing API endpoints
- Testing complex workflows
- Testing multiple crates together

### Test Naming Conventions

Follow Rust conventions:

- **Test functions**: `test_<what_is_being_tested>`
- **Describe behavior**: `test_find_by_user_returns_all_accounts`
- **Include context**: `test_set_default_clears_previous_default`
- **Edge cases**: `test_delete_account_unauthorized`

### Documentation Standards

Every test file should have module-level documentation:

````rust
//! Integration tests for provider account queries
//!
//! ## Prerequisites
//! - PostgreSQL database (or tests will be skipped)
//! - Environment variables: DATABASE_URL, JWT_SECRET, ENCRYPTION_KEY
//!
//! ## Running These Tests
//! ```bash
//! cargo test -p ampel-db --test integration
//! ```
````

## Best Practices

### DO

- Create new TestDb for each test
- Use real database queries (not mocks) for integration tests
- Use fixture builders for consistent test data
- Run migrations in each test
- Write independent, parallel-safe tests
- Use descriptive test names
- Test both success and error cases
- Clean up resources explicitly
- Use `#[tokio::test]` for async test functions

### DON'T

- Share database connections between tests
- Use global state or static variables
- Mock database calls in integration tests
- Assume test execution order
- Skip error case testing
- Create tests that depend on each other
- Use production database for testing

### Comprehensive Test Example

```rust
#[tokio::test]
async fn test_comprehensive_scenario() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    // Setup: Create test data
    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();

    // Action: Perform the operation
    let result = some_operation(db, user.id).await;

    // Assert: Verify behavior
    assert!(result.is_ok(), "Operation should succeed");
    let data = result.unwrap();
    assert_eq!(data.user_id, user.id);

    // Assert: Verify side effects
    let records = verify_records(db, user.id).await.unwrap();
    assert_eq!(records.len(), 1);

    // Cleanup
    test_db.cleanup().await;
}
```

## Debugging

### Print Output

```bash
cargo test -- --nocapture
```

### Single Test Sequential

```bash
cargo test test_name -- --test-threads=1 --nocapture
```

### With Logging

```bash
RUST_LOG=debug cargo test -- --nocapture
```

### File-based Database (for inspection)

```rust
let test_db = TestDb::new_file().await?;
// Database file path in test_db.file_path
println!("Database at: {:?}", test_db.file_path);
```

## Coverage

### Generate Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --all-features --workspace --out Html --output-dir coverage

# Open report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

### Coverage Targets

- **Lines**: 80%+
- **Functions**: 75%+
- **Branches**: 75%+

### Coverage Focus

**Prioritize:**

- Critical paths (authentication, authorization, data validation)
- Complex logic (business rules, state machines)
- Error handling (all error cases should be tested)
- Database queries (integration tests with real data)

**Don't obsess over:**

- Trivial getters/setters
- Generated code (migrations, entities)
- Simple DTOs with no logic

## Troubleshooting

### "Sqlite doesn't support multiple alter options"

This error occurs when running integration tests with SQLite. Use PostgreSQL:

```bash
TEST_DATABASE_TYPE=postgres cargo test
```

### "Failed to create test database"

Ensure PostgreSQL is running:

```bash
pg_isready -h localhost -p 5432 -U ampel
```

### "Database locked" errors

- Ensure each test uses its own TestDb instance
- Check for concurrent writes to same database

### Permission denied on database creation

```sql
ALTER USER ampel CREATEDB;
```

### Flaky tests

- Remove shared state between tests
- Check for timing-dependent assertions
- Ensure proper cleanup

### Slow tests

- Review database operations (use indexing)
- Check for unnecessary migrations
- Profile: `cargo test -- --nocapture --test-threads=1`

## Performance Targets

- **Unit tests**: < 100ms per test
- **Integration tests**: < 500ms per test
- **Full suite**: < 5 minutes (CI)

## References

- [The Rust Book - Writing Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [The Rust Book - Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Rust by Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [Rust by Example - Integration Testing](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [SeaORM Testing](https://www.sea-ql.org/SeaORM/docs/write-test/testing/)
- [cargo-nextest](https://nexte.st/)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
