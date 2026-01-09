/**
 * Repositories API fixtures for MSW handlers
 *
 * Provides typed mock data for repository endpoints.
 * Includes repository CRUD operations and discovery.
 */

import type { Repository, RepositoryWithStatus, DiscoveredRepository, GitProvider } from '@/types';
import { successResponse, errorResponse } from './auth';
import {
  mockGitHubRepoGreen,
  mockGitHubRepoYellow,
  mockGitHubRepoRed,
  mockGitLabRepo,
  mockBitbucketRepo,
  createMockRepository,
} from './dashboard';

// Re-export utilities and dashboard fixtures
export {
  successResponse,
  errorResponse,
  createMockRepository,
  mockGitHubRepoGreen,
  mockGitHubRepoYellow,
  mockGitHubRepoRed,
  mockGitLabRepo,
  mockBitbucketRepo,
};

// ============================================================================
// Repository (without status) Fixtures
// ============================================================================

/** Convert RepositoryWithStatus to Repository (remove status fields) */
export function toRepository(repo: RepositoryWithStatus): Repository {
  const { status: _status, openPrCount: _openPrCount, ...repository } = repo;
  return repository;
}

/** Default repository (GitHub) */
export const mockRepository: Repository = toRepository(mockGitHubRepoGreen);

// ============================================================================
// Discovered Repository Fixtures
// ============================================================================

/** Create a discovered repository */
export function createDiscoveredRepository(
  overrides: Partial<DiscoveredRepository> = {}
): DiscoveredRepository {
  const name = overrides.name || 'discovered-repo';
  const owner = overrides.owner || 'discovered-owner';

  return {
    provider: 'github',
    providerId: `provider-discovered-${Math.random().toString(36).substring(7)}`,
    owner,
    name,
    fullName: `${owner}/${name}`,
    description: 'A discovered repository',
    url: `https://github.com/${owner}/${name}`,
    defaultBranch: 'main',
    isPrivate: false,
    isArchived: false,
    ...overrides,
  };
}

/** GitHub discovered repositories */
export const mockDiscoveredGitHub: DiscoveredRepository[] = [
  createDiscoveredRepository({
    provider: 'github',
    owner: 'ampel-org',
    name: 'new-project',
    description: 'A new project to track',
  }),
  createDiscoveredRepository({
    provider: 'github',
    owner: 'ampel-org',
    name: 'another-repo',
    description: 'Another repository',
    isPrivate: true,
  }),
  createDiscoveredRepository({
    provider: 'github',
    owner: 'ampel-org',
    name: 'archived-project',
    description: 'An archived project',
    isArchived: true,
  }),
];

/** GitLab discovered repositories */
export const mockDiscoveredGitLab: DiscoveredRepository[] = [
  createDiscoveredRepository({
    provider: 'gitlab',
    owner: 'ampel-team',
    name: 'gitlab-project',
    url: 'https://gitlab.com/ampel-team/gitlab-project',
  }),
  createDiscoveredRepository({
    provider: 'gitlab',
    owner: 'ampel-team',
    name: 'internal-tools',
    url: 'https://gitlab.com/ampel-team/internal-tools',
    isPrivate: true,
  }),
];

/** Bitbucket discovered repositories */
export const mockDiscoveredBitbucket: DiscoveredRepository[] = [
  createDiscoveredRepository({
    provider: 'bitbucket',
    owner: 'ampel-workspace',
    name: 'bb-project',
    url: 'https://bitbucket.org/ampel-workspace/bb-project',
  }),
];

/** Get discovered repositories by provider */
export function getDiscoveredByProvider(provider: GitProvider): DiscoveredRepository[] {
  switch (provider) {
    case 'github':
      return mockDiscoveredGitHub;
    case 'gitlab':
      return mockDiscoveredGitLab;
    case 'bitbucket':
      return mockDiscoveredBitbucket;
    default:
      return [];
  }
}

// ============================================================================
// Repository List Fixtures
// ============================================================================

/** Default list of repositories with status */
export const mockRepositoriesWithStatus: RepositoryWithStatus[] = [
  mockGitHubRepoGreen,
  mockGitHubRepoYellow,
  mockGitHubRepoRed,
  mockGitLabRepo,
  mockBitbucketRepo,
];

/** Empty repository list */
export const emptyRepositories: RepositoryWithStatus[] = [];

// ============================================================================
// Factory Functions
// ============================================================================

/**
 * Create a repository from add request parameters
 */
export function createRepositoryFromAdd(
  provider: GitProvider,
  owner: string,
  name: string,
  pollIntervalSeconds?: number
): Repository {
  return {
    id: `repo-new-${Math.random().toString(36).substring(7)}`,
    userId: 'user-123',
    provider,
    providerId: `provider-${provider}-${Date.now()}`,
    owner,
    name,
    fullName: `${owner}/${name}`,
    description: null,
    url: getProviderUrl(provider, owner, name),
    defaultBranch: 'main',
    isPrivate: false,
    isArchived: false,
    pollIntervalSeconds: pollIntervalSeconds ?? 300,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

/** Get provider URL */
function getProviderUrl(provider: GitProvider, owner: string, name: string): string {
  switch (provider) {
    case 'github':
      return `https://github.com/${owner}/${name}`;
    case 'gitlab':
      return `https://gitlab.com/${owner}/${name}`;
    case 'bitbucket':
      return `https://bitbucket.org/${owner}/${name}`;
    default:
      return `https://example.com/${owner}/${name}`;
  }
}

// ============================================================================
// Pre-built Responses
// ============================================================================

/** Successful repository list response */
export const repositoryListSuccessResponse = successResponse(mockRepositoriesWithStatus);

/** Empty repository list response */
export const emptyRepositoryListResponse = successResponse(emptyRepositories);

/** Successful single repository response */
export const repositoryDetailSuccessResponse = successResponse(mockGitHubRepoGreen);

/** Successful discovery response (GitHub) */
export const discoverGitHubSuccessResponse = successResponse(mockDiscoveredGitHub);

/** Successful discovery response (GitLab) */
export const discoverGitLabSuccessResponse = successResponse(mockDiscoveredGitLab);

/** Successful discovery response (Bitbucket) */
export const discoverBitbucketSuccessResponse = successResponse(mockDiscoveredBitbucket);

/** Empty discovery response */
export const emptyDiscoveryResponse = successResponse<DiscoveredRepository[]>([]);

// ============================================================================
// Error Responses
// ============================================================================

/** Repository not found error */
export const repositoryNotFoundError = errorResponse('Repository not found');

/** Repository already exists error */
export const repositoryExistsError = errorResponse('Repository already tracked');

/** Provider authentication error */
export const providerAuthError = errorResponse('Failed to authenticate with provider');

/** Discovery failed error */
export const discoveryFailedError = errorResponse('Failed to discover repositories');

/** Invalid repository error */
export const invalidRepositoryError = errorResponse('Invalid repository configuration');
