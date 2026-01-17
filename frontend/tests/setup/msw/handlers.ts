/**
 * MSW Request Handlers
 *
 * Defines mock API handlers for all Ampel API endpoints.
 * These handlers intercept HTTP requests and return mock responses.
 *
 * @example
 * ```typescript
 * // Use default handlers
 * import { handlers } from '@/tests/setup/msw/handlers';
 *
 * // Override specific handlers in tests
 * server.use(
 *   http.get('/api/auth/me', () => {
 *     return HttpResponse.json(customUser);
 *   })
 * );
 * ```
 */

import { http, HttpResponse, delay } from 'msw';
import type { GitProvider } from '@/types';
import type { MergeRequest } from '@/api/pullRequests';
import type { AddAccountRequest, UpdateAccountRequest } from '@/types/account';
import type {
  UpdateUserSettingsRequest,
  UpdateNotificationPreferencesRequest,
} from '@/api/settings';

// Import fixtures
import {
  mockUser,
  mockAuthTokens,
  mockRefreshedTokens,
  successResponse,
  errorResponse,
} from './fixtures/auth';
import { mockDashboardSummary, mockRepositories } from './fixtures/dashboard';
import {
  mockPullRequests,
  mockMergeSuccess,
  createPaginatedResponse,
} from './fixtures/pull-requests';
import {
  mockRepositoriesWithStatus,
  getDiscoveredByProvider,
  createRepositoryFromAdd,
} from './fixtures/repositories';
import {
  mockUserSettings,
  mockNotificationPreferences,
  applyUserSettingsUpdate,
  applyNotificationUpdate,
} from './fixtures/settings';
import { mockAccounts, createAccountFromAdd, mockValidationSuccess } from './fixtures/accounts';

// Base API URL (should match the client configuration)
const API_BASE = '/api';

// ============================================================================
// Auth Handlers
// ============================================================================

export const authHandlers = [
  // POST /auth/login
  http.post(`${API_BASE}/auth/login`, async ({ request }) => {
    await delay(100);
    const body = (await request.json()) as { email: string; password: string };

    // Simulate invalid credentials
    if (body.email === 'invalid@example.com' || body.password === 'wrong') {
      return HttpResponse.json(errorResponse('Invalid email or password'), { status: 401 });
    }

    return HttpResponse.json(successResponse(mockAuthTokens));
  }),

  // POST /auth/register
  http.post(`${API_BASE}/auth/register`, async ({ request }) => {
    await delay(100);
    const body = (await request.json()) as {
      email: string;
      password: string;
      displayName?: string;
    };

    // Simulate email already exists
    if (body.email === 'exists@example.com') {
      return HttpResponse.json(errorResponse('Email already registered'), { status: 409 });
    }

    return HttpResponse.json(successResponse(mockAuthTokens));
  }),

  // POST /auth/refresh
  http.post(`${API_BASE}/auth/refresh`, async ({ request }) => {
    await delay(50);
    const body = (await request.json()) as { refreshToken: string };

    // Simulate expired token
    if (body.refreshToken === 'expired') {
      return HttpResponse.json(errorResponse('Token has expired'), { status: 401 });
    }

    return HttpResponse.json(successResponse(mockRefreshedTokens));
  }),

  // GET /auth/me
  http.get(`${API_BASE}/auth/me`, async ({ request }) => {
    await delay(50);
    const authHeader = request.headers.get('Authorization');

    // Check for authorization
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return HttpResponse.json(errorResponse('Unauthorized'), { status: 401 });
    }

    return HttpResponse.json(successResponse(mockUser));
  }),

  // POST /auth/logout
  http.post(`${API_BASE}/auth/logout`, async () => {
    await delay(50);
    return HttpResponse.json(successResponse(null));
  }),

  // PUT /auth/me (update profile)
  http.put(`${API_BASE}/auth/me`, async ({ request }) => {
    await delay(100);
    const body = (await request.json()) as { email?: string; displayName?: string };

    const updatedUser = {
      ...mockUser,
      ...body,
    };

    return HttpResponse.json(successResponse(updatedUser));
  }),
];

// ============================================================================
// Dashboard Handlers
// ============================================================================

export const dashboardHandlers = [
  // GET /dashboard/summary
  http.get(`${API_BASE}/dashboard/summary`, async () => {
    await delay(100);
    return HttpResponse.json(successResponse(mockDashboardSummary));
  }),

  // GET /dashboard/grid
  http.get(`${API_BASE}/dashboard/grid`, async () => {
    await delay(100);
    return HttpResponse.json(successResponse(mockRepositories));
  }),
];

// ============================================================================
// Pull Requests Handlers
// ============================================================================

export const pullRequestsHandlers = [
  // GET /pull-requests
  http.get(`${API_BASE}/pull-requests`, async ({ request }) => {
    await delay(100);
    const url = new URL(request.url);
    const page = parseInt(url.searchParams.get('page') || '1', 10);
    const perPage = parseInt(url.searchParams.get('perPage') || '20', 10);

    const paginatedResponse = createPaginatedResponse(mockPullRequests, page, perPage);
    return HttpResponse.json(successResponse(paginatedResponse));
  }),

  // GET /repositories/:repoId/pull-requests
  http.get(`${API_BASE}/repositories/:repoId/pull-requests`, async ({ params }) => {
    await delay(100);
    const { repoId } = params;

    // Filter PRs by repository (in real impl)
    const filteredPRs = mockPullRequests.filter(
      (pr) => pr.repositoryId === repoId || true // Return all for simplicity
    );

    return HttpResponse.json(successResponse(filteredPRs));
  }),

  // GET /repositories/:repoId/pull-requests/:prId
  http.get(`${API_BASE}/repositories/:repoId/pull-requests/:prId`, async ({ params }) => {
    await delay(100);
    const { prId } = params;

    const pr = mockPullRequests.find((p) => p.id === prId);
    if (!pr) {
      return HttpResponse.json(errorResponse('Pull request not found'), { status: 404 });
    }

    return HttpResponse.json(successResponse(pr));
  }),

  // POST /repositories/:repoId/pull-requests/:prId/merge
  http.post(
    `${API_BASE}/repositories/:repoId/pull-requests/:prId/merge`,
    async ({ params, request }) => {
      await delay(200);
      const { prId } = params;
      const _body = (await request.json()) as MergeRequest;

      const pr = mockPullRequests.find((p) => p.id === prId);
      if (!pr) {
        return HttpResponse.json(errorResponse('Pull request not found'), { status: 404 });
      }

      // Simulate merge failure for red PRs
      if (pr.status === 'red') {
        return HttpResponse.json(
          successResponse({
            merged: false,
            message: 'Pull request cannot be merged due to failing checks',
          })
        );
      }

      return HttpResponse.json(successResponse(mockMergeSuccess));
    }
  ),

  // POST /repositories/:repoId/pull-requests/:prId/refresh
  http.post(`${API_BASE}/repositories/:repoId/pull-requests/:prId/refresh`, async ({ params }) => {
    await delay(100);
    const { prId } = params;

    const pr = mockPullRequests.find((p) => p.id === prId);
    if (!pr) {
      return HttpResponse.json(errorResponse('Pull request not found'), { status: 404 });
    }

    return HttpResponse.json(
      successResponse({
        ...pr,
        updatedAt: new Date().toISOString(),
      })
    );
  }),
];

// ============================================================================
// Repositories Handlers
// ============================================================================

export const repositoriesHandlers = [
  // GET /repositories
  http.get(`${API_BASE}/repositories`, async () => {
    await delay(100);
    return HttpResponse.json(successResponse(mockRepositoriesWithStatus));
  }),

  // GET /repositories/:id
  http.get(`${API_BASE}/repositories/:id`, async ({ params }) => {
    await delay(100);
    const { id } = params;

    const repo = mockRepositoriesWithStatus.find((r) => r.id === id);
    if (!repo) {
      return HttpResponse.json(errorResponse('Repository not found'), { status: 404 });
    }

    return HttpResponse.json(successResponse(repo));
  }),

  // GET /repositories/discover
  http.get(`${API_BASE}/repositories/discover`, async ({ request }) => {
    await delay(200);
    const url = new URL(request.url);
    const provider = url.searchParams.get('provider') as GitProvider;

    if (!provider) {
      return HttpResponse.json(errorResponse('Provider is required'), { status: 400 });
    }

    const discovered = getDiscoveredByProvider(provider);
    return HttpResponse.json(successResponse(discovered));
  }),

  // POST /repositories
  http.post(`${API_BASE}/repositories`, async ({ request }) => {
    await delay(150);
    const body = (await request.json()) as {
      provider: GitProvider;
      owner: string;
      name: string;
      pollIntervalSeconds?: number;
    };

    // Check if already exists
    const existing = mockRepositoriesWithStatus.find(
      (r) => r.provider === body.provider && r.owner === body.owner && r.name === body.name
    );
    if (existing) {
      return HttpResponse.json(errorResponse('Repository already tracked'), { status: 409 });
    }

    const newRepo = createRepositoryFromAdd(
      body.provider,
      body.owner,
      body.name,
      body.pollIntervalSeconds
    );

    return HttpResponse.json(successResponse(newRepo));
  }),

  // PUT /repositories/:id
  http.put(`${API_BASE}/repositories/:id`, async ({ params, request }) => {
    await delay(100);
    const { id } = params;
    const body = (await request.json()) as { pollIntervalSeconds?: number };

    const repo = mockRepositoriesWithStatus.find((r) => r.id === id);
    if (!repo) {
      return HttpResponse.json(errorResponse('Repository not found'), { status: 404 });
    }

    const updatedRepo = {
      ...repo,
      ...body,
      updatedAt: new Date().toISOString(),
    };

    return HttpResponse.json(successResponse(updatedRepo));
  }),

  // DELETE /repositories/:id
  http.delete(`${API_BASE}/repositories/:id`, async ({ params }) => {
    await delay(100);
    const { id } = params;

    const repo = mockRepositoriesWithStatus.find((r) => r.id === id);
    if (!repo) {
      return HttpResponse.json(errorResponse('Repository not found'), { status: 404 });
    }

    return new HttpResponse(null, { status: 204 });
  }),
];

// ============================================================================
// Settings Handlers
// ============================================================================

// Mutable state for settings (allows tests to verify updates)
let currentSettings = { ...mockUserSettings };
let currentNotifications = { ...mockNotificationPreferences };

export const settingsHandlers = [
  // GET /settings/behavior
  http.get(`${API_BASE}/settings/behavior`, async () => {
    await delay(100);
    return HttpResponse.json(successResponse(currentSettings));
  }),

  // PUT /settings/behavior
  http.put(`${API_BASE}/settings/behavior`, async ({ request }) => {
    await delay(100);
    const body = (await request.json()) as UpdateUserSettingsRequest;

    currentSettings = applyUserSettingsUpdate(currentSettings, body);
    return HttpResponse.json(successResponse(currentSettings));
  }),

  // GET /notifications/preferences
  http.get(`${API_BASE}/notifications/preferences`, async () => {
    await delay(100);
    return HttpResponse.json(successResponse(currentNotifications));
  }),

  // PUT /notifications/preferences
  http.put(`${API_BASE}/notifications/preferences`, async ({ request }) => {
    await delay(100);
    const body = (await request.json()) as UpdateNotificationPreferencesRequest;

    currentNotifications = applyNotificationUpdate(currentNotifications, body);
    return HttpResponse.json(successResponse(currentNotifications));
  }),

  // POST /notifications/test-slack
  http.post(`${API_BASE}/notifications/test-slack`, async () => {
    await delay(300);

    // Simulate failure if Slack is not configured
    if (!currentNotifications.slackEnabled || !currentNotifications.slackWebhookUrl) {
      return HttpResponse.json(successResponse(false));
    }

    return HttpResponse.json(successResponse(true));
  }),

  // POST /notifications/test-email
  http.post(`${API_BASE}/notifications/test-email`, async () => {
    await delay(300);

    // Simulate failure if email is not configured
    if (!currentNotifications.emailEnabled || !currentNotifications.smtpHost) {
      return HttpResponse.json(successResponse(false));
    }

    return HttpResponse.json(successResponse(true));
  }),
];

// Reset settings state (call between tests)
export const resetSettingsState = () => {
  currentSettings = { ...mockUserSettings };
  currentNotifications = { ...mockNotificationPreferences };
};

// ============================================================================
// Accounts Handlers
// ============================================================================

export const accountsHandlers = [
  // GET /accounts
  http.get(`${API_BASE}/accounts`, async () => {
    await delay(100);
    return HttpResponse.json(successResponse(mockAccounts));
  }),

  // GET /accounts/:id
  http.get(`${API_BASE}/accounts/:id`, async ({ params }) => {
    await delay(100);
    const { id } = params;

    const account = mockAccounts.find((a) => a.id === id);
    if (!account) {
      return HttpResponse.json(errorResponse('Account not found'), { status: 404 });
    }

    return HttpResponse.json(successResponse(account));
  }),

  // POST /accounts
  http.post(`${API_BASE}/accounts`, async ({ request }) => {
    await delay(150);
    const body = (await request.json()) as AddAccountRequest;

    // Simulate invalid token
    if (body.accessToken === 'invalid') {
      return HttpResponse.json(errorResponse('Invalid or expired access token'), { status: 400 });
    }

    const newAccount = createAccountFromAdd(body);
    return HttpResponse.json(successResponse(newAccount));
  }),

  // PATCH /accounts/:id
  http.patch(`${API_BASE}/accounts/:id`, async ({ params, request }) => {
    await delay(100);
    const { id } = params;
    const body = (await request.json()) as UpdateAccountRequest;

    const account = mockAccounts.find((a) => a.id === id);
    if (!account) {
      return HttpResponse.json(errorResponse('Account not found'), { status: 404 });
    }

    const updatedAccount = {
      ...account,
      ...body,
    };

    return HttpResponse.json(successResponse(updatedAccount));
  }),

  // DELETE /accounts/:id
  http.delete(`${API_BASE}/accounts/:id`, async ({ params }) => {
    await delay(100);
    const { id } = params;

    const account = mockAccounts.find((a) => a.id === id);
    if (!account) {
      return HttpResponse.json(errorResponse('Account not found'), { status: 404 });
    }

    return new HttpResponse(null, { status: 204 });
  }),

  // POST /accounts/:id/validate
  http.post(`${API_BASE}/accounts/:id/validate`, async ({ params }) => {
    await delay(200);
    const { id } = params;

    const account = mockAccounts.find((a) => a.id === id);
    if (!account) {
      return HttpResponse.json(errorResponse('Account not found'), { status: 404 });
    }

    return HttpResponse.json(successResponse(mockValidationSuccess));
  }),

  // POST /accounts/:id/set-default
  http.post(`${API_BASE}/accounts/:id/set-default`, async ({ params }) => {
    await delay(100);
    const { id } = params;

    const account = mockAccounts.find((a) => a.id === id);
    if (!account) {
      return HttpResponse.json(errorResponse('Account not found'), { status: 404 });
    }

    return HttpResponse.json(
      successResponse({
        ...account,
        isDefault: true,
      })
    );
  }),
];

// ============================================================================
// All Handlers
// ============================================================================

/**
 * All default handlers combined.
 * Use these as the base handlers for MSW server.
 */
export const handlers = [
  ...authHandlers,
  ...dashboardHandlers,
  ...pullRequestsHandlers,
  ...repositoriesHandlers,
  ...settingsHandlers,
  ...accountsHandlers,
];

// ============================================================================
// Handler Utilities
// ============================================================================

/**
 * Create a handler that returns an error for any endpoint
 */
export const createErrorHandler = (
  method: 'get' | 'post' | 'put' | 'patch' | 'delete',
  path: string,
  error: string,
  status = 500
) => {
  const httpMethod = http[method];
  return httpMethod(`${API_BASE}${path}`, async () => {
    await delay(50);
    return HttpResponse.json(errorResponse(error), { status });
  });
};

/**
 * Create a handler that simulates network delay
 */
export const createSlowHandler = (
  method: 'get' | 'post' | 'put' | 'patch' | 'delete',
  path: string,
  response: unknown,
  delayMs = 2000
) => {
  const httpMethod = http[method];
  return httpMethod(`${API_BASE}${path}`, async () => {
    await delay(delayMs);
    return HttpResponse.json(successResponse(response));
  });
};

/**
 * Create a handler that simulates a network error
 */
export const createNetworkErrorHandler = (
  method: 'get' | 'post' | 'put' | 'patch' | 'delete',
  path: string
) => {
  const httpMethod = http[method];
  return httpMethod(`${API_BASE}${path}`, () => {
    return HttpResponse.error();
  });
};
