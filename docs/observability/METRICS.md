# Metrics Catalog

## Table of Contents

1. [Overview](#overview)
2. [Metrics Naming Convention](#metrics-naming-convention)
3. [Application Metrics](#application-metrics)
4. [Database Metrics](#database-metrics)
5. [Background Job Metrics](#background-job-metrics)
6. [Business Metrics](#business-metrics)
7. [System Metrics](#system-metrics)
8. [Custom Metrics Guide](#custom-metrics-guide)
9. [SLIs and SLOs](#slis-and-slos)
10. [Best Practices](#best-practices)

---

## Overview

This document catalogs all available metrics in the Ampel system, their meanings, and recommended usage for monitoring and alerting.

**Metrics Endpoint:** `http://localhost:8080/metrics`

**Format:** Prometheus text exposition format

---

## Metrics Naming Convention

Ampel follows Prometheus naming conventions:

```text
<namespace>_<subsystem>_<name>_<unit>_<suffix>
```

**Examples:**

- `ampel_http_requests_total` - Counter
- `ampel_http_request_duration_seconds` - Histogram
- `ampel_db_connections_active` - Gauge

**Suffixes:**

- `_total` - Counter (cumulative)
- `_seconds` - Time duration
- `_bytes` - Size
- `_ratio` - Percentage (0-1)

**Labels:**

Use labels for dimensions, not separate metrics:

```prometheus
# ✅ Good: One metric with labels
http_requests_total{method="GET", status="200", path="/api/prs"}

# ❌ Bad: Separate metrics per status
http_requests_200_total
http_requests_404_total
http_requests_500_total
```

---

## Application Metrics

### HTTP Server Metrics

#### `ampel_http_requests_total`

**Type:** Counter

**Description:** Total number of HTTP requests received

**Labels:**

- `method` - HTTP method (GET, POST, PUT, DELETE, PATCH)
- `path` - Request path (e.g., `/api/prs`, `/api/auth/login`)
- `status` - HTTP status code (200, 404, 500, etc.)

**Example:**

```prometheus
ampel_http_requests_total{method="GET",path="/api/prs",status="200"} 1234
ampel_http_requests_total{method="POST",path="/api/auth/login",status="401"} 15
```

**Usage:**

```prometheus
# Request rate per second
rate(ampel_http_requests_total[5m])

# Success rate
sum(rate(ampel_http_requests_total{status=~"2.."}[5m]))
/ sum(rate(ampel_http_requests_total[5m]))

# Error rate
sum(rate(ampel_http_requests_total{status=~"5.."}[5m]))
/ sum(rate(ampel_http_requests_total[5m]))
```

---

#### `ampel_http_request_duration_seconds`

**Type:** Histogram

**Description:** HTTP request duration in seconds

**Labels:**

- `method` - HTTP method
- `path` - Request path
- `status` - HTTP status code

**Buckets:** 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10

**Example:**

```prometheus
ampel_http_request_duration_seconds_bucket{method="GET",path="/api/prs",le="0.1"} 500
ampel_http_request_duration_seconds_bucket{method="GET",path="/api/prs",le="0.5"} 800
ampel_http_request_duration_seconds_sum{method="GET",path="/api/prs"} 245.5
ampel_http_request_duration_seconds_count{method="GET",path="/api/prs"} 1000
```

**Usage:**

```prometheus
# P95 latency
histogram_quantile(0.95,
  rate(ampel_http_request_duration_seconds_bucket[5m])
)

# P99 latency
histogram_quantile(0.99,
  rate(ampel_http_request_duration_seconds_bucket[5m])
)

# Average latency
rate(ampel_http_request_duration_seconds_sum[5m])
/ rate(ampel_http_request_duration_seconds_count[5m])
```

---

#### `ampel_http_request_size_bytes`

**Type:** Histogram

**Description:** HTTP request payload size in bytes

**Labels:**

- `method` - HTTP method
- `path` - Request path

**Buckets:** 100, 1000, 10000, 100000, 1000000, 10000000

---

#### `ampel_http_response_size_bytes`

**Type:** Histogram

**Description:** HTTP response payload size in bytes

**Labels:**

- `method` - HTTP method
- `path` - Request path
- `status` - HTTP status code

---

#### `ampel_http_connections_active`

**Type:** Gauge

**Description:** Number of currently active HTTP connections

**Example:**

```prometheus
ampel_http_connections_active 45
```

---

### Authentication Metrics

#### `ampel_auth_login_attempts_total`

**Type:** Counter

**Description:** Total number of login attempts

**Labels:**

- `success` - true/false
- `provider` - github/gitlab/bitbucket/local

**Example:**

```prometheus
ampel_auth_login_attempts_total{success="true",provider="github"} 1500
ampel_auth_login_attempts_total{success="false",provider="local"} 23
```

---

#### `ampel_auth_token_refreshes_total`

**Type:** Counter

**Description:** Total number of JWT token refreshes

**Labels:**

- `success` - true/false

---

#### `ampel_auth_active_sessions`

**Type:** Gauge

**Description:** Number of currently active user sessions

---

## Database Metrics

### Connection Pool Metrics

#### `ampel_db_connections_active`

**Type:** Gauge

**Description:** Number of active database connections

**Example:**

```prometheus
ampel_db_connections_active{database="ampel"} 8
```

---

#### `ampel_db_connections_idle`

**Type:** Gauge

**Description:** Number of idle database connections in pool

---

#### `ampel_db_connections_max`

**Type:** Gauge

**Description:** Maximum number of database connections allowed

**Example:**

```prometheus
ampel_db_connections_max{database="ampel"} 20
```

**Usage:**

```prometheus
# Connection pool utilization percentage
(ampel_db_connections_active / ampel_db_connections_max) * 100

# Alert when >90% utilized
(ampel_db_connections_active / ampel_db_connections_max) > 0.9
```

---

#### `ampel_db_connection_wait_duration_seconds`

**Type:** Histogram

**Description:** Time spent waiting for a database connection

**Buckets:** 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1

---

### Query Metrics

#### `ampel_db_query_duration_seconds`

**Type:** Histogram

**Description:** Database query execution duration

**Labels:**

- `operation` - Query operation (select, insert, update, delete)
- `table` - Primary table being queried

**Example:**

```prometheus
ampel_db_query_duration_seconds_bucket{operation="select",table="pull_requests",le="0.1"} 950
ampel_db_query_duration_seconds_sum{operation="select",table="pull_requests"} 87.5
```

**Usage:**

```prometheus
# Slow queries (>1s)
sum(rate(ampel_db_query_duration_seconds_count{
  le="+Inf",
  duration_seconds > 1
}[5m]))

# P95 query latency by table
histogram_quantile(0.95,
  sum(rate(ampel_db_query_duration_seconds_bucket[5m])) by (le, table)
)
```

---

#### `ampel_db_queries_total`

**Type:** Counter

**Description:** Total number of database queries executed

**Labels:**

- `operation` - Query operation
- `table` - Primary table
- `status` - success/error

---

#### `ampel_db_query_rows_returned`

**Type:** Histogram

**Description:** Number of rows returned by SELECT queries

**Labels:**

- `table` - Table being queried

**Buckets:** 1, 10, 100, 1000, 10000, 100000

---

### Transaction Metrics

#### `ampel_db_transactions_total`

**Type:** Counter

**Description:** Total number of database transactions

**Labels:**

- `status` - committed/rolled_back

---

#### `ampel_db_transaction_duration_seconds`

**Type:** Histogram

**Description:** Transaction duration from BEGIN to COMMIT/ROLLBACK

---

## Background Job Metrics

### Apalis Job Queue Metrics

#### `ampel_jobs_queued`

**Type:** Gauge

**Description:** Number of jobs currently in queue

**Labels:**

- `job_type` - Type of job (pr_sync, metrics_collection, webhook_delivery)
- `priority` - Job priority (low, normal, high)

**Example:**

```prometheus
ampel_jobs_queued{job_type="pr_sync",priority="normal"} 42
ampel_jobs_queued{job_type="metrics_collection",priority="low"} 5
```

**Alert:**

```prometheus
# Queue backlog alert
ampel_jobs_queued > 1000
```

---

#### `ampel_jobs_processed_total`

**Type:** Counter

**Description:** Total number of jobs processed

**Labels:**

- `job_type` - Type of job
- `status` - success/failure/retry

**Example:**

```prometheus
ampel_jobs_processed_total{job_type="pr_sync",status="success"} 5420
ampel_jobs_processed_total{job_type="pr_sync",status="failure"} 12
ampel_jobs_processed_total{job_type="pr_sync",status="retry"} 8
```

**Usage:**

```prometheus
# Job success rate
sum(rate(ampel_jobs_processed_total{status="success"}[5m]))
/ sum(rate(ampel_jobs_processed_total[5m]))

# Job failure rate
sum(rate(ampel_jobs_processed_total{status="failure"}[5m]))
/ sum(rate(ampel_jobs_processed_total[5m]))
```

---

#### `ampel_job_duration_seconds`

**Type:** Histogram

**Description:** Job execution duration

**Labels:**

- `job_type` - Type of job
- `status` - success/failure

**Buckets:** 0.1, 0.5, 1, 5, 10, 30, 60, 120, 300

---

#### `ampel_job_retry_count`

**Type:** Histogram

**Description:** Number of retries before job succeeded or permanently failed

**Labels:**

- `job_type` - Type of job

**Buckets:** 0, 1, 2, 3, 5, 10

---

## Business Metrics

### Pull Request Metrics

#### `ampel_prs_total`

**Type:** Gauge

**Description:** Total number of pull requests by status

**Labels:**

- `status` - ampel status (green, yellow, red)
- `state` - PR state (open, merged, closed)
- `provider` - github/gitlab/bitbucket

**Example:**

```prometheus
ampel_prs_total{status="green",state="open",provider="github"} 150
ampel_prs_total{status="yellow",state="open",provider="gitlab"} 45
ampel_prs_total{status="red",state="open",provider="bitbucket"} 10
```

---

#### `ampel_pr_time_to_merge_seconds`

**Type:** Histogram

**Description:** Time from PR creation to merge

**Labels:**

- `provider` - github/gitlab/bitbucket
- `repository` - Repository name

**Buckets:** 3600, 7200, 14400, 28800, 86400, 172800, 604800 (1h, 2h, 4h, 8h, 1d, 2d, 1w)

**Example:**

```prometheus
ampel_pr_time_to_merge_seconds_sum{provider="github"} 432000
ampel_pr_time_to_merge_seconds_count{provider="github"} 50
```

**Usage:**

```prometheus
# Average time to merge (in hours)
(
  rate(ampel_pr_time_to_merge_seconds_sum[24h])
  / rate(ampel_pr_time_to_merge_seconds_count[24h])
) / 3600

# P95 time to merge
histogram_quantile(0.95,
  rate(ampel_pr_time_to_merge_seconds_bucket[24h])
)
```

---

#### `ampel_pr_time_to_first_review_seconds`

**Type:** Histogram

**Description:** Time from PR creation to first review

**Labels:**

- `provider` - github/gitlab/bitbucket

**Buckets:** 600, 1800, 3600, 7200, 14400, 28800, 86400 (10m, 30m, 1h, 2h, 4h, 8h, 1d)

---

#### `ampel_pr_review_rounds`

**Type:** Histogram

**Description:** Number of review rounds before approval

**Buckets:** 0, 1, 2, 3, 5, 10

---

#### `ampel_pr_comments_count`

**Type:** Histogram

**Description:** Number of comments on a PR

**Buckets:** 0, 5, 10, 20, 50, 100

---

### Repository Metrics

#### `ampel_repos_total`

**Type:** Gauge

**Description:** Total number of repositories tracked

**Labels:**

- `provider` - github/gitlab/bitbucket
- `organization` - Organization name

---

#### `ampel_repos_synced_total`

**Type:** Counter

**Description:** Total number of successful repository syncs

**Labels:**

- `provider` - github/gitlab/bitbucket

---

#### `ampel_repos_sync_errors_total`

**Type:** Counter

**Description:** Total number of repository sync errors

**Labels:**

- `provider` - github/gitlab/bitbucket
- `error_type` - Rate limit, authentication, network, etc.

---

#### `ampel_repo_last_sync_timestamp`

**Type:** Gauge

**Description:** Unix timestamp of last successful repository sync

**Labels:**

- `repository` - Repository ID
- `provider` - github/gitlab/bitbucket

**Usage:**

```prometheus
# Repositories not synced in >1 hour
(time() - ampel_repo_last_sync_timestamp) > 3600

# Time since last sync (in minutes)
(time() - ampel_repo_last_sync_timestamp) / 60
```

---

### Provider API Metrics

#### `ampel_provider_api_requests_total`

**Type:** Counter

**Description:** Total API requests to Git providers

**Labels:**

- `provider` - github/gitlab/bitbucket
- `endpoint` - API endpoint
- `status` - HTTP status code

---

#### `ampel_provider_api_duration_seconds`

**Type:** Histogram

**Description:** API request duration to Git providers

**Labels:**

- `provider` - github/gitlab/bitbucket
- `endpoint` - API endpoint

---

#### `ampel_provider_rate_limit_remaining`

**Type:** Gauge

**Description:** Remaining API rate limit for provider

**Labels:**

- `provider` - github/gitlab/bitbucket
- `token_id` - Token identifier (hashed)

**Example:**

```prometheus
ampel_provider_rate_limit_remaining{provider="github",token_id="abc123"} 4500
```

**Alert:**

```prometheus
# Rate limit warning
ampel_provider_rate_limit_remaining < 1000
```

---

## System Metrics

### Process Metrics

#### `process_resident_memory_bytes`

**Type:** Gauge

**Description:** Resident memory size in bytes (RSS)

---

#### `process_virtual_memory_bytes`

**Type:** Gauge

**Description:** Virtual memory size in bytes

---

#### `process_cpu_seconds_total`

**Type:** Counter

**Description:** Total CPU time consumed in seconds

**Usage:**

```prometheus
# CPU usage percentage
rate(process_cpu_seconds_total[1m]) * 100
```

---

#### `process_open_fds`

**Type:** Gauge

**Description:** Number of open file descriptors

---

#### `process_max_fds`

**Type:** Gauge

**Description:** Maximum number of file descriptors

**Alert:**

```prometheus
# File descriptor exhaustion
process_open_fds / process_max_fds > 0.8
```

---

### Runtime Metrics (Tokio)

#### `tokio_tasks_active`

**Type:** Gauge

**Description:** Number of active async tasks

---

#### `tokio_tasks_spawned_total`

**Type:** Counter

**Description:** Total number of tasks spawned

---

#### `tokio_blocking_queue_depth`

**Type:** Gauge

**Description:** Number of tasks waiting in blocking thread pool queue

---

## Custom Metrics Guide

### Adding New Metrics

#### 1. Define Metric

```rust
use prometheus::{Counter, Histogram, Gauge, Registry};
use once_cell::sync::Lazy;

// Counter example
static PR_MERGED_COUNTER: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "ampel_prs_merged_total",
        "Total number of PRs merged"
    ).expect("metric creation failed")
});

// Histogram example
static PR_MERGE_TIME: Lazy<Histogram> = Lazy::new(|| {
    Histogram::with_opts(
        HistogramOpts::new(
            "ampel_pr_merge_time_seconds",
            "Time from PR creation to merge"
        )
        .buckets(vec![3600.0, 7200.0, 14400.0, 86400.0, 604800.0])
    ).expect("metric creation failed")
});

// Gauge example
static ACTIVE_USERS: Lazy<Gauge> = Lazy::new(|| {
    Gauge::new(
        "ampel_active_users",
        "Number of currently active users"
    ).expect("metric creation failed")
});
```

#### 2. Register Metric

```rust
use prometheus::Registry;

pub fn register_metrics(registry: &Registry) -> Result<()> {
    registry.register(Box::new(PR_MERGED_COUNTER.clone()))?;
    registry.register(Box::new(PR_MERGE_TIME.clone()))?;
    registry.register(Box::new(ACTIVE_USERS.clone()))?;
    Ok(())
}
```

#### 3. Instrument Code

```rust
// Increment counter
PR_MERGED_COUNTER.inc();

// Observe histogram value (seconds)
let duration = (merged_at - created_at).num_seconds() as f64;
PR_MERGE_TIME.observe(duration);

// Set gauge value
ACTIVE_USERS.set(current_user_count as f64);
```

#### 4. Add Labels

```rust
use prometheus::IntCounterVec;

static PR_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(
        Opts::new("ampel_prs_total", "Total PRs by status"),
        &["status", "provider"]
    ).expect("metric creation failed")
});

// Use with labels
PR_COUNTER.with_label_values(&["green", "github"]).inc();
PR_COUNTER.with_label_values(&["yellow", "gitlab"]).inc();
```

---

## SLIs and SLOs

### Service Level Indicators (SLIs)

#### Availability SLI

```prometheus
# Uptime percentage
100 * (
  sum(up{job="ampel-api"})
  / count(up{job="ampel-api"})
)
```

**Target:** 99.9% (3 nines)

---

#### Request Success Rate SLI

```prometheus
# Success rate percentage
100 * (
  sum(rate(ampel_http_requests_total{status=~"2.."}[5m]))
  / sum(rate(ampel_http_requests_total[5m]))
)
```

**Target:** 99.5%

---

#### Latency SLI

```prometheus
# P95 latency
histogram_quantile(0.95,
  rate(ampel_http_request_duration_seconds_bucket[5m])
)
```

**Target:** <500ms for 95% of requests

---

### Service Level Objectives (SLOs)

| SLI          | SLO    | Error Budget (30 days)                 |
| ------------ | ------ | -------------------------------------- |
| Availability | 99.9%  | 43.2 minutes                           |
| Success Rate | 99.5%  | 216,000 failed requests (at 100 req/s) |
| P95 Latency  | <500ms | 36,000 slow requests                   |

**Error Budget Calculation:**

```prometheus
# Availability error budget remaining
(
  (1 - 0.999) * 30 * 24 * 60  # Total allowed downtime
  - (30 * 24 * 60 - sum(up{job="ampel-api"}) * 30 * 24 * 60)  # Actual downtime
)
```

---

## Best Practices

### 1. Metric Naming

```prometheus
# ✅ Good: Clear, consistent naming
ampel_http_requests_total
ampel_db_query_duration_seconds
ampel_jobs_processed_total

# ❌ Bad: Inconsistent, unclear
requests
query_time
jobsProcessed
```

### 2. Use Appropriate Metric Types

```prometheus
# ✅ Counter for cumulative values
ampel_requests_total

# ✅ Gauge for current value
ampel_connections_active

# ✅ Histogram for distributions
ampel_request_duration_seconds
```

### 3. Label Cardinality

```prometheus
# ✅ Good: Low cardinality labels
{method="GET", status="200"}

# ❌ Bad: High cardinality (unbounded)
{user_id="550e8400-...", email="user@example.com"}
```

**Rule:** Keep unique label combinations <10,000 per metric

### 4. Use Histograms for Percentiles

```prometheus
# ✅ Good: Histogram for latency percentiles
histogram_quantile(0.95,
  rate(ampel_request_duration_seconds_bucket[5m])
)

# ❌ Bad: Gauge with average (loses distribution)
ampel_request_duration_seconds_avg
```

### 5. Document Your Metrics

Every custom metric should have:

- Clear description
- Unit in metric name
- Example usage in PromQL
- Alert threshold recommendations

---

## Resources

- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [Prometheus Metric Types](https://prometheus.io/docs/concepts/metric_types/)
- [PromQL Basics](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Ampel Monitoring Guide](MONITORING.md)
- [Ampel Observability Guide](OBSERVABILITY.md)

---

**Last Updated:** 2025-12-22
**Maintainer:** Ampel Metrics Team
