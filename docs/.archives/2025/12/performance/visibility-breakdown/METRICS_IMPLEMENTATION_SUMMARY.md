# Prometheus Metrics Implementation Summary

**Date:** December 24, 2025
**Feature:** Dashboard Visibility Breakdown Monitoring
**Status:** ✅ Implemented

---

## Overview

Prometheus metrics have been successfully implemented for the dashboard visibility breakdown feature as documented in [VISIBILITY_BREAKDOWN_MONITORING.md](./VISIBILITY_BREAKDOWN_MONITORING.md).

## Implemented Metrics

### 1. Response Duration Histogram

**Metric Name:** `ampel_dashboard_summary_duration_seconds`
**Type:** Histogram
**Description:** Tracks response time for the dashboard summary endpoint
**Location:** `crates/ampel-api/src/handlers/dashboard.rs:282`

```rust
histogram!("ampel_dashboard_summary_duration_seconds").record(duration.as_secs_f64());
```

**Buckets:** Default Prometheus buckets `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]`

**Usage Examples:**

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

### 2. Breakdown Count by Visibility Status

**Metric Name:** `ampel_dashboard_breakdown_total`
**Type:** Counter
**Description:** Total count of PRs by Ampel status (green/yellow/red)
**Location:** `crates/ampel-api/src/handlers/dashboard.rs:285-289`

```rust
counter!("ampel_dashboard_breakdown_total", "visibility" => "green").increment(green_count as u64);
counter!("ampel_dashboard_breakdown_total", "visibility" => "yellow").increment(yellow_count as u64);
counter!("ampel_dashboard_breakdown_total", "visibility" => "red").increment(red_count as u64);
```

**Labels:**

- `visibility`: `green`, `yellow`, or `red`

**Usage Examples:**

```promql
# Rate of green PRs per second
rate(ampel_dashboard_breakdown_total{visibility="green"}[5m])

# Percentage of red (blocked) PRs
sum(rate(ampel_dashboard_breakdown_total{visibility="red"}[5m])) / sum(rate(ampel_dashboard_breakdown_total[5m])) * 100

# Total breakdown by status
sum by (visibility) (ampel_dashboard_breakdown_total)
```

---

### 3. Error Counter

**Metric Name:** `ampel_dashboard_errors_total`
**Type:** Counter
**Description:** Total errors in dashboard operations
**Location:** `crates/ampel-api/src/handlers/mod.rs:101`

```rust
counter!("ampel_dashboard_errors_total", "error_type" => "database").increment(1);
```

**Labels:**

- `error_type`: `database`, `calculation`, `auth`, `other`

**Current Implementation:** Database errors only (in `From<sea_orm::DbErr> for ApiError`)

**Usage Examples:**

```promql
# Error rate per second
rate(ampel_dashboard_errors_total[5m])

# Error percentage
sum(rate(ampel_dashboard_errors_total[5m])) / sum(rate(http_requests_total{path="/api/dashboard/summary"}[5m])) * 100

# Errors by type
sum by (error_type) (ampel_dashboard_errors_total)
```

---

## Files Modified

### 1. `crates/ampel-api/src/handlers/dashboard.rs`

**Changes:**

- Added `metrics::{counter, histogram}` import
- Added `Deserialize` trait to structs for Redis caching compatibility
- Implemented histogram recording for response duration
- Implemented counter recording for PR status breakdown

**Lines Modified:**

- Line 2: Added metrics import
- Line 3: Added Deserialize import
- Lines 12, 25, 33, 41: Added Deserialize trait to structs
- Lines 282-289: Metrics collection code

### 2. `crates/ampel-api/src/handlers/mod.rs`

**Changes:**

- Added `metrics::counter` import
- Implemented error counter for database errors

**Lines Modified:**

- Line 15: Added metrics import
- Line 101: Database error counter

---

## Metrics Endpoint

The `/metrics` endpoint is already configured and available at:

```
GET http://localhost:8080/metrics
```

**Implementation:** `crates/ampel-api/src/observability.rs:98-100`

```rust
pub async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}
```

**Route:** Configured in `crates/ampel-api/src/routes/mod.rs:20`

---

## Testing the Metrics

### 1. Start the API Server

```bash
make dev-api
# API server starts on http://localhost:8080
```

### 2. Call the Dashboard Endpoint

```bash
# Login to get access token
TOKEN=$(curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password"}' \
  | jq -r '.data.accessToken')

# Call dashboard summary endpoint
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/dashboard/summary
```

### 3. Check Metrics Endpoint

```bash
curl http://localhost:8080/metrics | grep ampel_dashboard
```

**Expected Output:**

```prometheus
# HELP ampel_dashboard_summary_duration_seconds ampel_dashboard_summary_duration_seconds
# TYPE ampel_dashboard_summary_duration_seconds histogram
ampel_dashboard_summary_duration_seconds_bucket{le="0.005"} 0
ampel_dashboard_summary_duration_seconds_bucket{le="0.01"} 0
ampel_dashboard_summary_duration_seconds_bucket{le="0.025"} 1
ampel_dashboard_summary_duration_seconds_bucket{le="0.05"} 1
ampel_dashboard_summary_duration_seconds_bucket{le="0.1"} 1
ampel_dashboard_summary_duration_seconds_bucket{le="0.25"} 1
ampel_dashboard_summary_duration_seconds_bucket{le="0.5"} 1
ampel_dashboard_summary_duration_seconds_bucket{le="1.0"} 1
ampel_dashboard_summary_duration_seconds_bucket{le="2.5"} 1
ampel_dashboard_summary_duration_seconds_bucket{le="5.0"} 1
ampel_dashboard_summary_duration_seconds_bucket{le="10.0"} 1
ampel_dashboard_summary_duration_seconds_bucket{le="+Inf"} 1
ampel_dashboard_summary_duration_seconds_sum 0.023456
ampel_dashboard_summary_duration_seconds_count 1

# HELP ampel_dashboard_breakdown_total ampel_dashboard_breakdown_total
# TYPE ampel_dashboard_breakdown_total counter
ampel_dashboard_breakdown_total{visibility="green"} 45
ampel_dashboard_breakdown_total{visibility="yellow"} 62
ampel_dashboard_breakdown_total{visibility="red"} 20

# HELP ampel_dashboard_errors_total ampel_dashboard_errors_total
# TYPE ampel_dashboard_errors_total counter
ampel_dashboard_errors_total{error_type="database"} 0
```

---

## Prometheus Configuration

### scrape_configs

Add this configuration to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'ampel-api'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

### Alert Rules

Create `alerts/dashboard.rules.yml`:

```yaml
groups:
  - name: dashboard_performance
    interval: 30s
    rules:
      # High response time alert
      - alert: DashboardSummarySlowResponse
        expr: histogram_quantile(0.95, rate(ampel_dashboard_summary_duration_seconds_bucket[5m])) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: 'Dashboard summary API is slow'
          description: 'P95 response time is {{ $value }}s (threshold: 0.5s)'

      # High error rate alert
      - alert: DashboardSummaryHighErrorRate
        expr: |
          sum(rate(ampel_dashboard_errors_total[5m]))
          / sum(rate(http_requests_total{path="/api/dashboard/summary"}[5m])) > 0.01
        for: 3m
        labels:
          severity: critical
        annotations:
          summary: 'High error rate in dashboard summary API'
          description: 'Error rate is {{ $value | humanizePercentage }} (threshold: 1%)'

      # Database connection failures
      - alert: DashboardDatabaseConnectionFailure
        expr: rate(ampel_dashboard_errors_total{error_type="database"}[5m]) > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: 'Database connection failures in dashboard API'
```

---

## Grafana Dashboard

### Panel 1: Response Time (Line Graph)

**Query:**

```promql
# P50 (median)
histogram_quantile(0.50, rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))

# P95
histogram_quantile(0.95, rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))

# P99
histogram_quantile(0.99, rate(ampel_dashboard_summary_duration_seconds_bucket[5m]))
```

**Visualization:** Time series with legend showing P50, P95, P99

---

### Panel 2: PR Breakdown (Stacked Area Chart)

**Query:**

```promql
sum by (visibility) (rate(ampel_dashboard_breakdown_total[5m]))
```

**Visualization:** Stacked area chart with green/yellow/red colors

---

### Panel 3: Error Rate (Single Stat)

**Query:**

```promql
sum(rate(ampel_dashboard_errors_total[5m]))
/ sum(rate(http_requests_total{path="/api/dashboard/summary"}[5m])) * 100
```

**Visualization:** Single stat showing percentage with thresholds (green < 0.1%, yellow < 1%, red >= 1%)

---

## Build and Test Status

### Compilation

✅ **Status:** Successful

- All code compiles without errors
- Unit tests pass: 4/4 passed
- Dependencies resolved correctly

### Dependency Versions

```toml
metrics = "0.24"
metrics-exporter-prometheus = "0.16"
metrics-util = "0.19"
```

---

## Next Steps

### Immediate (Week 1)

- [x] Enable metrics collection code
- [ ] Deploy to staging environment
- [ ] Verify metrics appear in Prometheus
- [ ] Create Grafana dashboard
- [ ] Configure alert rules

### Short-term (Week 2-3)

- [ ] Add metrics to other error types (calculation, auth)
- [ ] Implement SQL query optimization (reduce N+1 queries)
- [ ] Add Redis caching layer with metrics
- [ ] Create performance benchmarks

### Long-term (Month 2+)

- [ ] Add request rate metrics
- [ ] Implement distributed tracing with OpenTelemetry
- [ ] Create SLO dashboards
- [ ] Implement auto-scaling based on metrics

---

## References

- [Prometheus Metrics Best Practices](https://prometheus.io/docs/practices/naming/)
- [Rust metrics Crate Documentation](https://docs.rs/metrics/latest/metrics/)
- [metrics-exporter-prometheus Documentation](https://docs.rs/metrics-exporter-prometheus/latest/metrics_exporter_prometheus/)
- [Original Monitoring Plan](./VISIBILITY_BREAKDOWN_MONITORING.md)

---

**Document Status:** Complete
**Implementation Date:** 2025-12-24
**Verified By:** Claude Code Agent
