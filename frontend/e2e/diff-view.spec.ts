/**
 * E2E tests for git diff integration
 *
 * Tests the complete diff viewing experience across different providers
 * using Playwright for browser automation.
 *
 * Run with: npx playwright test diff-view.spec.ts
 */

import { test, expect, Page } from '@playwright/test';

// Test configuration
const BASE_URL = 'http://localhost:5173';

// Helper to login
async function loginUser(page: Page) {
  await page.goto(`${BASE_URL}/login`);
  await page.fill('input[name="email"]', 'test@example.com');
  await page.fill('input[name="password"]', 'TestPassword123!');
  await page.click('button[type="submit"]');
  await page.waitForURL(`${BASE_URL}/dashboard`);
}

// Helper to navigate to a PR
async function navigateToPR(page: Page, prNumber: number) {
  await page.goto(`${BASE_URL}/pull-requests/${prNumber}`);
  await page.waitForLoadState('networkidle');
}

test.describe('Git Diff Integration E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Setup: Login before each test
    await loginUser(page);
  });

  test.describe('GitHub PR Diff Display', () => {
    test('displays GitHub PR diff correctly', async ({ page }) => {
      // Navigate to a GitHub PR
      await navigateToPR(page, 123);

      // Click on Files Changed tab
      await page.click('text=Files Changed');

      // Wait for diff to load
      await page.waitForSelector('[data-testid="diff-viewer"]', { timeout: 10000 });

      // Verify file list is displayed
      await expect(page.locator('[data-testid="file-list"]')).toBeVisible();

      // Verify at least one file is shown
      const fileCount = await page.locator('[data-testid="file-item"]').count();
      expect(fileCount).toBeGreaterThan(0);

      // Verify summary statistics
      await expect(page.locator('text=/files? changed/i')).toBeVisible();
      await expect(page.locator('text=/addition/i')).toBeVisible();
      await expect(page.locator('text=/deletion/i')).toBeVisible();
    });

    test('shows file additions and deletions', async ({ page }) => {
      await navigateToPR(page, 123);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Find a file with additions
      const addedLine = page.locator('text=/^\\+/').first();
      await expect(addedLine).toBeVisible();

      // Find a file with deletions
      const deletedLine = page.locator('text=/^-/').first();
      await expect(deletedLine).toBeVisible();
    });

    test('displays language-specific syntax highlighting', async ({ page }) => {
      await navigateToPR(page, 123);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Check for syntax-highlighted code (implementation-dependent)
      const codeBlock = page.locator('[data-testid="code-block"]').first();
      await expect(codeBlock).toBeVisible();

      // Verify language class is applied
      const languageClass = await codeBlock.getAttribute('class');
      expect(languageClass).toContain('language-');
    });

    test('allows expanding and collapsing files', async ({ page }) => {
      await navigateToPR(page, 123);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Click on a file to expand
      const fileHeader = page.locator('[data-testid="file-header"]').first();
      await fileHeader.click();

      // Verify diff content is visible
      await expect(page.locator('[data-testid="file-diff"]').first()).toBeVisible();

      // Click again to collapse
      await fileHeader.click();

      // Verify diff content is hidden
      await expect(page.locator('[data-testid="file-diff"]').first()).not.toBeVisible();
    });
  });

  test.describe('GitLab MR Diff with Renamed Files', () => {
    test('displays renamed file information', async ({ page }) => {
      // Navigate to a GitLab MR (mock or test instance)
      await navigateToPR(page, 456);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Look for renamed file indicator
      const renamedBadge = page.locator('text=/renamed/i').first();
      await expect(renamedBadge).toBeVisible();

      // Verify old and new filenames are shown
      await expect(page.locator('text=/old_name/i')).toBeVisible();
      await expect(page.locator('text=/new_name/i')).toBeVisible();
    });

    test('shows changes within renamed file', async ({ page }) => {
      await navigateToPR(page, 456);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Expand renamed file
      const renamedFile = page.locator('text=/renamed/i').first();
      await renamedFile.click();

      // Verify patch is shown
      await expect(page.locator('text=/@@ /').first()).toBeVisible();
    });
  });

  test.describe('Bitbucket PR Diff with Binary Files', () => {
    test('displays binary file indicator', async ({ page }) => {
      await navigateToPR(page, 789);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Look for binary file
      const binaryFile = page.locator('[data-testid="file-item"]:has-text(".png")').first();
      await expect(binaryFile).toBeVisible();

      // Verify binary indicator
      await expect(binaryFile.locator('text=/binary/i')).toBeVisible();
    });

    test('does not show patch for binary files', async ({ page }) => {
      await navigateToPR(page, 789);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Click on binary file
      const binaryFile = page.locator('[data-testid="file-item"]:has-text(".png")').first();
      await binaryFile.click();

      // Verify no patch content is shown
      await expect(page.locator('text=/@@ /').first()).not.toBeVisible({ timeout: 2000 });
    });
  });

  test.describe('Large Diff Performance', () => {
    test('handles 500+ files efficiently', async ({ page }) => {
      // Navigate to PR with many files
      await navigateToPR(page, 999);
      await page.click('text=Files Changed');

      // Start performance measurement
      const startTime = Date.now();

      // Wait for diff to load
      await page.waitForSelector('[data-testid="diff-viewer"]', { timeout: 15000 });

      const loadTime = Date.now() - startTime;

      // Verify loaded within reasonable time (< 5 seconds)
      expect(loadTime).toBeLessThan(5000);

      // Verify file count
      const fileCountText = await page.locator('text=/files? changed/i').textContent();
      expect(fileCountText).toContain('500');

      // Verify virtualization is working (not all files rendered at once)
      const renderedFiles = await page.locator('[data-testid="file-item"]').count();
      expect(renderedFiles).toBeLessThan(100); // Should use virtual scrolling
    });

    test('allows smooth scrolling through large diff', async ({ page }) => {
      await navigateToPR(page, 999);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Scroll down
      await page.evaluate(() => window.scrollBy(0, 1000));
      await page.waitForTimeout(500);

      // Verify more files are loaded
      const filesAfterScroll = await page.locator('[data-testid="file-item"]').count();
      expect(filesAfterScroll).toBeGreaterThan(0);

      // Scroll back up
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.waitForTimeout(500);

      // Should still be responsive
      await expect(page.locator('[data-testid="diff-viewer"]')).toBeVisible();
    });

    test('filters large diff efficiently', async ({ page }) => {
      await navigateToPR(page, 999);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Use filter input
      const filterInput = page.locator('input[placeholder*="filter"]');
      await filterInput.fill('.rs');

      // Wait for filter to apply
      await page.waitForTimeout(500);

      // Verify only Rust files are shown
      const visibleFiles = await page.locator('[data-testid="file-item"]').all();
      for (const file of visibleFiles) {
        const text = await file.textContent();
        expect(text).toContain('.rs');
      }
    });
  });

  test.describe('Offline Graceful Degradation', () => {
    test('shows error message when offline', async ({ page, context }) => {
      await navigateToPR(page, 123);

      // Simulate offline
      await context.setOffline(true);

      // Try to load diff
      await page.click('text=Files Changed');

      // Should show error state
      await expect(page.locator('text=/failed to load|offline|connection error/i')).toBeVisible({
        timeout: 10000,
      });
    });

    test('retries when connection restored', async ({ page, context }) => {
      await navigateToPR(page, 123);

      // Go offline
      await context.setOffline(true);
      await page.click('text=Files Changed');

      // Wait for error
      await page.waitForSelector('text=/error/i');

      // Restore connection
      await context.setOffline(false);

      // Click retry button
      const retryButton = page.locator('button:has-text("Retry")');
      if (await retryButton.isVisible()) {
        await retryButton.click();
      }

      // Should load successfully
      await expect(page.locator('[data-testid="diff-viewer"]')).toBeVisible({ timeout: 10000 });
    });

    test('shows cached diff when offline after initial load', async ({ page, context }) => {
      await navigateToPR(page, 123);
      await page.click('text=Files Changed');

      // Wait for diff to load and cache
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Navigate away
      await page.click('text=Conversation');

      // Go offline
      await context.setOffline(true);

      // Navigate back to Files Changed
      await page.click('text=Files Changed');

      // Should still show cached diff
      await expect(page.locator('[data-testid="diff-viewer"]')).toBeVisible();
    });
  });

  test.describe('Accessibility', () => {
    test('diff viewer is keyboard navigable', async ({ page }) => {
      await navigateToPR(page, 123);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Tab through files
      await page.keyboard.press('Tab');
      await page.keyboard.press('Tab');

      // Press Enter to expand file
      await page.keyboard.press('Enter');

      // Verify file expanded
      await expect(page.locator('[data-testid="file-diff"]').first()).toBeVisible();
    });

    test('screen reader announcements work', async ({ page }) => {
      await navigateToPR(page, 123);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Check for aria-live regions
      const liveRegion = page.locator('[aria-live="polite"]');
      await expect(liveRegion).toBeAttached();

      // Verify summary has proper labels
      const summary = page.locator('[role="status"]');
      if ((await summary.count()) > 0) {
        await expect(summary.first()).toBeVisible();
      }
    });
  });

  test.describe('Mobile Responsiveness', () => {
    test.use({ viewport: { width: 375, height: 667 } }); // iPhone SE size

    test('diff displays correctly on mobile', async ({ page }) => {
      await navigateToPR(page, 123);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Verify mobile layout
      const diffViewer = page.locator('[data-testid="diff-viewer"]');
      const box = await diffViewer.boundingBox();

      expect(box?.width).toBeLessThan(400); // Should fit mobile screen
    });

    test('file list is scrollable on mobile', async ({ page }) => {
      await navigateToPR(page, 123);
      await page.click('text=Files Changed');
      await page.waitForSelector('[data-testid="diff-viewer"]');

      // Swipe down
      await page.touchscreen.tap(100, 300);
      await page.evaluate(() => window.scrollBy(0, 500));

      // Should remain functional
      await expect(page.locator('[data-testid="diff-viewer"]')).toBeVisible();
    });
  });
});
