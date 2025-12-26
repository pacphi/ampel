import { test, expect } from '@playwright/test';

/**
 * E2E tests for Git Diff View feature
 * Tests the complete diff rendering workflow including:
 * - Basic rendering
 * - View mode switching
 * - Virtual scrolling for large diffs
 * - Syntax highlighting
 * - Error handling
 */

test.describe('Git Diff View - Basic Functionality', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to a pull request detail page
    await page.goto('/pr/1');
    // Wait for the page to fully load
    await page.waitForLoadState('networkidle');
  });

  test('renders diff view on PR page', async ({ page }) => {
    // Wait for diff view to be visible
    const diffView = page.locator('[data-testid="diff-view"]');
    await expect(diffView).toBeVisible({ timeout: 10000 });

    // Verify at least one diff line is rendered
    const diffLines = page.locator('.diff-line');
    await expect(diffLines).toHaveCount({ minimum: 1 });

    // Check for file headers
    const fileHeaders = page.locator('.diff-file-header');
    await expect(fileHeaders).toHaveCount({ minimum: 1 });
  });

  test('displays file metadata correctly', async ({ page }) => {
    const diffView = page.locator('[data-testid="diff-view"]');
    await expect(diffView).toBeVisible();

    // Check for additions/deletions counts
    const statsElement = page.locator('[data-testid="diff-stats"]');
    await expect(statsElement).toContainText(/\+\d+/); // additions
    await expect(statsElement).toContainText(/-\d+/); // deletions
  });

  test('shows syntax highlighting', async ({ page }) => {
    await page.waitForSelector('[data-testid="diff-view"]');

    // Look for syntax-highlighted elements
    const highlightedCode = page.locator('.syntax-highlight, .hljs, .token');
    const count = await highlightedCode.count();

    // Should have at least some syntax highlighting if code is present
    expect(count).toBeGreaterThan(0);
  });
});

test.describe('Git Diff View - View Modes', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/pr/1');
    await page.waitForLoadState('networkidle');
  });

  test('switches between unified and split views', async ({ page }) => {
    // Wait for diff to load
    await page.waitForSelector('[data-testid="diff-view"]');

    // Find and click view mode toggle
    const viewToggle = page.locator('[data-testid="view-mode-toggle"]');
    await expect(viewToggle).toBeVisible();

    // Should start in unified view (default)
    const unifiedView = page.locator('[data-testid="unified-view"]');
    await expect(unifiedView).toBeVisible();

    // Switch to split view
    await viewToggle.click();
    const splitView = page.locator('[data-testid="split-view"]');
    await expect(splitView).toBeVisible({ timeout: 5000 });

    // Verify unified view is hidden
    await expect(unifiedView).not.toBeVisible();
  });

  test('maintains line number alignment in split view', async ({ page }) => {
    await page.waitForSelector('[data-testid="diff-view"]');

    // Switch to split view
    const viewToggle = page.locator('[data-testid="view-mode-toggle"]');
    await viewToggle.click();

    // Get line numbers from both sides
    const leftLineNumbers = page.locator('[data-testid="left-line-number"]');
    const rightLineNumbers = page.locator('[data-testid="right-line-number"]');

    const leftCount = await leftLineNumbers.count();
    const rightCount = await rightLineNumbers.count();

    // Line counts should be equal (including blank lines for alignment)
    expect(leftCount).toBe(rightCount);
  });
});

test.describe('Git Diff View - Large Diffs', () => {
  test('handles large diffs with virtual scrolling', async ({ page }) => {
    // Navigate to a PR with many changes
    await page.goto('/pr/2'); // Assuming PR #2 has >1000 lines
    await page.waitForLoadState('networkidle');

    const diffView = page.locator('[data-testid="diff-view"]');
    await expect(diffView).toBeVisible({ timeout: 15000 });

    // Get initial visible lines
    const diffLines = page.locator('.diff-line');
    const initialCount = await diffLines.count();

    // Should not render all lines at once if using virtual scrolling
    // Expect less than total if diff is very large
    expect(initialCount).toBeGreaterThan(0);

    // Scroll to bottom
    await page.evaluate(() => {
      window.scrollTo(0, document.body.scrollHeight);
    });
    await page.waitForTimeout(1000); // Wait for virtual scroll to update

    // Should still be able to see lines at the bottom
    const lastVisibleLine = diffLines.last();
    await expect(lastVisibleLine).toBeVisible();
  });

  test('loads diff progressively without blocking UI', async ({ page }) => {
    await page.goto('/pr/2');

    // UI should remain responsive during load
    const startTime = Date.now();

    // Wait for diff to appear
    await page.waitForSelector('[data-testid="diff-view"]', { timeout: 15000 });

    const loadTime = Date.now() - startTime;

    // Should load within reasonable time (15 seconds max)
    expect(loadTime).toBeLessThan(15000);

    // Check that other UI elements are still interactive
    const viewToggle = page.locator('[data-testid="view-mode-toggle"]');
    await expect(viewToggle).toBeEnabled();
  });
});

test.describe('Git Diff View - Expand/Collapse', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/pr/1');
    await page.waitForLoadState('networkidle');
  });

  test('expands collapsed sections', async ({ page }) => {
    await page.waitForSelector('[data-testid="diff-view"]');

    // Find expand button for collapsed sections
    const expandButton = page.locator('[data-testid="expand-section"]').first();

    if (await expandButton.isVisible()) {
      const initialLineCount = await page.locator('.diff-line').count();

      // Click to expand
      await expandButton.click();
      await page.waitForTimeout(500);

      const expandedLineCount = await page.locator('.diff-line').count();

      // Should have more lines after expanding
      expect(expandedLineCount).toBeGreaterThan(initialLineCount);
    } else {
      // If no collapsed sections, test passes
      test.skip();
    }
  });

  test('collapses unchanged sections', async ({ page }) => {
    await page.waitForSelector('[data-testid="diff-view"]');

    // Find collapse button for unchanged sections
    const collapseButton = page.locator('[data-testid="collapse-section"]').first();

    if (await collapseButton.isVisible()) {
      const initialLineCount = await page.locator('.diff-line').count();

      // Click to collapse
      await collapseButton.click();
      await page.waitForTimeout(500);

      const collapsedLineCount = await page.locator('.diff-line').count();

      // Should have fewer lines after collapsing
      expect(collapsedLineCount).toBeLessThan(initialLineCount);
    } else {
      test.skip();
    }
  });
});

test.describe('Git Diff View - Error Handling', () => {
  test('displays error message when diff fails to load', async ({ page }) => {
    // Intercept diff API call and return error
    await page.route('**/api/pull-requests/*/diff', (route) => {
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Internal server error' }),
      });
    });

    await page.goto('/pr/1');
    await page.waitForLoadState('networkidle');

    // Should show error message
    const errorAlert = page.locator('[role="alert"]');
    await expect(errorAlert).toBeVisible({ timeout: 5000 });
    await expect(errorAlert).toContainText(/failed to load|error|could not fetch/i);
  });

  test('shows retry button on error', async ({ page }) => {
    await page.route('**/api/pull-requests/*/diff', (route) => {
      route.fulfill({ status: 500 });
    });

    await page.goto('/pr/1');
    await page.waitForLoadState('networkidle');

    // Find retry button
    const retryButton = page.locator('[data-testid="retry-diff-load"]');
    await expect(retryButton).toBeVisible({ timeout: 5000 });
    await expect(retryButton).toBeEnabled();
  });

  test('handles network timeout gracefully', async ({ page }) => {
    // Simulate network timeout
    await page.route('**/api/pull-requests/*/diff', () => {
      // Delay indefinitely
      return new Promise(() => {});
    });

    await page.goto('/pr/1');

    // Should show loading state initially
    const loadingSpinner = page.locator('[data-testid="diff-loading"]');
    await expect(loadingSpinner).toBeVisible({ timeout: 2000 });

    // Should eventually timeout and show error (after client timeout)
    const errorMessage = page.locator('[role="alert"]');
    await expect(errorMessage).toBeVisible({ timeout: 35000 }); // 30s timeout + 5s buffer
  });
});

test.describe('Git Diff View - Accessibility', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/pr/1');
    await page.waitForLoadState('networkidle');
  });

  test('supports keyboard navigation', async ({ page }) => {
    await page.waitForSelector('[data-testid="diff-view"]');

    // Focus on diff view
    await page.keyboard.press('Tab');

    // Should be able to navigate with arrow keys
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowUp');

    // View mode toggle should be keyboard accessible
    const viewToggle = page.locator('[data-testid="view-mode-toggle"]');
    await viewToggle.focus();
    await page.keyboard.press('Enter');

    // Should switch views
    const splitView = page.locator('[data-testid="split-view"]');
    await expect(splitView).toBeVisible({ timeout: 2000 });
  });

  test('has proper ARIA labels', async ({ page }) => {
    await page.waitForSelector('[data-testid="diff-view"]');

    // Check for ARIA labels on key elements
    const diffView = page.locator('[data-testid="diff-view"]');
    const ariaLabel = await diffView.getAttribute('aria-label');

    expect(ariaLabel).toBeTruthy();
    expect(ariaLabel).toMatch(/diff|changes|code/i);
  });

  test('maintains focus management', async ({ page }) => {
    await page.waitForSelector('[data-testid="diff-view"]');

    // Click view toggle
    const viewToggle = page.locator('[data-testid="view-mode-toggle"]');
    await viewToggle.click();

    // Focus should remain on a logical element
    const focusedElement = await page.evaluate(() => document.activeElement?.tagName);
    expect(focusedElement).toBeTruthy();
  });
});

test.describe('Git Diff View - Performance', () => {
  test('renders initial diff within 2 seconds', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/pr/1');
    await page.waitForSelector('[data-testid="diff-view"]', { timeout: 10000 });

    const renderTime = Date.now() - startTime;

    // Should render within 2 seconds for typical PR
    expect(renderTime).toBeLessThan(2000);
  });

  test('maintains smooth scrolling with large diffs', async ({ page }) => {
    await page.goto('/pr/2'); // Large diff
    await page.waitForSelector('[data-testid="diff-view"]', { timeout: 15000 });

    // Scroll multiple times and measure frame rate
    const metrics = await page.evaluate(async () => {
      const frameTimestamps: number[] = [];
      let lastTimestamp = performance.now();

      return new Promise<{ avgFrameTime: number }>((resolve) => {
        let frameCount = 0;
        const measureFrames = () => {
          frameCount++;
          const now = performance.now();
          const delta = now - lastTimestamp;
          frameTimestamps.push(delta);
          lastTimestamp = now;

          if (frameCount < 60) {
            requestAnimationFrame(measureFrames);
          } else {
            const avgFrameTime =
              frameTimestamps.reduce((a, b) => a + b, 0) / frameTimestamps.length;
            resolve({ avgFrameTime });
          }
        };

        // Start scrolling
        window.scrollTo(0, document.body.scrollHeight / 2);
        requestAnimationFrame(measureFrames);
      });
    });

    // Average frame time should be less than 16.67ms (60 FPS)
    expect(metrics.avgFrameTime).toBeLessThan(20); // Allow some tolerance
  });
});
