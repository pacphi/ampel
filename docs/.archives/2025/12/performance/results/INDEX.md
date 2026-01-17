# Performance Benchmark Results Index

This directory contains all performance benchmark results and analysis for the Ampel Git Diff integration feature.

## Quick Links

- **[SUMMARY.md](./SUMMARY.md)** - Executive summary of all tests
- **[bundle-analysis.json](./bundle-analysis.json)** - Detailed bundle size metrics (JSON)
- **[../../docs/performance/BENCHMARK_RESULTS.md](../../docs/performance/BENCHMARK_RESULTS.md)** - Comprehensive results report
- **[../../docs/performance/BENCHMARKS.md](../../docs/performance/BENCHMARKS.md)** - Performance targets reference

## Results Overview

### Completed Tests ✅

| Test                     | Status      | Result | Report                                         |
| ------------------------ | ----------- | ------ | ---------------------------------------------- |
| Frontend Bundle Analysis | ✅ Complete | PASS   | [bundle-analysis.json](./bundle-analysis.json) |

### Pending Tests ⏳

| Test                    | Status     | Required Services        |
| ----------------------- | ---------- | ------------------------ |
| Backend API Performance | ⏳ Pending | PostgreSQL, Redis, API   |
| Redis Cache Performance | ⏳ Pending | Redis                    |
| Lighthouse CI           | ⏳ Pending | Frontend server          |
| Load Testing            | ⏳ Pending | All services + test data |
| Scroll Performance      | ⏳ Pending | Frontend server          |

## File Descriptions

### SUMMARY.md

Executive summary of benchmark execution including:

- Test completion status
- Key metrics and targets
- Infrastructure created
- Next steps

### bundle-analysis.json

Detailed JSON metrics for frontend bundle analysis:

- Individual chunk sizes
- Code splitting status
- Target validation
- Optimization flags
- Recommendations

### Future Files (Generated After Service Startup)

When you run the full benchmark suite, these files will be generated:

- `backend_metrics.txt` - API response times
- `redis_stats.txt` - Cache hit rates and latency
- `redis_latency.txt` - Redis latency history
- `frontend_build.log` - Build output
- `frontend_metrics.txt` - Frontend performance metrics
- `lighthouse-report.json` - Lighthouse audit results
- `performance_report.md` - Unified performance report
- `api_load_results.csv` - API load test results
- `load_test_report.md` - Load testing summary

## Running Tests

### Quick Start

```bash
# From project root
./scripts/performance/run-benchmarks.sh
```

### Individual Tests

```bash
# Backend API tests
./scripts/performance/api-load-test.sh

# Lighthouse audit
npx lighthouse http://localhost:5173 \
  --config-path=./scripts/performance/lighthouse-config.json \
  --output-path=./scripts/performance/results/lighthouse-report.json

# Bundle analysis (already done)
cd frontend && pnpm run build
```

## Results Timeline

| Date       | Test            | Result  | Notes                                               |
| ---------- | --------------- | ------- | --------------------------------------------------- |
| 2025-12-25 | Bundle Analysis | ✅ PASS | Diff chunk: 11.70 KB gzipped, 114.10 KB under limit |

## Metrics Storage

Performance metrics are stored in the memory namespace: `git-diff-completion/performance`

Access via claude-flow:

```bash
npx claude-flow@alpha hooks session-restore --session-id "git-diff-completion"
```

## CI/CD Integration

To add performance testing to CI pipeline, see:

- [GitHub Actions example](../README.md#continuous-integration)
- [Lighthouse CI configuration](../lighthouse-config.json)

## Contact

For questions about performance benchmarks:

- See [testing documentation](../../docs/TESTING.md)
- Review [performance benchmarks](../../docs/performance/BENCHMARKS.md)
- Check [API documentation](http://localhost:8080/api/docs) when services running
