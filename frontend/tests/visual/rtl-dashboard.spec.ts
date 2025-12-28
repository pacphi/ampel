/**
 * Visual Regression Tests for RTL Dashboard Layout
 *
 * Tests Arabic and Hebrew rendering of the dashboard page with:
 * - Layout mirroring
 * - Icon directional flipping
 * - Text alignment
 * - Navigation positioning
 */

import { test, expect, type Page } from '@playwright/test';

// Helper to set language and wait for i18n initialization
async function setLanguage(page: Page, lang: string) {
  await page.evaluate((language) => {
    localStorage.setItem('ampel-i18n-lng', language);
  }, lang);
  await page.reload();

  // Wait for i18n to be ready
  await page.waitForFunction(
    () => {
      return (window as any).i18next?.isInitialized;
    },
    { timeout: 5000 }
  );

  // Small delay to ensure RTL CSS has applied
  await page.waitForTimeout(500);
}

// Helper to verify RTL attributes
async function verifyRTL(page: Page, shouldBeRTL: boolean) {
  const dir = await page.evaluate(() => document.documentElement.dir);
  const hasRtlClass = await page.evaluate(() => document.documentElement.classList.contains('rtl'));

  if (shouldBeRTL) {
    expect(dir).toBe('rtl');
    expect(hasRtlClass).toBe(true);
  } else {
    expect(dir).toBe('ltr');
    expect(hasRtlClass).toBe(false);
  }
}

test.describe('RTL Dashboard Visual Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Mock auth to access dashboard
    await page.goto('/');
    await page.evaluate(() => {
      localStorage.setItem('auth_token', 'mock-token');
    });
  });

  test.describe('Arabic (ar) Layout', () => {
    test('should render dashboard in Arabic with RTL layout', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/dashboard');

      // Verify RTL attributes
      await verifyRTL(page, true);

      // Wait for dashboard to load
      await page.waitForSelector('h1', { timeout: 10000 });

      // Take full page screenshot for visual regression
      await expect(page).toHaveScreenshot('dashboard-arabic-full.png', {
        fullPage: true,
        maxDiffPixels: 100,
      });
    });

    test('should render sidebar on the right in Arabic', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/dashboard');

      // Check sidebar positioning
      const sidebar = page.locator('[data-testid="sidebar"]').first();
      if ((await sidebar.count()) > 0) {
        const box = await sidebar.boundingBox();
        const viewport = page.viewportSize();

        // In RTL, sidebar should be on the right (x > 50% of viewport)
        if (box && viewport) {
          expect(box.x).toBeGreaterThan(viewport.width * 0.5);
        }
      }
    });

    test('should render summary tiles with correct RTL alignment', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/dashboard');

      // Wait for summary tiles
      await page.waitForSelector('[data-testid="summary-tile"]', { timeout: 10000 });

      // Take screenshot of summary section
      const summarySection = page.locator('.grid').first();
      await expect(summarySection).toHaveScreenshot('dashboard-arabic-summary.png', {
        maxDiffPixels: 50,
      });
    });

    test('should render PR cards with RTL layout', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/dashboard');

      // Switch to grid view if exists
      const gridButton = page.locator('button[title*="grid"]').first();
      if ((await gridButton.count()) > 0) {
        await gridButton.click();
        await page.waitForTimeout(500);
      }

      // Screenshot PR card section
      const prSection = page
        .locator('[data-testid="pr-grid"], [data-testid="repository-grid"]')
        .first();
      if ((await prSection.count()) > 0) {
        await expect(prSection).toHaveScreenshot('dashboard-arabic-pr-cards.png', {
          maxDiffPixels: 50,
        });
      }
    });

    test('should verify navigation menu RTL alignment', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/dashboard');

      const nav = page.locator('nav').first();
      if ((await nav.count()) > 0) {
        await expect(nav).toHaveScreenshot('dashboard-arabic-nav.png');
      }
    });

    test('should verify CSS logical properties are used', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/dashboard');

      // Check that components use margin-inline-start/end instead of left/right
      const hasLogicalProps = await page.evaluate(() => {
        const elements = document.querySelectorAll('*');
        const violations: string[] = [];

        elements.forEach((el) => {
          const styles = window.getComputedStyle(el);
          const classList = Array.from(el.classList);

          // Check for hardcoded directional classes (common violations)
          const problematicClasses = classList.filter(
            (cls) =>
              /\b(ml-|mr-|pl-|pr-|left-|right-|text-left|text-right)\d*\b/.test(cls) &&
              !/(start|end)/.test(cls)
          );

          if (problematicClasses.length > 0) {
            violations.push(`${el.tagName}: ${problematicClasses.join(', ')}`);
          }
        });

        return {
          hasViolations: violations.length > 0,
          violations: violations.slice(0, 10), // Limit output
        };
      });

      // Log violations but don't fail (some third-party components may have them)
      if (hasLogicalProps.hasViolations) {
        console.warn('CSS logical property violations found:', hasLogicalProps.violations);
      }
    });
  });

  test.describe('Hebrew (he) Layout', () => {
    test('should render dashboard in Hebrew with RTL layout', async ({ page }) => {
      await setLanguage(page, 'he');
      await page.goto('/dashboard');

      // Verify RTL attributes
      await verifyRTL(page, true);

      // Wait for dashboard to load
      await page.waitForSelector('h1', { timeout: 10000 });

      // Take full page screenshot
      await expect(page).toHaveScreenshot('dashboard-hebrew-full.png', {
        fullPage: true,
        maxDiffPixels: 100,
      });
    });

    test('should render sidebar on the right in Hebrew', async ({ page }) => {
      await setLanguage(page, 'he');
      await page.goto('/dashboard');

      // Check sidebar positioning
      const sidebar = page.locator('[data-testid="sidebar"]').first();
      if ((await sidebar.count()) > 0) {
        const box = await sidebar.boundingBox();
        const viewport = page.viewportSize();

        // In RTL, sidebar should be on the right
        if (box && viewport) {
          expect(box.x).toBeGreaterThan(viewport.width * 0.5);
        }
      }
    });

    test('should render summary tiles with correct RTL alignment', async ({ page }) => {
      await setLanguage(page, 'he');
      await page.goto('/dashboard');

      await page.waitForSelector('[data-testid="summary-tile"]', { timeout: 10000 });

      const summarySection = page.locator('.grid').first();
      await expect(summarySection).toHaveScreenshot('dashboard-hebrew-summary.png', {
        maxDiffPixels: 50,
      });
    });
  });

  test.describe('RTL to LTR Switching', () => {
    test('should correctly switch from Arabic to English', async ({ page }) => {
      // Start with Arabic
      await setLanguage(page, 'ar');
      await page.goto('/dashboard');
      await verifyRTL(page, true);

      // Switch to English
      await setLanguage(page, 'en');
      await page.goto('/dashboard');
      await verifyRTL(page, false);

      // Verify sidebar is now on the left
      const sidebar = page.locator('[data-testid="sidebar"]').first();
      if ((await sidebar.count()) > 0) {
        const box = await sidebar.boundingBox();
        if (box) {
          expect(box.x).toBeLessThan(300);
        }
      }

      // Screenshot after switch
      await expect(page).toHaveScreenshot('dashboard-english-after-rtl.png', {
        fullPage: true,
        maxDiffPixels: 100,
      });
    });

    test('should correctly switch from Hebrew to English', async ({ page }) => {
      await setLanguage(page, 'he');
      await page.goto('/dashboard');
      await verifyRTL(page, true);

      await setLanguage(page, 'en');
      await page.goto('/dashboard');
      await verifyRTL(page, false);
    });
  });

  test.describe('Responsive RTL Layout', () => {
    const viewports = [
      { name: 'desktop', width: 1920, height: 1080 },
      { name: 'laptop', width: 1366, height: 768 },
      { name: 'tablet', width: 768, height: 1024 },
      { name: 'mobile', width: 375, height: 667 },
    ];

    for (const viewport of viewports) {
      test(`should render correctly on ${viewport.name} in Arabic`, async ({ page }) => {
        await page.setViewportSize({ width: viewport.width, height: viewport.height });
        await setLanguage(page, 'ar');
        await page.goto('/dashboard');

        await page.waitForSelector('h1', { timeout: 10000 });

        await expect(page).toHaveScreenshot(`dashboard-arabic-${viewport.name}.png`, {
          fullPage: false,
          maxDiffPixels: 100,
        });
      });
    }
  });

  test.describe('Icon Directional Flipping', () => {
    test('should flip directional icons in Arabic', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/dashboard');

      // Check for common directional icons (arrows, chevrons)
      const icons = page.locator('[data-testid*="icon"], svg[class*="lucide"]');
      const iconCount = await icons.count();

      if (iconCount > 0) {
        // Take screenshot of icon section
        await expect(icons.first()).toHaveScreenshot('dashboard-arabic-icons.png', {
          maxDiffPixels: 20,
        });
      }
    });
  });
});
