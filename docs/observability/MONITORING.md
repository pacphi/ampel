# Monitoring Guide

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Monitoring Stack](#monitoring-stack)
4. [Available Metrics](#available-metrics)
5. [Grafana Dashboards](#grafana-dashboards)
6. [Alert Rules](#alert-rules)
7. [Local vs Production](#local-vs-production)
8. [Troubleshooting](#troubleshooting)

---

## Overview

Ampel uses a comprehensive monitoring stack to track application health, performance, and business metrics. This document covers how to set up, access, and use the monitoring infrastructure.

**Key Technologies:**

- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization and dashboards
- **Loki**: Log aggregation
- **Rust tracing**: Structured logging and instrumentation

---

## Quick Start

### Local Development

```bash
# Start monitoring stack with Docker
docker-compose -f docker/docker-compose.monitoring.yml up -d

# Access Grafana
open http://localhost:3001
# Default credentials: admin/admin (change on first login)

# Access Prometheus
open http://localhost:9090

# View application logs
docker logs -f ampel-api
docker logs -f ampel-worker
```

### Check Application Health

```bash
# Health check endpoint
curl http://localhost:8080/health

# Metrics endpoint
curl http://localhost:8080/metrics
```

---

## Monitoring Stack

### Architecture

```
┌─────────────┐
│   Ampel     │
│  (API/Worker)│──── Metrics ────▶ Prometheus ──────┐
│             │                                      │
│  tracing    │──── Logs ──────▶ Loki              │
└─────────────┘                                      │
                                                     ▼
                                              ┌─────────────┐
                                              │   Grafana   │
                                              │ (Dashboards)│
                                              └─────────────┘
```

### Components

#### 1. Application Instrumentation

Ampel uses the `tracing` ecosystem for structured logging and metrics:

```rust
// Example: Instrumented function
#[tracing::instrument(skip(db))]
async fn fetch_pull_requests(db: &DatabaseConnection) -> Result<Vec<PullRequest>> {
    tracing::info!("Fetching pull requests");
    // Function body
}
```

**Log Levels:**

- `ERROR`: Critical failures requiring immediate attention
- `WARN`: Non-critical issues (degraded performance, retries)
- `INFO`: Normal operational events
- `DEBUG`: Detailed diagnostic information
- `TRACE`: Very detailed trace information

#### 2. Prometheus Metrics

Metrics are exposed at `http://localhost:8080/metrics` in Prometheus format.

**Metric Types:**

- **Counter**: Monotonically increasing values (requests, errors)
- **Gauge**: Values that can increase or decrease (active connections)
- **Histogram**: Distribution of values (request duration, payload size)

#### 3. Grafana Dashboards

Pre-configured dashboards for:

- Application overview (requests, errors, latency)
- Database performance (query time, connection pool)
- Background jobs (queue depth, processing time)
- Business metrics (PR metrics, sync status)

---

## Available Metrics

### HTTP Server Metrics

```prometheus
# Total HTTP requests
http_requests_total{method="GET",path="/api/prs",status="200"} 1234

# Request duration histogram
http_request_duration_seconds_bucket{le="0.1"} 500
http_request_duration_seconds_bucket{le="0.5"} 800
http_request_duration_seconds_bucket{le="1.0"} 1000

# Active connections
http_connections_active 15
```

### Database Metrics

```prometheus
# Database connection pool
db_connections_active{database="ampel"} 8
db_connections_idle{database="ampel"} 2
db_connections_max{database="ampel"} 10

# Query execution time
db_query_duration_seconds{query="select_prs"} 0.045

# Query errors
db_query_errors_total{error_type="timeout"} 3
```

### Background Job Metrics

```prometheus
# Job queue depth
apalis_jobs_queued{job_type="pr_sync"} 42

# Job processing time
apalis_job_duration_seconds{job_type="metrics_collection"} 1.234

# Job success/failure
apalis_jobs_processed_total{job_type="pr_sync",status="success"} 1000
apalis_jobs_processed_total{job_type="pr_sync",status="failure"} 5
```

### Business Metrics

```prometheus
# PR metrics
ampel_prs_total{status="green"} 150
ampel_prs_total{status="yellow"} 45
ampel_prs_total{status="red"} 10

# Repository sync status
ampel_repos_synced_total{provider="github"} 25
ampel_repos_sync_errors_total{provider="gitlab"} 2

# Average PR metrics
ampel_pr_time_to_merge_seconds_avg 86400  # 1 day
ampel_pr_time_to_first_review_seconds_avg 3600  # 1 hour
```

### System Metrics

```prometheus
# Memory usage
process_resident_memory_bytes 52428800  # 50 MB

# CPU usage
process_cpu_seconds_total 125.5

# File descriptors
process_open_fds 45
process_max_fds 1024
```

---

## Grafana Dashboards

### Accessing Dashboards

1. Navigate to http://localhost:3001 (local) or your production Grafana URL
2. Login with credentials (default: admin/admin)
3. Go to **Dashboards** → **Browse**

### Pre-configured Dashboards

#### 1. Ampel Overview

**Metrics Displayed:**

- Total requests/sec
- Error rate (%)
- P50, P95, P99 latency
- Active users
- PR status distribution

**PromQL Examples:**

```prometheus
# Request rate
rate(http_requests_total[5m])

# Error rate percentage
100 * (
  rate(http_requests_total{status=~"5.."}[5m])
  / rate(http_requests_total[5m])
)

# P95 latency
histogram_quantile(0.95,
  rate(http_request_duration_seconds_bucket[5m])
)
```

#### 2. Database Performance

**Metrics:**

- Query latency by operation
- Connection pool utilization
- Slow query count (>1s)
- Lock wait time

**Visualization:**

```
┌─────────────────────────────────────┐
│  Connection Pool Status             │
│  ▓▓▓▓▓▓▓▓░░  80% utilized          │
└─────────────────────────────────────┘
┌─────────────────────────────────────┐
│  Query Duration (P95)               │
│  150ms ──────────────▲──────        │
│        0     50    100    150  200  │
└─────────────────────────────────────┘
```

#### 3. Background Jobs

**Metrics:**

- Jobs queued by type
- Average processing time
- Success/failure rate
- Retry count

**Alert Thresholds:**

- Queue depth > 1000 (warning)
- Queue depth > 5000 (critical)
- Failure rate > 5% (warning)
- Failure rate > 20% (critical)

#### 4. Business Metrics Dashboard

**PR Analytics:**

- Average time to merge (trend over time)
- Average time to first review
- PR distribution by status (green/yellow/red)
- Most active repositories

**Repository Sync:**

- Last sync timestamp per repo
- Sync success rate
- Failed syncs requiring retry

---

## Alert Rules

### Critical Alerts

```yaml
groups:
  - name: ampel_critical
    interval: 30s
    rules:
      # API is down
      - alert: APIDown
        expr: up{job="ampel-api"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: 'Ampel API is down'
          description: 'API has been down for more than 1 minute'

      # High error rate
      - alert: HighErrorRate
        expr: |
          100 * (
            rate(http_requests_total{status=~"5.."}[5m])
            / rate(http_requests_total[5m])
          ) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: 'High HTTP error rate (>10%)'

      # Database connection pool exhausted
      - alert: DatabasePoolExhausted
        expr: |
          db_connections_active / db_connections_max > 0.95
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: 'Database connection pool nearly exhausted'
```

### Warning Alerts

```yaml
- name: ampel_warnings
  interval: 1m
  rules:
    # High latency
    - alert: HighLatency
      expr: |
        histogram_quantile(0.95,
          rate(http_request_duration_seconds_bucket[5m])
        ) > 1.0
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: 'P95 latency > 1s'

    # High job queue depth
    - alert: HighJobQueueDepth
      expr: |
        apalis_jobs_queued > 1000
      for: 10m
      labels:
        severity: warning
      annotations:
        summary: 'Job queue depth exceeds 1000'

    # Stale repository sync
    - alert: StaleRepositorySync
      expr: |
        (time() - ampel_repo_last_sync_timestamp) > 3600
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "Repository hasn't synced in over 1 hour"
```

### Alert Notification Channels

Configure in Grafana → Alerting → Contact points:

- **Slack**: #ampel-alerts
- **Email**: ops@example.com
- **PagerDuty**: For critical alerts only

---

## Local vs Production

### Local Development

**Setup:**

```bash
# Start monitoring stack
docker-compose -f docker/docker-compose.monitoring.yml up -d

# Run application with debug logging
export RUST_LOG=debug,ampel=trace
make dev-api
make dev-worker
```

**Access:**

- Grafana: http://localhost:3001
- Prometheus: http://localhost:9090
- Loki: http://localhost:3100

**Note:** Local monitoring uses ephemeral storage. Data is lost when containers are removed.

### Production

**Setup:**

Production uses persistent volumes and secure authentication.

```yaml
# docker-compose.production.yml
services:
  prometheus:
    volumes:
      - /data/prometheus:/prometheus
    environment:
      - PROMETHEUS_RETENTION_TIME=30d

  grafana:
    volumes:
      - /data/grafana:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD}
      - GF_AUTH_ANONYMOUS_ENABLED=false
```

**Environment Variables:**

```bash
# Production .env
GRAFANA_ADMIN_PASSWORD=<strong-password>
PROMETHEUS_RETENTION_TIME=30d
LOKI_RETENTION_PERIOD=14d
ALERT_WEBHOOK_URL=https://hooks.slack.com/services/...
```

**Data Retention:**

- Metrics: 30 days
- Logs: 14 days
- Alerts: 90 days

**Backup:**

```bash
# Backup Prometheus data
docker exec prometheus tar czf /backup/prometheus-$(date +%Y%m%d).tar.gz /prometheus

# Backup Grafana dashboards
docker exec grafana grafana-cli admin export-dashboard > grafana-backup.json
```

---

## Troubleshooting

### Metrics Not Appearing

**Check metrics endpoint:**

```bash
# Verify metrics are being exported
curl http://localhost:8080/metrics | grep http_requests_total

# Expected output:
# http_requests_total{method="GET",path="/health",status="200"} 42
```

**Check Prometheus scraping:**

1. Open Prometheus UI: http://localhost:9090
2. Go to **Status** → **Targets**
3. Verify `ampel-api` target is **UP**

**Common Issues:**

- **Firewall blocking port 8080**: Check Docker network settings
- **Metrics endpoint returning 404**: Verify metrics middleware is enabled
- **Invalid metric format**: Check for metric naming violations (use snake_case, no spaces)

### High Memory Usage

**Diagnose:**

```bash
# Check container memory
docker stats ampel-api

# Check heap allocation
curl http://localhost:8080/debug/pprof/heap
```

**Solutions:**

- Reduce log retention period
- Adjust `RUST_LOG` to `info` instead of `debug`
- Increase container memory limit
- Review code for memory leaks (use `valgrind` or `heaptrack`)

### Slow Queries

**Find slow queries in logs:**

```bash
# Filter logs for slow queries
docker logs ampel-api 2>&1 | grep "slow query"
```

**Check query metrics:**

```prometheus
# Queries taking >1s
db_query_duration_seconds > 1.0
```

**Solutions:**

- Add database indexes
- Optimize query (avoid N+1 queries)
- Implement pagination
- Use query result caching (Redis)

### Missing Logs

**Verify logging configuration:**

```rust
// Check tracing-subscriber initialization in main.rs
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .json()  // Structured JSON logs for production
    .init();
```

**Environment:**

```bash
# Enable verbose logging
export RUST_LOG=trace

# Filter specific modules
export RUST_LOG=ampel=debug,sea_orm=info
```

**Loki not receiving logs:**

```bash
# Check Loki health
curl http://localhost:3100/ready

# Verify promtail is running
docker logs promtail
```

### Dashboard Not Loading

**Check Grafana datasources:**

1. Grafana → Configuration → Data sources
2. Verify Prometheus datasource is configured:
   - URL: http://prometheus:9090 (Docker network name)
   - Access: Server (not browser)

**Test connection:**

```bash
# From within Grafana container
docker exec grafana curl http://prometheus:9090/-/healthy
```

**Import dashboard JSON:**

```bash
# If dashboard is missing, re-import
curl -X POST http://localhost:3001/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @grafana/dashboards/ampel-overview.json
```

### Alert Not Firing

**Check alert rules in Prometheus:**

1. Prometheus UI → Alerts
2. Verify rule is loaded and expression evaluates correctly
3. Check "for" duration hasn't been met yet

**Test alert manually:**

```prometheus
# Simulate high error rate
100 * (
  rate(http_requests_total{status=~"5.."}[5m])
  / rate(http_requests_total[5m])
) > 10
```

**Verify notification channel:**

```bash
# Send test notification from Grafana
Grafana → Alerting → Contact points → Test
```

---

## Advanced Topics

### Custom Metrics

Add custom business metrics:

```rust
use prometheus::{Counter, register_counter};

lazy_static! {
    static ref PR_MERGED_COUNTER: Counter = register_counter!(
        "ampel_prs_merged_total",
        "Total number of PRs merged"
    ).unwrap();
}

// Increment when PR is merged
PR_MERGED_COUNTER.inc();
```

### Distributed Tracing

For microservices or multi-service deployments, consider adding OpenTelemetry:

```toml
[dependencies]
opentelemetry = "0.20"
tracing-opentelemetry = "0.21"
```

See [OBSERVABILITY.md](OBSERVABILITY.md) for distributed tracing setup.

### External Monitoring

For production, integrate with external monitoring:

- **Datadog**: APM and infrastructure monitoring
- **New Relic**: Application performance monitoring
- **Sentry**: Error tracking and crash reporting

---

## Resources

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Dashboards](https://grafana.com/docs/grafana/latest/dashboards/)
- [Rust tracing](https://docs.rs/tracing/latest/tracing/)
- [Ampel Observability Guide](OBSERVABILITY.md)
- [Ampel Metrics Catalog](METRICS.md)

---

**Last Updated:** 2025-12-22
**Maintainer:** Ampel DevOps Team
