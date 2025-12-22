# API Observability Endpoints

## Health Check Endpoints

### GET /health

Returns the health status of the service.

**Response (200 OK):**

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "checks": {
    "database": true
  }
}
```

**Response (503 Service Unavailable):**

```json
{
  "status": "unhealthy",
  "version": "0.1.0",
  "checks": {
    "database": false
  }
}
```

**Usage:**

```bash
curl http://localhost:8080/health
```

**Use Case:** Load balancer health checks, basic service availability

---

### GET /ready

Returns the readiness status of the service (suitable for Kubernetes readiness probes).

**Response (200 OK):**

```json
{
  "ready": true,
  "checks": {
    "database": true
  }
}
```

**Response (503 Service Unavailable):**

```json
{
  "ready": false,
  "checks": {
    "database": false
  }
}
```

**Usage:**

```bash
curl http://localhost:8080/ready
```

**Use Case:** Kubernetes readiness probes, deployment validation

---

### GET /metrics

Returns Prometheus-formatted metrics for scraping.

**Response (200 OK):**

```
# HELP http_requests_total Total number of HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",path="/api/pull-requests",status="200"} 42

# HELP http_request_duration_seconds HTTP request duration in seconds
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{method="GET",path="/api/pull-requests",status="200",le="0.005"} 12
http_request_duration_seconds_bucket{method="GET",path="/api/pull-requests",status="200",le="0.01"} 28
http_request_duration_seconds_bucket{method="GET",path="/api/pull-requests",status="200",le="0.025"} 38
http_request_duration_seconds_sum{method="GET",path="/api/pull-requests",status="200"} 0.856
http_request_duration_seconds_count{method="GET",path="/api/pull-requests",status="200"} 42
```

**Usage:**

```bash
curl http://localhost:8080/metrics
```

**Use Case:** Prometheus scraping, metrics analysis

---

## Metrics Tracked

### HTTP Metrics

**http_requests_total**

- **Type:** Counter
- **Labels:** `method`, `path`, `status`
- **Description:** Total number of HTTP requests handled by the API

**http_request_duration_seconds**

- **Type:** Histogram
- **Labels:** `method`, `path`, `status`
- **Buckets:** [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
- **Description:** HTTP request duration in seconds

### Example PromQL Queries

**Request rate by endpoint:**

```promql
rate(http_requests_total[5m])
```

**Error rate:**

```promql
sum(rate(http_requests_total{status=~"5.."}[5m]))
/
sum(rate(http_requests_total[5m]))
```

**P95 latency:**

```promql
histogram_quantile(0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, path)
)
```

**Slowest endpoints (P99):**

```promql
topk(5,
  histogram_quantile(0.99,
    sum(rate(http_request_duration_seconds_bucket[5m])) by (le, path)
  )
)
```

---

## Integration Examples

### Prometheus Configuration

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'ampel-api'
    static_configs:
      - targets: ['api:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Kubernetes Probes

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: ampel-api
spec:
  containers:
    - name: api
      image: ampel-api:latest
      livenessProbe:
        httpGet:
          path: /health
          port: 8080
        initialDelaySeconds: 10
        periodSeconds: 30
      readinessProbe:
        httpGet:
          path: /ready
          port: 8080
        initialDelaySeconds: 5
        periodSeconds: 10
```

### Docker Healthcheck

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1
```

### Fly.io Configuration

Add to `fly.toml`:

```toml
[metrics]
  port = 8080
  path = "/metrics"

[checks]
  [checks.alive]
    type = "http"
    port = 8080
    method = "get"
    path = "/health"
    interval = "30s"
    timeout = "2s"
    grace_period = "5s"
```

---

## Custom Application Metrics

Add custom business metrics in your Rust code:

### Counter

```rust
use metrics::counter;

counter!("pull_requests_synced_total",
    "provider" => "github",
    "repository" => repo_name,
    "status" => "success"
).increment(1);
```

### Histogram

```rust
use metrics::histogram;
use std::time::Instant;

let start = Instant::now();
// ... perform operation
let duration = start.elapsed();

histogram!("repository_sync_duration_seconds",
    "provider" => "github"
).record(duration.as_secs_f64());
```

### Gauge

```rust
use metrics::gauge;

gauge!("active_repositories",
    "status" => "enabled"
).set(count as f64);
```

---

## Monitoring Best Practices

### 1. Label Cardinality

Keep label cardinality low to avoid excessive memory usage:

❌ **Bad:** Using user IDs as labels

```rust
counter!("requests_total", "user_id" => user_id.to_string())
```

✅ **Good:** Aggregate by user type

```rust
counter!("requests_total", "user_type" => "premium")
```

### 2. Meaningful Metric Names

Follow Prometheus naming conventions:

- Use `_total` suffix for counters
- Use base unit (seconds, bytes, not ms or MB)
- Use descriptive names

### 3. Appropriate Metric Types

- **Counter:** Monotonically increasing values (requests, errors)
- **Gauge:** Values that go up and down (memory usage, active connections)
- **Histogram:** Distribution of values (request duration, response size)

### 4. Error Tracking

Always track errors with proper labels:

```rust
if let Err(e) = sync_repository(&db, repo_id).await {
    counter!("sync_errors_total",
        "error_type" => classify_error(&e)
    ).increment(1);
}
```

---

## Troubleshooting

### Metrics not appearing in Prometheus

1. Check metrics endpoint returns data:

   ```bash
   curl http://localhost:8080/metrics | grep http_requests_total
   ```

2. Verify Prometheus is scraping:

   ```bash
   # Check targets page
   open http://localhost:9090/targets
   ```

3. Check for scrape errors in Prometheus logs:
   ```bash
   docker logs ampel-prometheus
   ```

### High cardinality issues

If Prometheus is using too much memory:

1. Check label cardinality:

   ```promql
   count({__name__=~".+"}) by (__name__)
   ```

2. Identify high-cardinality metrics:

   ```bash
   curl http://localhost:9090/api/v1/label/__name__/values
   ```

3. Review metric labels and reduce unique combinations

---

## See Also

- [Observability Guide](OBSERVABILITY.md)
- [Monitoring Quick Start](QUICKSTART.md)
- [Metrics Catalog](METRICS.md)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
