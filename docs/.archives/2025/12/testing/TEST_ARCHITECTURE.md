# Test Architecture

## Overview

Ampel uses a dual-database testing strategy: SQLite for fast CI testing and PostgreSQL for production parity validation.

## Directory Structure

```
crates/ampel-db/
├── tests/
│   ├── common/
│   │   ├── mod.rs           # Core test utilities
│   │   └── fixtures.rs      # Data fixtures and builders
│   └── integration/
│       ├── mod.rs           # Integration test entry point
│       └── provider_account_queries.rs  # Example integration tests
│
frontend/
├── tests/
│   ├── setup.ts             # Global test setup
│   └── fixtures/            # Frontend test data
├── vitest.config.ts         # Vitest configuration
└── src/**/*.test.tsx        # Component tests
```

## Test Organization

### Backend Tests (Rust)

#### 1. Common Test Utilities (`tests/common/mod.rs`)

**TestDb Struct**: Core testing infrastructure

```rust
pub struct TestDb {
    pub connection: DatabaseConnection,
    pub file_path: Option<PathBuf>,
    is_in_memory: bool,
}

impl TestDb {
    pub async fn new() -> Result<Self, DbErr>
    pub async fn new_in_memory() -> Result<Self, DbErr>
    pub async fn new_file() -> Result<Self, DbErr>
    pub async fn run_migrations(&self) -> Result<(), DbErr>
    pub fn connection(&self) -> &DatabaseConnection
    pub async fn cleanup(self)
}
```

**Features**:

- Unique database per test (atomic counter + UUID)
- Automatic migration execution
- CI environment detection
- Automatic cleanup via Drop trait
- In-memory for speed, file-based for debugging

#### 2. Test Fixtures (`tests/common/fixtures.rs`)

**Builder Pattern for Test Data**:

```rust
// User fixture
let user = UserFixture::new("test@example.com", "testuser")
    .with_full_name("Test User")
    .unverified()
    .create(db)
    .await?;

// Provider account fixture
let account = ProviderAccountFixture::new(user.id, "github", "Work")
    .as_default()
    .with_scopes(r#"["repo","read:user"]"#)
    .inactive()
    .create(db)
    .await?;
```

**Benefits**:

- Fluent API for readability
- Consistent test data
- Reduces boilerplate
- Easy to extend

#### 3. Integration Tests (`tests/integration/`)

**Structure**:

- One test file per query module or domain
- Tests use real database operations
- Each test is fully isolated
- Parallel execution safe

**Example**:

```rust
#[tokio::test]
async fn test_find_by_user() {
    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");
    let db = test_db.connection();

    let user = create_test_user(db, "test@example.com", "testuser")
        .await
        .unwrap();

    // Test logic here...

    test_db.cleanup().await;
}
```

### Frontend Tests (Vitest)

#### 1. Global Setup (`tests/setup.ts`)

**Features**:

- DOM mocking (matchMedia, IntersectionObserver, ResizeObserver)
- Automatic cleanup after each test
- CI environment detection
- Test utilities (wait, flushPromises)

#### 2. Component Tests

**Location**: `src/**/*.test.tsx`

**Structure**:

```typescript
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';

describe('ComponentName', () => {
  it('should render correctly', () => {
    render(<ComponentName />);
    expect(screen.getByText('Expected Text')).toBeInTheDocument();
  });
});
```

## Test Isolation Strategy

### Backend (Rust)

**Database Isolation**:

1. Each test creates unique SQLite database
2. Atomic counter ensures no collisions
3. UUID adds additional uniqueness
4. In-memory databases for speed
5. Automatic cleanup after test

**Parallel Execution**:

```bash
# Default: parallel with all CPU cores
cargo test

# Explicit thread count
cargo test -- --test-threads=4

# Sequential (for debugging)
cargo test -- --test-threads=1
```

### Frontend (TypeScript)

**Test Isolation**:

1. Each test runs in isolated environment
2. Mocks are reset between tests
3. DOM cleanup after each test
4. Up to 4 parallel threads

**Configuration** (vitest.config.ts):

```typescript
{
  test: {
    isolate: true,
    threads: true,
    maxThreads: 4,
    clearMocks: true,
    mockReset: true,
    restoreMocks: true,
  }
}
```

## Running Tests

### Backend

```bash
# All tests (parallel)
cargo test

# Specific package
cargo test --package ampel-db

# Specific test
cargo test test_find_by_user

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test integration

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html --output-dir coverage
```

### Frontend

```bash
# All tests
pnpm test

# Watch mode
pnpm test -- --watch

# Coverage
pnpm test -- --coverage

# Specific file
pnpm test src/components/Dashboard.test.tsx

# UI mode
pnpm test -- --ui
```

## CI/CD Integration

### GitHub Actions Workflow

**Backend Unit Tests**:

- Uses SQLite in-memory databases
- Parallel execution (4 threads)
- Fast feedback (< 5 minutes)
- Runs on all PRs

**PostgreSQL Integration Tests**:

- Uses real PostgreSQL container
- Sequential execution for safety
- Runs on main/develop branches
- Validates production compatibility

**Frontend Tests**:

- Uses jsdom environment
- Parallel execution
- Coverage reporting
- Artifact upload

### Environment Variables

```yaml
# Backend (SQLite CI)
DATABASE_URL: sqlite::memory:
JWT_SECRET: test-jwt-secret-for-ci-minimum-32-chars
ENCRYPTION_KEY: dGVzdC1lbmNyeXB0aW9uLWtleS0zMi1ieXRlcw==
CI: true

# Frontend
CI: true
NODE_ENV: test
```

## Performance Targets

### Backend

- **Unit tests**: < 100ms per test
- **Integration tests**: < 500ms per test
- **Full suite**: < 5 minutes (CI)

### Frontend

- **Component tests**: < 200ms per test
- **Full suite**: < 2 minutes (CI)

## Best Practices

### DO

✅ Create new TestDb for each test
✅ Use real database queries (not mocks) for integration tests
✅ Use fixture builders for consistent test data
✅ Run migrations in each test
✅ Write independent, parallel-safe tests
✅ Use descriptive test names
✅ Test both success and error cases
✅ Clean up resources explicitly when needed

### DON'T

❌ Share database connections between tests
❌ Use global state or static variables
❌ Mock database calls in integration tests
❌ Assume test execution order
❌ Skip error case testing
❌ Create tests that depend on each other
❌ Use production database for testing

## Debugging Tests

### Backend

**Print Output**:

```bash
cargo test -- --nocapture
```

**Single Test Sequential**:

```bash
cargo test test_name -- --test-threads=1 --nocapture
```

**With Logging**:

```bash
RUST_LOG=debug cargo test -- --nocapture
```

**File-based Database** (for inspection):

```rust
let test_db = TestDb::new_file().await?;
// Database file path in test_db.file_path
```

### Frontend

**Debug Mode**:

```bash
pnpm test -- --watch --reporter=verbose
```

**Single Test**:

```bash
pnpm test -- -t "test name pattern"
```

**Browser UI**:

```bash
pnpm test -- --ui
```

## Extending Tests

### Adding New Integration Test Module

1. Create new file in `tests/integration/`:

```rust
// tests/integration/user_queries.rs
mod common;
use common::fixtures::{UserFixture};
use common::TestDb;

#[tokio::test]
async fn test_user_creation() {
    let test_db = TestDb::new().await.unwrap();
    test_db.run_migrations().await.unwrap();
    // Test implementation
}
```

2. Register in `tests/integration/mod.rs`:

```rust
mod user_queries;
```

### Adding New Fixture

1. Add to `tests/common/fixtures.rs`:

```rust
pub struct RepositoryFixture {
    // Fields
}

impl RepositoryFixture {
    pub fn new(...) -> Self { ... }
    pub fn with_...(...) -> Self { ... }
    pub async fn create(...) -> Result<...> { ... }
}
```

## Migration Compatibility

### Ensuring SQLite/PostgreSQL Compatibility

**Compatible SQL**:

- Standard SQL types (TEXT, INTEGER, TIMESTAMP)
- Avoid DB-specific functions
- Use SeaORM abstractions

**Testing Both**:

```rust
#[tokio::test]
async fn test_with_sqlite() {
    let db = Database::connect("sqlite::memory:").await?;
    // Test logic
}

#[tokio::test]
#[ignore] // Run with --ignored for PostgreSQL tests
async fn test_with_postgres() {
    let db = Database::connect(&env::var("DATABASE_URL")?).await?;
    // Same test logic
}
```

## Coverage

### Backend Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html
```

### Frontend Coverage

```bash
# Generate coverage
pnpm test -- --coverage

# Open report
open coverage/index.html
```

### Coverage Targets

- **Lines**: 80%+
- **Functions**: 75%+
- **Branches**: 75%+
- **Statements**: 80%+

## Troubleshooting

### Common Issues

**"Database locked" errors**:

- Ensure each test uses its own TestDb instance
- Check for concurrent writes to same database

**Migration failures**:

- Verify migrations are SQLite-compatible
- Check migration order and dependencies

**Flaky tests**:

- Remove shared state between tests
- Check for timing-dependent assertions
- Ensure proper cleanup

**Slow tests**:

- Review database operations (use indexing)
- Check for unnecessary migrations
- Profile with `cargo test -- --nocapture --test-threads=1`

## Future Improvements

- [ ] Snapshot testing for database states
- [ ] Performance regression tracking
- [ ] Automated compatibility testing (SQLite ↔ PostgreSQL)
- [ ] Test data generation utilities
- [ ] Property-based testing with proptest
- [ ] Mutation testing
- [ ] Visual regression testing (frontend)

## References

- [SeaORM Testing Docs](https://www.sea-ql.org/SeaORM/docs/write-test/testing/)
- [Vitest Documentation](https://vitest.dev/)
- [Testing Library](https://testing-library.com/)
- [Cargo Test Book](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
