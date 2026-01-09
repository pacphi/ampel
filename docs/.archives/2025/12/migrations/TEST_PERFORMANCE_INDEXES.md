# Testing Performance Indexes Migration

This document describes how to test the performance indexes migration `m20251224_000001_performance_indexes`.

## Overview

The migration adds two critical performance indexes:

1. **`idx_repositories_user_id`** - Index on `repositories(user_id)`
   - Improves performance of user-specific repository lookups
   - Expected improvement: 10-100x for users with many repositories

2. **`idx_pull_requests_repository_state`** - Composite index on `pull_requests(repository_id, state)`
   - Enables efficient filtered PR queries like "find all open PRs for repository X"
   - Expected improvement: 5-50x for repositories with many PRs

## Prerequisites

- PostgreSQL 16+ running locally or via Docker
- Environment variable `DATABASE_URL` set to your test database

## Testing Steps

### 1. Setup Test Database

```bash
# Using Docker
docker compose -f docker/docker-compose.yml up -d postgres

# Or use local PostgreSQL
createdb ampel_test
export DATABASE_URL="postgres://ampel:ampel@localhost:5432/ampel_test"
```

### 2. Run Migration

```bash
# From project root
cd crates/ampel-db

# Run all migrations (including the new one)
cargo run --bin sea-orm-cli migrate up

# Or using the Makefile
make db-migrate
```

### 3. Verify Indexes Created

Connect to the database and check that indexes exist:

```sql
-- Connect to database
psql $DATABASE_URL

-- List all indexes on repositories table
\d repositories

-- Expected output should include:
-- "idx_repositories_user_id" btree (user_id)

-- List all indexes on pull_requests table
\d pull_requests

-- Expected output should include:
-- "idx_pull_requests_repository_state" btree (repository_id, state)

-- View index definitions
SELECT
    schemaname,
    tablename,
    indexname,
    indexdef
FROM pg_indexes
WHERE tablename IN ('repositories', 'pull_requests')
ORDER BY tablename, indexname;
```

### 4. Test Query Performance

```sql
-- Test 1: Query repositories by user (should use idx_repositories_user_id)
EXPLAIN ANALYZE
SELECT * FROM repositories
WHERE user_id = 'some-uuid-here';

-- Expected: Index Scan using idx_repositories_user_id

-- Test 2: Query open PRs for a repository (should use idx_pull_requests_repository_state)
EXPLAIN ANALYZE
SELECT * FROM pull_requests
WHERE repository_id = 'some-uuid-here'
  AND state = 'open';

-- Expected: Index Scan using idx_pull_requests_repository_state

-- Test 3: Count open PRs per repository (should use composite index)
EXPLAIN ANALYZE
SELECT repository_id, COUNT(*)
FROM pull_requests
WHERE state = 'open'
GROUP BY repository_id;

-- Expected: Index scan on idx_pull_requests_repository_state
```

### 5. Test Rollback

```bash
# Test the down migration
cargo run --bin sea-orm-cli migrate down

# Verify indexes are dropped
psql $DATABASE_URL -c "\d repositories"
psql $DATABASE_URL -c "\d pull_requests"

# Re-apply migration
cargo run --bin sea-orm-cli migrate up
```

## Performance Benchmarks

To measure actual performance improvements, create test data:

```sql
-- Insert test user
INSERT INTO users (id, email, password_hash, created_at, updated_at)
VALUES ('00000000-0000-0000-0000-000000000001', 'test@example.com', 'hash', NOW(), NOW());

-- Insert 100 test repositories
INSERT INTO repositories (id, user_id, provider, provider_id, owner, name, full_name, url, default_branch, created_at, updated_at)
SELECT
    gen_random_uuid(),
    '00000000-0000-0000-0000-000000000001',
    'github',
    'provider-' || i,
    'owner-' || i,
    'repo-' || i,
    'owner-' || i || '/repo-' || i,
    'https://github.com/owner-' || i || '/repo-' || i,
    'main',
    NOW(),
    NOW()
FROM generate_series(1, 100) AS i;

-- Insert 10,000 test pull requests (100 per repo)
INSERT INTO pull_requests (
    id, repository_id, provider, provider_id, number, title,
    url, state, source_branch, target_branch, author,
    created_at, updated_at, last_synced_at
)
SELECT
    gen_random_uuid(),
    r.id,
    'github',
    'pr-' || (i * 100 + j),
    j,
    'Test PR #' || j,
    'https://github.com/test/pr/' || j,
    CASE WHEN random() < 0.3 THEN 'open'
         WHEN random() < 0.6 THEN 'closed'
         ELSE 'merged' END,
    'feature-' || j,
    'main',
    'test-author',
    NOW() - (random() * interval '30 days'),
    NOW() - (random() * interval '7 days'),
    NOW()
FROM repositories r, generate_series(1, 100) AS i, generate_series(1, 100) AS j
WHERE r.user_id = '00000000-0000-0000-0000-000000000001'
LIMIT 10000;

-- Benchmark query: Find all repositories for a user
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM repositories
WHERE user_id = '00000000-0000-0000-0000-000000000001';

-- Benchmark query: Find open PRs for each repository
EXPLAIN (ANALYZE, BUFFERS)
SELECT r.full_name, COUNT(pr.id) as open_prs
FROM repositories r
LEFT JOIN pull_requests pr ON pr.repository_id = r.id AND pr.state = 'open'
WHERE r.user_id = '00000000-0000-0000-0000-000000000001'
GROUP BY r.id, r.full_name;
```

## Expected Results

### Without Indexes

- Repository lookup: Sequential scan, ~10-50ms for 100 repos
- PR state filter: Sequential scan, ~50-200ms for 10,000 PRs

### With Indexes

- Repository lookup: Index scan, ~0.5-2ms (10-25x faster)
- PR state filter: Index scan, ~1-5ms (10-40x faster)

## Cleanup

```bash
# Drop test database
dropdb ampel_test
```

## CI Integration

The migration is automatically tested in CI via:

- `make test-backend` - Runs all backend tests including migration tests
- GitHub Actions workflow checks migration applies cleanly

## Troubleshooting

### Index not being used

Check that PostgreSQL query planner is using the index:

```sql
-- Enable detailed query planning
SET enable_seqscan = off;

-- Re-run query and check EXPLAIN output
EXPLAIN ANALYZE SELECT ...;
```

### Migration fails

Common issues:

1. **Index already exists**: Check if you've run migration before
2. **Foreign key constraint**: Ensure parent tables exist
3. **Permissions**: Database user needs CREATE INDEX privilege

## Related Files

- Migration: `/alt/home/developer/workspace/projects/ampel/crates/ampel-db/src/migrations/m20251224_000001_performance_indexes.rs`
- Entity models: `/alt/home/developer/workspace/projects/ampel/crates/ampel-db/src/entities/`
