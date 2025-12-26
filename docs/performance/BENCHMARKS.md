# Git Diff Integration Performance Benchmarks

## Overview

This document tracks performance metrics for the Git Diff integration feature across different PR sizes.

## Target Metrics (from Appendix F)

| PR Size | Files  | Target Load Time | Target Scroll | Bundle Impact |
| ------- | ------ | ---------------- | ------------- | ------------- |
| Small   | 1-10   | <500ms           | 60fps         | <150KB        |
| Medium  | 10-50  | <1s              | 60fps         | <150KB        |
| Large   | 50-200 | <2s              | 60fps         | <150KB        |

## Optimization Strategies Implemented

### Backend Optimizations

1. **Redis Caching**
   - Open PRs: 5-minute TTL
   - Closed/Merged PRs: 1-hour TTL
   - Cache key format: `pr:diff:{repo_id}:{pr_id}`

2. **Cache Invalidation**
   - Triggered on PR refresh endpoint
   - Webhook-based invalidation (future)

### Frontend Optimizations

1. **Code Splitting**
   - Lazy loading of GitDiffView component
   - Vendor chunks separation
   - Route-based code splitting

2. **Virtual Scrolling**
   - Only renders visible files (5 at a time)
   - ~400px per file item
   - Smooth scrolling at 60fps

3. **Lazy Rendering**
   - Diff patches only rendered when file is expanded
   - Collapsed files show summary only

4. **TanStack Query Optimization**
   - `staleTime: 5 minutes` for open PRs
   - `gcTime: 1 hour` for closed PRs
   - No refetch on window focus

## Bundle Size Analysis

### Before Optimization

- Total: ~500KB (baseline)

### After Optimization (Target)

- React vendors: ~140KB
- UI vendors: ~80KB
- Query vendors: ~60KB
- Diff view: ~50KB (lazy loaded)
- **Total increase: <150KB**

## Performance Measurement Tools

### Lighthouse CI

```bash
# Run Lighthouse audit
npx lighthouse-ci autorun --config=.lighthouserc.json
```

### Chrome DevTools Performance

1. Open DevTools → Performance tab
2. Record page load
3. Measure:
   - First Contentful Paint (FCP)
   - Time to Interactive (TTI)
   - Total Blocking Time (TBT)
   - Cumulative Layout Shift (CLS)

### React DevTools Profiler

1. Open React DevTools → Profiler
2. Start recording
3. Scroll through diff view
4. Measure render times per file

## Benchmark Results

### Small PR (1-10 files)

**Target: <500ms**

| Metric               | Result | Status |
| -------------------- | ------ | ------ |
| Cache Hit Load Time  | TBD    | ⏳     |
| Cache Miss Load Time | TBD    | ⏳     |
| Scroll FPS           | TBD    | ⏳     |
| Bundle Impact        | TBD    | ⏳     |

### Medium PR (10-50 files)

**Target: <1s**

| Metric               | Result | Status |
| -------------------- | ------ | ------ |
| Cache Hit Load Time  | TBD    | ⏳     |
| Cache Miss Load Time | TBD    | ⏳     |
| Scroll FPS           | TBD    | ⏳     |
| Bundle Impact        | TBD    | ⏳     |

### Large PR (50-200 files)

**Target: <2s**

| Metric               | Result | Status |
| -------------------- | ------ | ------ |
| Cache Hit Load Time  | TBD    | ⏳     |
| Cache Miss Load Time | TBD    | ⏳     |
| Scroll FPS           | TBD    | ⏳     |
| Bundle Impact        | TBD    | ⏳     |

## How to Run Benchmarks

### 1. Setup Test PRs

Create test PRs with different sizes:

```bash
# Small PR (5 files)
git checkout -b test-small-pr
# ... create 5 file changes
git push origin test-small-pr

# Medium PR (30 files)
git checkout -b test-medium-pr
# ... create 30 file changes
git push origin test-medium-pr

# Large PR (100 files)
git checkout -b test-large-pr
# ... create 100 file changes
git push origin test-large-pr
```

### 2. Measure Load Times

```javascript
// In browser console
performance.mark('diff-start');
// Load diff view
performance.mark('diff-end');
performance.measure('diff-load', 'diff-start', 'diff-end');
console.table(performance.getEntriesByType('measure'));
```

### 3. Measure Scroll Performance

```javascript
// Enable FPS meter in Chrome DevTools
// Settings → More tools → Rendering → FPS meter
// Scroll through diff view and observe FPS
```

### 4. Bundle Size Analysis

```bash
# Build and analyze
cd frontend
pnpm run build
npx vite-bundle-analyzer
```

## Continuous Monitoring

### CI Integration

Add to `.github/workflows/performance.yml`:

```yaml
name: Performance Tests
on: [pull_request]
jobs:
  lighthouse:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Lighthouse CI
        uses: treosh/lighthouse-ci-action@v9
        with:
          urls: |
            http://localhost:5173
          uploadArtifacts: true
          temporaryPublicStorage: true
```

## Future Optimizations

1. **Progressive Loading**
   - Load first 10 files immediately
   - Stream remaining files

2. **Web Workers**
   - Parse diffs in background thread
   - Offload syntax highlighting

3. **Compression**
   - Gzip/Brotli compression for API responses
   - Delta compression for similar files

4. **CDN Caching**
   - Cache static diff assets
   - Edge caching for public repositories

## References

- [Web Vitals](https://web.dev/vitals/)
- [Lighthouse Scoring](https://web.dev/performance-scoring/)
- [React Performance Optimization](https://react.dev/learn/render-and-commit)
- [Virtual Scrolling Best Practices](https://web.dev/virtualize-long-lists-react-window/)
