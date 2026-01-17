/**
 * MSW Fixtures Index
 *
 * Central export point for all API fixtures.
 * Use these fixtures to create consistent mock data across tests.
 *
 * @example
 * ```typescript
 * import { fixtures } from '@/tests/setup/msw/fixtures';
 *
 * // Use default fixtures
 * const user = fixtures.auth.mockUser;
 * const repos = fixtures.dashboard.mockRepositories;
 *
 * // Or use factory functions
 * const customRepo = fixtures.dashboard.createMockRepository({
 *   status: 'red',
 *   openPrCount: 10,
 * });
 * ```
 */

// Auth fixtures
export * as auth from './auth';
export {
  mockUser,
  mockUser2,
  mockAuthTokens,
  mockRefreshedTokens,
  successResponse,
  errorResponse,
  loginSuccessResponse,
  registerSuccessResponse,
  refreshSuccessResponse,
  meSuccessResponse,
  invalidCredentialsError,
  emailExistsError,
  expiredTokenError,
  unauthorizedError,
} from './auth';

// Dashboard fixtures
export * as dashboard from './dashboard';
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
  createRepositoriesWithStatuses,
  dashboardSummarySuccessResponse,
  dashboardGridSuccessResponse,
} from './dashboard';

// Pull Requests fixtures
export * as pullRequests from './pull-requests';
export {
  mockPullRequests,
  mockPRGreen,
  mockPRYellow,
  mockPRRedCIFailed,
  mockPRRedChangesRequested,
  mockPRRedConflicts,
  mockPRDraft,
  mockPaginatedPRs,
  mockCICheckSuccess,
  mockCICheckFailed,
  mockCICheckInProgress,
  mockReviewApproved,
  mockReviewChangesRequested,
  mockMergeSuccess,
  mockMergeFailedConflicts,
  createMockPullRequest,
  createPaginatedResponse,
  prListSuccessResponse,
  prDetailSuccessResponse,
  mergeSuccessResponse,
  prNotFoundError,
} from './pull-requests';

// Repositories fixtures
export * as repositories from './repositories';
export {
  mockRepositoriesWithStatus,
  mockDiscoveredGitHub,
  mockDiscoveredGitLab,
  mockDiscoveredBitbucket,
  getDiscoveredByProvider,
  createRepositoryFromAdd,
  createDiscoveredRepository,
  repositoryListSuccessResponse,
  repositoryDetailSuccessResponse,
  discoverGitHubSuccessResponse,
  repositoryNotFoundError,
  repositoryExistsError,
} from './repositories';

// Settings fixtures
export * as settings from './settings';
export {
  mockUserSettings,
  mockConservativeSettings,
  mockAggressiveSettings,
  mockNotificationPreferences,
  mockFullNotificationPreferences,
  mockDisabledNotificationPreferences,
  createUserSettings,
  createNotificationPreferences,
  applyUserSettingsUpdate,
  applyNotificationUpdate,
  behaviorSettingsSuccessResponse,
  notificationPreferencesSuccessResponse,
  testSlackSuccessResponse,
  testEmailSuccessResponse,
} from './settings';

// Accounts fixtures
export * as accounts from './accounts';
export {
  mockAccounts,
  mockAllAccounts,
  mockGitHubAccount,
  mockGitLabAccount,
  mockBitbucketAccount,
  mockInvalidAccount,
  mockPendingAccount,
  mockValidationSuccess,
  mockValidationFailed,
  createMockAccount,
  createAccountFromAdd,
  getAccountsByProvider,
  getDefaultAccount,
  accountsListSuccessResponse,
  accountDetailSuccessResponse,
  validateSuccessResponse,
  accountNotFoundError,
  invalidTokenError,
} from './accounts';

/**
 * Grouped fixtures namespace for convenient access
 */
import * as authFixtures from './auth';
import * as dashboardFixtures from './dashboard';
import * as pullRequestsFixtures from './pull-requests';
import * as repositoriesFixtures from './repositories';
import * as settingsFixtures from './settings';
import * as accountsFixtures from './accounts';

export const fixtures = {
  auth: authFixtures,
  dashboard: dashboardFixtures,
  pullRequests: pullRequestsFixtures,
  repositories: repositoriesFixtures,
  settings: settingsFixtures,
  accounts: accountsFixtures,
} as const;
