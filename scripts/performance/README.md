# Ampel Performance Testing Suite

Comprehensive performance benchmarking tools for the Ampel PR management dashboard.

## Overview

This directory contains scripts and tools for measuring and validating performance targets defined in `/docs/performance/BENCHMARKS.md`.

## Prerequisites

### Required Services

- PostgreSQL (port 5432)
- Redis (port 6379)
- API server (port 8080)
- Frontend dev server (port 5173) - for frontend tests only

### Required Tools

```bash
# Core tools (required)
curl
redis-cli
jq
bc

# Optional tools (for enhanced testing)
npm install -g lighthouse
npm install -g lighthouse-ci
```

## Quick Start

### 1. Start All Services

```bash
# In terminal 1: Start database
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=ampel postgres:16

# In terminal 2: Start Redis
redis-server

# In terminal 3: Start API
cd /path/to/ampel
make dev-api

# In terminal 4: Start frontend (optional, for frontend tests)
make dev-frontend

# In terminal 5: Start worker (optional)
make dev-worker
```

### 2. Run Complete Benchmark Suite

```bash
./scripts/performance/run-benchmarks.sh
```

This will:

- Check service availability
- Run backend performance tests
- Measure Redis cache performance
- Analyze frontend bundle size
- Run Lighthouse audit (if frontend is running)
- Generate comprehensive report

### 3. View Results

Results are saved to `./scripts/performance/results/`:

- `performance_report.md` - Main report
- `backend_metrics.txt` - Backend timing metrics
- `frontend_metrics.txt` - Frontend metrics
- `bundle_sizes.txt` - Bundle size breakdown
- `redis_stats.txt` - Cache statistics
- `lighthouse-report.json` - Lighthouse audit (if run)

## Individual Test Scripts

### Backend API Load Testing

```bash
./scripts/performance/api-load-test.sh
```

Tests API endpoints with varying load patterns:

- Health endpoint baseline
- Authentication endpoints
- Diff endpoints (with/without cache)

### Frontend Bundle Analysis

```bash
cd frontend
pnpm run build
npx vite-bundle-visualizer
```

Analyzes:

- Total bundle size
- Code splitting effectiveness
- Vendor chunk sizes
- Lazy-loaded chunk sizes

### Lighthouse CI

```bash
# With config file
npx lighthouse-ci autorun --config=./scripts/performance/lighthouse-config.json

# Manual run
lighthouse http://localhost:5173 \
  --output=html \
  --output-path=./results/lighthouse.html \
  --only-categories=performance
```

## Performance Targets

### Backend (API)

| Metric                 | Target |
| ---------------------- | ------ |
| Uncached diff endpoint | <2s    |
| Cached diff endpoint   | <500ms |
| Redis cache hit rate   | >87%   |
| Redis cache latency    | <10ms  |

### Frontend

| Metric                         | Target |
| ------------------------------ | ------ |
| Bundle size increase           | <150KB |
| Largest Contentful Paint (LCP) | <2.5s  |
| First Input Delay (FID)        | <100ms |
| Cumulative Layout Shift (CLS)  | <0.1   |
| Performance Score              | >90    |

### Load Time by PR Size

| PR Size    | Files  | Target |
| ---------- | ------ | ------ |
| Small      | 1-10   | <500ms |
| Medium     | 10-50  | <1s    |
| Large      | 50-200 | <2s    |
| Very Large | 200+   | <3s    |

### Scroll Performance

- Frame rate: 60fps
- Virtual scrolling smooth with 500+ files

## Continuous Integration

### GitHub Actions Integration

Add to `.github/workflows/performance.yml`:

```yaml
name: Performance Tests
on:
  pull_request:
    paths:
      - 'frontend/**'
      - 'crates/ampel-api/**'

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup services
        run: |
          docker-compose up -d postgres redis

      - name: Build backend
        run: cargo build --release

      - name: Build frontend
        run: |
          cd frontend
          pnpm install
          pnpm run build

      - name: Run benchmarks
        run: ./scripts/performance/run-benchmarks.sh

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: performance-results
          path: scripts/performance/results/
```

## Troubleshooting

### Services Not Running

```bash
# Check services
curl http://localhost:8080/health  # API
redis-cli ping                      # Redis
curl http://localhost:5173          # Frontend

# Start missing services
make dev-api        # API
redis-server        # Redis
make dev-frontend   # Frontend
```

### No Test Data

The benchmarks work best with actual PR data in the database. To seed test data:

```bash
# Use the API to create test PRs or
# Import sample data if available
```

### Lighthouse Fails

```bash
# Install Lighthouse
npm install -g lighthouse

# Ensure frontend is running
curl http://localhost:5173

# Run with verbose output
lighthouse http://localhost:5173 --view
```

### Redis Cache Empty

```bash
# Check Redis status
redis-cli info stats

# Populate cache by accessing endpoints
curl http://localhost:8080/api/v1/pull-requests

# Clear cache and test cold start
redis-cli flushdb
```

## Advanced Usage

### Custom Load Testing

```bash
# High concurrency test
ab -n 1000 -c 10 http://localhost:8080/api/v1/pull-requests

# Sustained load test
wrk -t4 -c100 -d30s http://localhost:8080/health
```

### Bundle Size Monitoring

```bash
# Compare before/after
cd frontend
git checkout main
pnpm run build
du -sh dist/assets > /tmp/before.txt

git checkout feature-branch
pnpm run build
du -sh dist/assets > /tmp/after.txt

diff /tmp/before.txt /tmp/after.txt
```

### Memory Profiling

```bash
# Backend memory
ps aux | grep ampel-api

# Redis memory
redis-cli info memory

# Frontend memory (use Chrome DevTools)
```

## References

- [Performance Benchmarks Documentation](/docs/performance/BENCHMARKS.md)
- [Web Vitals](https://web.dev/vitals/)
- [Lighthouse CI](https://github.com/GoogleChrome/lighthouse-ci)
- [Apache Bench](https://httpd.apache.org/docs/2.4/programs/ab.html)
