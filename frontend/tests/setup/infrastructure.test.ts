/**
 * Test Infrastructure Verification Tests
 *
 * These tests verify that the test infrastructure (MSW, fixtures, utilities)
 * is working correctly. Run these first when setting up a new environment.
 */

import { describe, it, expect } from 'vitest';

// Import MSW components
import { server, http, HttpResponse } from './msw/server';
import {
  mockUser,
  mockRepositories,
  mockDashboardSummary,
  successResponse,
  errorResponse,
} from './msw/fixtures';

// Import i18n utilities
import { hasTranslation, getTranslation, expectTranslationKey } from './i18n-test-utils';

// Import BDD helpers (use capitalized Then to avoid Promise.then conflict)
import { Feature, Scenario, Given, When, Then } from './bdd-helpers';

// Import API clients
import { authApi } from '@/api/auth';
import { dashboardApi } from '@/api/dashboard';

// ============================================================================
// MSW Infrastructure Tests
// ============================================================================

describe('MSW Infrastructure', () => {
  it('intercepts auth API calls and returns mock data', async () => {
    // Setup authenticated user (sets localStorage tokens)
    localStorage.setItem('accessToken', 'test-token');

    // Call the actual API client
    const user = await authApi.me();

    // Verify mock data is returned
    expect(user).toEqual(mockUser);
    expect(user.id).toBe('user-123');
    expect(user.email).toBe('test@example.com');
  });

  it('intercepts dashboard API calls and returns mock data', async () => {
    const summary = await dashboardApi.getSummary();

    expect(summary).toEqual(mockDashboardSummary);
    expect(summary.totalRepositories).toBe(5);
    expect(summary.statusCounts.green).toBe(2);
  });

  it('returns dashboard grid with repositories', async () => {
    const grid = await dashboardApi.getGrid();

    expect(grid).toEqual(mockRepositories);
    expect(grid).toHaveLength(5);
    expect(grid[0].status).toBe('green');
  });

  it('allows overriding handlers in individual tests', async () => {
    // Override the me endpoint for this test only
    const customUser = {
      ...mockUser,
      displayName: 'Custom User',
    };

    server.use(
      http.get('/api/auth/me', () => {
        return HttpResponse.json(successResponse(customUser));
      })
    );

    const user = await authApi.me();

    expect(user.displayName).toBe('Custom User');
  });

  it('resets handlers after each test', async () => {
    // Setup authenticated user (sets localStorage tokens)
    localStorage.setItem('accessToken', 'test-token');

    // This test should get the original mock data, not the overridden one
    const user = await authApi.me();

    expect(user.displayName).toBe('Test User');
  });

  it('can simulate API errors', async () => {
    server.use(
      http.get('/api/dashboard/summary', () => {
        return HttpResponse.json(errorResponse('Server error'), { status: 500 });
      })
    );

    await expect(dashboardApi.getSummary()).rejects.toThrow();
  });
});

// ============================================================================
// Fixtures Tests
// ============================================================================

describe('Fixtures', () => {
  it('provides typed mock user data', () => {
    expect(mockUser).toHaveProperty('id');
    expect(mockUser).toHaveProperty('email');
    expect(mockUser).toHaveProperty('displayName');
    expect(mockUser).toHaveProperty('createdAt');
  });

  it('provides mock repositories with different statuses', () => {
    const greenRepos = mockRepositories.filter((r) => r.status === 'green');
    const yellowRepos = mockRepositories.filter((r) => r.status === 'yellow');
    const redRepos = mockRepositories.filter((r) => r.status === 'red');

    expect(greenRepos.length).toBeGreaterThan(0);
    expect(yellowRepos.length).toBeGreaterThan(0);
    expect(redRepos.length).toBeGreaterThan(0);
  });

  it('provides mock dashboard summary with correct structure', () => {
    expect(mockDashboardSummary).toHaveProperty('totalRepositories');
    expect(mockDashboardSummary).toHaveProperty('totalOpenPrs');
    expect(mockDashboardSummary).toHaveProperty('statusCounts');
    expect(mockDashboardSummary).toHaveProperty('providerCounts');

    expect(mockDashboardSummary.statusCounts).toHaveProperty('green');
    expect(mockDashboardSummary.statusCounts).toHaveProperty('yellow');
    expect(mockDashboardSummary.statusCounts).toHaveProperty('red');
  });
});

// ============================================================================
// i18n Infrastructure Tests
// ============================================================================

describe('i18n Infrastructure', () => {
  it('loads English translations', () => {
    expect(hasTranslation('common:app.title')).toBe(true);
    expect(hasTranslation('dashboard:title')).toBe(true);
  });

  it('returns correct translation text', () => {
    const title = getTranslation('common:app.title');
    expect(title).toBe('Ampel PR Dashboard');
  });

  it('handles interpolation', () => {
    const text = getTranslation('common:time.minutesAgo', { count: 5 });
    expect(text).toContain('5');
  });

  it('returns marked string for missing translations', () => {
    const missing = getTranslation('nonexistent:key');
    expect(missing).toContain('MISSING');
  });

  it('expectTranslationKey helper works correctly', () => {
    const result = expectTranslationKey('common:app.title');
    expect(result.getTranslatedText()).toBe('Ampel PR Dashboard');
  });
});

// ============================================================================
// BDD Helpers Tests
// ============================================================================

describe('BDD Helpers', () => {
  Feature('BDD Test Structure', () => {
    Scenario('Given/When/Then steps execute in order', async () => {
      const executionOrder: string[] = [];

      await Given('setup step', () => {
        executionOrder.push('given');
      });

      await When('action step', () => {
        executionOrder.push('when');
      });

      await Then('assertion step', () => {
        executionOrder.push('then');
      });

      expect(executionOrder).toEqual(['given', 'when', 'then']);
    });
  });

  Feature('Async Step Support', () => {
    Scenario('Async steps are properly awaited', async () => {
      let value = 0;

      await Given('async setup', async () => {
        await new Promise((resolve) => setTimeout(resolve, 10));
        value = 1;
      });

      await When('async action', async () => {
        await new Promise((resolve) => setTimeout(resolve, 10));
        value = 2;
      });

      await Then('value is updated', () => {
        expect(value).toBe(2);
      });
    });
  });
});

// ============================================================================
// Response Helpers Tests
// ============================================================================

describe('Response Helpers', () => {
  it('successResponse creates correct structure', () => {
    const response = successResponse({ test: 'data' });

    expect(response).toEqual({
      success: true,
      data: { test: 'data' },
    });
  });

  it('errorResponse creates correct structure', () => {
    const response = errorResponse('Something went wrong');

    expect(response).toEqual({
      success: false,
      error: 'Something went wrong',
    });
  });
});
