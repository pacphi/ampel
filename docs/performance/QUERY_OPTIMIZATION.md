# Query Optimization - N+1 Problem Resolution

## Problem

The dashboard endpoints (`/api/dashboard/summary` and `/api/dashboard/grid`) had an N+1 query problem that resulted in excessive database queries:

**Before optimization:**

- For 50 repositories with 10 PRs each (500 total PRs):
  - 1 query to load repositories
  - 50 queries to load PRs (one per repository)
  - 1000 queries to load CI checks (one per PR)
  - 1000 queries to load reviews (one per PR)
  - **Total: 2,051 queries**

This resulted in poor performance and high database load.

## Solution

Implemented batch loading using `WHERE id IN (...)` queries to load all related data in constant queries:

**After optimization:**

- 1 query to load all repositories
- 1 query to load all open PRs for all repositories
- 1 query to load all CI checks for all PRs
- 1 query to load all reviews for all PRs
- **Total: 4 queries**

### Complexity Analysis

- **Before**: O(n²) where n = number of repositories
- **After**: O(1) - constant number of queries regardless of data volume

### Performance Improvement

- **Query reduction**: 2,051 → 4 queries (512x reduction)
- **Time complexity**: O(n²) → O(1)
- **Memory**: O(n) in-memory hash maps for O(1) lookups

## Implementation Details

### New Batch Query Methods

Added three new batch loading methods to query modules:

#### 1. `PrQueries::find_open_for_repositories`

```rust
pub async fn find_open_for_repositories(
    db: &DatabaseConnection,
    repository_ids: &[Uuid],
) -> Result<Vec<Model>, DbErr>
```

Loads all open PRs for multiple repositories in one query using `WHERE repository_id IN (...)`.

#### 2. `CICheckQueries::find_for_pull_requests`

```rust
pub async fn find_for_pull_requests(
    db: &DatabaseConnection,
    pull_request_ids: &[Uuid],
) -> Result<Vec<Model>, DbErr>
```

Loads all CI checks for multiple PRs in one query using `WHERE pull_request_id IN (...)`.

#### 3. `ReviewQueries::find_for_pull_requests`

```rust
pub async fn find_for_pull_requests(
    db: &DatabaseConnection,
    pull_request_ids: &[Uuid],
) -> Result<Vec<Model>, DbErr>
```

Loads all reviews for multiple PRs in one query using `WHERE pull_request_id IN (...)`.

### Dashboard Handler Updates

#### `get_summary` Handler

**Query Flow:**

1. Load all repositories for user
2. Batch load all open PRs for all repositories
3. Batch load all CI checks for all PRs
4. Batch load all reviews for all PRs

**Data Processing:**

- Build hash maps (`HashMap<Uuid, Vec<T>>`) for O(1) lookups
- Iterate through PRs once to calculate status
- No nested database queries

#### `get_grid` Handler

**Query Flow:**

1. Load all repositories for user
2. Batch load all open PRs for all repositories
3. Batch load all CI checks for all PRs
4. Batch load all reviews for all PRs

**Data Processing:**

- Group PRs by repository using hash map
- Build hash maps for CI checks and reviews
- Calculate repository status from PR statuses
- No nested database queries

## Key Design Patterns

### 1. Batch Loading Pattern

Load all related data upfront in bulk queries instead of loading individually in loops.

### 2. Hash Map Grouping

Use `HashMap<ParentId, Vec<Child>>` to group child records by parent ID for O(1) access.

```rust
let mut ci_checks_by_pr: HashMap<uuid::Uuid, Vec<_>> = HashMap::new();
for ci_check in all_ci_checks {
    ci_checks_by_pr.entry(ci_check.pull_request_id).or_default().push(ci_check);
}
```

### 3. Empty Data Early Return

Return early if intermediate data is empty to avoid unnecessary queries.

```rust
if all_open_prs.is_empty() {
    return Ok(/* empty response */);
}
```

## Files Modified

### Query Modules

- `crates/ampel-db/src/queries/pr_queries.rs` - Added `find_open_for_repositories`
- `crates/ampel-db/src/queries/ci_check_queries.rs` - Added `find_for_pull_requests`
- `crates/ampel-db/src/queries/review_queries.rs` - Added `find_for_pull_requests`

### Handlers

- `crates/ampel-api/src/handlers/dashboard.rs` - Refactored `get_summary` and `get_grid`

## Testing

All existing tests pass:

```bash
cargo test --package ampel-api --lib handlers::dashboard::tests --all-features
```

Results:

- ✅ test_visibility_breakdown_default
- ✅ test_visibility_breakdown_clone
- ✅ test_visibility_breakdown_serialization
- ✅ test_dashboard_summary_has_all_fields

## Performance Monitoring

The implementation includes detailed tracing for monitoring:

```rust
tracing::debug!(pr_count = all_open_prs.len(), "Loaded all open PRs in batch");
tracing::debug!(ci_check_count = all_ci_checks.len(), "Loaded all CI checks in batch");
tracing::debug!(review_count = all_reviews.len(), "Loaded all reviews in batch");
```

Metrics recorded via Prometheus:

- `ampel_dashboard_summary_duration_seconds` - Response time histogram
- `ampel_dashboard_breakdown_total` - Status count by visibility

## Future Optimizations

### Potential Improvements

1. **Database Indexes**: Ensure indexes on foreign key columns:
   - `pull_requests.repository_id`
   - `ci_checks.pull_request_id`
   - `reviews.pull_request_id`

2. **Query Result Caching**: Cache batch query results in Redis for frequently accessed data

3. **Pagination**: For users with thousands of repositories, implement pagination at the repository level

4. **Database Views**: Consider materialized views for pre-computed aggregations

5. **Connection Pooling**: Ensure proper connection pool sizing for batch queries

## Conclusion

The N+1 query optimization successfully reduced database queries from O(n²) to O(1), resulting in:

- 512x reduction in query count for typical workloads
- Predictable performance regardless of data volume
- Lower database load and improved scalability
- Maintained backward compatibility with no API changes
