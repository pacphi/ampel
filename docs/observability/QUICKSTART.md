# Observability Quick Start

Get Ampel's observability stack running in 5 minutes.

## Prerequisites

- Docker and Docker Compose installed
- Ampel services running (API, Worker, PostgreSQL, Redis)

## Start Monitoring Stack

```bash
# Start monitoring services
make monitoring-up

# Or manually
docker-compose -f docker/docker-compose.monitoring.yml up -d
```

This starts:

- **Prometheus** (http://localhost:9090) - Metrics storage
- **Grafana** (http://localhost:3000) - Dashboards (admin/admin)
- **Postgres Exporter** (http://localhost:9187/metrics)
- **Redis Exporter** (http://localhost:9121/metrics)
- **Loki** (http://localhost:3100) - Log aggregation

## Verify Setup

### Check Health Endpoints

```bash
# API health check
curl http://localhost:8080/health

# API readiness check
curl http://localhost:8080/ready

# View Prometheus metrics
curl http://localhost:8080/metrics
```

### Access Grafana

1. Open http://localhost:3000
2. Login with admin/admin
3. Navigate to Dashboards > Ampel Overview

## View Metrics

### Prometheus UI

1. Open http://localhost:9090
2. Try these queries:

```promql
# Request rate
rate(http_requests_total[5m])

# Error rate
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))

# Database connections
pg_stat_database_numbackends
```

### Grafana Dashboards

Pre-configured dashboard shows:

- HTTP request rate by endpoint
- Request duration (p95)
- Status code distribution
- Database connection count
- Active pull requests

## Common Commands

```bash
# View logs
make monitoring-logs

# Restart services
make monitoring-restart

# Check health
make monitoring-health

# Stop monitoring
make monitoring-down

# Clean all data
make monitoring-clean
```

## Alerts

Alerts are configured in `/monitoring/alerts/ampel.yml`:

- High error rate (>5% for 5min)
- High latency (p95 >1s for 10min)
- Database down
- Service unavailable

## Next Steps

- [Full Observability Guide](OBSERVABILITY.md)
- [Custom Metrics](API-ENDPOINTS.md#custom-application-metrics)
- [Monitoring Setup](MONITORING.md)
- [Distributed Tracing](OBSERVABILITY.md#distributed-tracing)

## Troubleshooting

### Prometheus not scraping metrics

Check prometheus.yml targets match your service ports:

```yaml
- targets: ['api:8080'] # Should match your API port
```

### Grafana dashboard empty

1. Verify Prometheus datasource: Configuration > Data Sources
2. Check Prometheus is scraping: http://localhost:9090/targets
3. Verify metrics endpoint returns data: curl http://localhost:8080/metrics

### High memory usage

Reduce retention period in prometheus.yml:

```yaml
global:
  scrape_interval: 30s # Increase from 15s
```

Or add storage retention flags:

```bash
--storage.tsdb.retention.time=15d
--storage.tsdb.retention.size=10GB
```
