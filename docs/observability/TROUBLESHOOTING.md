# Observability Troubleshooting Guide

Common issues and solutions for Ampel's observability stack.

## Table of Contents

1. [Metrics Issues](#metrics-issues)
2. [Prometheus Issues](#prometheus-issues)
3. [Grafana Issues](#grafana-issues)
4. [Performance Issues](#performance-issues)
5. [Alert Issues](#alert-issues)
6. [Logging Issues](#logging-issues)

---

## Metrics Issues

### Metrics Not Appearing

**Symptom:** `/metrics` endpoint returns empty or metrics missing in Prometheus

**Diagnosis:**

```bash
# Check metrics endpoint
curl http://localhost:8080/metrics

# Expected output should include:
# http_requests_total{...}
# http_request_duration_seconds{...}
```

**Solutions:**

1. **Verify metrics middleware is enabled:**

   ```rust
   // In main.rs, ensure metrics recorder is initialized
   PrometheusBuilder::new()
       .install_recorder()
       .expect("failed to install Prometheus recorder");
   ```

2. **Check application is running:**

   ```bash
   curl http://localhost:8080/health
   ```

3. **Verify port mapping:**

   ```yaml
   # In docker-compose.yml
   ports:
     - '8080:8080' # Ensure port is exposed
   ```

4. **Check firewall/network:**
   ```bash
   # Test connectivity
   telnet localhost 8080
   ```

---

### High Cardinality Warnings

**Symptom:** Prometheus logs show cardinality warnings

```
level=warn msg="Many time series for metric" metric=http_requests_total
```

**Diagnosis:**

```promql
# Count unique time series per metric
count({__name__=~".+"}) by (__name__)
```

**Solutions:**

1. **Reduce label cardinality:**

   ```rust
   // ❌ Bad: User ID in labels
   counter!("requests_total", "user_id" => user_id.to_string())

   // ✅ Good: User type instead
   counter!("requests_total", "user_type" => "premium")
   ```

2. **Drop unnecessary labels:**

   ```yaml
   # In prometheus.yml
   metric_relabel_configs:
     - source_labels: [high_cardinality_label]
       action: labeldrop
   ```

3. **Aggregate metrics:**
   ```rust
   // Group by status instead of full error message
   counter!("errors_total", "error_type" => classify_error(&e))
   ```

---

### Metrics Showing Incorrect Values

**Symptom:** Counter decreasing or gauge values incorrect

**Solutions:**

1. **Counter reset:** Prometheus handles counter resets automatically
   - Use `rate()` or `increase()` instead of raw counter value

   ```promql
   # ✅ Correct: Handles resets
   rate(http_requests_total[5m])

   # ❌ Incorrect: Raw counter value
   http_requests_total
   ```

2. **Histogram bucket issues:**
   ```rust
   // Ensure buckets are in ascending order
   let buckets = vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];
   ```

---

## Prometheus Issues

### Prometheus Not Scraping

**Symptom:** Targets show as "DOWN" in Prometheus UI

**Diagnosis:**

```bash
# Check Prometheus targets
open http://localhost:9090/targets

# Or via API
curl http://localhost:9090/api/v1/targets
```

**Solutions:**

1. **Verify target configuration:**

   ```yaml
   # In prometheus.yml
   scrape_configs:
     - job_name: 'ampel-api'
       static_configs:
         - targets: ['api:8080'] # Check hostname and port
   ```

2. **Check network connectivity:**

   ```bash
   # From Prometheus container
   docker exec prometheus curl http://api:8080/metrics
   ```

3. **Verify scrape interval and timeout:**

   ```yaml
   scrape_interval: 15s
   scrape_timeout: 10s # Must be < scrape_interval
   ```

4. **Check Prometheus logs:**
   ```bash
   docker logs prometheus | grep -i error
   ```

---

### High Memory Usage

**Symptom:** Prometheus consuming excessive memory

**Diagnosis:**

```promql
# Check memory usage
process_resident_memory_bytes{job="prometheus"}

# Check time series count
prometheus_tsdb_head_series
```

**Solutions:**

1. **Reduce retention time:**

   ```yaml
   # docker/docker-compose.monitoring.yml
   command:
     - '--storage.tsdb.retention.time=15d' # Reduce from 30d
   ```

2. **Limit storage size:**

   ```yaml
   command:
     - '--storage.tsdb.retention.size=10GB'
   ```

3. **Drop high-cardinality metrics:**

   ```yaml
   # In prometheus.yml
   metric_relabel_configs:
     - source_labels: [__name__]
       regex: 'high_cardinality_metric.*'
       action: drop
   ```

4. **Increase scrape interval:**

   ```yaml
   scrape_interval: 30s # From 15s
   ```

5. **Enable WAL compression:**
   ```yaml
   command:
     - '--storage.tsdb.wal-compression'
   ```

---

### Slow Queries

**Symptom:** Grafana dashboards load slowly

**Diagnosis:**

```bash
# Enable query logging
# In prometheus.yml
--query.log-queries-longer-than=1s
```

**Solutions:**

1. **Use recording rules:**

   ```yaml
   # In recording-rules.yml
   - record: job:http_requests:rate5m
     expr: sum(rate(http_requests_total[5m])) by (job)
   ```

2. **Limit query range:**

   ```promql
   # ❌ Slow: Large range
   rate(http_requests_total[1d])

   # ✅ Fast: Smaller range
   rate(http_requests_total[5m])
   ```

3. **Reduce aggregation dimensions:**

   ```promql
   # ❌ Slow: Many labels
   sum(rate(http_requests_total[5m])) by (method, path, status, instance)

   # ✅ Fast: Fewer labels
   sum(rate(http_requests_total[5m])) by (method)
   ```

---

## Grafana Issues

### Dashboard Shows "No Data"

**Symptom:** Panels show "No data" even though Prometheus has data

**Diagnosis:**

1. **Check data source connection:**

   ```
   Configuration → Data Sources → Prometheus → Test
   ```

2. **Verify data exists in Prometheus:**
   ```bash
   open http://localhost:9090
   # Run same query in Explore tab
   ```

**Solutions:**

1. **Check time range:**
   - Dashboard time picker matches data availability
   - Adjust to wider range (e.g., Last 6 hours)

2. **Verify query syntax:**

   ```promql
   # Test query in Prometheus UI first
   rate(http_requests_total[5m])
   ```

3. **Check data source URL:**

   ```
   URL: http://prometheus:9090  # For Docker network
   OR
   URL: http://localhost:9090   # For host network
   ```

4. **Refresh dashboard:**
   - Click refresh button
   - Check auto-refresh interval

---

### Grafana Won't Start

**Symptom:** Cannot access Grafana at http://localhost:3000

**Diagnosis:**

```bash
# Check container status
docker ps | grep grafana

# Check logs
docker logs grafana
```

**Solutions:**

1. **Check port binding:**

   ```yaml
   # In docker/docker-compose.monitoring.yml
   ports:
     - '3000:3000'
   ```

2. **Verify data directory permissions:**

   ```bash
   # Grafana needs write access
   sudo chown -R 472:472 /data/grafana
   ```

3. **Check database migration:**

   ```bash
   # Look for migration errors in logs
   docker logs grafana | grep -i migration
   ```

4. **Reset Grafana database:**

   ```bash
   # Stop Grafana
   docker-compose down

   # Remove data
   rm -rf /data/grafana/*

   # Restart
   docker-compose up -d grafana
   ```

---

### Alerts Not Firing

**Symptom:** Alert rule configured but not triggering

**Diagnosis:**

1. **Check alert rule in Prometheus:**

   ```bash
   open http://localhost:9090/alerts
   ```

2. **Verify condition:**
   ```promql
   # Run alert query manually
   100 * (
     sum(rate(http_requests_total{status=~"5.."}[5m]))
     / sum(rate(http_requests_total[5m]))
   ) > 5
   ```

**Solutions:**

1. **Check "for" duration:**

   ```yaml
   - alert: HighErrorRate
     expr: error_rate > 5
     for: 5m # Must be breached for 5 minutes
   ```

2. **Verify notification channel:**

   ```
   Alerting → Contact points → Test
   ```

3. **Check alert state:**
   - **Inactive:** Condition not met
   - **Pending:** Condition met, waiting for "for" duration
   - **Firing:** Alert active

4. **Review Grafana alert logs:**
   ```bash
   docker logs grafana | grep -i alert
   ```

---

## Performance Issues

### Slow Application Response

**Symptom:** API responses slow, high latency in metrics

**Diagnosis:**

```promql
# P95 latency
histogram_quantile(0.95,
  rate(http_request_duration_seconds_bucket[5m])
)

# Slow queries
http_request_duration_seconds_bucket{le="+Inf"} - ignoring(le)
http_request_duration_seconds_bucket{le="1.0"}
```

**Solutions:**

1. **Identify slow endpoints:**

   ```promql
   topk(5,
     histogram_quantile(0.95,
       sum(rate(http_request_duration_seconds_bucket[5m])) by (le, path)
     )
   )
   ```

2. **Check database performance:**

   ```promql
   # Query duration
   db_query_duration_seconds

   # Connection pool
   db_connections_active / db_connections_max
   ```

3. **Review slow query logs:**

   ```bash
   docker logs ampel-api | grep "slow query"
   ```

4. **Check external API latency:**
   ```promql
   provider_api_duration_seconds{provider="github"}
   ```

---

### High Memory Usage

**Symptom:** Application memory growing continuously

**Diagnosis:**

```promql
# Memory usage
process_resident_memory_bytes{job="ampel-api"}

# Memory growth rate
rate(process_resident_memory_bytes[5m])
```

**Solutions:**

1. **Check for memory leaks:**

   ```bash
   # Use heaptrack
   heaptrack ./target/release/ampel-api
   ```

2. **Review memory-intensive operations:**

   ```bash
   # Look for large allocations in logs
   docker logs ampel-api | grep -i memory
   ```

3. **Adjust log verbosity:**

   ```bash
   # Reduce from debug to info
   export RUST_LOG=info
   ```

4. **Monitor Tokio tasks:**
   ```promql
   # Active async tasks
   tokio_tasks_active
   ```

---

## Alert Issues

### Too Many Alerts

**Symptom:** Alert fatigue from frequent notifications

**Solutions:**

1. **Increase "for" duration:**

   ```yaml
   - alert: HighErrorRate
     for: 10m # Increase from 5m
   ```

2. **Adjust thresholds:**

   ```yaml
   expr: error_rate > 10 # Increase from 5
   ```

3. **Add alert inhibition:**

   ```yaml
   # In alertmanager.yml
   inhibit_rules:
     - source_match:
         severity: 'critical'
       target_match:
         severity: 'warning'
   ```

4. **Group notifications:**
   ```yaml
   route:
     group_by: ['alertname', 'cluster']
     group_wait: 30s
     group_interval: 5m
   ```

---

### Missing Alerts

**Symptom:** Should be alerting but not receiving notifications

**Solutions:**

1. **Verify alert rule is loaded:**

   ```bash
   # Check Prometheus rules
   curl http://localhost:9090/api/v1/rules
   ```

2. **Check Alertmanager:**

   ```bash
   # View active alerts
   curl http://localhost:9093/api/v2/alerts
   ```

3. **Test notification channel:**

   ```bash
   # Send test alert
   curl -XPOST http://localhost:9093/api/v1/alerts -d '[...]'
   ```

4. **Review Alertmanager logs:**
   ```bash
   docker logs alertmanager
   ```

---

## Logging Issues

### Logs Not Appearing in Loki

**Symptom:** Grafana Explore shows no logs from Loki

**Diagnosis:**

```bash
# Check Loki health
curl http://localhost:3100/ready

# Check Promtail logs
docker logs promtail
```

**Solutions:**

1. **Verify Promtail configuration:**

   ```yaml
   # In promtail-config.yaml
   clients:
     - url: http://loki:3100/loki/api/v1/push
   ```

2. **Check log file paths:**

   ```yaml
   scrape_configs:
     - job_name: system
       static_configs:
         - targets:
             - localhost
           labels:
             __path__: /var/log/*.log
   ```

3. **Test log pipeline:**

   ```bash
   # Send test log
   curl -X POST http://localhost:3100/loki/api/v1/push \
     -H 'Content-Type: application/json' \
     -d '{"streams":[{"stream":{"job":"test"},"values":[["'$(date +%s)000000000'","test log"]]}]}'
   ```

4. **Check Loki limits:**
   ```yaml
   # In loki-config.yaml
   limits_config:
     ingestion_rate_mb: 10
     ingestion_burst_size_mb: 20
   ```

---

### Application Not Logging

**Symptom:** No structured logs appearing

**Diagnosis:**

```bash
# Check log output
docker logs ampel-api

# Verify RUST_LOG is set
docker exec ampel-api env | grep RUST_LOG
```

**Solutions:**

1. **Set log level:**

   ```bash
   export RUST_LOG=info,ampel=debug
   ```

2. **Verify tracing initialization:**

   ```rust
   tracing_subscriber::fmt()
       .with_env_filter(EnvFilter::from_default_env())
       .json()
       .init();
   ```

3. **Check log format:**

   ```rust
   // Ensure JSON format for production
   .json()
   ```

4. **Add instrumentation:**
   ```rust
   #[tracing::instrument]
   async fn my_function() {
       tracing::info!("Function called");
   }
   ```

---

## Quick Reference

### Common Commands

```bash
# Check all service health
make monitoring-health

# View logs
docker logs prometheus
docker logs grafana
docker logs ampel-api

# Restart services
make monitoring-restart

# Clean data and restart
make monitoring-clean && make monitoring-up

# Test metrics endpoint
curl http://localhost:8080/metrics | head -20

# Test Prometheus query
curl -G http://localhost:9090/api/v1/query --data-urlencode 'query=up'
```

### Health Check URLs

- Prometheus: http://localhost:9090/-/healthy
- Grafana: http://localhost:3000/api/health
- Loki: http://localhost:3100/ready
- API: http://localhost:8080/health

---

## Getting Help

If issues persist:

1. **Check documentation:**
   - [Monitoring Guide](MONITORING.md)
   - [Observability Guide](OBSERVABILITY.md)
   - [Metrics Catalog](METRICS.md)

2. **Review logs:**

   ```bash
   make monitoring-logs
   ```

3. **Search for similar issues:**
   - [Prometheus Troubleshooting](https://prometheus.io/docs/prometheus/latest/troubleshooting/)
   - [Grafana Troubleshooting](https://grafana.com/docs/grafana/latest/troubleshooting/)

4. **Create an issue:**
   - Include error messages
   - Provide configuration files
   - Share relevant logs

---

**Last Updated:** 2025-12-22
**Maintainer:** Ampel Platform Team
