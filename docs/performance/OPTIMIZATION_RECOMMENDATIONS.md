# Dashboard Performance Optimization Recommendations

**Document Version:** 1.0
**Date:** December 24, 2025
**Component:** Dashboard Summary API
**Current State:** N+1 Query Pattern

---

## Executive Summary

The current dashboard summary endpoint exhibits **N+1 query patterns** that will cause significant performance degradation as user repositories and pull requests scale. This document provides specific, actionable optimization recommendations with expected performance improvements.

### Current Performance Profile

| Dataset Size | Repositories | PRs  | Queries | Response Time |
| ------------ | ------------ | ---- | ------- | ------------- |
| Small        | 10           | 50   | 111     | ~100ms        |
| Medium       | 50           | 500  | 551     | ~300ms        |
| Large        | 100          | 2000 | 2101    | ~500ms ⚠️     |
| Very Large   | 200          | 6000 | 4201    | ~1000ms ❌    |

**Critical Issue:** Query count scales linearly with repository and PR count, leading to unacceptable response times at scale.

---

## Optimization Strategy Overview

| Priority | Optimization        | Complexity | Expected Improvement        | Timeline |
| -------- | ------------------- | ---------- | --------------------------- | -------- |
| **P0**   | Database Indexing   | Low        | 70-90% query speedup        | 1 day    |
| **P1**   | SQL Aggregation     | Medium     | 95% query reduction         | 1 week   |
| **P2**   | Redis Caching       | Medium     | 98% response time reduction | 1 week   |
| **P3**   | Materialized Views  | High       | 99% response time reduction | 2 weeks  |
| **P4**   | Parallel Processing | Medium     | 4x speedup (8 cores)        | 1 week   |

---

## P0: Database Indexing (IMMEDIATE)

### Problem

Current queries perform full table scans on:

- `repositories.user_id`
- `pull_requests.repository_id` and `state`
- `ci_checks.pull_request_id`
- `reviews.pull_request_id`

### Solution

Create the following indexes:

```sql
-- Migration: 20251224_add_dashboard_indexes.sql

-- Index for user repository lookups
CREATE INDEX IF NOT EXISTS idx_repositories_user_id
ON repositories(user_id)
WHERE deleted_at IS NULL;

-- Composite index for open PR queries
CREATE INDEX IF NOT EXISTS idx_pull_requests_repo_state
ON pull_requests(repository_id, state)
WHERE state = 'open';

-- Index for Ampel status filtering
CREATE INDEX IF NOT EXISTS idx_pull_requests_ampel_status
ON pull_requests(ampel_status)
WHERE state = 'open';

-- Index for CI checks per PR
CREATE INDEX IF NOT EXISTS idx_ci_checks_pr_id
ON ci_checks(pull_request_id);

-- Index for reviews per PR
CREATE INDEX IF NOT EXISTS idx_reviews_pr_id
ON reviews(pull_request_id);

-- Composite index for provider counts
CREATE INDEX IF NOT EXISTS idx_repositories_user_provider
ON repositories(user_id, provider)
WHERE deleted_at IS NULL;
```

### Implementation

1. **Create migration file:**

   ```bash
   cd crates/ampel-db
   sea-orm-cli migrate generate add_dashboard_indexes
   ```

2. **Add indexes to migration:**
   Copy SQL above into migration file

3. **Test locally:**

   ```bash
   make db-migrate
   EXPLAIN ANALYZE SELECT * FROM repositories WHERE user_id = 'uuid';
   ```

4. **Deploy to staging:**

   ```bash
   # Staging deployment
   make deploy-staging
   ```

5. **Monitor impact:**
   - Query execution time should drop 70-90%
   - Check with `EXPLAIN ANALYZE` before/after

### Expected Impact

| Metric                | Before    | After    | Improvement    |
| --------------------- | --------- | -------- | -------------- |
| Repository query      | 50ms      | 5ms      | 90% faster     |
| PR query              | 30ms      | 3ms      | 90% faster     |
| CI check query        | 20ms      | 2ms      | 90% faster     |
| **Total (100 repos)** | **500ms** | **50ms** | **90% faster** |

**Risk:** Low - Indexes only improve read performance

---

## P1: SQL Aggregation (HIGH PRIORITY)

### Problem

Current implementation:

1. Fetch all repositories for user (1 query)
2. For each repository, fetch all open PRs (N queries)
3. For each PR, fetch CI checks and reviews (2M queries)
4. Calculate status in application code

**Total queries for 100 repos with 10 PRs each:** 1 + 100 + (100 × 10 × 2) = **2101 queries**

### Solution

Replace with a single aggregated SQL query:

```rust
// File: crates/ampel-api/src/handlers/dashboard_optimized.rs

use sea_orm::*;
use ampel_db::prelude::*;

pub async fn get_summary_optimized(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    let start = std::time::Instant::now();

    // Single query with JOINs and aggregations
    let summary = Repository::find()
        .filter(repository::Column::UserId.eq(auth.user_id))
        .filter(repository::Column::DeletedAt.is_null())
        .left_join(PullRequest)
        .filter(
            pull_request::Column::State.eq("open")
            .or(pull_request::Column::Id.is_null()) // Include repos with no PRs
        )
        .select_only()
        // Repository counts
        .column_as(
            repository::Column::Id.count_distinct(),
            "total_repositories"
        )
        // PR counts
        .column_as(
            pull_request::Column::Id.count(),
            "total_open_prs"
        )
        // Status counts (using CASE WHEN)
        .column_as(
            Expr::case(
                pull_request::Column::AmpelStatus.eq("green"),
                1
            )
            .finally(0)
            .sum(),
            "green_count"
        )
        .column_as(
            Expr::case(
                pull_request::Column::AmpelStatus.eq("yellow"),
                1
            )
            .finally(0)
            .sum(),
            "yellow_count"
        )
        .column_as(
            Expr::case(
                pull_request::Column::AmpelStatus.eq("red"),
                1
            )
            .finally(0)
            .sum(),
            "red_count"
        )
        // Provider counts
        .column_as(
            Expr::case(
                repository::Column::Provider.eq("github"),
                1
            )
            .finally(0)
            .sum(),
            "github_count"
        )
        .column_as(
            Expr::case(
                repository::Column::Provider.eq("gitlab"),
                1
            )
            .finally(0)
            .sum(),
            "gitlab_count"
        )
        .column_as(
            Expr::case(
                repository::Column::Provider.eq("bitbucket"),
                1
            )
            .finally(0)
            .sum(),
            "bitbucket_count"
        )
        .into_tuple::<(i64, i64, i64, i64, i64, i64, i64, i64)>()
        .one(&state.db)
        .await?
        .unwrap_or((0, 0, 0, 0, 0, 0, 0, 0));

    let (
        total_repositories,
        total_open_prs,
        green_count,
        yellow_count,
        red_count,
        github_count,
        gitlab_count,
        bitbucket_count,
    ) = summary;

    let duration = start.elapsed();

    tracing::info!(
        duration_ms = duration.as_millis(),
        total_repos = total_repositories,
        total_open_prs,
        "Dashboard summary generated (optimized)"
    );

    Ok(Json(ApiResponse::success(DashboardSummary {
        total_repositories: total_repositories as i32,
        total_open_prs: total_open_prs as i32,
        status_counts: StatusCounts {
            green: green_count as i32,
            yellow: yellow_count as i32,
            red: red_count as i32,
        },
        provider_counts: ProviderCounts {
            github: github_count as i32,
            gitlab: gitlab_count as i32,
            bitbucket: bitbucket_count as i32,
        },
    })))
}
```

### Raw SQL Equivalent

For reference, the generated SQL will look like:

```sql
SELECT
    COUNT(DISTINCT r.id) AS total_repositories,
    COUNT(pr.id) AS total_open_prs,
    SUM(CASE WHEN pr.ampel_status = 'green' THEN 1 ELSE 0 END) AS green_count,
    SUM(CASE WHEN pr.ampel_status = 'yellow' THEN 1 ELSE 0 END) AS yellow_count,
    SUM(CASE WHEN pr.ampel_status = 'red' THEN 1 ELSE 0 END) AS red_count,
    SUM(CASE WHEN r.provider = 'github' THEN 1 ELSE 0 END) AS github_count,
    SUM(CASE WHEN r.provider = 'gitlab' THEN 1 ELSE 0 END) AS gitlab_count,
    SUM(CASE WHEN r.provider = 'bitbucket' THEN 1 ELSE 0 END) AS bitbucket_count
FROM repositories r
LEFT JOIN pull_requests pr ON pr.repository_id = r.id AND pr.state = 'open'
WHERE r.user_id = $1 AND r.deleted_at IS NULL
GROUP BY r.user_id;
```

### Implementation Steps

1. **Create new handler file:**

   ```bash
   touch crates/ampel-api/src/handlers/dashboard_optimized.rs
   ```

2. **Add to mod.rs:**

   ```rust
   pub mod dashboard_optimized;
   ```

3. **A/B Test Setup:**

   ```rust
   // Add feature flag for gradual rollout
   if state.config.use_optimized_dashboard {
       dashboard_optimized::get_summary_optimized(state, auth).await
   } else {
       dashboard::get_summary(state, auth).await
   }
   ```

4. **Test with EXPLAIN:**

   ```sql
   EXPLAIN (ANALYZE, BUFFERS)
   SELECT ... -- paste full query
   ```

5. **Benchmark comparison:**

   ```bash
   cargo test --test test_dashboard_performance -- --nocapture
   ```

6. **Gradual rollout:**
   - 10% of users for 1 day
   - 50% of users for 1 day
   - 100% if no issues

### Expected Impact

| Metric                  | Before | After   | Improvement      |
| ----------------------- | ------ | ------- | ---------------- |
| Query count (100 repos) | 2101   | 1       | 99.95% reduction |
| Response time           | 500ms  | 30ms    | 94% faster       |
| Database load           | High   | Minimal | 99% reduction    |
| Memory usage            | 50MB   | 5MB     | 90% reduction    |

**Risk:** Medium - Requires careful testing of SQL logic

---

## P2: Redis Caching (HIGH PRIORITY)

### Problem

Dashboard summary is requested on every page load but data changes infrequently (only when PRs update).

### Solution

Implement Redis caching with smart invalidation:

```rust
// File: crates/ampel-api/src/handlers/dashboard_cached.rs

use redis::AsyncCommands;

pub async fn get_summary_cached(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    let cache_key = format!("dashboard:summary:{}", auth.user_id);

    // Try cache first
    if let Ok(cached) = state.redis.get::<_, String>(&cache_key).await {
        if let Ok(summary) = serde_json::from_str::<DashboardSummary>(&cached) {
            tracing::debug!("Cache hit for dashboard summary");

            // METRIC: Cache hit
            // counter!("ampel_dashboard_cache_hit_total").increment(1);

            return Ok(Json(ApiResponse::success(summary)));
        }
    }

    // Cache miss - compute
    tracing::debug!("Cache miss for dashboard summary");

    // METRIC: Cache miss
    // counter!("ampel_dashboard_cache_miss_total").increment(1);

    let summary = compute_summary(&state, auth.user_id).await?;

    // Store in cache with 5-minute TTL
    let cache_value = serde_json::to_string(&summary)?;
    let _: () = state.redis
        .set_ex(&cache_key, cache_value, 300)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to cache dashboard summary: {}", e);
        });

    Ok(Json(ApiResponse::success(summary)))
}

async fn compute_summary(
    state: &AppState,
    user_id: Uuid,
) -> Result<DashboardSummary, ApiError> {
    // Use optimized query from P1
    dashboard_optimized::compute_summary_internal(state, user_id).await
}
```

### Cache Invalidation Strategy

```rust
// File: crates/ampel-worker/src/jobs/cache_invalidation.rs

/// Invalidate dashboard cache when PR state changes
pub async fn invalidate_dashboard_cache(
    redis: &RedisClient,
    user_id: Uuid,
) -> Result<(), Error> {
    let cache_key = format!("dashboard:summary:{}", user_id);

    redis.del(&cache_key).await?;

    tracing::debug!(
        user_id = %user_id,
        "Invalidated dashboard cache"
    );

    Ok(())
}

// Call from webhook handlers
impl PullRequestWebhookHandler {
    async fn handle_pr_update(&self, event: PrUpdateEvent) -> Result<()> {
        // ... existing logic ...

        // Invalidate cache for all users with access to this repository
        let users = self.db.find_users_with_repo_access(event.repo_id).await?;
        for user in users {
            invalidate_dashboard_cache(&self.redis, user.id).await?;
        }

        Ok(())
    }
}
```

### Expected Impact

| Metric           | Before (No Cache) | After (Cache Hit) | Improvement    |
| ---------------- | ----------------- | ----------------- | -------------- |
| Response time    | 30ms              | 2ms               | 93% faster     |
| Database queries | 1                 | 0                 | 100% reduction |
| Server load      | High              | Minimal           | 95% reduction  |
| Cache hit rate   | N/A               | 85-95%            | N/A            |

**Cache Miss Performance:** Same as P1 (30ms with optimized query)

**Risk:** Low - Cache invalidation must be robust

---

## P3: Materialized Views (ADVANCED)

### Problem

Even with SQL aggregation, large datasets require scanning millions of rows.

### Solution

Create a materialized view that pre-aggregates dashboard data:

```sql
-- Migration: 20251224_create_dashboard_summary_view.sql

CREATE MATERIALIZED VIEW dashboard_summary_cache AS
SELECT
    u.id AS user_id,
    COUNT(DISTINCT r.id) AS total_repositories,
    COUNT(pr.id) FILTER (WHERE pr.state = 'open') AS total_open_prs,
    COUNT(pr.id) FILTER (WHERE pr.ampel_status = 'green' AND pr.state = 'open') AS green_count,
    COUNT(pr.id) FILTER (WHERE pr.ampel_status = 'yellow' AND pr.state = 'open') AS yellow_count,
    COUNT(pr.id) FILTER (WHERE pr.ampel_status = 'red' AND pr.state = 'open') AS red_count,
    COUNT(r.id) FILTER (WHERE r.provider = 'github') AS github_count,
    COUNT(r.id) FILTER (WHERE r.provider = 'gitlab') AS gitlab_count,
    COUNT(r.id) FILTER (WHERE r.provider = 'bitbucket') AS bitbucket_count,
    NOW() AS last_updated
FROM users u
LEFT JOIN repositories r ON r.user_id = u.id AND r.deleted_at IS NULL
LEFT JOIN pull_requests pr ON pr.repository_id = r.id
GROUP BY u.id;

-- Create unique index for fast lookups
CREATE UNIQUE INDEX idx_dashboard_summary_cache_user
ON dashboard_summary_cache(user_id);

-- Refresh concurrently to avoid blocking
REFRESH MATERIALIZED VIEW CONCURRENTLY dashboard_summary_cache;
```

### Refresh Strategy

```rust
// File: crates/ampel-worker/src/jobs/refresh_dashboard_cache.rs

use apalis::prelude::*;

/// Background job to refresh materialized view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshDashboardCacheJob;

#[async_trait]
impl Job for RefreshDashboardCacheJob {
    const NAME: &'static str = "refresh_dashboard_cache";

    async fn execute(&self, ctx: JobContext) -> Result<JobResult> {
        let db = ctx.data::<DatabaseConnection>()?;

        // Refresh materialized view
        db.execute_unprepared(
            "REFRESH MATERIALIZED VIEW CONCURRENTLY dashboard_summary_cache"
        ).await?;

        tracing::info!("Dashboard summary materialized view refreshed");

        Ok(JobResult::Success)
    }
}

// Schedule refresh every 1 minute
pub fn register_dashboard_cache_refresh(monitor: Monitor<PostgresStorage<RefreshDashboardCacheJob>>) {
    monitor
        .run_at("0 * * * * *", |_| async { RefreshDashboardCacheJob })
        .await;
}
```

### Query from Materialized View

```rust
pub async fn get_summary_from_view(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    let summary = sqlx::query_as!(
        DashboardSummaryCached,
        r#"
        SELECT
            total_repositories,
            total_open_prs,
            green_count,
            yellow_count,
            red_count,
            github_count,
            gitlab_count,
            bitbucket_count
        FROM dashboard_summary_cache
        WHERE user_id = $1
        "#,
        auth.user_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ApiResponse::success(summary.into())))
}
```

### Expected Impact

| Metric        | Before  | After    | Improvement   |
| ------------- | ------- | -------- | ------------- |
| Response time | 30ms    | 3ms      | 90% faster    |
| Database load | High    | Minimal  | 98% reduction |
| Scalability   | Limited | Infinite | N/A           |

**Trade-off:** Data may be up to 1 minute stale

**Risk:** Medium - Requires PostgreSQL-specific features

---

## P4: Parallel Processing (OPTIONAL)

### Problem

For users with many repositories, sequential processing is slow.

### Solution

Process repositories in parallel using Tokio:

```rust
use futures::future::join_all;

pub async fn get_summary_parallel(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;

    // Process repositories in parallel batches
    let chunk_size = 10; // Process 10 repos at a time
    let mut all_results = Vec::new();

    for chunk in repos.chunks(chunk_size) {
        let futures: Vec<_> = chunk.iter()
            .map(|repo| {
                let db = state.db.clone();
                let repo_id = repo.id;

                async move {
                    let prs = PrQueries::find_open_by_repository(&db, repo_id).await?;

                    // Calculate statuses for this repo
                    let mut statuses = Vec::new();
                    for pr_model in &prs {
                        let ci_checks = CICheckQueries::find_by_pull_request(&db, pr_model.id).await?;
                        let reviews = ReviewQueries::find_by_pull_request(&db, pr_model.id).await?;

                        let pr: ampel_core::models::PullRequest = pr_model.clone().into();
                        let ci_checks: Vec<_> = ci_checks.into_iter().map(Into::into).collect();
                        let reviews: Vec<_> = reviews.into_iter().map(Into::into).collect();

                        let status = AmpelStatus::for_pull_request(&pr, &ci_checks, &reviews);
                        statuses.push(status);
                    }

                    Ok::<_, ApiError>((repo.provider.clone(), statuses))
                }
            })
            .collect();

        let chunk_results = join_all(futures).await;
        all_results.extend(chunk_results);
    }

    // Aggregate results
    // ... rest of implementation
}
```

### Expected Impact

| Metric                    | Before | After (8 cores) | Improvement        |
| ------------------------- | ------ | --------------- | ------------------ |
| CPU usage                 | 12%    | 60%             | Higher utilization |
| Response time (100 repos) | 500ms  | 125ms           | 4x faster          |

**Note:** This optimization is superseded by P1 (SQL aggregation) which is more effective.

**Risk:** Medium - Can overload database with concurrent queries

---

## Implementation Roadmap

### Week 1: P0 - Database Indexing

- **Day 1:** Create migration, test locally
- **Day 2:** Deploy to staging, monitor
- **Day 3:** Deploy to production with monitoring
- **Days 4-5:** Measure impact, adjust if needed

**Success Criteria:** 70%+ query speedup

---

### Week 2: P1 - SQL Aggregation

- **Days 1-2:** Implement optimized handler
- **Day 3:** A/B testing framework
- **Day 4:** Test with 10% traffic
- **Day 5:** Gradual rollout to 100%

**Success Criteria:** 95%+ query reduction, < 50ms response time

---

### Week 3: P2 - Redis Caching

- **Days 1-2:** Implement cache layer
- **Day 3:** Implement cache invalidation
- **Days 4-5:** Test and deploy

**Success Criteria:** 85%+ cache hit rate, < 5ms cache hit response

---

### Week 4: P3 - Materialized Views (Optional)

- **Days 1-2:** Create materialized view
- **Day 3:** Implement refresh job
- **Days 4-5:** Test and deploy

**Success Criteria:** < 5ms response time with acceptable staleness

---

## Monitoring & Validation

### Key Metrics to Track

```promql
# Response time improvement
histogram_quantile(0.95,
  rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))

# Query count reduction (via database metrics)
rate(postgres_queries_total{query_type="dashboard"}[5m])

# Cache hit rate
rate(ampel_dashboard_cache_hit_total[5m]) /
  (rate(ampel_dashboard_cache_hit_total[5m]) +
   rate(ampel_dashboard_cache_miss_total[5m]))
```

### Success Criteria

| Optimization     | Metric              | Target           |
| ---------------- | ------------------- | ---------------- |
| **P0: Indexes**  | Query time          | < 10ms per query |
| **P1: SQL Agg**  | Total queries       | 1 (vs 2101)      |
| **P1: SQL Agg**  | Response time       | < 50ms           |
| **P2: Caching**  | Cache hit rate      | > 85%            |
| **P2: Caching**  | Response time (hit) | < 5ms            |
| **P3: Mat View** | Response time       | < 5ms            |

---

## Risk Assessment

| Optimization     | Risk Level | Mitigation                                    |
| ---------------- | ---------- | --------------------------------------------- |
| **P0: Indexes**  | Low        | Standard practice, monitor disk usage         |
| **P1: SQL Agg**  | Medium     | Extensive testing, A/B rollout, rollback plan |
| **P2: Caching**  | Medium     | Robust invalidation, cache warming            |
| **P3: Mat View** | High       | PostgreSQL-specific, staleness trade-off      |
| **P4: Parallel** | Low        | Connection pool limits, query timeout         |

---

## Conclusion

Implementing **P0** (Indexes) and **P1** (SQL Aggregation) will provide **95%+ performance improvement** with acceptable risk. These optimizations should be prioritized for immediate implementation.

**P2** (Caching) provides additional benefits for frequently accessed data and should be implemented after P1 is stable.

**P3** and **P4** are advanced optimizations that may not be necessary if P0-P2 achieve performance targets.

---

**Document Owner:** Backend Team
**Reviewers:** Database Team, QE Team
**Next Review:** After P1 implementation
