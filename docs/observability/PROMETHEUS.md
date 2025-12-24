# Prometheus Setup and Configuration

Complete guide to Prometheus configuration, queries, and best practices for Ampel.

## Table of Contents

1. [Overview](#overview)
2. [Configuration](#configuration)
3. [Scrape Configuration](#scrape-configuration)
4. [PromQL Queries](#promql-queries)
5. [Recording Rules](#recording-rules)
6. [Alert Rules](#alert-rules)
7. [Best Practices](#best-practices)

---

## Overview

Prometheus is the metrics storage and query engine for Ampel's observability stack.

**Key Features:**

- Time-series database optimized for metrics
- Powerful PromQL query language
- Pull-based metrics collection
- Alert rule evaluation
- Service discovery

**Access:** http://localhost:9090 (local development)

---

## Configuration

### Basic Configuration

Located at `/monitoring/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s # How often to scrape targets
  evaluation_interval: 15s # How often to evaluate rules

  external_labels:
    environment: 'production'
    cluster: 'ampel'

# Alert manager configuration
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

# Load alert rules
rule_files:
  - '/etc/prometheus/alerts/*.yml'

# Scrape configurations
scrape_configs:
  - job_name: 'ampel-api'
    static_configs:
      - targets: ['api:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
    scrape_timeout: 10s

  - job_name: 'ampel-worker'
    static_configs:
      - targets: ['worker:8081']
    metrics_path: '/metrics'

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
```

### Retention Settings

Configure data retention in docker-compose or command line:

```yaml
# docker/docker-compose.monitoring.yml
prometheus:
  command:
    - '--config.file=/etc/prometheus/prometheus.yml'
    - '--storage.tsdb.path=/prometheus'
    - '--storage.tsdb.retention.time=30d' # Keep 30 days
    - '--storage.tsdb.retention.size=50GB' # Or 50GB max
    - '--web.enable-lifecycle'
```

**Retention Policies:**

- **Development:** 7 days
- **Staging:** 15 days
- **Production:** 30 days (or more)

---

## Scrape Configuration

### Static Targets

Simple configuration for known services:

```yaml
scrape_configs:
  - job_name: 'ampel-api'
    static_configs:
      - targets:
          - 'api-1:8080'
          - 'api-2:8080'
          - 'api-3:8080'
```

### Relabeling

Modify labels during scraping:

```yaml
scrape_configs:
  - job_name: 'ampel-api'
    static_configs:
      - targets: ['api:8080']
    relabel_configs:
      # Add instance label
      - source_labels: [__address__]
        target_label: instance
        regex: '([^:]+):\d+'
        replacement: '${1}'

      # Add custom labels
      - target_label: 'service'
        replacement: 'ampel-api'

      - target_label: 'environment'
        replacement: 'production'
```

### Metric Relabeling

Filter or modify metrics after scraping:

```yaml
scrape_configs:
  - job_name: 'ampel-api'
    metric_relabel_configs:
      # Drop high-cardinality metrics
      - source_labels: [__name__]
        regex: 'http_request_duration_seconds_bucket'
        action: drop

      # Rename label
      - source_labels: [old_label]
        target_label: new_label
```

---

## PromQL Queries

### Basic Queries

**Instant Vector (current value):**

```promql
# Current request rate
rate(http_requests_total[5m])

# Error count
http_requests_total{status=~"5.."}

# Database connections
db_connections_active
```

**Range Vector (time series):**

```promql
# Last 5 minutes of requests
http_requests_total[5m]

# Last hour of latency
http_request_duration_seconds[1h]
```

### Aggregation

```promql
# Sum across all instances
sum(http_requests_total)

# Average by path
avg(http_request_duration_seconds) by (path)

# Count by status code
count(http_requests_total) by (status)

# Max latency by endpoint
max(http_request_duration_seconds) by (path)
```

### Rate and Increase

```promql
# Rate per second
rate(http_requests_total[5m])

# Total increase over time
increase(http_requests_total[1h])

# Rate of rate (acceleration)
rate(rate(http_requests_total[5m])[5m:1m])
```

### Percentiles (Histograms)

```promql
# P50 (median)
histogram_quantile(0.50,
  rate(http_request_duration_seconds_bucket[5m])
)

# P95
histogram_quantile(0.95,
  rate(http_request_duration_seconds_bucket[5m])
)

# P99
histogram_quantile(0.99,
  rate(http_request_duration_seconds_bucket[5m])
)

# P95 by endpoint
histogram_quantile(0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, path)
)
```

### Filtering and Matching

```promql
# Exact match
http_requests_total{method="GET", path="/api/prs"}

# Regex match
http_requests_total{status=~"5.."}  # 5xx errors
http_requests_total{path=~"/api/.*"}  # All /api paths

# Negative match
http_requests_total{status!="200"}  # Not 200
http_requests_total{path!~"/health|/ready"}  # Not health checks
```

### Arithmetic

```promql
# Error rate percentage
100 * (
  sum(rate(http_requests_total{status=~"5.."}[5m]))
  / sum(rate(http_requests_total[5m]))
)

# Connection pool utilization
(db_connections_active / db_connections_max) * 100

# Request rate difference
rate(http_requests_total[5m]) - rate(http_requests_total[5m] offset 1h)
```

### Time Shifting

```promql
# Current vs 1 hour ago
http_requests_total - http_requests_total offset 1h

# Week-over-week comparison
rate(http_requests_total[5m]) / rate(http_requests_total[5m] offset 1w)
```

### Useful Patterns

**Top N slowest endpoints:**

```promql
topk(5,
  histogram_quantile(0.95,
    sum(rate(http_request_duration_seconds_bucket[5m])) by (le, path)
  )
)
```

**Error budget (SLO tracking):**

```promql
# 99.9% availability means 0.1% error budget
100 - (
  100 * (
    sum(rate(http_requests_total{status=~"5.."}[30d]))
    / sum(rate(http_requests_total[30d]))
  )
) >= 99.9
```

**Request rate by status code:**

```promql
sum(rate(http_requests_total[5m])) by (status)
```

**Predict linear trend:**

```promql
# Predict memory usage in 4 hours
predict_linear(process_resident_memory_bytes[1h], 4*3600)
```

---

## Recording Rules

Pre-compute expensive queries for faster dashboards.

### Configuration

Located at `/monitoring/rules/recording.yml`:

```yaml
groups:
  - name: ampel_recording_rules
    interval: 30s
    rules:
      # Pre-compute request rate
      - record: job:http_requests:rate5m
        expr: sum(rate(http_requests_total[5m])) by (job)

      # Pre-compute error rate
      - record: job:http_errors:rate5m
        expr: sum(rate(http_requests_total{status=~"5.."}[5m])) by (job)

      # Pre-compute P95 latency
      - record: job:http_latency:p95
        expr: |
          histogram_quantile(0.95,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (le, job)
          )
```

### Best Practices

1. **Name consistently:** Use `level:metric:operations` format
2. **Keep cardinality low:** Aggregate appropriately
3. **Set reasonable intervals:** Match dashboard refresh rates
4. **Document purpose:** Add comments explaining complex rules

---

## Alert Rules

See [MONITORING.md](MONITORING.md) for detailed alert configuration.

### Example Alert Rule

```yaml
groups:
  - name: ampel_alerts
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: |
          100 * (
            sum(rate(http_requests_total{status=~"5.."}[5m]))
            / sum(rate(http_requests_total[5m]))
          ) > 5
        for: 5m
        labels:
          severity: critical
          team: platform
        annotations:
          summary: 'High HTTP error rate ({{ $value | humanizePercentage }})'
          description: 'Error rate is {{ $value | humanizePercentage }} (threshold: 5%)'
          dashboard: 'https://grafana.example.com/d/ampel-overview'
```

---

## Best Practices

### 1. Query Optimization

**Use recording rules for expensive queries:**

```promql
# ❌ Bad: Expensive calculation every dashboard refresh
histogram_quantile(0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, job, path)
)

# ✅ Good: Pre-computed recording rule
job_path:http_latency:p95
```

**Limit time ranges:**

```promql
# ❌ Bad: Query last 7 days
rate(http_requests_total[7d])

# ✅ Good: Query last 5 minutes
rate(http_requests_total[5m])
```

### 2. Label Management

**Keep cardinality low:**

```promql
# ✅ Good: ~10 unique values
{method="GET", status="200"}

# ❌ Bad: Millions of unique values
{user_id="550e8400-...", request_id="..."}
```

**Use appropriate labels:**

- Fixed dimensions: method, status, path
- No timestamps or UUIDs in labels
- Aggregate high-cardinality data

### 3. Scrape Configuration

**Set appropriate intervals:**

- **Default:** 15s (good for most services)
- **High-frequency:** 5s (critical services)
- **Low-frequency:** 60s (batch jobs, exporters)

**Configure timeouts:**

```yaml
scrape_timeout: 10s # Must be < scrape_interval
```

**Handle slow targets:**

```yaml
scrape_configs:
  - job_name: 'slow-service'
    scrape_interval: 30s
    scrape_timeout: 20s
```

### 4. Storage Optimization

**Reduce retention for development:**

```bash
--storage.tsdb.retention.time=7d
```

**Limit storage size:**

```bash
--storage.tsdb.retention.size=10GB
```

**Enable compression:**

```bash
--storage.tsdb.wal-compression
```

### 5. Monitoring Prometheus

**Track Prometheus health:**

```promql
# Scrape duration
scrape_duration_seconds

# Samples ingested
prometheus_tsdb_head_samples_appended_total

# Memory usage
process_resident_memory_bytes{job="prometheus"}

# Failed scrapes
up == 0
```

---

## Troubleshooting

### Metrics Not Appearing

**Check target status:**

```bash
# View in UI
open http://localhost:9090/targets

# Or via API
curl http://localhost:9090/api/v1/targets
```

**Common issues:**

- Target down (check `up` metric)
- Wrong scrape endpoint
- Network connectivity
- Firewall blocking port

### High Memory Usage

**Identify high-cardinality metrics:**

```promql
# Count unique time series per metric
count({__name__=~".+"}) by (__name__)
```

**Solutions:**

- Reduce label cardinality
- Drop unnecessary metrics
- Decrease retention time
- Use recording rules

### Slow Queries

**Check query performance:**

```bash
# Enable query logging
--query.log-queries-longer-than=1s
```

**Optimize queries:**

- Use recording rules
- Limit time ranges
- Reduce aggregation dimensions
- Avoid complex regex

### Missing Data Points

**Check scrape interval:**

```promql
# Time since last scrape
time() - timestamp(up)
```

**Verify scrape success:**

```promql
# Scrape success rate
avg(up) by (job)
```

---

## Resources

- [Prometheus Documentation](https://prometheus.io/docs/)
- [PromQL Basics](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Best Practices](https://prometheus.io/docs/practices/naming/)
- [Query Examples](https://prometheus.io/docs/prometheus/latest/querying/examples/)

---

**Last Updated:** 2025-12-22
**Maintainer:** Ampel Platform Team
