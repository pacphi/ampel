/**
 * Test Setup Module Index
 *
 * Central export point for all test utilities and setup files.
 *
 * @example
 * ```typescript
 * // Import everything you need from one place
 * import {
 *   render,
 *   screen,
 *   userEvent,
 *   server,
 *   http,
 *   HttpResponse,
 *   mockUser,
 *   mockRepositories,
 *   Feature,
 *   Scenario,
 *   given,
 *   when,
 *   then,
 *   expectTranslationKey,
 * } from '@/tests/setup';
 * ```
 */

// ============================================================================
// Test Utilities
// ============================================================================

// Custom render and testing library utilities
export {
  render,
  screen,
  within,
  waitFor,
  waitForElementToBeRemoved,
  fireEvent,
  userEvent,
  createTestQueryClient,
  waitForLoadingToFinish,
  waitForAndClick,
  fillField,
  submitForm,
  setupLocalStorage,
  clearLocalStorage,
  setupAuthenticatedUser,
  clearAuthenticatedUser,
  type CustomRenderOptions,
  type CustomRenderResult,
} from './test-utils';

// ============================================================================
// MSW (Mock Service Worker)
// ============================================================================

// Server and utilities
export {
  server,
  startServer,
  stopServer,
  resetHandlers,
  serverLifecycle,
  http,
  HttpResponse,
  delay,
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
} from './msw';

// Fixtures - Auth
export {
  mockUser,
  mockUser2,
  mockAuthTokens,
  mockRefreshedTokens,
  successResponse,
  errorResponse,
  loginSuccessResponse,
  registerSuccessResponse,
  meSuccessResponse,
  invalidCredentialsError,
  unauthorizedError,
} from './msw/fixtures';

// Fixtures - Dashboard
export {
  mockDashboardSummary,
  mockEmptyDashboardSummary,
  mockRepositories,
  mockGitHubRepoGreen,
  mockGitHubRepoYellow,
  mockGitHubRepoRed,
  mockGitLabRepo,
  mockBitbucketRepo,
  createMockRepository,
  createDashboardSummary,
  dashboardSummarySuccessResponse,
  dashboardGridSuccessResponse,
} from './msw/fixtures';

// Fixtures - Pull Requests
export {
  mockPullRequests,
  mockPRGreen,
  mockPRYellow,
  mockPRRedCIFailed,
  mockPRDraft,
  mockCICheckSuccess,
  mockCICheckFailed,
  mockReviewApproved,
  mockReviewChangesRequested,
  mockMergeSuccess,
  createMockPullRequest,
  createPaginatedResponse,
  prListSuccessResponse,
  prNotFoundError,
} from './msw/fixtures';

// Fixtures - Repositories
export {
  mockRepositoriesWithStatus,
  mockDiscoveredGitHub,
  mockDiscoveredGitLab,
  getDiscoveredByProvider,
  createDiscoveredRepository,
  createRepositoryFromAdd,
  repositoryListSuccessResponse,
  repositoryNotFoundError,
} from './msw/fixtures';

// Fixtures - Settings
export {
  mockUserSettings,
  mockNotificationPreferences,
  createUserSettings,
  createNotificationPreferences,
  behaviorSettingsSuccessResponse,
  notificationPreferencesSuccessResponse,
} from './msw/fixtures';

// Fixtures - Accounts
export {
  mockAccounts,
  mockGitHubAccount,
  mockGitLabAccount,
  mockBitbucketAccount,
  createMockAccount,
  accountsListSuccessResponse,
  accountNotFoundError,
} from './msw/fixtures';

// ============================================================================
// i18n Testing
// ============================================================================

export {
  testI18n,
  hasTranslation,
  getTranslation,
  changeTestLanguage,
  getByTranslationKey,
  queryByTranslationKey,
  findByTranslationKey,
  getAllByTranslationKey,
  expectTranslationKey,
  verifyNamespaceKeys,
  translationTracker,
  testRTLLayout,
  expectRTLDirection,
  expectLTRDirection,
  testPluralization,
  pluralizationTestCases,
  type TranslationKey,
  type TranslationKeyOptions,
} from './i18n-test-utils';

// ============================================================================
// BDD Helpers
// ============================================================================

export {
  Feature,
  Scenario,
  xScenario,
  fScenario,
  // Capitalized versions (recommended to avoid Promise.then conflict)
  Given,
  When,
  Then,
  And,
  But,
  // Lowercase aliases (except 'then' which conflicts with Promise.then)
  given,
  when,
  and,
  but,
  Background,
  ScenarioOutline,
  dataTable,
  stepRegistry,
  defineGiven,
  defineWhen,
  defineThen,
  useGiven,
  useWhen,
  useThen,
  resetBDDState,
  type TestContext,
} from './bdd-helpers';

// ============================================================================
// Setup Utilities
// ============================================================================

export { isCI, wait, flushPromises, createDeferred, setMatchMedia } from './vitest.setup';
