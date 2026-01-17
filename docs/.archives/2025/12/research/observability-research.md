# Observability Research Report: Ampel PR Dashboard

**Generated**: 2025-12-22
**Research Specialist**: Observability Research Agent
**Target Deployment**: Docker Compose (local) + Fly.io (production)
**Tech Stack**: Rust (Axum + SeaORM + Apalis) + React 19 + PostgreSQL + Redis

---

## Executive Summary

This research provides comprehensive recommendations for implementing observability, monitoring, and metrics for the Ampel PR dashboard. The strategy balances developer experience with production reliability while minimizing costs on Fly.io.

### Key Recommendations

| Component        | Local (Docker Compose) | Production (Fly.io)                 |
| ---------------- | ---------------------- | ----------------------------------- |
| **Metrics**      | Prometheus + Grafana   | Fly.io managed Prometheus + Grafana |
| **Tracing**      | Jaeger/Tempo           | OpenTelemetry → Grafana Cloud       |
| **Logs**         | Loki                   | Fly.io logs + Loki (optional)       |
| **Frontend RUM** | web-vitals dev mode    | Sentry / Datadog RUM                |
| **Alerts**       | Grafana Alerts         | Grafana Cloud Alerts                |
| **Cost**         | $0 (local infra)       | ~$10-30/month                       |

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      AMPEL APPLICATION                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Axum API    │  │ React Frontend│  │ Apalis Worker│      │
│  │              │  │               │  │              │      │
│  │ • tracing    │  │ • web-vitals  │  │ • tracing    │      │
│  │ • metrics    │  │ • error-boundary│ • metrics    │      │
│  │ • OpenTelemetry│ • Sentry       │  │ • OpenTelemetry│   │
│  └──────┬───────┘  └──────┬────────┘  └──────┬───────┘      │
│         │                 │                  │               │
└─────────┼─────────────────┼──────────────────┼───────────────┘
          │                 │                  │
          ▼                 ▼                  ▼
┌─────────────────────────────────────────────────────────────┐
│              OBSERVABILITY INFRASTRUCTURE                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Prometheus  │  │    Loki      │  │  Jaeger/Tempo│      │
│  │              │  │              │  │              │      │
│  │ Metrics TSDB │  │ Log Storage  │  │ Trace Storage│      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                  │               │
│         └─────────────────┴──────────────────┘               │
│                           │                                  │
│                  ┌────────▼────────┐                         │
│                  │    Grafana      │                         │
│                  │  Visualization  │                         │
│                  └─────────────────┘                         │
│                                                              │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                   EXTERNAL SERVICES                          │
├──────────────────────────────────────────────────────────────┤
│  • Sentry (error tracking)                                   │
│  • Datadog RUM / Grafana Cloud (frontend monitoring)         │
│  • PagerDuty / Opsgenie (alerting)                          │
└──────────────────────────────────────────────────────────────┘
```

---

## 1. Rust Backend Observability

### 1.1 Tracing Architecture

#### Recommended Stack (2025)

**Core Dependencies:**

```toml
# Cargo.toml
[dependencies]
# Core tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.27"

# OpenTelemetry
opentelemetry = { version = "0.27", features = ["trace", "metrics"] }
opentelemetry-otlp = { version = "0.27", features = ["tokio", "tls"] }
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }

# Prometheus metrics
metrics = "0.24"
metrics-exporter-prometheus = { version = "0.16", default-features = false }

# Axum integration
axum-prometheus = "0.8"  # or axum-otel-metrics for OpenTelemetry

# Optional: Sentry for error tracking
sentry = { version = "0.35", features = ["tracing", "tower"] }
sentry-tower = "0.35"
```

#### Key Best Practices (Based on Research)

1. **Use tracing-subscriber Registry Pattern**
   - Registry stores span metadata and relationships
   - Allows downstream layers to focus on filtering/formatting
   - Source: [OpenTelemetry Rust Docs](https://opentelemetry.io/docs/languages/rust/)

2. **Direct OpenTelemetry Integration (2025 Recommendation)**
   - Datadog recommends dropping intermediate tracing dependency
   - Direct integration provides simpler, more consistent telemetry
   - Source: [Datadog Rust Monitoring Guide](https://www.datadoghq.com/blog/monitor-rust-otel/)

3. **Use OTLP Protocol (Deprecated: Jaeger)**
   - `opentelemetry_jaeger` is deprecated
   - `opentelemetry-otlp` is the official exporter
   - Source: [Getting Started with OpenTelemetry in Rust](https://last9.io/blog/opentelemetry-in-rust/)

4. **Special OpenTelemetry Fields**
   - `otel.name`: Override span names
   - `otel.kind`: Set span kind (client/server)
   - `otel.status_code`: Set span status
   - Source: [tracing-opentelemetry docs](https://docs.rs/tracing-opentelemetry)

#### Implementation Example

**File: `crates/ampel-api/src/telemetry.rs`**

```rust
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub fn init_telemetry(
    service_name: &str,
    otlp_endpoint: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Environment filter for log levels
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,ampel=debug"));

    // JSON formatting for production, pretty for development
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_line_number(true)
        .json();

    // Initialize OpenTelemetry tracer
    let tracer = if let Some(endpoint) = otlp_endpoint {
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint)
            )
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_resource(Resource::new(vec![
                        KeyValue::new("service.name", service_name.to_string()),
                        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    ])),
            )
            .install_batch(runtime::Tokio)?
    } else {
        // Development: no-op tracer
        opentelemetry::global::tracer(service_name)
    };

    // OpenTelemetry tracing layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Combine layers
    Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    Ok(())
}

pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
}
```

**Usage in `main.rs`:**

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry
    telemetry::init_telemetry(
        "ampel-api",
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
    )?;

    // Your application code
    let app = create_app().await?;

    // Graceful shutdown
    let shutdown_signal = shutdown_signal();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    // Cleanup
    telemetry::shutdown_telemetry();
    Ok(())
}
```

**Instrumentation Example:**

```rust
use tracing::{instrument, info, warn, error};

#[instrument(
    skip(pool, user_id),
    fields(
        user_id = %user_id,
        otel.kind = "server"
    )
)]
pub async fn get_user_repositories(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Repository>, AppError> {
    info!("Fetching repositories for user");

    let repos = sqlx::query_as!(
        Repository,
        "SELECT * FROM repositories WHERE user_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to fetch repositories");
        AppError::Database(e)
    })?;

    info!(count = repos.len(), "Repositories fetched successfully");
    Ok(repos)
}
```

### 1.2 Prometheus Metrics for Axum

#### Recommended Approach: axum-prometheus

**Comparison of Crates (2025):**

| Crate                        | Backend                                  | Pros                              | Cons                    |
| ---------------------------- | ---------------------------------------- | --------------------------------- | ----------------------- |
| `axum-prometheus`            | metrics.rs + metrics_exporter_prometheus | Easier integration, more flexible | Indirect implementation |
| `prometheus-axum-middleware` | prometheus crate directly                | Direct Prometheus API             | Less flexible           |
| `axum-otel-metrics`          | OpenTelemetry + Prometheus               | Unified telemetry                 | More complex            |

Source: [axum-prometheus GitHub](https://github.com/Ptrskay3/axum-prometheus)

**Implementation:**

```rust
use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;

pub fn create_app() -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/health", get(health_check))
        // Metrics endpoint
        .route("/metrics", get(|| async move { metric_handle.render() }))
        // Apply metrics layer to all routes
        .layer(prometheus_layer)
}
```

**Standard Metrics Collected:**

- `axum_http_requests_total` (counter): Total HTTP requests
- `axum_http_requests_duration_seconds` (histogram): Request duration
- `axum_http_requests_pending` (gauge): In-flight requests

Source: [Axum App Monitoring with Prometheus and Grafana](https://carlosmv.hashnode.dev/how-to-monitor-your-axum-app-with-prometheus-and-grafana)

### 1.3 SeaORM Metrics

**Custom Metrics for Database Operations:**

```rust
use metrics::{counter, histogram};
use sea_orm::{DatabaseConnection, DbErr};

pub struct InstrumentedDatabase {
    conn: DatabaseConnection,
}

impl InstrumentedDatabase {
    pub async fn query<T, F>(&self, op_name: &str, f: F) -> Result<T, DbErr>
    where
        F: FnOnce(&DatabaseConnection) -> Result<T, DbErr>,
    {
        let start = std::time::Instant::now();

        let result = f(&self.conn);

        let duration = start.elapsed();
        histogram!("db_query_duration_seconds", duration.as_secs_f64(), "operation" => op_name);

        match &result {
            Ok(_) => counter!("db_queries_total", 1, "operation" => op_name, "status" => "success"),
            Err(_) => counter!("db_queries_total", 1, "operation" => op_name, "status" => "error"),
        }

        result
    }
}
```

**Key Metrics to Track:**

- Query execution time by operation type
- Connection pool statistics (active, idle, waiting)
- Transaction success/failure rates
- Database error rates by type

### 1.4 Apalis Worker Metrics

**Custom Job Metrics:**

```rust
use apalis::prelude::*;
use metrics::{counter, histogram, gauge};

#[derive(Clone)]
pub struct MetricsMiddleware;

impl<T> Middleware<T> for MetricsMiddleware
where
    T: Job,
{
    async fn call(&self, job: T, ctx: JobContext, next: Next<T>) -> Result<(), JobError> {
        let job_type = std::any::type_name::<T>();
        let start = std::time::Instant::now();

        gauge!("worker_jobs_active", 1.0, "job_type" => job_type);
        counter!("worker_jobs_total", 1, "job_type" => job_type, "status" => "started");

        let result = next.call(job, ctx).await;

        let duration = start.elapsed();
        histogram!("worker_job_duration_seconds", duration.as_secs_f64(), "job_type" => job_type);

        match &result {
            Ok(_) => counter!("worker_jobs_total", 1, "job_type" => job_type, "status" => "success"),
            Err(_) => counter!("worker_jobs_total", 1, "job_type" => job_type, "status" => "failed"),
        }

        gauge!("worker_jobs_active", -1.0, "job_type" => job_type);

        result
    }
}
```

**Key Worker Metrics:**

- Job execution duration by type
- Job success/failure rates
- Queue depth
- Worker pool utilization
- Retry counts

### 1.5 Error Tracking: Sentry Integration

```rust
use sentry::{integrations::tower::SentryHttpLayer, ClientOptions};
use tower::ServiceBuilder;

pub fn init_sentry(dsn: &str) -> sentry::ClientInitGuard {
    let guard = sentry::init((
        dsn,
        ClientOptions {
            release: sentry::release_name!(),
            environment: Some(std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".into()).into()),
            traces_sample_rate: 0.1, // 10% of traces
            ..Default::default()
        },
    ));

    guard
}

// In router:
pub fn create_app() -> Router {
    Router::new()
        .route("/api/users", get(get_users))
        .layer(
            ServiceBuilder::new()
                .layer(SentryHttpLayer::new())
        )
}
```

---

## 2. React Frontend Monitoring

### 2.1 Real User Monitoring (RUM) in 2025

#### Key Changes for 2025

**Important Update**: Starting March 2024, **Interaction to Next Paint (INP)** replaced FID as a Core Web Vital.

**Current Core Web Vitals:**

1. **Largest Contentful Paint (LCP)**: Loading performance (< 2.5s good)
2. **Interaction to Next Paint (INP)**: Interactivity (< 200ms good) - **NEW**
3. **Cumulative Layout Shift (CLS)**: Visual stability (< 0.1 good)

Source: [React Performance Monitoring 2025](https://medium.com/@mernstackdevbykevin/react-performance-monitoring-with-web-vitals-the-2025-developers-guide-661559271932)

#### Recommended Implementation: web-vitals Library

**Installation:**

```bash
pnpm add web-vitals
```

**Implementation (`frontend/src/lib/vitals.ts`):**

```typescript
import { onCLS, onINP, onLCP, onFCP, onTTFB, Metric } from 'web-vitals';

type VitalsCallback = (metric: Metric) => void;

const vitalsUrl = import.meta.env.VITE_VITALS_ENDPOINT || '/api/vitals';

function sendToAnalytics(metric: Metric) {
  const body = JSON.stringify({
    name: metric.name,
    value: metric.value,
    rating: metric.rating,
    delta: metric.delta,
    id: metric.id,
    navigationType: metric.navigationType,
    url: window.location.href,
    userAgent: navigator.userAgent,
  });

  // Use `navigator.sendBeacon()` if available, falling back to `fetch()`
  if (navigator.sendBeacon) {
    navigator.sendBeacon(vitalsUrl, body);
  } else {
    fetch(vitalsUrl, {
      body,
      method: 'POST',
      keepalive: true,
      headers: { 'Content-Type': 'application/json' },
    });
  }
}

export function initVitals() {
  onCLS(sendToAnalytics);
  onINP(sendToAnalytics); // NEW: Replaces FID
  onLCP(sendToAnalytics);
  onFCP(sendToAnalytics);
  onTTFB(sendToAnalytics);
}
```

**Usage in `main.tsx`:**

```typescript
import { initVitals } from './lib/vitals';

// Initialize vitals reporting
if (import.meta.env.PROD) {
  initVitals();
}
```

**Backend Endpoint (`crates/ampel-api/src/handlers/vitals.rs`):**

```rust
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct WebVital {
    name: String,
    value: f64,
    rating: String,
    delta: f64,
    id: String,
    #[serde(rename = "navigationType")]
    navigation_type: String,
    url: String,
    #[serde(rename = "userAgent")]
    user_agent: String,
}

pub async fn record_vitals(
    Json(vital): Json<WebVital>,
) -> StatusCode {
    info!(
        vital.name = %vital.name,
        vital.value = %vital.value,
        vital.rating = %vital.rating,
        url = %vital.url,
        "Web vital recorded"
    );

    // TODO: Store in database or send to metrics backend
    // metrics::histogram!("frontend_web_vitals", vital.value, "metric" => vital.name);

    StatusCode::NO_CONTENT
}
```

Source: [web.dev Web Vitals Measurement](https://web.dev/articles/vitals-measurement-getting-started)

### 2.2 Error Boundary + Error Reporting

**React Error Boundary (`frontend/src/components/ErrorBoundary.tsx`):**

```typescript
import React, { Component, ErrorInfo, ReactNode } from 'react';
import * as Sentry from '@sentry/react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);

    // Send to Sentry
    Sentry.captureException(error, {
      contexts: {
        react: {
          componentStack: errorInfo.componentStack,
        },
      },
    });
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div className="error-fallback">
          <h2>Something went wrong</h2>
          <details>
            <summary>Error details</summary>
            <pre>{this.state.error?.message}</pre>
          </details>
          <button onClick={() => window.location.reload()}>Reload page</button>
        </div>
      );
    }

    return this.props.children;
  }
}
```

**Sentry Integration:**

```typescript
// frontend/src/main.tsx
import * as Sentry from '@sentry/react';

if (import.meta.env.PROD) {
  Sentry.init({
    dsn: import.meta.env.VITE_SENTRY_DSN,
    environment: import.meta.env.VITE_ENVIRONMENT || 'production',
    tracesSampleRate: 0.1,
    integrations: [
      Sentry.browserTracingIntegration(),
      Sentry.replayIntegration({
        maskAllText: false,
        blockAllMedia: false,
      }),
    ],
    replaysSessionSampleRate: 0.1,
    replaysOnErrorSampleRate: 1.0,
  });
}
```

### 2.3 Performance Monitoring Patterns

**React Query Instrumentation:**

```typescript
// frontend/src/lib/queryClient.ts
import { QueryClient } from '@tanstack/react-query';
import * as Sentry from '@sentry/react';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 3,
      staleTime: 5 * 60 * 1000, // 5 minutes
      onError: (error) => {
        console.error('Query error:', error);
        Sentry.captureException(error);
      },
      onSettled: (data, error, query) => {
        // Track query performance
        if (query.state.fetchStatus === 'idle') {
          const duration = Date.now() - (query.state.dataUpdatedAt || 0);
          // Send metric to backend
          fetch('/api/metrics/query-performance', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              queryKey: query.queryKey,
              duration,
              hasError: !!error,
            }),
          });
        }
      },
    },
  },
});
```

### 2.4 Session Replay Considerations

**Recommended Approach**: Use Sentry Session Replay or LogRocket

**Privacy-First Configuration:**

```typescript
Sentry.init({
  // ... other config
  integrations: [
    Sentry.replayIntegration({
      // Privacy settings
      maskAllText: true, // Mask all text content
      maskAllInputs: true, // Mask form inputs
      blockAllMedia: true, // Block images/videos

      // Network recording
      networkDetailAllowUrls: [window.location.origin],
      networkCaptureBodies: false, // Don't capture request bodies

      // Performance settings
      compress: true,
      maxReplayDuration: 60 * 1000, // 1 minute
    }),
  ],
  replaysSessionSampleRate: 0.1, // 10% of sessions
  replaysOnErrorSampleRate: 1.0, // 100% of error sessions
});
```

**Cost Considerations**:

- Sentry: ~$26/month for 50K events + replays
- LogRocket: ~$99/month for 10K sessions
- FullStory: Enterprise pricing

---

## 3. Infrastructure Monitoring

### 3.1 Docker Compose Stack (Local Development)

**Complete Observability Stack:**

```yaml
# docker-compose.observability.yml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:v2.50.0
    container_name: ampel-prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/usr/share/prometheus/console_libraries'
      - '--web.console.templates=/usr/share/prometheus/consoles'
      - '--storage.tsdb.retention.time=30d'
    volumes:
      - ./observability/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    ports:
      - '9090:9090'
    networks:
      - ampel-network
    restart: unless-stopped

  grafana:
    image: grafana/grafana:10.3.0
    container_name: ampel-grafana
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
      - GF_SERVER_ROOT_URL=http://localhost:3000
    volumes:
      - ./observability/grafana/provisioning:/etc/grafana/provisioning:ro
      - ./observability/grafana/dashboards:/var/lib/grafana/dashboards:ro
      - grafana-data:/var/lib/grafana
    ports:
      - '3000:3000'
    networks:
      - ampel-network
    depends_on:
      - prometheus
      - loki
    restart: unless-stopped

  loki:
    image: grafana/loki:2.9.5
    container_name: ampel-loki
    command: -config.file=/etc/loki/local-config.yaml
    volumes:
      - ./observability/loki.yml:/etc/loki/local-config.yaml:ro
      - loki-data:/loki
    ports:
      - '3100:3100'
    networks:
      - ampel-network
    restart: unless-stopped

  promtail:
    image: grafana/promtail:2.9.5
    container_name: ampel-promtail
    command: -config.file=/etc/promtail/config.yml
    volumes:
      - ./observability/promtail.yml:/etc/promtail/config.yml:ro
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - /var/run/docker.sock:/var/run/docker.sock
    networks:
      - ampel-network
    depends_on:
      - loki
    restart: unless-stopped

  tempo:
    image: grafana/tempo:2.3.1
    container_name: ampel-tempo
    command: ['-config.file=/etc/tempo.yaml']
    volumes:
      - ./observability/tempo.yml:/etc/tempo.yaml:ro
      - tempo-data:/tmp/tempo
    ports:
      - '4317:4317' # OTLP gRPC
      - '4318:4318' # OTLP HTTP
      - '3200:3200' # Tempo HTTP
    networks:
      - ampel-network
    restart: unless-stopped

  # PostgreSQL Exporter
  postgres-exporter:
    image: prometheuscommunity/postgres-exporter:v0.15.0
    container_name: ampel-postgres-exporter
    environment:
      DATA_SOURCE_NAME: 'postgresql://ampel:ampel@postgres:5432/ampel?sslmode=disable'
    ports:
      - '9187:9187'
    networks:
      - ampel-network
    depends_on:
      - postgres
    restart: unless-stopped

  # Redis Exporter
  redis-exporter:
    image: oliver006/redis_exporter:v1.57.0
    container_name: ampel-redis-exporter
    environment:
      REDIS_ADDR: 'redis:6379'
    ports:
      - '9121:9121'
    networks:
      - ampel-network
    depends_on:
      - redis
    restart: unless-stopped

volumes:
  prometheus-data:
  grafana-data:
  loki-data:
  tempo-data:

networks:
  ampel-network:
    external: true
```

**Prometheus Configuration (`observability/prometheus.yml`):**

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'local'
    environment: 'development'

scrape_configs:
  # Ampel API metrics
  - job_name: 'ampel-api'
    static_configs:
      - targets: ['ampel-api:8080']
    metrics_path: '/metrics'

  # Ampel Worker metrics
  - job_name: 'ampel-worker'
    static_configs:
      - targets: ['ampel-worker:8081']
    metrics_path: '/metrics'

  # PostgreSQL metrics
  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  # Redis metrics
  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

  # Prometheus itself
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
```

Source: [Redis Monitoring with Prometheus and Grafana](https://nelsoncode.medium.com/how-to-monitor-redis-with-prometheus-and-grafana-docker-6eb33a5ea998)

### 3.2 Grafana Dashboards

**Pre-configured Dashboards (`observability/grafana/provisioning/dashboards/dashboard.yml`):**

```yaml
apiVersion: 1

providers:
  - name: 'Ampel Dashboards'
    orgId: 1
    folder: 'Ampel'
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /var/lib/grafana/dashboards
      foldersFromFilesStructure: true
```

**Dashboard Templates to Include:**

1. **Ampel API Overview**
   - Request rate, duration, error rate
   - Database query performance
   - Background job metrics

2. **PostgreSQL Dashboard** (ID: 9628)
   - Connection pool stats
   - Query performance
   - Cache hit ratios

3. **Redis Dashboard** (Grafana Redis plugin)
   - Memory usage
   - Commands per second
   - Key expiration rates

4. **Frontend Performance**
   - Core Web Vitals (LCP, INP, CLS)
   - API call latencies
   - Error rates by page

Source: [Grafana Redis Data Source](https://grafana.com/grafana/plugins/redis-datasource/)

### 3.3 Loki Configuration for Logs

**Loki Config (`observability/loki.yml`):**

```yaml
auth_enabled: false

server:
  http_listen_port: 3100

common:
  path_prefix: /loki
  storage:
    filesystem:
      chunks_directory: /loki/chunks
      rules_directory: /loki/rules
  replication_factor: 1
  ring:
    kvstore:
      store: inmemory

schema_config:
  configs:
    - from: 2024-01-01
      store: tsdb
      object_store: filesystem
      schema: v13
      index:
        prefix: index_
        period: 24h

limits_config:
  retention_period: 168h # 7 days
  ingestion_rate_mb: 16
  ingestion_burst_size_mb: 32
```

**Promtail Config (`observability/promtail.yml`):**

```yaml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: docker
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      - source_labels: ['__meta_docker_container_name']
        regex: '/(.*)'
        target_label: 'container'
      - source_labels: ['__meta_docker_container_log_stream']
        target_label: 'stream'
```

Source: [Monitoring n8n with Prometheus and Grafana](https://community.n8n.io/t/automation-script-to-self-hosting-n8n-with-docker-compose-both-single-and-queue-mode-and-monitoring-with-prometheus-grafana/187720)

### 3.4 Tempo Configuration for Tracing

**Tempo Config (`observability/tempo.yml`):**

```yaml
server:
  http_listen_port: 3200

distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: 0.0.0.0:4317
        http:
          endpoint: 0.0.0.0:4318

storage:
  trace:
    backend: local
    local:
      path: /tmp/tempo/blocks

metrics_generator:
  registry:
    external_labels:
      source: tempo
  storage:
    path: /tmp/tempo/generator/wal
  traces_storage:
    path: /tmp/tempo/generator/traces
```

**Environment Variables for Ampel Services:**

```bash
# .env.local
OTEL_EXPORTER_OTLP_ENDPOINT=http://tempo:4317
OTEL_SERVICE_NAME=ampel-api
OTEL_TRACES_SAMPLER=parentbased_traceidratio
OTEL_TRACES_SAMPLER_ARG=0.1
```

---

## 4. Fly.io Production Deployment

### 4.1 Fly.io's Managed Monitoring

**Key Features (2025):**

1. **Managed Prometheus** (FREE tier available)
   - VictoriaMetrics backend
   - CPU, memory, network metrics out-of-the-box
   - Custom metrics via `/metrics` endpoint
   - Source: [Fly.io Metrics Docs](https://fly.io/docs/monitoring/metrics/)

2. **Managed Grafana**
   - Pre-configured dashboards
   - Searchable app logs (beta)
   - Alert manager integration
   - Source: [Fly App Grafana Dashboard](https://grafana.com/grafana/dashboards/14741-fly-app/)

3. **Cost Structure**
   - Prometheus metrics: **Free** (basic tier forever)
   - High-volume metrics: Future paid tiers
   - Grafana: Included
   - Source: [Fly.io Blog - Hooking Up Fly Metrics](https://fly.io/blog/hooking-up-fly-metrics/)

### 4.2 Custom Metrics on Fly.io

**Expose Metrics Endpoint:**

```rust
// crates/ampel-api/src/main.rs
async fn main() {
    let app = Router::new()
        // ... your routes
        .route("/metrics", get(metrics_handler));

    // Bind to Fly.io's internal network
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**Fly.io Configuration (`fly.toml`):**

```toml
app = "ampel-prod"
primary_region = "sjc"

[build]
  dockerfile = "Dockerfile"

[env]
  RUST_LOG = "info,ampel=debug"
  OTEL_EXPORTER_OTLP_ENDPOINT = "https://otel-collector.fly.dev"

[[services]]
  internal_port = 8080
  protocol = "tcp"

  [[services.ports]]
    port = 80
    handlers = ["http"]

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

  # Health checks
  [[services.http_checks]]
    interval = "10s"
    timeout = "2s"
    grace_period = "5s"
    method = "GET"
    path = "/health"

  # Expose metrics for Fly's Prometheus
  [[services.http_checks]]
    interval = "15s"
    timeout = "5s"
    method = "GET"
    path = "/metrics"

[metrics]
  port = 8080
  path = "/metrics"
```

### 4.3 Cost-Effective Monitoring Strategy

**Recommended Setup:**

```
┌─────────────────────────────────────────────────┐
│              FLY.IO PRODUCTION                  │
├─────────────────────────────────────────────────┤
│                                                 │
│  Ampel App → /metrics → Fly Prometheus (FREE)  │
│                              ↓                  │
│                       Fly Grafana (FREE)        │
│                                                 │
│  Logs → fly logs → Grafana (FREE)              │
│                                                 │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│           EXTERNAL SERVICES (PAID)              │
├─────────────────────────────────────────────────┤
│                                                 │
│  • Sentry (errors + performance)  ~$26/mo      │
│  • Grafana Cloud (traces)         FREE tier    │
│  • Datadog RUM (optional)         ~$15/mo      │
│                                                 │
│  TOTAL: ~$26-41/month                          │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Alternative: Grafana Cloud Free Tier**

For side projects, use Fly.io's Prometheus with Grafana Cloud's generous free tier:

- 50GB logs
- 50GB traces
- 10K series metrics
- 14-day retention

Source: [Grafana Cloud Pricing](https://grafana.com/pricing/)

### 4.4 Database Monitoring on Fly.io

**Fly Postgres Monitoring:**

Fly.io provides managed Postgres with built-in metrics:

- Connection pool stats
- Query performance
- Replication lag
- Disk usage

**Custom Metrics with pg_stat_statements:**

```sql
-- Enable query statistics
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Query for slow queries
SELECT
    query,
    calls,
    total_exec_time,
    mean_exec_time,
    max_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 20;
```

**Export to Prometheus:**
Use `postgres_exporter` as a Fly.io app:

```toml
# fly.postgres-exporter.toml
app = "ampel-postgres-exporter"

[env]
  DATA_SOURCE_NAME = "postgresql://user:pass@ampel-db.internal:5432/ampel?sslmode=disable"

[[services]]
  internal_port = 9187
  protocol = "tcp"
```

### 4.5 Redis Monitoring on Fly.io

**Upstash Redis (Fly.io Partner):**

- Built-in metrics dashboard
- REST API for metrics
- Free tier: 10K commands/day

**Self-hosted Redis:**
Deploy `redis_exporter` as a Fly.io app similar to postgres-exporter.

---

## 5. Alerting Strategy

### 5.1 Alert Definitions

**Critical Alerts (Page immediately):**

```yaml
# observability/grafana/provisioning/alerting/alerts.yml
apiVersion: 1

groups:
  - name: ampel-critical
    interval: 1m
    rules:
      - alert: HighErrorRate
        expr: |
          sum(rate(axum_http_requests_total{status=~"5.."}[5m]))
          /
          sum(rate(axum_http_requests_total[5m])) > 0.05
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: 'High error rate ({{ $value | humanizePercentage }})'
          description: 'API is returning 5xx errors at {{ $value | humanizePercentage }} rate'

      - alert: DatabaseDown
        expr: up{job="postgres"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: 'PostgreSQL is down'

      - alert: HighDatabaseConnections
        expr: |
          sum(pg_stat_database_numbackends{datname="ampel"})
          /
          pg_settings_max_connections > 0.8
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: 'Database connection pool nearly exhausted'

      - alert: WorkerJobsFailing
        expr: |
          sum(rate(worker_jobs_total{status="failed"}[5m]))
          /
          sum(rate(worker_jobs_total[5m])) > 0.2
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: 'Worker jobs failing at {{ $value | humanizePercentage }} rate'
```

**Warning Alerts (Investigate during business hours):**

```yaml
- name: ampel-warning
  interval: 5m
  rules:
    - alert: HighP95Latency
      expr: |
        histogram_quantile(0.95,
          sum(rate(axum_http_requests_duration_seconds_bucket[5m])) by (le)
        ) > 1.0
      for: 10m
      labels:
        severity: warning
      annotations:
        summary: 'P95 latency is {{ $value }}s'

    - alert: HighMemoryUsage
      expr: |
        (process_resident_memory_bytes / node_memory_MemTotal_bytes) > 0.85
      for: 10m
      labels:
        severity: warning
      annotations:
        summary: 'Memory usage is {{ $value | humanizePercentage }}'

    - alert: LowCacheHitRatio
      expr: redis_keyspace_hits_total / (redis_keyspace_hits_total + redis_keyspace_misses_total) < 0.8
      for: 15m
      labels:
        severity: warning
      annotations:
        summary: 'Redis cache hit ratio is {{ $value | humanizePercentage }}'
```

### 5.2 Alert Routing

**Grafana Contact Points:**

```yaml
# observability/grafana/provisioning/alerting/contact-points.yml
apiVersion: 1

contactPoints:
  - name: pagerduty-critical
    uid: pagerduty-critical
    type: pagerduty
    settings:
      integrationKey: ${PAGERDUTY_INTEGRATION_KEY}
      severity: critical

  - name: slack-warnings
    uid: slack-warnings
    type: slack
    settings:
      url: ${SLACK_WEBHOOK_URL}
      title: 'Ampel Warning Alert'

  - name: email-team
    uid: email-team
    type: email
    settings:
      addresses: ops@yourcompany.com
```

**Notification Policies:**

```yaml
# observability/grafana/provisioning/alerting/policies.yml
apiVersion: 1

policies:
  - receiver: pagerduty-critical
    group_by: ['alertname', 'severity']
    group_wait: 10s
    group_interval: 10s
    repeat_interval: 12h
    matchers:
      - severity = critical

  - receiver: slack-warnings
    group_by: ['alertname']
    group_wait: 30s
    group_interval: 5m
    repeat_interval: 4h
    matchers:
      - severity = warning

  - receiver: email-team
    group_by: ['alertname']
    repeat_interval: 24h
    matchers:
      - severity = info
```

---

## 6. Implementation Roadmap

### Phase 1: Foundation (Week 1)

**Backend:**

- [ ] Add `tracing`, `tracing-subscriber` dependencies
- [ ] Implement `telemetry.rs` module with OpenTelemetry
- [ ] Add structured logging to all handlers
- [ ] Add `axum-prometheus` middleware
- [ ] Expose `/metrics` endpoint
- [ ] Add basic custom metrics (DB queries, job execution)

**Frontend:**

- [ ] Install `web-vitals` library
- [ ] Implement Core Web Vitals tracking
- [ ] Add `/api/vitals` backend endpoint
- [ ] Set up React Error Boundary
- [ ] Add console error tracking

**Infrastructure:**

- [ ] Create `docker-compose.observability.yml`
- [ ] Set up Prometheus + Grafana locally
- [ ] Configure basic Grafana dashboards

**Estimated Effort:** 2-3 days

### Phase 2: Enhanced Metrics (Week 2)

**Backend:**

- [ ] Instrument all Axum handlers with spans
- [ ] Add SeaORM query metrics
- [ ] Add Apalis worker metrics middleware
- [ ] Configure log levels per environment
- [ ] Add request ID propagation

**Frontend:**

- [ ] Integrate Sentry (free tier)
- [ ] Add session replay (Sentry or LogRocket)
- [ ] Track React Query performance
- [ ] Add custom performance marks

**Infrastructure:**

- [ ] Add Loki + Promtail for logs
- [ ] Add Tempo for distributed tracing
- [ ] Configure postgres-exporter
- [ ] Configure redis-exporter
- [ ] Create comprehensive Grafana dashboards

**Estimated Effort:** 3-4 days

### Phase 3: Production Setup (Week 3)

**Fly.io Configuration:**

- [ ] Configure Fly.io metrics endpoint
- [ ] Set up Fly Grafana dashboards
- [ ] Configure production log aggregation
- [ ] Set up Sentry production project
- [ ] Configure alert rules in Grafana

**Cost Optimization:**

- [ ] Evaluate Grafana Cloud free tier vs self-hosted
- [ ] Configure metric retention policies
- [ ] Set up log sampling for high-volume endpoints
- [ ] Review trace sampling rates

**Documentation:**

- [ ] Document observability architecture
- [ ] Create runbook for common alerts
- [ ] Document metric naming conventions
- [ ] Create dashboard screenshots for docs

**Estimated Effort:** 2-3 days

### Phase 4: Refinement (Week 4)

**Optimization:**

- [ ] Fine-tune alert thresholds based on real data
- [ ] Add SLO/SLI dashboards
- [ ] Implement error budget tracking
- [ ] Add capacity planning dashboards

**Advanced Features:**

- [ ] Implement custom business metrics (PRs merged, polling frequency)
- [ ] Add user journey tracking
- [ ] Implement feature flag tracking
- [ ] Add A/B test metrics

**Team Enablement:**

- [ ] Train team on using Grafana
- [ ] Establish on-call rotation
- [ ] Create incident response playbook
- [ ] Set up weekly metrics review

**Estimated Effort:** 3-4 days

---

## 7. Cost Analysis

### 7.1 Local Development (Docker Compose)

| Service    | Resource Usage | Cost   |
| ---------- | -------------- | ------ |
| Prometheus | ~200MB RAM     | $0     |
| Grafana    | ~150MB RAM     | $0     |
| Loki       | ~100MB RAM     | $0     |
| Tempo      | ~150MB RAM     | $0     |
| Exporters  | ~50MB RAM      | $0     |
| **TOTAL**  | ~650MB RAM     | **$0** |

### 7.2 Fly.io Production

**Minimum Setup (Cost-Effective):**

| Service              | Configuration    | Monthly Cost   |
| -------------------- | ---------------- | -------------- |
| Fly.io Metrics       | Free tier        | $0             |
| Fly.io Grafana       | Included         | $0             |
| Fly.io Logs          | Included         | $0             |
| Sentry (Errors)      | 5K events/month  | $0 (free tier) |
| Sentry (Performance) | 10K transactions | $26            |
| **TOTAL**            |                  | **$26/month**  |

**Recommended Setup:**

| Service                | Configuration                      | Monthly Cost     |
| ---------------------- | ---------------------------------- | ---------------- |
| Fly.io Metrics         | Free tier                          | $0               |
| Fly.io Grafana         | Included                           | $0               |
| Grafana Cloud          | Free tier (50GB logs, 10K metrics) | $0               |
| Sentry                 | Developer plan                     | $26              |
| Datadog RUM (optional) | 10K sessions                       | $15              |
| **TOTAL**              |                                    | **$26-41/month** |

**Enterprise Setup:**

| Service            | Configuration    | Monthly Cost    |
| ------------------ | ---------------- | --------------- |
| Fly.io Metrics     | High volume      | ~$20            |
| Grafana Cloud      | Pro tier         | $50             |
| Datadog Full Stack | APM + RUM + Logs | $200            |
| PagerDuty          | Professional     | $25/user        |
| **TOTAL**          |                  | **$295+/month** |

Source: [Grafana Pricing](https://grafana.com/pricing/)

### 7.3 Cost Optimization Tips

1. **Use Sampling Strategically**
   - Traces: 10% in production (0.1 sample rate)
   - Logs: Sample verbose logs, keep errors at 100%
   - Metrics: Aggregate high-cardinality labels

2. **Retention Policies**
   - Metrics: 30 days (local), 15 days (production)
   - Logs: 7 days (local), 14 days (production)
   - Traces: 7 days

3. **Free Tier Maximization**
   - Sentry: 5K errors/month free
   - Grafana Cloud: 50GB logs, 10K metrics, 50GB traces
   - Fly.io: Prometheus metrics free forever

---

## 8. Best Practices Summary

### 8.1 Naming Conventions

**Metrics:**

```
<namespace>_<subsystem>_<name>_<unit>

Examples:
- ampel_http_requests_total
- ampel_db_query_duration_seconds
- ampel_worker_jobs_active
- ampel_cache_hit_ratio
```

**Logs:**

```
# Use structured logging with consistent fields
{
  "timestamp": "2025-12-22T12:00:00Z",
  "level": "info",
  "target": "ampel_api::handlers::auth",
  "message": "User logged in",
  "user_id": "uuid-here",
  "request_id": "req-123",
  "duration_ms": 45
}
```

**Traces:**

```
# Span naming: <service>.<operation>
ampel-api.http.request
ampel-api.db.query
ampel-worker.job.execute
```

### 8.2 Instrumentation Guidelines

**What to Instrument:**

✅ **DO Instrument:**

- All HTTP endpoints (automatic with middleware)
- Database queries (add custom metrics for slow queries)
- Background jobs (duration, success/failure)
- External API calls (GitHub, GitLab, Bitbucket)
- Cache operations (hits, misses)
- Authentication attempts (success, failure)
- Business metrics (PRs merged, repositories added)

❌ **DON'T Instrument:**

- Health check endpoints (creates noise)
- Internal service calls (unless debugging)
- Every single function (causes overhead)

**Metric Types:**

- **Counter**: Monotonically increasing (requests_total, errors_total)
- **Gauge**: Point-in-time value (active_connections, memory_usage)
- **Histogram**: Distribution of values (request_duration, response_size)
- **Summary**: Similar to histogram, client-side quantiles

### 8.3 Performance Overhead

**Expected Impact:**

| Component                    | CPU Overhead | Memory Overhead | Latency Impact |
| ---------------------------- | ------------ | --------------- | -------------- |
| tracing-subscriber           | <1%          | ~5MB            | <1ms           |
| Prometheus metrics           | <2%          | ~10MB           | <1ms           |
| OpenTelemetry (10% sampling) | <3%          | ~15MB           | <2ms           |
| Sentry (error tracking)      | <1%          | ~5MB            | 0ms (async)    |

**Total Overhead:** <5% CPU, ~35MB RAM, <5ms latency

### 8.4 Security Considerations

**Sensitive Data in Logs:**

```rust
use tracing::info;

// ❌ BAD: Logs sensitive data
info!(password = %user.password, "User login");

// ✅ GOOD: Redact sensitive fields
info!(user_id = %user.id, "User login successful");
```

**Metrics with PII:**

```rust
// ❌ BAD: Email in metric label
counter!("user_logins_total", 1, "email" => user.email);

// ✅ GOOD: Hash or omit PII
counter!("user_logins_total", 1, "user_hash" => hash(user.email));
```

**Sentry Scrubbing:**

```typescript
Sentry.init({
  beforeSend(event, hint) {
    // Remove sensitive data
    if (event.request?.headers) {
      delete event.request.headers['Authorization'];
      delete event.request.headers['Cookie'];
    }
    return event;
  },
});
```

---

## 9. Recommended Libraries & Versions (2025)

### 9.1 Rust Backend

```toml
[dependencies]
# Core tracing
tracing = "0.1.40"
tracing-subscriber = { version = "0.3.18", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.27.0"

# OpenTelemetry
opentelemetry = { version = "0.27.0", features = ["trace", "metrics"] }
opentelemetry-otlp = { version = "0.27.0", features = ["tokio", "tls"] }
opentelemetry_sdk = { version = "0.27.0", features = ["rt-tokio"] }

# Metrics
metrics = "0.24.0"
metrics-exporter-prometheus = { version = "0.16.0", default-features = false }
axum-prometheus = "0.8.0"

# Error tracking
sentry = { version = "0.35.0", features = ["tracing", "tower"] }
sentry-tower = "0.35.0"

# Async runtime
tokio = { version = "1.41.1", features = ["full", "tracing"] }
```

### 9.2 Frontend (React)

```json
{
  "dependencies": {
    "web-vitals": "^4.2.4",
    "@sentry/react": "^8.43.0",
    "react": "^19.0.0"
  }
}
```

### 9.3 Docker Images

```yaml
# Recommended versions (2025)
prometheus: prom/prometheus:v2.50.0
grafana: grafana/grafana:10.3.0
loki: grafana/loki:2.9.5
tempo: grafana/tempo:2.3.1
postgres-exporter: prometheuscommunity/postgres-exporter:v0.15.0
redis-exporter: oliver006/redis_exporter:v1.57.0
```

---

## 10. Code Examples

### 10.1 Complete Axum Service with Observability

```rust
// crates/ampel-api/src/main.rs
use axum::{
    Router,
    routing::{get, post},
    middleware,
};
use axum_prometheus::PrometheusMetricLayer;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;

mod telemetry;
mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry
    telemetry::init_telemetry(
        "ampel-api",
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
    )?;

    // Initialize Sentry
    let _guard = if let Ok(dsn) = std::env::var("SENTRY_DSN") {
        Some(telemetry::init_sentry(&dsn))
    } else {
        None
    };

    // Create Prometheus metrics layer
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    // Build router
    let app = Router::new()
        // Health check (no metrics)
        .route("/health", get(handlers::health_check))

        // Metrics endpoint
        .route("/metrics", get(|| async move { metric_handle.render() }))

        // API routes
        .route("/api/users", get(handlers::get_users))
        .route("/api/repositories", get(handlers::get_repositories))

        // Apply middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(prometheus_layer)
                .layer(sentry_tower::NewSentryLayer::new_from_top())
        );

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Starting server on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Cleanup
    telemetry::shutdown_telemetry();
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    tracing::info!("Shutdown signal received");
}
```

### 10.2 Instrumented Handler

```rust
// crates/ampel-api/src/handlers/repositories.rs
use axum::{Extension, Json};
use tracing::{instrument, info, error};
use uuid::Uuid;
use metrics::{counter, histogram};

#[instrument(
    skip(pool),
    fields(
        user_id = %user_id,
        otel.kind = "server"
    )
)]
pub async fn get_repositories(
    Extension(pool): Extension<PgPool>,
    user_id: Uuid,
) -> Result<Json<Vec<Repository>>, AppError> {
    info!("Fetching repositories");

    let start = std::time::Instant::now();

    let repos = sqlx::query_as!(
        Repository,
        "SELECT * FROM repositories WHERE user_id = $1",
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Database query failed");
        counter!("db_errors_total", 1, "operation" => "fetch_repositories");
        AppError::Database(e)
    })?;

    let duration = start.elapsed();
    histogram!("db_query_duration_seconds", duration.as_secs_f64(), "query" => "fetch_repositories");
    counter!("repositories_fetched_total", repos.len() as u64);

    info!(count = repos.len(), "Repositories fetched successfully");
    Ok(Json(repos))
}
```

### 10.3 Worker Job Instrumentation

```rust
// crates/ampel-worker/src/jobs/poll_repository.rs
use apalis::prelude::*;
use tracing::{instrument, info, error, warn};
use metrics::{counter, histogram, gauge};

#[instrument(
    skip(ctx),
    fields(
        repo_id = %job.repository_id,
        otel.kind = "worker"
    )
)]
pub async fn poll_repository(
    job: PollRepositoryJob,
    ctx: JobContext,
) -> Result<(), JobError> {
    let start = std::time::Instant::now();
    gauge!("worker_active_jobs", 1.0, "job_type" => "poll_repository");

    info!("Starting repository poll");

    match execute_poll(&job).await {
        Ok(pr_count) => {
            let duration = start.elapsed();
            histogram!("worker_job_duration_seconds", duration.as_secs_f64(), "job_type" => "poll_repository", "status" => "success");
            counter!("worker_jobs_total", 1, "job_type" => "poll_repository", "status" => "success");
            counter!("pull_requests_polled_total", pr_count as u64);

            info!(pr_count, duration_ms = duration.as_millis(), "Poll completed successfully");
            Ok(())
        }
        Err(e) => {
            let duration = start.elapsed();
            histogram!("worker_job_duration_seconds", duration.as_secs_f64(), "job_type" => "poll_repository", "status" => "error");
            counter!("worker_jobs_total", 1, "job_type" => "poll_repository", "status" => "error");

            error!(error = %e, duration_ms = duration.as_millis(), "Poll failed");
            Err(JobError::Failed(e.to_string()))
        }
    }
    .map(|r| {
        gauge!("worker_active_jobs", -1.0, "job_type" => "poll_repository");
        r
    })
    .map_err(|e| {
        gauge!("worker_active_jobs", -1.0, "job_type" => "poll_repository");
        e
    })
}
```

### 10.4 Frontend Web Vitals Integration

```typescript
// frontend/src/lib/vitals.ts
import { onCLS, onINP, onLCP, onFCP, onTTFB, type Metric } from 'web-vitals';

interface VitalMetric extends Metric {
  url?: string;
  userAgent?: string;
}

function sendToAnalytics(metric: Metric) {
  const vitalsUrl = import.meta.env.VITE_VITALS_ENDPOINT || '/api/vitals';

  const body = JSON.stringify({
    name: metric.name,
    value: metric.value,
    rating: metric.rating,
    delta: metric.delta,
    id: metric.id,
    navigationType: metric.navigationType,
    url: window.location.href,
    userAgent: navigator.userAgent,
  });

  // Use beacon API for reliability
  if (navigator.sendBeacon) {
    navigator.sendBeacon(vitalsUrl, body);
  } else {
    fetch(vitalsUrl, {
      body,
      method: 'POST',
      keepalive: true,
      headers: { 'Content-Type': 'application/json' },
    }).catch(console.error);
  }
}

export function initVitals() {
  // Core Web Vitals (2025)
  onCLS(sendToAnalytics);
  onINP(sendToAnalytics); // Replaces FID as of March 2024
  onLCP(sendToAnalytics);

  // Additional metrics
  onFCP(sendToAnalytics);
  onTTFB(sendToAnalytics);
}

// In main.tsx:
// import { initVitals } from './lib/vitals';
// if (import.meta.env.PROD) { initVitals(); }
```

---

## 11. Monitoring Mermaid Diagrams

### 11.1 Data Flow Architecture

```mermaid
graph TB
    subgraph "Ampel Application"
        API[Axum API<br/>Port 8080]
        Worker[Apalis Worker<br/>Port 8081]
        Frontend[React Frontend<br/>Port 5173]
    end

    subgraph "Observability Stack"
        Prometheus[Prometheus<br/>Port 9090]
        Loki[Loki<br/>Port 3100]
        Tempo[Tempo<br/>Port 4317/4318]
        Grafana[Grafana<br/>Port 3000]
    end

    subgraph "Exporters"
        PgExp[PostgreSQL<br/>Exporter]
        RedisExp[Redis<br/>Exporter]
    end

    subgraph "External Services"
        Sentry[Sentry<br/>Error Tracking]
    end

    API -->|/metrics| Prometheus
    Worker -->|/metrics| Prometheus
    API -->|logs| Loki
    Worker -->|logs| Loki
    API -->|traces OTLP| Tempo
    Worker -->|traces OTLP| Tempo
    Frontend -->|web-vitals| API
    Frontend -->|errors| Sentry

    PgExp -->|metrics| Prometheus
    RedisExp -->|metrics| Prometheus

    Prometheus --> Grafana
    Loki --> Grafana
    Tempo --> Grafana

    style API fill:#bbf
    style Worker fill:#bbf
    style Frontend fill:#bfb
    style Grafana fill:#fb9
```

### 11.2 Alerting Flow

```mermaid
graph LR
    Prometheus[Prometheus] -->|evaluates| Rules[Alert Rules]
    Rules -->|fires| AlertManager[Alert Manager]
    AlertManager -->|routes| Router{Severity?}

    Router -->|critical| PagerDuty[PagerDuty<br/>On-Call]
    Router -->|warning| Slack[Slack<br/>#alerts]
    Router -->|info| Email[Email<br/>Team Inbox]

    PagerDuty -->|acknowledges| Incident[Incident Response]
    Slack -->|investigates| Dashboard[Grafana Dashboard]

    style Router fill:#f96
    style PagerDuty fill:#f66
    style Slack fill:#fc6
    style Email fill:#9cf
```

---

## 12. Key Takeaways

### For Local Development

1. **Start Simple**: Prometheus + Grafana is sufficient
2. **Add Gradually**: Loki for logs, Tempo for traces as needed
3. **Use Exporters**: postgres-exporter and redis-exporter for database metrics
4. **Test Alerts**: Validate alert rules locally before production

### For Production (Fly.io)

1. **Use Managed Services**: Fly's Prometheus and Grafana are free and reliable
2. **External for Errors**: Sentry for error tracking (better UX than self-hosted)
3. **Optimize Costs**: Use Grafana Cloud free tier for traces
4. **Sample Wisely**: 10% trace sampling is usually sufficient

### For the Team

1. **Instrument Early**: Add observability during development, not after
2. **Document Alerts**: Every alert should have a runbook
3. **Review Metrics**: Weekly metric reviews catch issues early
4. **Train Everyone**: Entire team should know how to use Grafana

---

## 13. References & Sources

### Rust Observability

- [OpenTelemetry Rust Documentation](https://opentelemetry.io/docs/languages/rust/)
- [tracing-opentelemetry crate](https://docs.rs/tracing-opentelemetry)
- [Datadog: Monitor Rust with OpenTelemetry](https://www.datadoghq.com/blog/monitor-rust-otel/)
- [Last9: Getting Started with OpenTelemetry in Rust](https://last9.io/blog/opentelemetry-in-rust/)
- [Flexible Tracing with Rust and OpenTelemetry OTLP](https://broch.tech/posts/rust-tracing-opentelemetry/)

### Axum & Prometheus

- [axum-prometheus GitHub](https://github.com/Ptrskay3/axum-prometheus)
- [Axum Prometheus metrics example](https://github.com/tokio-rs/axum/blob/main/examples/prometheus-metrics/src/main.rs)
- [Exporting Prometheus metrics with Axum](https://ellie.wtf/notes/exporting-prometheus-metrics-with-axum)
- [Axum App Monitoring with Prometheus and Grafana](https://carlosmv.hashnode.dev/how-to-monitor-your-axum-app-with-prometheus-and-grafana)

### React & RUM

- [React Performance Monitoring 2025](https://medium.com/@mernstackdevbykevin/react-performance-monitoring-with-web-vitals-the-2025-developers-guide-661559271932)
- [web.dev: Measuring Web Vitals](https://web.dev/articles/vitals-measurement-getting-started)
- [Datadog: Monitor Core Web Vitals](https://www.datadoghq.com/blog/core-web-vitals-monitoring-datadog-rum-synthetics/)
- [Top 8 Real User Monitoring Tools 2025](https://cubeapm.com/blog/top-real-user-monitoring-tools-features-pricing/)

### Fly.io

- [Fly.io Metrics Documentation](https://fly.io/docs/monitoring/metrics/)
- [Fly.io: Hooking Up Fly Metrics](https://fly.io/blog/hooking-up-fly-metrics/)
- [Fly App Grafana Dashboard](https://grafana.com/grafana/dashboards/14741-fly-app/)
- [Fly's Prometheus Metrics](https://fly.io/blog/measuring-fly/)

### Docker & Infrastructure

- [Redis Monitoring with Prometheus and Grafana](https://nelsoncode.medium.com/how-to-monitor-redis-with-prometheus-and-grafana-docker-6eb33a5ea998)
- [Setup Redis Monitoring with Grafana in Docker Compose](https://medium.com/the-quantified-world/setup-redis-monitoring-with-grafana-in-docker-compose-e10d3e386bf3)
- [n8n Monitoring with Prometheus & Grafana](https://community.n8n.io/t/automation-script-to-self-hosting-n8n-with-docker-compose-both-single-and-queue-mode-and-monitoring-with-prometheus-grafana/187720)
- [Grafana Redis Data Source](https://grafana.com/grafana/plugins/redis-datasource/)

### Pricing & Tools

- [Grafana Pricing](https://grafana.com/pricing/)
- [Sentry Pricing](https://sentry.io/pricing/)

---

## 14. Conclusion

This research provides a comprehensive, production-ready observability strategy for Ampel tailored to:

1. **Local Development**: Docker Compose with Prometheus, Grafana, Loki, and Tempo
2. **Production**: Fly.io managed services + Sentry + optional Grafana Cloud
3. **Cost-Effectiveness**: ~$26-41/month for full observability

The recommended stack balances:

- **Developer Experience**: Easy local setup with familiar tools
- **Production Reliability**: Proven, battle-tested components
- **Cost Efficiency**: Maximizing free tiers and managed services
- **Future Scalability**: Can upgrade to Datadog/New Relic if needed

### Next Steps

1. Review this research with the team
2. Prioritize implementation phases
3. Start with Phase 1 (Foundation) in local environment
4. Test thoroughly before Fly.io deployment
5. Iterate based on real production data

**Research Complete.** Ready for implementation planning.
