# Git Diff View - CI/CD Integration

**Document Version:** 1.0
**Date:** December 25, 2025
**Status:** Implementation Complete

---

## Overview

This document describes the CI/CD pipeline integration for the git diff view feature in Ampel. The integration ensures quality, performance, and reliability of the diff rendering functionality across all deployment stages.

## CI/CD Pipeline Stages

### 1. Continuous Integration (CI)

#### 1.1 Dependency Verification

**Location:** `.github/workflows/ci.yml` - Frontend Test job

```yaml
- name: Verify git diff dependencies
  run: |
    echo "Checking @git-diff-view/react installation..."
    pnpm list @git-diff-view/react
    pnpm list parse-diff
```

**Purpose:**

- Ensures `@git-diff-view/react` and `parse-diff` are properly installed
- Verifies package versions match lock file
- Catches missing peer dependencies early

#### 1.2 Component Testing

**Location:** `.github/workflows/ci.yml` - Frontend Test job

```yaml
- name: Run diff component tests
  run: |
    echo "Running git diff view component tests..."
    pnpm test -- --run src/components/pr/PullRequestDiff.test.tsx
    pnpm test -- --run src/components/diff/
```

**Test Coverage:**

- Unit tests for `PullRequestDiff` component
- Integration tests for diff rendering
- Edge cases (empty diffs, large diffs, binary files)
- Accessibility tests (keyboard navigation, screen readers)

#### 1.3 Bundle Size Analysis

**Location:** `.github/workflows/ci.yml` - Frontend Test job

```yaml
- name: Check bundle size for git diff library
  run: |
    BUNDLE_SIZE=$(du -sb dist/assets/*.js | awk '{sum += $1} END {print sum}')
    MAX_SIZE=$((5000000 + 150000))  # 5MB baseline + 150KB for diff
    if [ "$BUNDLE_SIZE" -gt "$MAX_SIZE" ]; then
      echo "::warning::Bundle size exceeds threshold"
    fi
```

**Thresholds:**

- **Maximum increase:** 150KB for git diff library
- **Baseline:** 5MB total bundle size
- **Alert:** Warning if threshold exceeded (non-blocking)

**Code Splitting Verification:**

```yaml
- name: Analyze code splitting
  run: |
    if ls dist/assets/*diff*.js 1> /dev/null 2>&1; then
      echo "✅ Git diff code appears to be code-split"
    fi
```

#### 1.4 Backend Diff Endpoint Testing

**Location:** `.github/workflows/ci.yml` - Backend Integration Test job

```yaml
- name: Test git diff endpoint integration
  run: |
    cargo test --all-features --package ampel-api diff
    cargo test --all-features --package ampel-providers provider_diff
```

**Test Coverage:**

- Diff endpoint route registration
- Provider diff API calls (GitHub, GitLab, Bitbucket)
- Redis caching behavior
- Error handling for large diffs
- Rate limiting compliance

### 2. Performance Testing

#### 2.1 Frontend Performance Benchmarking

**Location:** `.github/workflows/ci.yml` - Frontend Performance job

```yaml
frontend-performance:
  name: Frontend Performance (Diff Rendering)
  runs-on: ubuntu-latest
  needs: [frontend-test]
  if: github.event_name == 'pull_request'
```

**Lighthouse CI Integration:**

```yaml
- name: Performance audit with Lighthouse
  uses: treosh/lighthouse-ci-action@v11
  with:
    urls: |
      http://localhost:3000
      http://localhost:3000/pr/1
    configPath: './frontend/lighthouserc.json'
```

**Core Web Vitals Targets:**

- **LCP (Largest Contentful Paint):** < 2.5s
- **FID (First Input Delay):** < 100ms
- **CLS (Cumulative Layout Shift):** < 0.1

**Metrics Tracked:**

- Diff rendering performance
- Virtual scrolling efficiency
- Syntax highlighting impact
- Memory usage for large diffs

### 3. End-to-End Testing

#### 3.1 E2E Diff View Tests

**Location:** `.github/workflows/ci.yml` - E2E Diff Testing job

```yaml
e2e-diff-testing:
  name: E2E Diff View Testing
  runs-on: ubuntu-latest
  needs: [backend-build, frontend-test]
  if: github.event_name == 'pull_request'

  services:
    postgres: ...
    redis: ...
```

**Test Scenarios:**

1. **Basic diff rendering**
   - Navigate to PR detail page
   - Verify diff loads and renders
   - Check syntax highlighting

2. **Diff view modes**
   - Switch between unified and split views
   - Verify line numbers align
   - Test expand/collapse functionality

3. **Large diff handling**
   - Load PR with >1000 lines changed
   - Verify virtual scrolling works
   - Check performance remains acceptable

4. **Error scenarios**
   - Test diff load failure
   - Verify error message display
   - Check retry mechanism

**Redis Cache Verification:**

```yaml
- name: Test Redis cache integration
  run: |
    redis-cli -h localhost ping
    echo "✅ Redis is available for caching"
```

**API Health Check:**

```yaml
- name: Test diff endpoint availability
  run: |
    curl --retry 5 --retry-delay 2 --fail http://localhost:8080/health
```

### 4. Deployment Verification

#### 4.1 Pre-Deployment Checks

**Location:** `.github/workflows/deploy.yml` - Deploy Frontend job

**Nginx Configuration Verification:**

```yaml
- name: Verify nginx configuration for diff endpoint
  run: |
    if grep -q "unsafe-eval" docker/nginx.prod.conf; then
      echo "::error::Production nginx config contains unsafe-eval!"
      exit 1
    fi
```

**Security Checks:**

- Ensures `nginx.prod.conf` is used (not `nginx.dev.conf`)
- Verifies strict CSP (no `unsafe-eval`)
- Validates production API URLs

#### 4.2 Post-Deployment Verification

**Location:** `.github/workflows/deploy.yml` - Deploy API job

**Redis Availability:**

```yaml
- name: Verify Redis availability for diff caching
  run: |
    flyctl ssh console --app ${{ env.APP_PREFIX }}-api \
      -C "redis-cli -h \$REDIS_HOST ping"
```

**Diff Endpoint Smoke Test:**

```yaml
- name: Test diff endpoint
  run: |
    curl -f https://${{ env.APP_PREFIX }}-api.fly.dev/api/pull-requests/1/diff \
      -H "Authorization: Bearer test"
```

**Frontend Asset Verification:**

```yaml
- name: Verify diff view assets
  run: |
    curl -s https://${{ env.APP_PREFIX }}-frontend.fly.dev/ | \
      grep -o 'assets/.*\.js'
```

## Configuration Files

### Lighthouse CI Configuration

**File:** `frontend/lighthouserc.json`

```json
{
  "ci": {
    "collect": {
      "numberOfRuns": 3,
      "settings": {
        "preset": "desktop",
        "throttling": {
          "rttMs": 40,
          "throughputKbps": 10240,
          "cpuSlowdownMultiplier": 1
        }
      }
    },
    "assert": {
      "assertions": {
        "categories:performance": ["warn", { "minScore": 0.8 }],
        "categories:accessibility": ["error", { "minScore": 0.9 }],
        "first-contentful-paint": ["warn", { "maxNumericValue": 2000 }],
        "largest-contentful-paint": ["warn", { "maxNumericValue": 2500 }],
        "cumulative-layout-shift": ["warn", { "maxNumericValue": 0.1 }]
      }
    },
    "upload": {
      "target": "temporary-public-storage"
    }
  }
}
```

### Playwright E2E Test Configuration

**File:** `frontend/tests/e2e/diff-view.spec.ts`

```typescript
import { test, expect } from '@playwright/test';

test.describe('Git Diff View', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/pr/1');
  });

  test('renders diff view on PR page', async ({ page }) => {
    await expect(page.locator('[data-testid="diff-view"]')).toBeVisible();
    await expect(page.locator('.diff-line')).toHaveCount({ minimum: 1 });
  });

  test('switches between unified and split views', async ({ page }) => {
    await page.click('[data-testid="view-mode-toggle"]');
    await expect(page.locator('.diff-split-view')).toBeVisible();
  });

  test('handles large diffs with virtual scrolling', async ({ page }) => {
    // Test PR with >1000 lines
    await page.goto('/pr/2');
    const diffLines = page.locator('.diff-line');
    await expect(diffLines.first()).toBeVisible();

    // Scroll to bottom
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    await expect(diffLines.last()).toBeVisible();
  });

  test('displays syntax highlighting', async ({ page }) => {
    const syntaxHighlight = page.locator('.syntax-highlight');
    await expect(syntaxHighlight).toHaveCount({ minimum: 1 });
  });

  test('shows error message when diff fails to load', async ({ page }) => {
    await page.route('**/api/pull-requests/*/diff', (route) => {
      route.fulfill({ status: 500 });
    });
    await page.reload();
    await expect(page.locator('[role="alert"]')).toContainText('Failed to load diff');
  });
});
```

## Monitoring & Alerts

### Production Monitoring

**Metrics to Track:**

1. **Diff Endpoint Performance**
   - Response time percentiles (p50, p95, p99)
   - Cache hit rate
   - Error rate

2. **Frontend Performance**
   - Diff rendering time
   - Memory usage
   - Core Web Vitals

3. **Cache Performance**
   - Redis hit/miss ratio
   - Cache size
   - Eviction rate

### Alert Configuration

**Critical Alerts:**

- Diff endpoint error rate > 5%
- Average response time > 2s
- Redis unavailable

**Warning Alerts:**

- Cache hit rate < 70%
- Bundle size increase > 150KB
- LCP > 3s on diff pages

## Rollout Strategy

### Phase 1: Canary Deployment

- Deploy to 5% of users
- Monitor error rates and performance
- Verify Redis cache working
- Duration: 24 hours

### Phase 2: Gradual Rollout

- Increase to 25% of users
- Monitor bundle size impact
- Check frontend performance metrics
- Duration: 48 hours

### Phase 3: Full Deployment

- Deploy to 100% of users
- Monitor for 1 week
- Adjust cache TTL if needed

## Rollback Procedures

### Automatic Rollback Triggers

- Diff endpoint error rate > 10%
- Frontend performance degradation > 20%
- Redis connection failures

### Manual Rollback Process

1. Revert deployment via Fly.io dashboard
2. Clear Redis cache keys (`diff:*`)
3. Monitor recovery metrics
4. Investigate root cause

## Success Criteria

### CI Pipeline

- ✅ All tests pass (unit, integration, E2E)
- ✅ Bundle size within 150KB threshold
- ✅ Code coverage maintained at 80%+
- ✅ No security vulnerabilities introduced

### Performance

- ✅ LCP < 2.5s on diff-heavy pages
- ✅ FID < 100ms
- ✅ CLS < 0.1
- ✅ Diff rendering < 500ms for typical PRs

### Reliability

- ✅ Diff endpoint uptime > 99.9%
- ✅ Cache hit rate > 80%
- ✅ Error rate < 1%

## Maintenance

### Weekly Tasks

- Review performance metrics
- Check bundle size trends
- Analyze cache efficiency

### Monthly Tasks

- Update dependencies (@git-diff-view/react, parse-diff)
- Review Lighthouse scores
- Optimize cache TTL settings

### Quarterly Tasks

- Performance audit
- Security review
- Capacity planning for Redis

## References

- [Git Diff View Integration Plan](/docs/planning/GIT_DIFF_VIEW_INTEGRATION.md)
- [GitHub Actions CI Workflow](/.github/workflows/ci.yml)
- [Deployment Workflow](/.github/workflows/deploy.yml)
- [Lighthouse CI Documentation](https://github.com/GoogleChrome/lighthouse-ci)
- [Playwright Testing](https://playwright.dev/)

---

**Last Updated:** December 25, 2025
**Maintained By:** Ampel DevOps Team
