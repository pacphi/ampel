# Integration Testing Guide

## Overview

Ampel uses PostgreSQL for production and integration tests. SQLite is used for simple unit tests.

## Test Database Configuration

The test suite automatically selects the appropriate database backend based on environment variables:

### PostgreSQL (Integration Tests)

Integration tests require PostgreSQL and will automatically use it when:

- `TEST_DATABASE_TYPE=postgres` is set, OR
- `DATABASE_URL` starts with `postgres://` or `postgresql://`, OR
- `TEST_DATABASE_URL` starts with `postgres://` or `postgresql://`

### SQLite (Unit Tests)

Simple unit tests (like fixture tests in `common/`) use SQLite by default when PostgreSQL is not configured.

## Running Tests Locally

### Option 1: With Docker (Recommended)

```bash
# Start PostgreSQL
docker run -d --name ampel-test-postgres \
  -e POSTGRES_USER=ampel \
  -e POSTGRES_PASSWORD=ampel \
  -e POSTGRES_DB=ampel_test \
  -p 5432:5432 \
  postgres:16-alpine

# Wait for PostgreSQL to be ready
sleep 5

# Run integration tests
TEST_DATABASE_TYPE=postgres \
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test \
cargo test --all-features

# Cleanup
docker rm -f ampel-test-postgres
```

### Option 2: Using docker compose

```bash
# Start PostgreSQL
docker compose -f docker/docker-compose.yml up -d postgres

# Run tests
TEST_DATABASE_TYPE=postgres \
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel_test \
cargo test --all-features

# Cleanup
docker compose -f docker/docker-compose.yml down
```

### Option 3: SQLite Only (Limited)

Run only unit tests that don't require migrations:

```bash
# This will skip integration tests due to migration incompatibility
cargo test --all-features --lib
```

## CI Configuration

The GitHub Actions CI runs two separate test jobs:

1. **backend-unit-test**: Uses SQLite for fast unit tests
2. **backend-integration-test**: Uses PostgreSQL for full integration tests

See `.github/workflows/ci.yml` for the complete configuration.

## Test Database Isolation

Each test gets a completely isolated database:

- **PostgreSQL**: Creates a unique database per test (e.g., `ampel_test_1234_abcdef`)
- **SQLite**: Creates a unique temporary file per test

All databases are automatically cleaned up after tests complete.

## Troubleshooting

### "Sqlite doesn't support multiple alter options"

This error occurs when running integration tests with SQLite. Solution: Use PostgreSQL for integration tests.

### "Failed to create test database"

Ensure PostgreSQL is running and accessible:

```bash
pg_isready -h localhost -p 5432 -U ampel
```

### Permission denied on database creation

Ensure the PostgreSQL user has CREATEDB permission:

```sql
ALTER USER ampel CREATEDB;
```

## Environment Variables

| Variable             | Purpose                              | Example                                            |
| -------------------- | ------------------------------------ | -------------------------------------------------- |
| `TEST_DATABASE_TYPE` | Primary way to select backend in CI  | `postgres` or `sqlite`                             |
| `DATABASE_URL`       | Connection string (used in CI)       | `postgres://ampel:ampel@localhost:5432/ampel_test` |
| `TEST_DATABASE_URL`  | Base URL for creating test databases | `postgres://ampel:ampel@localhost:5432`            |
| `USE_POSTGRES_TESTS` | Explicit opt-in to PostgreSQL        | Set to any value                                   |

## Test Structure

```
crates/ampel-db/tests/
├── common/
│   ├── mod.rs          # TestDb helper (supports both SQLite & PostgreSQL)
│   └── fixtures.rs     # Test data fixtures
└── integration/
    ├── mod.rs          # Integration test module setup
    └── provider_account_queries.rs  # Integration tests (requires PostgreSQL)
```
