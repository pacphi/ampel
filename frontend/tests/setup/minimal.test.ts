/**
 * Minimal test to verify basic functionality
 */
import { describe, it, expect } from 'vitest';
import { mockUser } from './msw/fixtures/auth';
import { server, http, HttpResponse } from './msw/server';
import { hasTranslation, getTranslation } from './i18n-test-utils';
// Test API imports
import { authApi } from '@/api/auth';
import { dashboardApi } from '@/api/dashboard';

describe('Minimal Test', () => {
  it('should pass', () => {
    expect(1 + 1).toBe(2);
  });

  it('should import fixtures', () => {
    expect(mockUser.id).toBe('user-123');
  });

  it('should have server', () => {
    expect(server).toBeDefined();
    expect(http).toBeDefined();
    expect(HttpResponse).toBeDefined();
  });

  it('should have i18n', () => {
    expect(hasTranslation).toBeDefined();
    expect(getTranslation).toBeDefined();
  });

  it('should have API clients', () => {
    expect(authApi).toBeDefined();
    expect(dashboardApi).toBeDefined();
  });
});
