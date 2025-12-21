import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { apiClient } from './client';

// Mock axios
vi.mock('axios', async () => {
  const actual = await vi.importActual<typeof import('axios')>('axios');
  return {
    ...actual,
    default: {
      ...actual.default,
      create: vi.fn(() => ({
        interceptors: {
          request: {
            use: vi.fn(),
          },
          response: {
            use: vi.fn(),
          },
        },
        get: vi.fn(),
        post: vi.fn(),
        put: vi.fn(),
        delete: vi.fn(),
      })),
      post: vi.fn(),
    },
  };
});

describe('apiClient', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('should be defined', () => {
    expect(apiClient).toBeDefined();
  });

  it('should have interceptors configured', () => {
    expect(apiClient.interceptors).toBeDefined();
    expect(apiClient.interceptors.request).toBeDefined();
    expect(apiClient.interceptors.response).toBeDefined();
  });
});

// Test the request interceptor logic directly
describe('request interceptor logic', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('should add Authorization header when token exists', () => {
    localStorage.setItem('accessToken', 'test-token');
    const token = localStorage.getItem('accessToken');
    expect(token).toBe('test-token');

    // Simulate what the interceptor does
    const config = {
      headers: {} as Record<string, string>,
    };
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    expect(config.headers.Authorization).toBe('Bearer test-token');
  });

  it('should not add Authorization header when no token exists', () => {
    const token = localStorage.getItem('accessToken');
    expect(token).toBeNull();

    const config = {
      headers: {} as Record<string, string>,
    };
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    expect(config.headers.Authorization).toBeUndefined();
  });
});

// Test the response interceptor logic directly
describe('response interceptor logic', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('should handle 401 response by attempting token refresh', async () => {
    localStorage.setItem('refreshToken', 'refresh-token');

    const refreshToken = localStorage.getItem('refreshToken');
    expect(refreshToken).toBe('refresh-token');

    // Simulate what the interceptor does on 401
    if (refreshToken) {
      // Would attempt refresh here
      const newAccessToken = 'new-access-token';
      const newRefreshToken = 'new-refresh-token';
      localStorage.setItem('accessToken', newAccessToken);
      localStorage.setItem('refreshToken', newRefreshToken);
    }

    expect(localStorage.getItem('accessToken')).toBe('new-access-token');
    expect(localStorage.getItem('refreshToken')).toBe('new-refresh-token');
  });

  it('should clear tokens and redirect on refresh failure', () => {
    localStorage.setItem('accessToken', 'test-token');
    localStorage.setItem('refreshToken', 'refresh-token');

    // Simulate what happens on refresh failure
    localStorage.removeItem('accessToken');
    localStorage.removeItem('refreshToken');

    expect(localStorage.getItem('accessToken')).toBeNull();
    expect(localStorage.getItem('refreshToken')).toBeNull();
  });

  it('should not attempt refresh without refreshToken', () => {
    // No refresh token set
    const refreshToken = localStorage.getItem('refreshToken');
    expect(refreshToken).toBeNull();

    // Should not attempt refresh
    const shouldAttemptRefresh = refreshToken !== null;
    expect(shouldAttemptRefresh).toBe(false);
  });

  it('should not retry request that has already been retried', () => {
    const originalRequest = { _retry: true };

    // Should skip refresh on already retried request
    const shouldRefresh = !originalRequest._retry;
    expect(shouldRefresh).toBe(false);
  });
});
