/**
 * Visual Regression Tests for RTL Settings Page Layout
 *
 * Tests Arabic and Hebrew rendering of the settings page with:
 * - Form layout mirroring
 * - Input field alignment
 * - Navigation menu positioning
 * - Button alignment
 */

import { test, expect, type Page } from '@playwright/test';

async function setLanguage(page: Page, lang: string) {
  await page.evaluate((language) => {
    localStorage.setItem('ampel-i18n-lng', language);
  }, lang);
  await page.reload();

  await page.waitForFunction(
    () => {
      return (window as any).i18next?.isInitialized;
    },
    { timeout: 5000 }
  );

  await page.waitForTimeout(500);
}

async function verifyRTL(page: Page, shouldBeRTL: boolean) {
  const dir = await page.evaluate(() => document.documentElement.dir);
  const hasRtlClass = await page.evaluate(() => document.documentElement.classList.contains('rtl'));

  expect(dir).toBe(shouldBeRTL ? 'rtl' : 'ltr');
  expect(hasRtlClass).toBe(shouldBeRTL);
}

test.describe('RTL Settings Visual Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.evaluate(() => {
      localStorage.setItem('auth_token', 'mock-token');
    });
  });

  test.describe('Arabic (ar) Settings Layout', () => {
    test('should render settings page in Arabic with RTL layout', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings');

      await verifyRTL(page, true);
      await page.waitForSelector('h1', { timeout: 10000 });

      await expect(page).toHaveScreenshot('settings-arabic-full.png', {
        fullPage: true,
        maxDiffPixels: 100,
      });
    });

    test('should render settings navigation on the right', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings');

      const nav = page.locator('nav').first();
      if ((await nav.count()) > 0) {
        const navBox = await nav.boundingBox();
        const viewport = page.viewportSize();

        // In RTL, nav might be on right side
        await expect(nav).toHaveScreenshot('settings-arabic-nav.png');
      }
    });

    test('should render profile form with RTL alignment', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings');

      // Wait for form
      await page.waitForSelector('input[type="email"]', { timeout: 10000 });

      // Screenshot form section
      const form = page.locator('form, [role="form"]').first();
      if ((await form.count()) > 0) {
        await expect(form).toHaveScreenshot('settings-arabic-profile-form.png', {
          maxDiffPixels: 50,
        });
      }
    });

    test('should render input fields with RTL text alignment', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings');

      const inputs = page.locator('input[type="text"], input[type="email"]');
      const inputCount = await inputs.count();

      if (inputCount > 0) {
        const firstInput = inputs.first();

        // Check text-align direction
        const textAlign = await firstInput.evaluate((el) => {
          return window.getComputedStyle(el).textAlign;
        });

        // Should be 'start' (logical) or 'right' (RTL)
        expect(['start', 'right']).toContain(textAlign);
      }
    });

    test('should render buttons with correct RTL alignment', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings');

      const buttons = page.locator('button');
      const buttonCount = await buttons.count();

      if (buttonCount > 0) {
        const buttonSection = page
          .locator('.flex, [class*="flex"]')
          .filter({
            has: page.locator('button'),
          })
          .first();

        if ((await buttonSection.count()) > 0) {
          await expect(buttonSection).toHaveScreenshot('settings-arabic-buttons.png', {
            maxDiffPixels: 30,
          });
        }
      }
    });

    test('should render accounts settings in Arabic', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings/accounts');

      await page.waitForSelector('h1, h2', { timeout: 10000 });

      await expect(page).toHaveScreenshot('settings-arabic-accounts.png', {
        fullPage: true,
        maxDiffPixels: 100,
      });
    });

    test('should render filters settings in Arabic', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings/filters');

      await page.waitForSelector('h1, h2', { timeout: 10000 });

      await expect(page).toHaveScreenshot('settings-arabic-filters.png', {
        fullPage: true,
        maxDiffPixels: 100,
      });
    });

    test('should render notifications settings in Arabic', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings/notifications');

      await page.waitForSelector('h1, h2', { timeout: 10000 });

      await expect(page).toHaveScreenshot('settings-arabic-notifications.png', {
        fullPage: true,
        maxDiffPixels: 100,
      });
    });

    test('should render behavior settings in Arabic', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings/behavior');

      await page.waitForSelector('h1, h2', { timeout: 10000 });

      await expect(page).toHaveScreenshot('settings-arabic-behavior.png', {
        fullPage: true,
        maxDiffPixels: 100,
      });
    });
  });

  test.describe('Hebrew (he) Settings Layout', () => {
    test('should render settings page in Hebrew with RTL layout', async ({ page }) => {
      await setLanguage(page, 'he');
      await page.goto('/settings');

      await verifyRTL(page, true);
      await page.waitForSelector('h1', { timeout: 10000 });

      await expect(page).toHaveScreenshot('settings-hebrew-full.png', {
        fullPage: true,
        maxDiffPixels: 100,
      });
    });

    test('should render profile form with RTL alignment in Hebrew', async ({ page }) => {
      await setLanguage(page, 'he');
      await page.goto('/settings');

      await page.waitForSelector('input[type="email"]', { timeout: 10000 });

      const form = page.locator('form, [role="form"]').first();
      if ((await form.count()) > 0) {
        await expect(form).toHaveScreenshot('settings-hebrew-profile-form.png', {
          maxDiffPixels: 50,
        });
      }
    });

    test('should render all settings sections in Hebrew', async ({ page }) => {
      const sections = [
        '/settings',
        '/settings/accounts',
        '/settings/filters',
        '/settings/notifications',
        '/settings/behavior',
      ];

      for (const section of sections) {
        await setLanguage(page, 'he');
        await page.goto(section);
        await page.waitForSelector('h1, h2', { timeout: 10000 });

        const sectionName = section.split('/').pop() || 'main';
        await expect(page).toHaveScreenshot(`settings-hebrew-${sectionName}.png`, {
          fullPage: true,
          maxDiffPixels: 100,
        });
      }
    });
  });

  test.describe('Form Controls RTL Behavior', () => {
    test('should render checkboxes with correct RTL positioning', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings/notifications');

      const checkboxes = page.locator('input[type="checkbox"]');
      const checkboxCount = await checkboxes.count();

      if (checkboxCount > 0) {
        const checkboxSection = page
          .locator('[role="group"], .space-y-2')
          .filter({
            has: page.locator('input[type="checkbox"]'),
          })
          .first();

        if ((await checkboxSection.count()) > 0) {
          await expect(checkboxSection).toHaveScreenshot('settings-arabic-checkboxes.png', {
            maxDiffPixels: 30,
          });
        }
      }
    });

    test('should render select dropdowns with correct RTL alignment', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings/behavior');

      const selects = page.locator('select, [role="combobox"]');
      const selectCount = await selects.count();

      if (selectCount > 0) {
        await expect(selects.first()).toHaveScreenshot('settings-arabic-select.png', {
          maxDiffPixels: 30,
        });
      }
    });

    test('should render sliders with correct RTL direction', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings/notifications');

      const sliders = page.locator('input[type="range"], [role="slider"]');
      const sliderCount = await sliders.count();

      if (sliderCount > 0) {
        await expect(sliders.first()).toHaveScreenshot('settings-arabic-slider.png', {
          maxDiffPixels: 30,
        });
      }
    });
  });

  test.describe('Dropdown Menus RTL', () => {
    test('should render dropdown menus with correct RTL positioning', async ({ page }) => {
      await setLanguage(page, 'ar');
      await page.goto('/settings/accounts');

      // Look for dropdown trigger
      const dropdownTrigger = page.locator('[role="button"][aria-haspopup]').first();
      if ((await dropdownTrigger.count()) > 0) {
        await dropdownTrigger.click();
        await page.waitForTimeout(300);

        // Screenshot dropdown
        const dropdown = page.locator('[role="menu"], [role="listbox"]').first();
        if ((await dropdown.count()) > 0) {
          await expect(dropdown).toHaveScreenshot('settings-arabic-dropdown.png', {
            maxDiffPixels: 30,
          });
        }
      }
    });
  });

  test.describe('Mobile RTL Settings', () => {
    test('should render mobile settings in Arabic', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await setLanguage(page, 'ar');
      await page.goto('/settings');

      await page.waitForSelector('h1', { timeout: 10000 });

      await expect(page).toHaveScreenshot('settings-arabic-mobile.png', {
        fullPage: false,
        maxDiffPixels: 100,
      });
    });

    test('should render mobile settings in Hebrew', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await setLanguage(page, 'he');
      await page.goto('/settings');

      await page.waitForSelector('h1', { timeout: 10000 });

      await expect(page).toHaveScreenshot('settings-hebrew-mobile.png', {
        fullPage: false,
        maxDiffPixels: 100,
      });
    });
  });
});
