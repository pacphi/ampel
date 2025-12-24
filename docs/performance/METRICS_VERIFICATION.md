# Prometheus Metrics Verification Guide

**Date:** December 24, 2025
**Purpose:** Quick verification that dashboard metrics are working correctly

---

## Quick Verification Steps

### 1. Build and Start the API Server

```bash
# Build the project
make build

# Start the API server
make dev-api
```

**Expected Output:**

```
Starting Ampel API server...
Metrics exporter initialized
Configuration loaded
Database connection established
Database migrations applied
Listening on http://0.0.0.0:8080
```

---

### 2. Verify Metrics Endpoint is Accessible

```bash
curl http://localhost:8080/metrics
```

**Expected:** You should see Prometheus-formatted metrics output. Look for the metrics infrastructure to be working:

```prometheus
# HELP http_requests_total http_requests_total
# TYPE http_requests_total counter
...
```

---

### 3. Trigger Dashboard Summary Endpoint

First, register and login to get an access token:

```bash
# Register a test user
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test User",
    "email": "metrics-test@example.com",
    "password": "TestPassword123!"
  }'

# Login to get access token
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "metrics-test@example.com",
    "password": "TestPassword123!"
  }' | jq -r '.data.accessToken')

echo "Access token: $TOKEN"
```

---

### 4. Call Dashboard Summary Multiple Times

```bash
# Call the dashboard summary endpoint 5 times
for i in {1..5}; do
  echo "Request $i:"
  curl -s -H "Authorization: Bearer $TOKEN" \
    http://localhost:8080/api/dashboard/summary | jq '.success'
  sleep 1
done
```

**Expected Output:**

```
Request 1:
true
Request 2:
true
Request 3:
true
Request 4:
true
Request 5:
true
```

---

### 5. Verify Dashboard Metrics Appear

```bash
curl -s http://localhost:8080/metrics | grep -A 15 "ampel_dashboard"
```

**Expected Output:**

```prometheus
# HELP ampel_dashboard_breakdown_total ampel_dashboard_breakdown_total
# TYPE ampel_dashboard_breakdown_total counter
ampel_dashboard_breakdown_total{visibility="green"} 0
ampel_dashboard_breakdown_total{visibility="red"} 0
ampel_dashboard_breakdown_total{visibility="yellow"} 0

# HELP ampel_dashboard_summary_duration_seconds ampel_dashboard_summary_duration_seconds
# TYPE ampel_dashboard_summary_duration_seconds histogram
ampel_dashboard_summary_duration_seconds_bucket{le="0.005"} 0
ampel_dashboard_summary_duration_seconds_bucket{le="0.01"} 2
ampel_dashboard_summary_duration_seconds_bucket{le="0.025"} 5
ampel_dashboard_summary_duration_seconds_bucket{le="0.05"} 5
ampel_dashboard_summary_duration_seconds_bucket{le="0.1"} 5
ampel_dashboard_summary_duration_seconds_bucket{le="0.25"} 5
ampel_dashboard_summary_duration_seconds_bucket{le="0.5"} 5
ampel_dashboard_summary_duration_seconds_bucket{le="1.0"} 5
ampel_dashboard_summary_duration_seconds_bucket{le="2.5"} 5
ampel_dashboard_summary_duration_seconds_bucket{le="5.0"} 5
ampel_dashboard_summary_duration_seconds_bucket{le="10.0"} 5
ampel_dashboard_summary_duration_seconds_bucket{le="+Inf"} 5
ampel_dashboard_summary_duration_seconds_sum 0.05234
ampel_dashboard_summary_duration_seconds_count 5
```

**Key Observations:**

- `ampel_dashboard_summary_duration_seconds_count` should equal the number of requests (5 in this example)
- `ampel_dashboard_summary_duration_seconds_sum` shows total time spent
- The bucket values show the distribution of response times
- `ampel_dashboard_breakdown_total` counters show PR counts by status (0 if no repos/PRs exist)

---

### 6. Verify Metrics are Increasing

Call the dashboard endpoint again and check if the count increases:

```bash
# Before
BEFORE=$(curl -s http://localhost:8080/metrics | grep "ampel_dashboard_summary_duration_seconds_count" | awk '{print $2}')
echo "Count before: $BEFORE"

# Make another request
curl -s -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/dashboard/summary > /dev/null

# After
AFTER=$(curl -s http://localhost:8080/metrics | grep "ampel_dashboard_summary_duration_seconds_count" | awk '{print $2}')
echo "Count after: $AFTER"

# Verify increment
if [ "$AFTER" -gt "$BEFORE" ]; then
  echo "✅ Metrics are incrementing correctly!"
else
  echo "❌ Metrics are not incrementing"
fi
```

**Expected Output:**

```
Count before: 5
Count after: 6
✅ Metrics are incrementing correctly!
```

---

### 7. Test with Data (If Available)

If you have test data with repositories and PRs:

```bash
# Add a test repository (example - adjust based on your provider setup)
curl -X POST http://localhost:8080/api/repositories \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-repo",
    "provider": "github",
    "externalId": "12345",
    "defaultBranch": "main"
  }'

# Call dashboard summary
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/dashboard/summary | jq '.data.statusCounts'

# Check breakdown metrics
curl -s http://localhost:8080/metrics | grep "ampel_dashboard_breakdown_total"
```

**Expected:** Breakdown counters should reflect the actual PR status distribution:

```prometheus
ampel_dashboard_breakdown_total{visibility="green"} 15
ampel_dashboard_breakdown_total{visibility="yellow"} 8
ampel_dashboard_breakdown_total{visibility="red"} 3
```

---

### 8. Test Error Metrics

Simulate a database error (optional - requires stopping the database):

```bash
# Stop the database temporarily
docker-compose stop postgres

# Try to call dashboard (should fail)
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/dashboard/summary

# Check error metrics
curl -s http://localhost:8080/metrics | grep "ampel_dashboard_errors_total"

# Restart database
docker-compose start postgres
```

**Expected:**

```prometheus
# HELP ampel_dashboard_errors_total ampel_dashboard_errors_total
# TYPE ampel_dashboard_errors_total counter
ampel_dashboard_errors_total{error_type="database"} 1
```

---

## Automated Verification Script

Save this as `scripts/verify-metrics.sh`:

```bash
#!/bin/bash
set -e

echo "=== Prometheus Metrics Verification ==="
echo ""

# Check if API is running
echo "1. Checking API server..."
if curl -s http://localhost:8080/health > /dev/null; then
  echo "   ✅ API server is running"
else
  echo "   ❌ API server is not running. Start it with: make dev-api"
  exit 1
fi

# Check metrics endpoint
echo ""
echo "2. Checking metrics endpoint..."
if curl -s http://localhost:8080/metrics | grep -q "# TYPE"; then
  echo "   ✅ Metrics endpoint is accessible"
else
  echo "   ❌ Metrics endpoint is not working"
  exit 1
fi

# Check for dashboard metrics
echo ""
echo "3. Checking for dashboard metrics..."
METRICS=$(curl -s http://localhost:8080/metrics | grep "ampel_dashboard" | wc -l)
if [ "$METRICS" -gt 0 ]; then
  echo "   ✅ Found $METRICS dashboard metric lines"
  echo ""
  echo "   Dashboard Metrics:"
  curl -s http://localhost:8080/metrics | grep "ampel_dashboard" | grep -E "(TYPE|HELP|_count|_sum)"
else
  echo "   ⚠️  No dashboard metrics found yet (call /api/dashboard/summary first)"
fi

echo ""
echo "=== Verification Complete ==="
```

**Usage:**

```bash
chmod +x scripts/verify-metrics.sh
./scripts/verify-metrics.sh
```

---

## Expected Metrics Summary

After successfully calling the dashboard endpoint, you should see these three metrics:

### 1. Duration Histogram

```prometheus
ampel_dashboard_summary_duration_seconds_bucket{le="..."} N
ampel_dashboard_summary_duration_seconds_sum X.XXX
ampel_dashboard_summary_duration_seconds_count N
```

### 2. Breakdown Counters

```prometheus
ampel_dashboard_breakdown_total{visibility="green"} N
ampel_dashboard_breakdown_total{visibility="yellow"} N
ampel_dashboard_breakdown_total{visibility="red"} N
```

### 3. Error Counter

```prometheus
ampel_dashboard_errors_total{error_type="database"} N
```

---

## Troubleshooting

### Metrics not appearing

**Problem:** `/metrics` endpoint returns empty or no dashboard metrics

**Solutions:**

1. Verify the API server is running: `curl http://localhost:8080/health`
2. Ensure you've called the dashboard endpoint at least once
3. Check server logs for errors: `make dev-api` (check terminal output)
4. Verify metrics initialization in logs: Look for "Metrics exporter initialized"

---

### Zero counts in breakdown metrics

**Problem:** `ampel_dashboard_breakdown_total` shows 0 for all statuses

**Expected Behavior:** This is normal if:

- User has no repositories configured
- Repositories have no open pull requests

**To Test with Data:**

1. Add repositories via `/api/repositories`
2. Ensure repositories have pull requests
3. Call dashboard summary again
4. Verify counters are now > 0

---

### Histogram shows no requests

**Problem:** `ampel_dashboard_summary_duration_seconds_count` is 0

**Solution:**

1. Verify you're authenticated: `curl -H "Authorization: Bearer $TOKEN" ...`
2. Check for 401/403 errors in response
3. Ensure the endpoint returns 200 OK
4. Check server logs for request handling

---

## Performance Targets

Based on the monitoring plan, verify these targets:

| Metric            | Target  | How to Check                          |
| ----------------- | ------- | ------------------------------------- |
| P50 Response Time | < 200ms | `histogram_quantile(0.50, ...)`       |
| P95 Response Time | < 500ms | `histogram_quantile(0.95, ...)`       |
| P99 Response Time | < 800ms | `histogram_quantile(0.99, ...)`       |
| Error Rate        | < 1%    | `errors_total / requests_total * 100` |

**Note:** Performance targets are for production with optimization. Development builds may be slower.

---

## Next Steps After Verification

1. ✅ **Verified Metrics Working** → Set up Prometheus scraping
2. ✅ **Prometheus Scraping** → Create Grafana dashboards
3. ✅ **Dashboards Created** → Configure alert rules
4. ✅ **Alerts Configured** → Monitor in production
5. ✅ **Production Monitoring** → Optimize based on metrics

---

**Document Status:** Complete
**Last Updated:** 2025-12-24
