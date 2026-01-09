/**
 * Auth API fixtures for MSW handlers
 *
 * Provides typed mock data for authentication endpoints.
 * All data follows the actual API response structure.
 */

import type { AuthTokens, User, ApiResponse } from '@/types';

/** Default test user */
export const mockUser: User = {
  id: 'user-123',
  email: 'test@example.com',
  displayName: 'Test User',
  avatarUrl: 'https://avatars.example.com/user-123.png',
  createdAt: '2024-01-01T00:00:00Z',
};

/** Secondary test user for multi-user scenarios */
export const mockUser2: User = {
  id: 'user-456',
  email: 'other@example.com',
  displayName: 'Other User',
  avatarUrl: null,
  createdAt: '2024-02-15T00:00:00Z',
};

/** Default auth tokens */
export const mockAuthTokens: AuthTokens = {
  accessToken: 'mock-access-token-xyz123',
  refreshToken: 'mock-refresh-token-abc456',
  tokenType: 'Bearer',
  expiresIn: 900, // 15 minutes
};

/** Refreshed tokens (different values for testing token refresh) */
export const mockRefreshedTokens: AuthTokens = {
  accessToken: 'mock-refreshed-access-token-new789',
  refreshToken: 'mock-refreshed-refresh-token-new012',
  tokenType: 'Bearer',
  expiresIn: 900,
};

// ============================================================================
// Response Builders
// ============================================================================

/**
 * Build a successful API response
 */
export function successResponse<T>(data: T): ApiResponse<T> {
  return {
    success: true,
    data,
  };
}

/**
 * Build an error API response
 */
export function errorResponse(error: string): ApiResponse<never> {
  return {
    success: false,
    error,
  };
}

// ============================================================================
// Pre-built Responses
// ============================================================================

/** Successful login response */
export const loginSuccessResponse = successResponse(mockAuthTokens);

/** Successful registration response */
export const registerSuccessResponse = successResponse(mockAuthTokens);

/** Successful token refresh response */
export const refreshSuccessResponse = successResponse(mockRefreshedTokens);

/** Successful me endpoint response */
export const meSuccessResponse = successResponse(mockUser);

/** Successful profile update response */
export const updateProfileSuccessResponse = (updates: Partial<User> = {}): ApiResponse<User> =>
  successResponse({ ...mockUser, ...updates });

// ============================================================================
// Error Responses
// ============================================================================

/** Invalid credentials error */
export const invalidCredentialsError = errorResponse('Invalid email or password');

/** Email already exists error */
export const emailExistsError = errorResponse('Email already registered');

/** Expired token error */
export const expiredTokenError = errorResponse('Token has expired');

/** Unauthorized error */
export const unauthorizedError = errorResponse('Unauthorized');

/** User not found error */
export const userNotFoundError = errorResponse('User not found');
