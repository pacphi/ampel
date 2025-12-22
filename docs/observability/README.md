# Observability at Ampel

**See it. Fix it. Ship it.**

Production systems are black boxes. Observability opens them up—giving you the power to understand system behavior, detect issues before users do, and optimize with data-driven confidence.

## Why This Matters

Modern applications fail in complex ways. Without observability, you're flying blind. With it, you gain superpowers:

- **Detect Issues Before Users Do** - Catch errors, latency spikes, and anomalies in real-time
- **Reduce MTTR by 10x** - Jump from alert to root cause in seconds, not hours
- **Understand System Behavior** - Trace requests across services, analyze patterns, optimize bottlenecks
- **Data-Driven Optimization** - Make decisions based on real production metrics, not guesswork
- **Prevent Incidents** - Identify trends before they become outages

In production, you don't get second chances. Observability gives you visibility, control, and confidence.

## What You Get

Ampel ships with a battle-tested observability stack built on industry-standard tools:

### 📊 **Real-time Metrics**

Prometheus-powered metrics collection tracking every request, database query, and business event. Histogram-based latency tracking gives you accurate percentile calculations (P50, P95, P99). Monitor HTTP requests, database connections, background jobs, and custom business metrics—all in one place.

### 🔍 **Distributed Tracing**

OpenTelemetry integration for tracing requests across services. Follow a request from frontend through API to database and external providers. Identify bottlenecks, understand dependencies, and debug distributed failures with precision.

### 📝 **Structured Logging**

JSON-formatted logs with correlation IDs for request tracking. Centralized log aggregation with Loki. Query logs by user ID, request ID, error type, or any custom field. Debug production issues with the context you need.

### 🚨 **Smart Alerting**

Pre-configured alerts for critical scenarios: high error rates, latency spikes, database issues, service downtime. Alerts fire only when they matter—reducing noise and alert fatigue.

### 📈 **Beautiful Dashboards**

Grafana dashboards showing system health at a glance. Request rates, error rates, latency percentiles, database performance, and business metrics—all visualized in real-time. Drill down from overview to individual requests in seconds.

### 🎯 **Error Tracking**

Frontend and backend error capture with stack traces. React ErrorBoundary catches UI crashes. Web Vitals tracking (CLS, FID, LCP) for performance monitoring. Know when users hit problems before they report them.

## Quick Start

**Be up and running in 5 minutes.**

### Local Development

```bash
# Start monitoring stack
make monitoring-up

# Start Ampel services
make dev-api
make dev-worker

# Access dashboards
open http://localhost:3000  # Grafana (admin/admin)
open http://localhost:9090  # Prometheus

# Check health
curl http://localhost:8080/health
curl http://localhost:8080/metrics
```

**That's it.** Your entire observability stack is running.

### Verify Everything Works

```bash
# Health endpoints
curl http://localhost:8080/health
curl http://localhost:8080/ready

# View metrics
curl http://localhost:8080/metrics | grep http_requests

# Check Prometheus targets
open http://localhost:9090/targets
```

### Generate Sample Traffic

```bash
# Hit some endpoints
curl http://localhost:8080/api/pull-requests
curl http://localhost:8080/api/repositories

# Watch metrics update in real-time
open http://localhost:3000/d/ampel-overview
```

## Documentation

### Getting Started

- **[Quick Start Guide](QUICKSTART.md)** - Be up and running in 5 minutes
- **[Monitoring Overview](MONITORING.md)** - Complete monitoring setup and architecture
- **[Metrics Catalog](METRICS.md)** - All available metrics with usage examples

### Deep Dives

- **[Observability Guide](OBSERVABILITY.md)** - Observability principles, patterns, and implementation
- **[API Endpoints](API-ENDPOINTS.md)** - Health checks and metrics endpoints reference
- **[Prometheus Guide](PROMETHEUS.md)** - Prometheus configuration and PromQL queries
- **[Grafana Guide](GRAFANA.md)** - Dashboard creation and visualization

### Reference

- **[Troubleshooting](TROUBLESHOOTING.md)** - Common issues and solutions
- **[Implementation Summary](IMPLEMENTATION-SUMMARY.md)** - Technical implementation details

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Ampel Application                      │
├─────────────────────────────────────────────────────────────┤
│  API Server          Background Worker      Frontend        │
│  ├─ /metrics         ├─ /metrics           ├─ ErrorBoundary│
│  ├─ /health          └─ /health            ├─ Web Vitals   │
│  └─ /ready                                  └─ Event Track  │
└────────┬─────────────────┬─────────────────────┬───────────┘
         │                 │                     │
         │ Scrape 15s      │ Scrape 15s         │ Push events
         ▼                 ▼                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    Prometheus (9090)                         │
│  Metrics Storage • PromQL Queries • Alert Evaluation        │
└────────┬──────────────────────────────────┬─────────────────┘
         │                                  │
         │ Query                            │ Alert
         ▼                                  ▼
┌──────────────────────┐          ┌──────────────────────────┐
│  Grafana (3000)      │          │  Alertmanager (Optional) │
│  Dashboards          │          │  Slack/Email/PagerDuty   │
│  Visualizations      │          │  Notification Routing    │
└──────────────────────┘          └──────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    Additional Exporters                      │
├─────────────────────────────────────────────────────────────┤
│  PostgreSQL (9187)   Redis (9121)   Loki (3100)            │
└─────────────────────────────────────────────────────────────┘
```

**Data Flow:**

1. Application exposes metrics at `/metrics` endpoint
2. Prometheus scrapes metrics every 15 seconds
3. Grafana queries Prometheus and renders dashboards
4. Alerts fire when conditions are met
5. Notifications sent to configured channels

## Key Features

### Metrics That Matter

Track what impacts users and business:

```rust
// HTTP metrics (automatic)
http_requests_total{method="GET", path="/api/prs", status="200"}
http_request_duration_seconds{method="GET", path="/api/prs"}

// Database metrics (automatic)
db_connections_active{database="ampel"}
db_query_duration_seconds{operation="select", table="pull_requests"}

// Custom business metrics (add your own)
ampel_prs_total{status="green"}
ampel_pr_time_to_merge_seconds
ampel_repos_synced_total{provider="github"}
```

### Powerful Queries

PromQL makes complex analysis simple:

```promql
# Request rate per second
rate(http_requests_total[5m])

# Error rate percentage
100 * (
  sum(rate(http_requests_total{status=~"5.."}[5m]))
  / sum(rate(http_requests_total[5m]))
)

# P95 latency by endpoint
histogram_quantile(0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, path)
)

# Database connection pool utilization
(db_connections_active / db_connections_max) * 100
```

### Pre-configured Alerts

Alerts that fire when it matters:

- **HighErrorRate** - Error rate >5% for 5 minutes → Critical
- **HighLatency** - P95 latency >1s for 10 minutes → Warning
- **DatabaseDown** - PostgreSQL unavailable for 1 minute → Critical
- **ServiceDown** - API unavailable for 2 minutes → Critical
- **HighDatabaseConnections** - >80 connections for 5 minutes → Warning

Each alert includes:

- Clear trigger condition
- Recommended severity level
- Actionable description
- Relevant metrics for debugging

## Production Deployment

### Fly.io (Native Support)

Ampel's `/metrics` endpoint works seamlessly with Fly.io's native monitoring:

```toml
# fly.toml
[metrics]
  port = 8080
  path = "/metrics"

[checks]
  [checks.alive]
    type = "http"
    port = 8080
    path = "/health"
    interval = "30s"
```

Access metrics at: `https://fly.io/apps/[APP-NAME]/monitoring`

### Self-Hosted

Run the full stack with Docker Compose:

```bash
# Production docker-compose
docker-compose -f docker/docker-compose.monitoring.yml up -d

# Configure persistent storage
volumes:
  - /data/prometheus:/prometheus
  - /data/grafana:/var/lib/grafana

# Set retention policies
--storage.tsdb.retention.time=30d
--storage.tsdb.retention.size=50GB
```

### Security Checklist

Before going to production:

- [ ] Change Grafana admin password
- [ ] Enable authentication on Prometheus
- [ ] Restrict metrics endpoint to monitoring network
- [ ] Configure TLS for Grafana and Prometheus
- [ ] Set up backup for Prometheus data
- [ ] Configure Alertmanager with notification channels
- [ ] Test alert delivery (Slack/PagerDuty/Email)
- [ ] Create runbooks for common incidents

## Adding Custom Metrics

Instrument your code in seconds:

```rust
use metrics::{counter, histogram, gauge};

// Counter - monotonically increasing
counter!("prs_merged_total",
    "provider" => "github",
    "status" => "success"
).increment(1);

// Histogram - distribution of values
let start = Instant::now();
// ... perform operation
histogram!("sync_duration_seconds",
    "provider" => "github"
).record(start.elapsed().as_secs_f64());

// Gauge - current value
gauge!("active_repositories").set(count as f64);
```

**Metrics appear automatically** in Prometheus and Grafana.

## Best Practices

### 1. Name Metrics Consistently

```rust
// ✅ Good: Clear namespace, base unit, suffix
ampel_http_requests_total
ampel_db_query_duration_seconds
ampel_jobs_processed_total

// ❌ Bad: Inconsistent, unclear
requests
query_time
jobsProcessed
```

### 2. Use Appropriate Types

- **Counter** - Cumulative values (requests, errors, events)
- **Gauge** - Current value (memory, connections, queue depth)
- **Histogram** - Distribution (latency, size, duration)

### 3. Keep Label Cardinality Low

```rust
// ✅ Good: Low cardinality (< 100 unique values)
{method="GET", status="200"}

// ❌ Bad: High cardinality (unbounded)
{user_id="550e8400-...", email="user@example.com"}
```

**Rule:** Keep unique label combinations under 10,000 per metric.

### 4. Track What Matters

Focus on metrics that impact:

- **User experience** - Latency, errors, availability
- **Business outcomes** - Conversions, feature usage, revenue
- **System health** - Resource usage, error rates, dependencies

Avoid vanity metrics that don't drive action.

## Support

### Documentation

- Complete guides in this directory
- [Prometheus Docs](https://prometheus.io/docs/)
- [Grafana Docs](https://grafana.com/docs/)
- [OpenTelemetry Docs](https://opentelemetry.io/docs/)

### Troubleshooting

Common issues and solutions in [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

### Getting Help

- Check documentation first
- Review existing dashboards for examples
- Test queries in Prometheus UI
- Verify metrics endpoint returns data

---

**Built on proven tools. Designed for production. Ready when you are.**

_Last Updated: 2025-12-22_
