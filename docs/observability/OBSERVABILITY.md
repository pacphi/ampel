# Observability Guide

## Table of Contents

1. [Overview](#overview)
2. [Observability Pillars](#observability-pillars)
3. [Distributed Tracing](#distributed-tracing)
4. [Structured Logging](#structured-logging)
5. [Log Aggregation](#log-aggregation)
6. [Debugging Production Issues](#debugging-production-issues)
7. [Performance Profiling](#performance-profiling)
8. [Error Tracking](#error-tracking)
9. [Best Practices](#best-practices)

---

## Overview

Observability is the ability to understand the internal state of a system by examining its outputs. For Ampel, this means comprehensive logging, tracing, and metrics to quickly diagnose and resolve issues.

**Three Pillars of Observability:**

1. **Logs**: Discrete events (errors, warnings, info)
2. **Metrics**: Aggregated measurements over time
3. **Traces**: Request flow through distributed system

---

## Observability Pillars

### 1. Logs

**What to Log:**

- User actions (login, PR view, settings change)
- System events (startup, shutdown, configuration reload)
- Errors and warnings
- Performance-critical operations
- External API calls

**What NOT to Log:**

- Passwords or secrets
- Personal identifiable information (PII)
- Full request/response bodies (use sampling)
- High-frequency events (use metrics instead)

### 2. Metrics

**Types:**

- **Counters**: Total requests, errors, events
- **Gauges**: Current value (active connections, queue depth)
- **Histograms**: Distribution (latency, payload size)

See [METRICS.md](METRICS.md) for complete catalog.

### 3. Traces

**Purpose:**

- Track request across services
- Identify bottlenecks
- Understand dependencies
- Debug distributed failures

---

## Distributed Tracing

### Architecture

```
┌──────────────┐
│   Frontend   │ (Trace ID: abc123)
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Axum API   │ ◄── Start Span: "GET /api/prs"
└──────┬───────┘
       │
       ├──▶ ┌──────────────┐
       │    │  PostgreSQL  │ ◄── Child Span: "SELECT prs"
       │    └──────────────┘
       │
       └──▶ ┌──────────────┐
            │ GitHub API   │ ◄── Child Span: "GET github.com/repos"
            └──────────────┘
```

### Implementation with OpenTelemetry

#### 1. Add Dependencies

```toml
# Cargo.toml
[dependencies]
opentelemetry = "0.20"
opentelemetry-jaeger = "0.19"
tracing-opentelemetry = "0.21"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

#### 2. Initialize Tracer

```rust
use opentelemetry::global;
use opentelemetry::sdk::trace::Tracer;
use opentelemetry_jaeger::new_agent_pipeline;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

fn init_tracer() -> Result<Tracer> {
    let tracer = new_agent_pipeline()
        .with_service_name("ampel-api")
        .with_endpoint("jaeger:6831")
        .install_batch(opentelemetry::runtime::Tokio)?;

    Ok(tracer)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracer
    let tracer = init_tracer()?;

    // Create OpenTelemetry layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Combine with logging
    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().json())
        .with(telemetry);

    tracing::subscriber::set_global_default(subscriber)?;

    // Application code
    Ok(())
}
```

#### 3. Instrument Functions

```rust
use tracing::{instrument, info, error};

#[instrument(skip(db))]
async fn fetch_pull_requests(
    db: &DatabaseConnection,
    repo_id: Uuid,
) -> Result<Vec<PullRequest>> {
    info!("Fetching PRs for repository {}", repo_id);

    let prs = pull_request::Entity::find()
        .filter(pull_request::Column::RepositoryId.eq(repo_id))
        .all(db)
        .await?;

    info!(count = prs.len(), "Fetched PRs successfully");
    Ok(prs)
}
```

#### 4. Propagate Context

```rust
use opentelemetry::global;
use tracing::Span;

#[instrument]
async fn sync_repository(repo: &Repository) -> Result<()> {
    // Current span is automatically propagated

    // Create child span for external API call
    let github_span = tracing::info_span!("github_api_call");
    let _guard = github_span.enter();

    let response = reqwest::get(&repo.url).await?;

    Ok(())
}
```

### Viewing Traces

#### Jaeger UI

```bash
# Start Jaeger (all-in-one Docker image)
docker run -d --name jaeger \
  -p 5775:5775/udp \
  -p 6831:6831/udp \
  -p 6832:6832/udp \
  -p 5778:5778 \
  -p 16686:16686 \
  -p 14268:14268 \
  -p 9411:9411 \
  jaegertracing/all-in-one:latest

# Access UI
open http://localhost:16686
```

**Search traces:**

1. Select service: `ampel-api`
2. Select operation: `GET /api/prs`
3. Apply filters: min duration, tags, time range
4. Click trace to view waterfall

**Example Trace View:**

```
ampel-api: GET /api/prs                    [=============================] 245ms
  ├─ db: SELECT pull_requests              [=====]                        45ms
  ├─ github: GET /repos/.../pulls          [              =========]      120ms
  └─ redis: GET cache:prs:123              [==]                           15ms
```

---

## Structured Logging

### Why Structured Logs?

Traditional text logs:

```
2025-12-22 10:30:45 INFO User login successful
```

Structured (JSON) logs:

```json
{
  "timestamp": "2025-12-22T10:30:45.123Z",
  "level": "INFO",
  "target": "ampel_api::auth",
  "fields": {
    "message": "User login successful",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "ip_address": "192.168.1.100",
    "user_agent": "Mozilla/5.0..."
  }
}
```

**Benefits:**

- Machine-parseable
- Easy to filter and search
- Aggregation and analytics
- Consistent schema

### Implementation

```rust
use tracing::{info, warn, error};
use serde_json::json;

// Simple structured log
info!(
    user_id = %user.id,
    email = %user.email,
    "User logged in successfully"
);

// Complex structured data
let metadata = json!({
    "repository": repo.name,
    "pr_count": prs.len(),
    "sync_duration_ms": duration.as_millis(),
});

info!(
    repo_id = %repo.id,
    metadata = %metadata,
    "Repository sync completed"
);

// Error with context
error!(
    error = %e,
    user_id = %user_id,
    operation = "pr_sync",
    "Failed to sync repository"
);
```

### Log Levels

```rust
// TRACE: Very detailed diagnostic info
tracing::trace!(query = %sql, params = ?params, "Executing query");

// DEBUG: Detailed information for debugging
tracing::debug!(cache_hit = true, "Retrieved from cache");

// INFO: General informational messages
tracing::info!(user_count = users.len(), "Loaded users");

// WARN: Warning messages (non-critical)
tracing::warn!(retry_count = 3, "Retrying failed request");

// ERROR: Error messages (requires attention)
tracing::error!(error = %e, "Database connection failed");
```

### Dynamic Log Levels

```rust
// Use environment variable
// export RUST_LOG=info,ampel_api=debug,sea_orm=warn

use tracing_subscriber::EnvFilter;

tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::from_default_env()
            .add_directive("ampel=debug".parse()?)
    )
    .json()
    .init();
```

---

## Log Aggregation

### Loki Setup

#### Docker Compose

```yaml
# docker/docker-compose.monitoring.yml
services:
  loki:
    image: grafana/loki:latest
    ports:
      - '3100:3100'
    volumes:
      - ./loki/loki-config.yaml:/etc/loki/local-config.yaml
      - loki_data:/loki

  promtail:
    image: grafana/promtail:latest
    volumes:
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - ./loki/promtail-config.yaml:/etc/promtail/config.yml
    command: -config.file=/etc/promtail/config.yml
```

#### Loki Configuration

```yaml
# loki/loki-config.yaml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
  chunk_idle_period: 5m
  chunk_retain_period: 30s

schema_config:
  configs:
    - from: 2025-01-01
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/index
    cache_location: /loki/cache
    shared_store: filesystem
  filesystem:
    directory: /loki/chunks

limits_config:
  retention_period: 14d
```

#### Promtail Configuration

```yaml
# loki/promtail-config.yaml
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: ampel-api
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      - source_labels: ['__meta_docker_container_name']
        regex: '/(ampel-api|ampel-worker).*'
        action: keep
      - source_labels: ['__meta_docker_container_name']
        target_label: container
    pipeline_stages:
      - json:
          expressions:
            timestamp: timestamp
            level: level
            target: target
            message: fields.message
      - labels:
          level:
          target:
      - timestamp:
          source: timestamp
          format: RFC3339
```

### Querying Logs in Grafana

#### LogQL Examples

```logql
# All logs from ampel-api
{container="ampel-api"}

# Error logs only
{container="ampel-api"} |= "level=\"ERROR\""

# Logs containing specific user ID
{container="ampel-api"} | json | user_id="550e8400-e29b-41d4-a716-446655440000"

# Count errors per minute
sum(rate({container="ampel-api"} |= "ERROR" [1m]))

# Top 10 error messages
topk(10,
  sum by (message) (
    rate({container="ampel-api"} |= "ERROR" [5m])
  )
)

# Slow queries (>1s)
{container="ampel-api"}
  | json
  | duration_ms > 1000
```

---

## Debugging Production Issues

### Common Debugging Workflows

#### 1. High Error Rate Alert

**Steps:**

1. **Check Grafana dashboard** for error spike timing
2. **Query Loki** for error logs in that timeframe:

```logql
{container="ampel-api"}
  |= "ERROR"
  | json
  | __timestamp__ > 1640000000
  | __timestamp__ < 1640003600
```

3. **Group errors by type**:

```logql
sum by (error_type) (
  rate({container="ampel-api"} |= "ERROR" [5m])
)
```

4. **Find affected users**:

```logql
{container="ampel-api"}
  |= "ERROR"
  | json
  | user_id != ""
```

5. **Check related traces** in Jaeger using trace ID from logs

#### 2. Slow Response Time

**Steps:**

1. **Identify slow endpoints** in Grafana:

```prometheus
histogram_quantile(0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, path)
)
```

2. **Find slow traces** in Jaeger:
   - Filter by service: `ampel-api`
   - Min duration: `1s`
   - Limit: 20

3. **Analyze trace spans** to find bottleneck:
   - Database query? Check query plan
   - External API? Check API status page
   - Lock contention? Review concurrent requests

4. **Check database metrics**:

```prometheus
db_query_duration_seconds{query="fetch_prs"} > 1.0
```

#### 3. Memory Leak

**Steps:**

1. **Monitor memory over time**:

```prometheus
process_resident_memory_bytes{job="ampel-api"}
```

2. **Check heap allocation**:

```bash
# Rust memory profiling with heaptrack
heaptrack ./target/release/ampel-api
# Analyze with heaptrack_gui
```

3. **Review logs for memory-intensive operations**:

```logql
{container="ampel-api"}
  | json
  | operation = "large_export"
```

4. **Add memory instrumentation**:

```rust
#[instrument(skip(db))]
async fn large_query(db: &DatabaseConnection) -> Result<Vec<Item>> {
    let start_mem = get_memory_usage();
    let items = fetch_all_items(db).await?;
    let end_mem = get_memory_usage();

    info!(
        memory_delta_mb = (end_mem - start_mem) / 1024 / 1024,
        item_count = items.len(),
        "Large query completed"
    );

    Ok(items)
}
```

---

## Performance Profiling

### CPU Profiling

#### Using `perf` (Linux)

```bash
# Record profile
perf record -g ./target/release/ampel-api

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > profile.svg

# View in browser
open profile.svg
```

#### Using `cargo flamegraph`

```bash
# Install
cargo install flamegraph

# Profile release build
cargo flamegraph --release -- --config config.toml

# Output: flamegraph.svg
```

### Memory Profiling

#### Using `heaptrack`

```bash
# Install
sudo apt install heaptrack

# Run profiler
heaptrack ./target/release/ampel-api

# Analyze
heaptrack_gui heaptrack.ampel-api.XXXXX.gz
```

#### Using `valgrind`

```bash
# Memory leak detection
valgrind --leak-check=full ./target/release/ampel-api

# Cache profiling
valgrind --tool=cachegrind ./target/release/ampel-api
```

### Benchmarking

```rust
// benches/api_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_fetch_prs(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = rt.block_on(setup_test_db());

    c.bench_function("fetch_prs", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(fetch_pull_requests(&db, repo_id).await)
        })
    });
}

criterion_group!(benches, bench_fetch_prs);
criterion_main!(benches);
```

```bash
# Run benchmarks
cargo bench

# Compare with baseline
cargo bench -- --baseline main
```

---

## Error Tracking

### Integration with Sentry

#### Setup

```toml
[dependencies]
sentry = { version = "0.31", features = ["tracing"] }
sentry-tracing = "0.31"
```

```rust
use sentry::IntegrationMeta;

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = sentry::init((
        "https://your-dsn@sentry.io/project-id",
        sentry::ClientOptions {
            release: Some(env!("CARGO_PKG_VERSION").into()),
            environment: Some(env::var("ENVIRONMENT").unwrap_or_else(|_| "development".into()).into()),
            traces_sample_rate: 0.1, // 10% of requests
            ..Default::default()
        },
    ));

    // Integrate with tracing
    sentry_tracing::init();

    // Application code
    run_server().await
}
```

#### Capturing Errors

```rust
use sentry::{capture_error, capture_message};

// Automatic error capture
if let Err(e) = sync_repository(&repo).await {
    tracing::error!(error = %e, repo_id = %repo.id, "Sync failed");
    // Sentry integration captures automatically via tracing
}

// Manual error capture with context
sentry::configure_scope(|scope| {
    scope.set_user(Some(sentry::User {
        id: Some(user.id.to_string()),
        email: Some(user.email.clone()),
        ..Default::default()
    }));
    scope.set_tag("repository", repo.name.clone());
});

capture_error(&error);

// Custom events
capture_message("Critical configuration error", sentry::Level::Error);
```

### Error Grouping and Prioritization

**Sentry Dashboard:**

1. Errors grouped by fingerprint (stack trace)
2. Prioritize by:
   - Frequency (errors/minute)
   - User impact (unique users affected)
   - Severity (error vs warning)

**Configure alerts:**

```yaml
# sentry-alerts.yml
alerts:
  - name: High error rate
    condition: event.count > 100
    interval: 5m
    notify: slack:#alerts

  - name: New error
    condition: is:unresolved is:new
    notify: slack:#errors
```

---

## Best Practices

### 1. Correlation IDs

Use correlation IDs to track requests across services:

```rust
use uuid::Uuid;
use axum::extract::Extension;

// Middleware to generate correlation ID
async fn correlation_id_middleware(
    mut req: Request<Body>,
    next: Next<Body>,
) -> Response {
    let correlation_id = req
        .headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.extensions_mut().insert(correlation_id.clone());

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        "x-correlation-id",
        correlation_id.parse().unwrap()
    );

    response
}

// Use in handler
#[instrument(skip(db), fields(correlation_id = %correlation_id))]
async fn get_prs(
    Extension(correlation_id): Extension<String>,
    db: Extension<DatabaseConnection>,
) -> Result<Json<Vec<PullRequest>>> {
    info!("Fetching PRs");
    // ...
}
```

### 2. Sampling for High-Volume Events

```rust
use rand::Rng;

// Sample 10% of requests
let should_log = rand::thread_rng().gen_range(0..100) < 10;
if should_log {
    info!(
        path = %req.uri(),
        method = %req.method(),
        "Request received"
    );
}

// Or use rate limiting
use ratelimit::Ratelimiter;
static LOG_LIMITER: Lazy<Ratelimiter> = Lazy::new(|| {
    Ratelimiter::builder(100, Duration::from_secs(60))
        .build()
        .unwrap()
});

if LOG_LIMITER.try_wait().is_ok() {
    info!("High-frequency event");
}
```

### 3. Context-Rich Logging

```rust
// ❌ Poor: Not enough context
error!("Failed to save");

// ✅ Good: Actionable information
error!(
    error = %e,
    user_id = %user.id,
    resource = "pull_request",
    operation = "update",
    pr_id = %pr.id,
    "Failed to update pull request"
);
```

### 4. Log Levels Appropriately

```rust
// ✅ Correct usage
trace!("Entering function with params: {:?}", params);  // Debug info
debug!("Cache miss for key: {}", key);                  // Debugging
info!("Server started on port {}", port);               // Lifecycle
warn!("API rate limit approaching: {}/100", count);     // Potential issue
error!("Database connection failed: {}", e);             // Requires action
```

### 5. Avoid Logging Sensitive Data

```rust
// ❌ NEVER log passwords
error!(password = %password, "Login failed");

// ❌ NEVER log tokens
info!(api_token = %token, "API call");

// ✅ Log redacted version
info!(
    user_email = %user.email,
    token_prefix = %&token[..8],
    "API authentication successful"
);
```

---

## Resources

- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [tracing Documentation](https://docs.rs/tracing/latest/tracing/)
- [Loki Documentation](https://grafana.com/docs/loki/latest/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Sentry Rust SDK](https://docs.sentry.io/platforms/rust/)
- [Ampel Monitoring Guide](MONITORING.md)
- [Ampel Metrics Catalog](METRICS.md)

---

**Last Updated:** 2025-12-22
**Maintainer:** Ampel Platform Team
