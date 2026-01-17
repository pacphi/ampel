/**
 * MSW Module Index
 *
 * Central export point for MSW testing utilities.
 *
 * @example
 * ```typescript
 * // Import server and utilities
 * import { server, http, HttpResponse } from '@/tests/setup/msw';
 *
 * // Import fixtures
 * import { mockUser, mockRepositories } from '@/tests/setup/msw/fixtures';
 *
 * // Override handlers in tests
 * server.use(
 *   http.get('/api/auth/me', () => {
 *     return HttpResponse.json({ success: true, data: customUser });
 *   })
 * );
 * ```
 */

// Server exports
export {
  server,
  startServer,
  stopServer,
  resetHandlers,
  serverLifecycle,
  http,
  HttpResponse,
  delay,
} from './server';

// Handler exports
export {
  handlers,
  authHandlers,
  dashboardHandlers,
  pullRequestsHandlers,
  repositoriesHandlers,
  settingsHandlers,
  accountsHandlers,
  resetSettingsState,
  createErrorHandler,
  createSlowHandler,
  createNetworkErrorHandler,
} from './handlers';

// Re-export fixtures for convenience
export * from './fixtures';
