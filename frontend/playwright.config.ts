import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for RTL visual regression testing
 */
export default defineConfig({
  testDir: './tests/visual',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['json', { outputFile: 'playwright-report/results.json' }],
    ['list'],
  ],

  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium-ltr',
      use: {
        ...devices['Desktop Chrome'],
        locale: 'en-US',
      },
    },
    {
      name: 'chromium-rtl-arabic',
      use: {
        ...devices['Desktop Chrome'],
        locale: 'ar-SA',
      },
    },
    {
      name: 'chromium-rtl-hebrew',
      use: {
        ...devices['Desktop Chrome'],
        locale: 'he-IL',
      },
    },
    {
      name: 'firefox-rtl-arabic',
      use: {
        ...devices['Desktop Firefox'],
        locale: 'ar-SA',
      },
    },
    {
      name: 'webkit-rtl-arabic',
      use: {
        ...devices['Desktop Safari'],
        locale: 'ar-SA',
      },
    },
  ],

  webServer: {
    command: 'pnpm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
});
