# Ampel Monitoring Configuration

This directory contains the configuration for Ampel's observability stack.

## Directory Structure

```
monitoring/
├── README.md                    # This file
├── prometheus.yml               # Prometheus configuration
├── alerts/
│   └── ampel.yml                # Alert rules
├── grafana/
│   ├── datasources/
│   │   └── prometheus.yml       # Prometheus datasource
│   └── dashboards/
│       ├── ampel-overview.json  # Main dashboard
│       └── dashboard-provider.yml
```

## Quick Start

```bash
# Start monitoring stack
make monitoring-up

# Access Grafana
open http://localhost:3000  # admin/admin

# Access Prometheus
open http://localhost:9090

# View logs
make monitoring-logs

# Stop monitoring
make monitoring-down
```

## Services

### Prometheus (port 9090)

Metrics storage and querying engine. Scrapes metrics from:

- Ampel API (:8080/metrics)
- Ampel Worker (:8081/metrics)
- PostgreSQL Exporter (:9187)
- Redis Exporter (:9121)

### Grafana (port 3000)

Visualization and dashboarding. Pre-configured with:

- Prometheus datasource
- Ampel Overview dashboard
- Default credentials: admin/admin

### Exporters

- **postgres-exporter** (port 9187) - PostgreSQL database metrics
- **redis-exporter** (port 9121) - Redis cache metrics

### Loki (port 3100)

Log aggregation service for centralized logging.

## Metrics

### HTTP Metrics

- `http_requests_total{method, path, status}` - Total requests
- `http_request_duration_seconds{method, path, status}` - Request duration histogram

### Custom Application Metrics

Add custom metrics in your Rust code:

```rust
use metrics::{counter, histogram, gauge};

// Increment counter
counter!("pull_requests_synced_total",
    "provider" => "github",
    "status" => "success"
).increment(1);

// Record histogram
histogram!("sync_duration_seconds",
    "provider" => "github"
).record(duration.as_secs_f64());

// Set gauge
gauge!("active_repositories").set(count as f64);
```

## Alerts

Configured alerts in `alerts/ampel.yml`:

1. **HighErrorRate** - Triggers when error rate >5% for 5 minutes
2. **HighLatency** - Triggers when P95 latency >1s for 10 minutes
3. **DatabaseDown** - Triggers when PostgreSQL is unreachable
4. **HighDatabaseConnections** - Triggers when connections >80
5. **ServiceDown** - Triggers when service is unavailable for 2 minutes

## Dashboards

### Ampel Overview

Main dashboard showing:

- HTTP request rate by endpoint
- Request duration (P95)
- HTTP status code distribution
- Database connections
- Active pull requests

### Creating Custom Dashboards

1. Open Grafana at http://localhost:3000
2. Create new dashboard
3. Add panels with PromQL queries
4. Export JSON to `grafana/dashboards/`

Example PromQL queries:

```promql
# Request rate
rate(http_requests_total[5m])

# Error rate percentage
(sum(rate(http_requests_total{status=~"5.."}[5m]))
/
sum(rate(http_requests_total[5m]))) * 100

# P95 latency
histogram_quantile(0.95,
  rate(http_request_duration_seconds_bucket[5m])
)

# Database connections by database
pg_stat_database_numbackends
```

## Configuration

### Prometheus Scrape Intervals

Edit `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s # How often to scrape targets
  evaluation_interval: 15s # How often to evaluate rules
```

### Retention Settings

Default retention: 15 days (configurable in docker-compose)

To change, update command in `docker/docker-compose.monitoring.yml`:

```yaml
prometheus:
  command:
    - '--storage.tsdb.retention.time=30d'
    - '--storage.tsdb.retention.size=10GB'
```

### Alertmanager (Optional)

To add alert notifications:

1. Add alertmanager service to docker-compose
2. Update `prometheus.yml` alerting section
3. Configure notification channels (email, Slack, PagerDuty)

Example alertmanager config:

```yaml
receivers:
  - name: 'team-notifications'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
        channel: '#alerts'
```

## Troubleshooting

### Prometheus not scraping

Check targets page: http://localhost:9090/targets

Common issues:

- Service not exposing /metrics endpoint
- Wrong port in prometheus.yml
- Network connectivity (check docker network)

### Grafana dashboard empty

1. Verify Prometheus datasource: Configuration > Data Sources
2. Check data exists: Explore > Run query
3. Verify time range matches your data

### High memory usage

Reduce scrape frequency or retention:

```yaml
global:
  scrape_interval: 30s  # Increase from 15s

# Or add retention limits
--storage.tsdb.retention.time=7d
--storage.tsdb.retention.size=5GB
```

## Production Deployment

### Security Checklist

- [ ] Change Grafana admin password
- [ ] Enable authentication on Prometheus
- [ ] Restrict metrics endpoint to monitoring network
- [ ] Use TLS for all connections
- [ ] Set up alerting with on-call rotation
- [ ] Configure backup for Prometheus data

### Fly.io Integration

For Fly.io deployments, metrics are automatically available:

```bash
# View Fly.io metrics
fly dashboard -a ampel-api

# Configure /metrics endpoint
# Add to fly.toml:
[metrics]
  port = 8080
  path = "/metrics"
```

See [docs/observability/OBSERVABILITY.md](../docs/observability/OBSERVABILITY.md) for details.

## Resources

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [PromQL Tutorial](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Ampel Observability Guide](../docs/observability/)
