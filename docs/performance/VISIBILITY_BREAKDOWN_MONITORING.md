# Visibility Breakdown Tiles - Performance Monitoring & Metrics

**Document Version:** 1.0
**Date:** December 24, 2025
**Feature:** Visibility Breakdown Tiles (Repository Filter by Ampel Status)
**Related:** [Git Diff View Integration Plan](../planning/GIT_DIFF_VIEW_INTEGRATION.md)

---

## Table of Contents

1. [Overview](#overview)
2. [Backend Logging](#backend-logging)
3. [Prometheus Metrics](#prometheus-metrics)
4. [Performance Testing](#performance-testing)
5. [Optimization Recommendations](#optimization-recommendations)
6. [Monitoring Dashboard](#monitoring-dashboard)
7. [Alerting Strategy](#alerting-strategy)

---

## Overview

The visibility breakdown tiles feature provides real-time filtering of repositories by their Ampel status (Green/Yellow/Red). This document outlines the performance monitoring, logging, and metrics strategy to ensure the feature meets performance targets.

### Performance Goals

| Metric                  | Target  | Critical Threshold |
| ----------------------- | ------- | ------------------ |
| API Response Time (P50) | < 200ms | < 500ms            |
| API Response Time (P95) | < 500ms | < 1000ms           |
| API Response Time (P99) | < 800ms | < 2000ms           |
| Database Query Time     | < 100ms | < 300ms            |
| CPU Usage               | < 30%   | < 70%              |
| Memory Usage            | < 200MB | < 500MB            |

---

## Backend Logging

### Current Implementation

The `get_summary` handler in `crates/ampel-api/src/handlers/dashboard.rs` now includes comprehensive structured logging:

```rust
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id))]
pub async fn get_summary(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    use std::time::Instant;
    let start = Instant::now();

    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;
    tracing::debug!(repo_count = repos.len(), "Retrieved repositories for user");

    // ... calculation logic ...

    let duration = start.elapsed();

    tracing::info!(
        duration_ms = duration.as_millis(),
        total_repos = repos.len(),
        total_open_prs,
        green_count,
        yellow_count,
        red_count,
        github_count,
        gitlab_count,
        bitbucket_count,
        "Dashboard summary generated"
    );

    // ... return response ...
}
```

### Log Fields

| Field             | Type  | Description                            |
| ----------------- | ----- | -------------------------------------- |
| `user_id`         | UUID  | User requesting the summary            |
| `duration_ms`     | u128  | Total request duration in milliseconds |
| `total_repos`     | usize | Number of repositories processed       |
| `total_open_prs`  | i32   | Total open pull requests               |
| `green_count`     | i32   | PRs with Green (ready to merge) status |
| `yellow_count`    | i32   | PRs with Yellow (in progress) status   |
| `red_count`       | i32   | PRs with Red (blocked) status          |
| `github_count`    | i32   | Repositories from GitHub               |
| `gitlab_count`    | i32   | Repositories from GitLab               |
| `bitbucket_count` | i32   | Repositories from Bitbucket            |

### Log Levels

- **DEBUG**: Repository retrieval, intermediate calculations
- **INFO**: Successful summary generation with metrics
- **WARN**: Slow response times (> 500ms)
- **ERROR**: Database errors, calculation failures

### Example Log Output (JSON)

```json
{
  "timestamp": "2025-12-24T10:30:15.123Z",
  "level": "INFO",
  "target": "ampel_api::handlers::dashboard",
  "fields": {
    "message": "Dashboard summary generated",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "duration_ms": 245,
    "total_repos": 42,
    "total_open_prs": 127,
    "green_count": 45,
    "yellow_count": 62,
    "red_count": 20,
    "github_count": 30,
    "gitlab_count": 8,
    "bitbucket_count": 4
  }
}
```

---

## Prometheus Metrics

### Metric Definitions

The following Prometheus metrics are documented and ready to be implemented:

#### 1. Response Duration Histogram

```rust
histogram!("ampel_dashboard_summary_duration_seconds").record(duration.as_secs_f64());
```

**Type:** Histogram
**Description:** Response time for dashboard summary endpoint
**Buckets:** `[0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0]` seconds
**Labels:** None (endpoint-specific)

**PromQL Queries:**

```promql
# P50 latency
histogram_quantile(0.50, rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))

# P95 latency
histogram_quantile(0.95, rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))

# P99 latency
histogram_quantile(0.99, rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))

# Average latency
rate(ampel_dashboard_summary_duration_seconds_sum[5m]) / rate(ampel_dashboard_summary_duration_seconds_count[5m])
```

---

#### 2. Breakdown Count by Visibility

```rust
counter!("ampel_dashboard_breakdown_total", &[("visibility", "green")]).increment(green_count as u64);
counter!("ampel_dashboard_breakdown_total", &[("visibility", "yellow")]).increment(yellow_count as u64);
counter!("ampel_dashboard_breakdown_total", &[("visibility", "red")]).increment(red_count as u64);
```

**Type:** Counter
**Description:** Total count of PRs by visibility status
**Labels:**

- `visibility`: `green`, `yellow`, or `red`

**PromQL Queries:**

```promql
# Rate of green PRs per second
rate(ampel_dashboard_breakdown_total{visibility="green"}[5m])

# Percentage of red PRs
sum(rate(ampel_dashboard_breakdown_total{visibility="red"}[5m])) / sum(rate(ampel_dashboard_breakdown_total[5m])) * 100

# Total breakdown by status
sum by (visibility) (ampel_dashboard_breakdown_total)
```

---

#### 3. Error Counter

```rust
counter!("ampel_dashboard_errors_total", &[("error_type", error_type)]).increment(1);
```

**Type:** Counter
**Description:** Total errors in dashboard summary endpoint
**Labels:**

- `error_type`: `database`, `calculation`, `auth`, `other`

**PromQL Queries:**

```promql
# Error rate
rate(ampel_dashboard_errors_total[5m])

# Error percentage
sum(rate(ampel_dashboard_errors_total[5m])) / sum(rate(http_requests_total{path="/api/dashboard/summary"}[5m])) * 100
```

---

### Metric Collection Location

Metrics are collected in `crates/ampel-api/src/handlers/dashboard.rs` at line 90-103, marked with:

```rust
// METRICS COLLECTION POINT:
// Future Prometheus metrics to be collected here
```

To enable metrics, uncomment the metric collection code and ensure the `metrics` crate is imported:

```rust
use metrics::{counter, histogram};
```

---

## Performance Testing

### Test Scenarios

#### 1. Small Dataset (Baseline)

**Configuration:**

- Repositories: 10
- Open PRs per repo: 5
- Total PRs: 50

**Expected Performance:**

- Response time: < 100ms
- Database queries: 11 (1 repo query + 10 PR queries)

**Test Command:**

```bash
cargo test --test test_dashboard_performance -- --nocapture test_summary_small_dataset
```

---

#### 2. Medium Dataset (Typical)

**Configuration:**

- Repositories: 50
- Open PRs per repo: 10
- Total PRs: 500

**Expected Performance:**

- Response time: < 300ms
- Database queries: 51 (1 repo query + 50 PR queries)

**Test Command:**

```bash
cargo test --test test_dashboard_performance -- --nocapture test_summary_medium_dataset
```

---

#### 3. Large Dataset (Stress Test)

**Configuration:**

- Repositories: 100
- Open PRs per repo: 20
- Total PRs: 2000

**Expected Performance:**

- Response time: < 500ms (CRITICAL THRESHOLD)
- Database queries: 101 (1 repo query + 100 PR queries)
- Memory usage: < 200MB

**Test Command:**

```bash
cargo test --test test_dashboard_performance -- --nocapture test_summary_large_dataset
```

---

#### 4. Very Large Dataset (Capacity Test)

**Configuration:**

- Repositories: 200
- Open PRs per repo: 30
- Total PRs: 6000

**Expected Performance:**

- Response time: < 1000ms
- Database queries: 201
- Memory usage: < 500MB

**Test Command:**

```bash
cargo test --test test_dashboard_performance -- --nocapture test_summary_very_large_dataset
```

---

### Performance Test Implementation

Create `crates/ampel-api/tests/test_dashboard_performance.rs`:

```rust
mod common;

use common::{create_test_app, TestDb};
use axum::{body::Body, http::{header, Request, StatusCode}};
use tower::ServiceExt;
use std::time::Instant;

#[tokio::test]
async fn test_summary_large_dataset_performance() {
    if TestDb::skip_if_sqlite() {
        return;
    }

    let test_db = TestDb::new().await.expect("Failed to create test DB");
    test_db.run_migrations().await.expect("Failed to run migrations");

    // Setup: Create 100 repositories with 20 open PRs each
    // ... setup code ...

    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;

    let start = Instant::now();

    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/summary")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    // Assert response is successful
    assert_eq!(response.status(), StatusCode::OK);

    // Assert performance target met
    assert!(
        duration.as_millis() < 500,
        "Response time {}ms exceeds 500ms target",
        duration.as_millis()
    );

    println!("✅ Large dataset test passed: {}ms", duration.as_millis());

    test_db.cleanup().await;
}
```

---

### Load Testing with k6

For realistic load testing, use k6:

```javascript
// scripts/load-test-dashboard.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 10 }, // Ramp-up to 10 users
    { duration: '1m', target: 50 }, // Ramp-up to 50 users
    { duration: '2m', target: 50 }, // Stay at 50 users
    { duration: '30s', target: 0 }, // Ramp-down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests < 500ms
    http_req_failed: ['rate<0.01'], // < 1% error rate
  },
};

export default function () {
  const token = __ENV.ACCESS_TOKEN;

  const res = http.get('http://localhost:8080/api/dashboard/summary', {
    headers: { Authorization: `Bearer ${token}` },
  });

  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
    'has green count': (r) => r.json('data.statusCounts.green') !== undefined,
  });

  sleep(1);
}
```

**Run:**

```bash
k6 run scripts/load-test-dashboard.js
```

---

## Optimization Recommendations

### 1. Database Query Optimization

#### Current Bottleneck

The handler makes **N+1 queries**:

- 1 query to fetch all repositories
- 1 query per repository to fetch open PRs
- 2 queries per PR (CI checks + reviews)

For 100 repositories with 10 PRs each:

- **Total queries:** 1 + 100 + (100 × 10 × 2) = **2101 queries**

#### Recommendation: SQL Aggregation

Replace the nested loop with a single SQL query using JOINs and aggregations:

```rust
// Optimized implementation
pub async fn get_summary_optimized(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    use sea_orm::*;

    // Single query to get all data
    let summary: (i64, i64, i64, i64, i64, i64, i64, i64) = ampel_db::repository::Entity::find()
        .left_join(ampel_db::pull_request::Entity)
        .left_join(ampel_db::ci_check::Entity)
        .left_join(ampel_db::review::Entity)
        .filter(ampel_db::repository::Column::UserId.eq(auth.user_id))
        .filter(ampel_db::pull_request::Column::State.eq("open"))
        .select_only()
        .column_as(ampel_db::repository::Column::Id.count(), "total_repos")
        .column_as(ampel_db::pull_request::Column::Id.count(), "total_prs")
        .column_as(
            Expr::case(
                Expr::col(ampel_db::pull_request::Column::AmpelStatus).eq("green"),
                1
            ).into(),
            "green_count"
        )
        // ... similar for yellow, red, providers
        .into_tuple()
        .one(&state.db)
        .await?
        .unwrap_or_default();

    // Map tuple to response struct
    // ...
}
```

**Performance Improvement:**

- Queries reduced from 2101 to **1 query**
- Expected response time: **< 100ms** for 100 repos

---

### 2. Redis Caching Strategy

#### Cache Key Design

```
dashboard:summary:{user_id}:{version}
```

**Version:** Increment on any PR state change

#### Implementation

```rust
use redis::AsyncCommands;

pub async fn get_summary_cached(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    let cache_key = format!("dashboard:summary:{}:v1", auth.user_id);

    // Try cache first
    if let Ok(cached) = state.redis.get::<_, String>(&cache_key).await {
        if let Ok(summary) = serde_json::from_str(&cached) {
            tracing::debug!("Cache hit for dashboard summary");
            return Ok(Json(ApiResponse::success(summary)));
        }
    }

    // Cache miss - compute
    let summary = compute_summary(&state, auth.user_id).await?;

    // Store in cache with 5-minute TTL
    let _ = state.redis.set_ex(&cache_key, serde_json::to_string(&summary)?, 300).await;

    Ok(Json(ApiResponse::success(summary)))
}
```

**Performance Improvement:**

- Cache hit response time: **< 10ms**
- Cache invalidation on PR updates via webhooks

---

### 3. Database Indexing

#### Required Indexes

```sql
-- Index for user repository lookups
CREATE INDEX idx_repositories_user_id ON repositories(user_id);

-- Index for open PR queries
CREATE INDEX idx_pull_requests_repo_state ON pull_requests(repository_id, state);

-- Index for CI checks per PR
CREATE INDEX idx_ci_checks_pr_id ON ci_checks(pull_request_id);

-- Index for reviews per PR
CREATE INDEX idx_reviews_pr_id ON reviews(pull_request_id);

-- Composite index for status calculation
CREATE INDEX idx_pull_requests_status ON pull_requests(repository_id, state, ampel_status);
```

**Performance Improvement:**

- Query execution time reduced by **70-90%**

---

### 4. Parallel Processing

For users with many repositories, process in parallel:

```rust
use futures::future::join_all;

let futures: Vec<_> = repos.iter()
    .map(|repo| {
        let db = state.db.clone();
        async move {
            let prs = PrQueries::find_open_by_repository(&db, repo.id).await?;
            // Calculate status...
            Ok((repo.id, status))
        }
    })
    .collect();

let results = join_all(futures).await;
```

**Performance Improvement:**

- With 8 CPU cores, ~**4x speedup** for CPU-bound calculations

---

### 5. Materialized View (Advanced)

For very large datasets, create a materialized view:

```sql
CREATE MATERIALIZED VIEW dashboard_summary_cache AS
SELECT
    u.id AS user_id,
    COUNT(DISTINCT r.id) AS total_repositories,
    COUNT(pr.id) FILTER (WHERE pr.state = 'open') AS total_open_prs,
    COUNT(pr.id) FILTER (WHERE pr.ampel_status = 'green') AS green_count,
    COUNT(pr.id) FILTER (WHERE pr.ampel_status = 'yellow') AS yellow_count,
    COUNT(pr.id) FILTER (WHERE pr.ampel_status = 'red') AS red_count,
    COUNT(r.id) FILTER (WHERE r.provider = 'github') AS github_count,
    COUNT(r.id) FILTER (WHERE r.provider = 'gitlab') AS gitlab_count,
    COUNT(r.id) FILTER (WHERE r.provider = 'bitbucket') AS bitbucket_count
FROM users u
LEFT JOIN repositories r ON r.user_id = u.id
LEFT JOIN pull_requests pr ON pr.repository_id = r.id
GROUP BY u.id;

-- Refresh on PR state changes
REFRESH MATERIALIZED VIEW CONCURRENTLY dashboard_summary_cache;
```

**Performance Improvement:**

- Response time: **< 5ms** (simple SELECT)
- Trade-off: Slightly stale data (refreshed every minute or on triggers)

---

## Monitoring Dashboard

### Grafana Dashboard JSON

Create a Grafana dashboard with the following panels:

#### Panel 1: Response Time (P50, P95, P99)

```promql
# P50
histogram_quantile(0.50, rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))

# P95
histogram_quantile(0.95, rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))

# P99
histogram_quantile(0.99, rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))
```

#### Panel 2: PR Breakdown Distribution (Stacked Area)

```promql
sum by (visibility) (rate(ampel_dashboard_breakdown_total[5m]))
```

#### Panel 3: Error Rate

```promql
sum(rate(ampel_dashboard_errors_total[5m])) / sum(rate(http_requests_total{path="/api/dashboard/summary"}[5m])) * 100
```

#### Panel 4: Request Rate

```promql
sum(rate(http_requests_total{path="/api/dashboard/summary"}[5m]))
```

---

## Alerting Strategy

### Alert Rules

#### 1. High Response Time

```yaml
- alert: DashboardSummarySlowResponse
  expr: histogram_quantile(0.95, rate(ampel_dashboard_summary_duration_seconds_bucket[5m])) > 0.5
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: 'Dashboard summary API is slow'
    description: 'P95 response time is {{ $value }}s (threshold: 0.5s)'
```

#### 2. Error Rate Spike

```yaml
- alert: DashboardSummaryHighErrorRate
  expr: sum(rate(ampel_dashboard_errors_total[5m])) / sum(rate(http_requests_total{path="/api/dashboard/summary"}[5m])) > 0.01
  for: 3m
  labels:
    severity: critical
  annotations:
    summary: 'High error rate in dashboard summary API'
    description: 'Error rate is {{ $value | humanizePercentage }} (threshold: 1%)'
```

#### 3. Database Connection Issues

```yaml
- alert: DashboardDatabaseConnectionFailure
  expr: rate(ampel_dashboard_errors_total{error_type="database"}[5m]) > 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: 'Database connection failures in dashboard API'
```

---

## Performance Monitoring Checklist

- [x] Structured logging added to `get_summary` handler
- [x] Performance timing metrics collected
- [x] Prometheus metrics documented
- [ ] Metrics collection code uncommented and enabled
- [ ] Performance tests created for 100+ repositories
- [ ] Database indexes verified
- [ ] Redis caching implemented
- [ ] Grafana dashboard created
- [ ] Alert rules configured
- [ ] Load testing with k6 completed
- [ ] Optimization baseline established

---

## Next Steps

1. **Enable Metrics Collection** (Week 1)
   - Uncomment metric collection code
   - Deploy to staging environment
   - Verify metrics appear in Prometheus

2. **Implement SQL Aggregation** (Week 2)
   - Write optimized query
   - A/B test against current implementation
   - Roll out if performance improves

3. **Add Redis Caching** (Week 3)
   - Implement cache layer
   - Set up cache invalidation on webhooks
   - Monitor cache hit rate

4. **Create Performance Tests** (Week 4)
   - Implement test scenarios in `test_dashboard_performance.rs`
   - Add to CI pipeline
   - Set performance SLOs

---

## References

- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [Rust Tracing Documentation](https://docs.rs/tracing/latest/tracing/)
- [Axum Middleware Guide](https://docs.rs/axum/latest/axum/middleware/)
- [k6 Load Testing](https://k6.io/docs/)
- [Grafana Dashboard Design](https://grafana.com/docs/grafana/latest/dashboards/)

---

**Document Status:** Draft
**Next Review:** 2025-01-07
**Owner:** QE Team
