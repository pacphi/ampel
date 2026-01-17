/**
 * MSW Server Setup for Node.js (Vitest)
 *
 * Configures Mock Service Worker for use in Node.js test environment.
 * This server intercepts HTTP requests during tests and returns mock responses.
 *
 * @example
 * ```typescript
 * // In test file
 * import { server } from '@/tests/setup/msw/server';
 * import { http, HttpResponse } from 'msw';
 *
 * // Override handler for specific test
 * server.use(
 *   http.get('/api/auth/me', () => {
 *     return HttpResponse.json({ success: false, error: 'Unauthorized' }, { status: 401 });
 *   })
 * );
 * ```
 */

import { setupServer } from 'msw/node';
import { handlers, resetSettingsState } from './handlers';
import { i18nHandlers } from './handlers-i18n';

/**
 * MSW Server instance for Node.js testing.
 *
 * - Starts before all tests
 * - Resets handlers after each test
 * - Closes after all tests
 */
export const server = setupServer(...handlers, ...i18nHandlers);

/**
 * Start the server with default handlers.
 * Call this in your test setup file.
 */
export const startServer = () => {
  server.listen({
    onUnhandledRequest: 'warn',
  });
};

/**
 * Reset handlers to defaults.
 * Call this after each test to ensure clean state.
 */
export const resetHandlers = () => {
  server.resetHandlers();
  resetSettingsState();
};

/**
 * Stop the server.
 * Call this after all tests complete.
 */
export const stopServer = () => {
  server.close();
};

/**
 * Server lifecycle hooks for vitest.
 *
 * Usage in vitest.setup.ts:
 * ```typescript
 * import { serverLifecycle } from '@/tests/setup/msw/server';
 *
 * beforeAll(() => serverLifecycle.beforeAll());
 * afterEach(() => serverLifecycle.afterEach());
 * afterAll(() => serverLifecycle.afterAll());
 * ```
 */
export const serverLifecycle = {
  /**
   * Call before all tests - starts the server
   */
  beforeAll: () => {
    startServer();
  },

  /**
   * Call after each test - resets handlers and state
   */
  afterEach: () => {
    resetHandlers();
  },

  /**
   * Call after all tests - stops the server
   */
  afterAll: () => {
    stopServer();
  },
};

// Export handlers for runtime override in tests
export { handlers } from './handlers';

// Re-export msw utilities for convenience
export { http, HttpResponse, delay } from 'msw';
