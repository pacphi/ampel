/**
 * Dashboard API fixtures for MSW handlers
 *
 * Provides typed mock data for dashboard endpoints.
 * Includes repository grid and summary statistics.
 */

import type { DashboardSummary, RepositoryWithStatus, AmpelStatus, GitProvider } from '@/types';
import { successResponse, errorResponse } from './auth';

// Re-export utilities for convenience
export { successResponse, errorResponse };

// ============================================================================
// Repository Fixtures
// ============================================================================

/** Create a mock repository with status */
export function createMockRepository(
  overrides: Partial<RepositoryWithStatus> = {}
): RepositoryWithStatus {
  const id = overrides.id || `repo-${Math.random().toString(36).substring(7)}`;
  const name = overrides.name || 'test-repo';
  const owner = overrides.owner || 'test-owner';

  return {
    id,
    userId: 'user-123',
    provider: 'github',
    providerId: `provider-${id}`,
    owner,
    name,
    fullName: `${owner}/${name}`,
    description: 'A test repository',
    url: `https://github.com/${owner}/${name}`,
    defaultBranch: 'main',
    isPrivate: false,
    isArchived: false,
    pollIntervalSeconds: 300,
    lastPolledAt: new Date().toISOString(),
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: new Date().toISOString(),
    status: 'green',
    openPrCount: 0,
    ...overrides,
  };
}

/** GitHub repository with green status (ready to merge) */
export const mockGitHubRepoGreen: RepositoryWithStatus = createMockRepository({
  id: 'repo-gh-1',
  provider: 'github',
  owner: 'ampel-org',
  name: 'ampel-frontend',
  description: 'Ampel frontend React application',
  isPrivate: false,
  status: 'green',
  openPrCount: 2,
});

/** GitHub repository with yellow status (in progress) */
export const mockGitHubRepoYellow: RepositoryWithStatus = createMockRepository({
  id: 'repo-gh-2',
  provider: 'github',
  owner: 'ampel-org',
  name: 'ampel-backend',
  description: 'Ampel backend Rust API',
  isPrivate: false,
  status: 'yellow',
  openPrCount: 5,
});

/** GitHub repository with red status (blocked) */
export const mockGitHubRepoRed: RepositoryWithStatus = createMockRepository({
  id: 'repo-gh-3',
  provider: 'github',
  owner: 'ampel-org',
  name: 'ampel-docs',
  description: 'Documentation repository',
  isPrivate: false,
  status: 'red',
  openPrCount: 1,
});

/** GitLab repository */
export const mockGitLabRepo: RepositoryWithStatus = createMockRepository({
  id: 'repo-gl-1',
  provider: 'gitlab',
  owner: 'ampel-team',
  name: 'infrastructure',
  description: 'Infrastructure as code',
  url: 'https://gitlab.com/ampel-team/infrastructure',
  isPrivate: true,
  status: 'green',
  openPrCount: 3,
});

/** Bitbucket repository */
export const mockBitbucketRepo: RepositoryWithStatus = createMockRepository({
  id: 'repo-bb-1',
  provider: 'bitbucket',
  owner: 'ampel-workspace',
  name: 'shared-libs',
  description: 'Shared libraries',
  url: 'https://bitbucket.org/ampel-workspace/shared-libs',
  isPrivate: true,
  status: 'yellow',
  openPrCount: 2,
});

/** Private repository */
export const mockPrivateRepo: RepositoryWithStatus = createMockRepository({
  id: 'repo-private-1',
  owner: 'private-org',
  name: 'secret-project',
  isPrivate: true,
  status: 'none',
  openPrCount: 0,
});

/** Archived repository */
export const mockArchivedRepo: RepositoryWithStatus = createMockRepository({
  id: 'repo-archived-1',
  owner: 'legacy-org',
  name: 'old-project',
  isArchived: true,
  status: 'none',
  openPrCount: 0,
});

/** Default list of mock repositories */
export const mockRepositories: RepositoryWithStatus[] = [
  mockGitHubRepoGreen,
  mockGitHubRepoYellow,
  mockGitHubRepoRed,
  mockGitLabRepo,
  mockBitbucketRepo,
];

// ============================================================================
// Dashboard Summary Fixtures
// ============================================================================

/** Default dashboard summary */
export const mockDashboardSummary: DashboardSummary = {
  totalRepositories: 5,
  totalOpenPrs: 13,
  statusCounts: {
    green: 2,
    yellow: 2,
    red: 1,
  },
  providerCounts: {
    github: 3,
    gitlab: 1,
    bitbucket: 1,
  },
  repositoryBreakdown: {
    public: 3,
    private: 2,
    archived: 0,
  },
  openPrsBreakdown: {
    public: 8,
    private: 5,
    archived: 0,
  },
  readyToMergeBreakdown: {
    public: 4,
    private: 1,
    archived: 0,
  },
  needsAttentionBreakdown: {
    public: 1,
    private: 0,
    archived: 0,
  },
};

/** Empty dashboard summary (no repositories) */
export const mockEmptyDashboardSummary: DashboardSummary = {
  totalRepositories: 0,
  totalOpenPrs: 0,
  statusCounts: {
    green: 0,
    yellow: 0,
    red: 0,
  },
  providerCounts: {
    github: 0,
    gitlab: 0,
    bitbucket: 0,
  },
};

// ============================================================================
// Pre-built Responses
// ============================================================================

/** Successful dashboard summary response */
export const dashboardSummarySuccessResponse = successResponse(mockDashboardSummary);

/** Empty dashboard summary response */
export const emptyDashboardSummaryResponse = successResponse(mockEmptyDashboardSummary);

/** Successful dashboard grid response */
export const dashboardGridSuccessResponse = successResponse(mockRepositories);

/** Empty dashboard grid response */
export const emptyDashboardGridResponse = successResponse<RepositoryWithStatus[]>([]);

// ============================================================================
// Factory Functions
// ============================================================================

/**
 * Create a dashboard summary with custom status counts
 */
export function createDashboardSummary(
  statusCounts: { green?: number; yellow?: number; red?: number },
  providerCounts: { github?: number; gitlab?: number; bitbucket?: number } = {}
): DashboardSummary {
  const counts = {
    green: statusCounts.green ?? 0,
    yellow: statusCounts.yellow ?? 0,
    red: statusCounts.red ?? 0,
  };

  const providers = {
    github: providerCounts.github ?? 0,
    gitlab: providerCounts.gitlab ?? 0,
    bitbucket: providerCounts.bitbucket ?? 0,
  };

  const totalRepositories = providers.github + providers.gitlab + providers.bitbucket;
  const totalOpenPrs = counts.green + counts.yellow + counts.red;

  return {
    totalRepositories,
    totalOpenPrs,
    statusCounts: counts,
    providerCounts: providers,
  };
}

/**
 * Create a list of repositories with specified statuses
 */
export function createRepositoriesWithStatuses(
  statuses: AmpelStatus[],
  provider: GitProvider = 'github'
): RepositoryWithStatus[] {
  return statuses.map((status, index) =>
    createMockRepository({
      id: `repo-${provider}-${index}`,
      provider,
      name: `repo-${index}`,
      status,
      openPrCount: status === 'none' ? 0 : Math.floor(Math.random() * 5) + 1,
    })
  );
}
