# Grafana Dashboards and Visualization

Complete guide to creating, managing, and optimizing Grafana dashboards for Ampel.

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Pre-configured Dashboards](#pre-configured-dashboards)
4. [Creating Custom Dashboards](#creating-custom-dashboards)
5. [Panel Types](#panel-types)
6. [Variables and Templating](#variables-and-templating)
7. [Alert Configuration](#alert-configuration)
8. [Best Practices](#best-practices)

---

## Overview

Grafana provides powerful visualization and dashboarding for Ampel's metrics.

**Key Features:**

- Interactive dashboards with drill-down capabilities
- Multiple visualization types (graphs, tables, heatmaps)
- Template variables for dynamic dashboards
- Alert rules with notification channels
- Dashboard sharing and embedding

**Access:** http://localhost:3000 (local development)
**Default Credentials:** admin/admin (change on first login)

---

## Getting Started

### First-Time Setup

1. **Access Grafana:**

   ```bash
   open http://localhost:3000
   ```

2. **Login:**
   - Username: `admin`
   - Password: `admin`
   - Set new password when prompted

3. **Verify Datasource:**
   - Navigate to Configuration → Data Sources
   - Verify "Prometheus" datasource exists
   - URL should be: `http://prometheus:9090`
   - Click "Test" to verify connection

4. **View Dashboards:**
   - Go to Dashboards → Browse
   - Open "Ampel Overview" dashboard

### Quick Commands

```bash
# Start Grafana
make monitoring-up

# View logs
docker logs grafana

# Restart Grafana
docker restart grafana

# Import dashboards
make monitoring-import-dashboards

# Export dashboards
make monitoring-export-dashboards
```

---

## Pre-configured Dashboards

### Ampel Overview Dashboard

Main dashboard showing system health at a glance.

**Panels:**

1. **Request Rate**
   - Total requests/second across all endpoints
   - Grouped by HTTP method

2. **Error Rate**
   - Percentage of 5xx errors
   - Alert threshold at 5%

3. **Request Duration (P95)**
   - 95th percentile latency
   - Grouped by endpoint

4. **Status Code Distribution**
   - Pie chart showing 2xx/4xx/5xx distribution
   - Updated in real-time

5. **Database Connections**
   - Active vs max connections
   - Pool utilization percentage

6. **Active Pull Requests**
   - PRs by status (green/yellow/red)
   - Business metric tracking

**Refresh Rate:** 5 seconds (configurable)

**Time Range:** Last 6 hours (default)

---

## Creating Custom Dashboards

### Step-by-Step Guide

#### 1. Create New Dashboard

```
Dashboards → New Dashboard → Add new panel
```

#### 2. Configure Query

Select data source: Prometheus

**Example queries:**

```promql
# Request rate
rate(http_requests_total[5m])

# Error rate
100 * (
  sum(rate(http_requests_total{status=~"5.."}[5m]))
  / sum(rate(http_requests_total[5m]))
)

# P95 latency
histogram_quantile(0.95,
  rate(http_request_duration_seconds_bucket[5m])
)
```

#### 3. Choose Visualization

- **Time series:** Line graphs, area charts
- **Stat:** Single value with sparkline
- **Gauge:** Progress indicators
- **Bar chart:** Categorical comparisons
- **Table:** Tabular data
- **Heatmap:** Time-based distributions

#### 4. Configure Panel Options

**Title:** Clear, descriptive name

**Description:** What the panel shows and why it matters

**Unit:** Select appropriate unit (seconds, bytes, percent)

**Thresholds:** Set warning/critical levels

**Legend:** Show/hide, position, format

#### 5. Save Dashboard

Give it a descriptive name and folder location.

### Example Panel Configurations

#### Request Rate Panel

```json
{
  "title": "Request Rate",
  "type": "timeseries",
  "targets": [
    {
      "expr": "sum(rate(http_requests_total[5m])) by (method)",
      "legendFormat": "{{method}}"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "reqps",
      "color": {
        "mode": "palette-classic"
      }
    }
  }
}
```

#### Error Rate Panel

```json
{
  "title": "Error Rate",
  "type": "stat",
  "targets": [
    {
      "expr": "100 * (sum(rate(http_requests_total{status=~\"5..\"}[5m])) / sum(rate(http_requests_total[5m])))"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "percent",
      "thresholds": {
        "steps": [
          { "value": 0, "color": "green" },
          { "value": 1, "color": "yellow" },
          { "value": 5, "color": "red" }
        ]
      }
    }
  }
}
```

---

## Panel Types

### Time Series

Best for: Metrics over time

**Configuration:**

- **Graph styles:** Lines, bars, points
- **Fill opacity:** 0-100%
- **Line width:** 1-10px
- **Point size:** 1-20px

**Example query:**

```promql
rate(http_requests_total[5m])
```

### Stat

Best for: Single value with context

**Features:**

- Big number display
- Sparkline for trend
- Threshold-based colors
- Comparison to previous value

**Example query:**

```promql
sum(rate(http_requests_total[5m]))
```

### Gauge

Best for: Percentage or capacity metrics

**Configuration:**

- Min/max values
- Threshold ranges
- Color stops

**Example query:**

```promql
(db_connections_active / db_connections_max) * 100
```

### Bar Chart

Best for: Categorical comparisons

**Example query:**

```promql
sum(http_requests_total) by (path)
```

### Table

Best for: Multiple metrics or dimensions

**Configuration:**

- Column overrides
- Cell display modes
- Sortable columns

**Example query:**

```promql
topk(10, rate(http_requests_total[5m]))
```

### Heatmap

Best for: Latency distributions over time

**Example query:**

```promql
sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
```

---

## Variables and Templating

Make dashboards dynamic with template variables.

### Creating Variables

Navigate to: Dashboard Settings → Variables → Add variable

#### Instance Variable

```
Name: instance
Type: Query
Query: label_values(up, instance)
```

**Usage in query:**

```promql
rate(http_requests_total{instance=~"$instance"}[5m])
```

#### Endpoint Variable

```
Name: endpoint
Type: Query
Query: label_values(http_requests_total, path)
Multi-value: true
Include All: true
```

**Usage in query:**

```promql
rate(http_requests_total{path=~"$endpoint"}[5m])
```

#### Time Range Variable

```
Name: time_range
Type: Interval
Values: 5m,15m,1h,6h,24h
```

**Usage in query:**

```promql
rate(http_requests_total[$time_range])
```

### Variable Options

**Types:**

- Query: From Prometheus labels
- Custom: Comma-separated values
- Constant: Single value
- Interval: Time ranges
- Data source: Select data source

**Options:**

- Multi-value: Select multiple values
- Include All: Add "All" option
- Regex: Filter values with regex

---

## Alert Configuration

### Creating Alerts

1. **Edit panel** → Alert tab
2. **Configure rule:**

   ```
   Name: High Error Rate
   Evaluate every: 1m
   For: 5m
   Condition: WHEN last() OF query(A) IS ABOVE 5
   ```

3. **Set notification channel:**
   - Slack
   - Email
   - PagerDuty
   - Webhook

### Alert States

- **OK:** Condition not met
- **Pending:** Condition met but waiting for "For" duration
- **Alerting:** Condition met for "For" duration
- **No Data:** No data received

### Notification Channels

#### Slack

```
Type: Slack
Webhook URL: https://hooks.slack.com/services/...
Channel: #ampel-alerts
```

#### Email

```
Type: Email
Addresses: ops@example.com
```

#### PagerDuty

```
Type: PagerDuty
Integration Key: <your-integration-key>
Severity: critical
```

---

## Best Practices

### Dashboard Design

**1. Use consistent naming:**

- Clear, descriptive titles
- Consistent capitalization
- Include units in panel titles

**2. Organize logically:**

- Most important metrics at top
- Group related panels in rows
- Use consistent panel sizes

**3. Optimize performance:**

- Limit time ranges
- Use recording rules for expensive queries
- Set reasonable refresh intervals

**4. Add context:**

- Include panel descriptions
- Document query logic
- Add links to runbooks

### Query Optimization

**Use recording rules:**

```promql
# ❌ Expensive query on every refresh
histogram_quantile(0.95,
  sum(rate(http_request_duration_seconds_bucket[5m])) by (le, path)
)

# ✅ Pre-computed recording rule
job_path:http_latency:p95
```

**Limit cardinality:**

```promql
# ❌ High cardinality
sum(rate(http_requests_total[5m])) by (method, path, status, instance)

# ✅ Lower cardinality
sum(rate(http_requests_total[5m])) by (method)
```

**Set appropriate intervals:**

- **5s refresh:** Critical metrics only
- **10s refresh:** Important dashboards
- **30s refresh:** Overview dashboards
- **1m refresh:** Long-term trends

### Color and Thresholds

**Standard thresholds:**

- **Green:** < 70% (OK)
- **Yellow:** 70-90% (Warning)
- **Red:** > 90% (Critical)

**Example:**

```json
"thresholds": {
  "steps": [
    { "value": 0, "color": "green" },
    { "value": 70, "color": "yellow" },
    { "value": 90, "color": "red" }
  ]
}
```

### Dashboard Variables

**Benefits:**

- Single dashboard for multiple environments
- Dynamic filtering by instance/endpoint
- Reduced maintenance burden

**Example:**

```promql
# Query with variables
rate(http_requests_total{
  instance=~"$instance",
  path=~"$endpoint",
  method=~"$method"
}[$time_range])
```

---

## Importing/Exporting Dashboards

### Export Dashboard

1. Open dashboard
2. Dashboard settings → JSON Model
3. Copy JSON or Download

Or via CLI:

```bash
make monitoring-export-dashboards
```

### Import Dashboard

1. Dashboards → Import
2. Paste JSON or upload file
3. Select data source
4. Import

Or via CLI:

```bash
make monitoring-import-dashboards
```

### Version Control

Store dashboard JSON in Git:

```
monitoring/grafana/dashboards/
├── ampel-overview.json
├── database-performance.json
└── business-metrics.json
```

---

## Troubleshooting

### Dashboard Shows "No Data"

**Check:**

1. Data source is connected
2. Prometheus has data for time range
3. Query syntax is correct
4. Time range matches available data

**Test query in Prometheus UI:**

```bash
open http://localhost:9090
```

### Panels Load Slowly

**Solutions:**

1. Use recording rules for expensive queries
2. Reduce time range
3. Limit cardinality (fewer `by` labels)
4. Increase refresh interval

### Variables Not Working

**Check:**

1. Variable query returns values
2. Variable name matches in queries
3. Regex is correct
4. All option is enabled if needed

---

## Resources

- [Grafana Documentation](https://grafana.com/docs/)
- [PromQL Basics](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)
- [Community Dashboards](https://grafana.com/grafana/dashboards/)

---

**Last Updated:** 2025-12-22
**Maintainer:** Ampel Platform Team
