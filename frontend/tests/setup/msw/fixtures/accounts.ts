/**
 * Accounts API fixtures for MSW handlers
 *
 * Provides typed mock data for provider account endpoints.
 * Includes account CRUD operations and validation.
 */

import type { GitProvider } from '@/types';
import type { ProviderAccount, AddAccountRequest, ValidateAccountResponse } from '@/types/account';
import { successResponse, errorResponse } from './auth';

// Re-export utilities
export { successResponse, errorResponse };

// ============================================================================
// Provider Account Fixtures
// ============================================================================

/** Create a mock provider account */
export function createMockAccount(overrides: Partial<ProviderAccount> = {}): ProviderAccount {
  const id = overrides.id || `account-${Math.random().toString(36).substring(7)}`;
  const provider = overrides.provider || 'github';

  return {
    id,
    provider,
    instanceUrl: null,
    accountLabel: `${provider} Account`,
    providerUsername: 'test-user',
    providerEmail: 'test@example.com',
    avatarUrl: 'https://avatars.example.com/test-user.png',
    scopes: ['repo', 'read:user'],
    tokenExpiresAt: null,
    validationStatus: 'valid',
    lastValidatedAt: new Date().toISOString(),
    isActive: true,
    isDefault: false,
    repositoryCount: 5,
    createdAt: '2024-01-01T00:00:00Z',
    ...overrides,
  };
}

/** Default GitHub account */
export const mockGitHubAccount: ProviderAccount = createMockAccount({
  id: 'account-gh-1',
  provider: 'github',
  accountLabel: 'GitHub Personal',
  providerUsername: 'ampel-user',
  providerEmail: 'user@github.com',
  scopes: ['repo', 'read:user', 'read:org'],
  isDefault: true,
  repositoryCount: 15,
});

/** Secondary GitHub account */
export const mockGitHubAccount2: ProviderAccount = createMockAccount({
  id: 'account-gh-2',
  provider: 'github',
  accountLabel: 'GitHub Work',
  providerUsername: 'work-user',
  providerEmail: 'work@company.com',
  scopes: ['repo', 'read:user'],
  isDefault: false,
  repositoryCount: 8,
});

/** GitLab account */
export const mockGitLabAccount: ProviderAccount = createMockAccount({
  id: 'account-gl-1',
  provider: 'gitlab',
  instanceUrl: null, // gitlab.com
  accountLabel: 'GitLab Main',
  providerUsername: 'gitlab-user',
  providerEmail: 'user@gitlab.com',
  scopes: ['api', 'read_repository'],
  isDefault: true,
  repositoryCount: 7,
});

/** Self-hosted GitLab account */
export const mockGitLabSelfHosted: ProviderAccount = createMockAccount({
  id: 'account-gl-2',
  provider: 'gitlab',
  instanceUrl: 'https://gitlab.company.com',
  accountLabel: 'Company GitLab',
  providerUsername: 'company-user',
  providerEmail: 'user@company.com',
  scopes: ['api', 'read_repository'],
  isDefault: false,
  repositoryCount: 12,
});

/** Bitbucket account */
export const mockBitbucketAccount: ProviderAccount = createMockAccount({
  id: 'account-bb-1',
  provider: 'bitbucket',
  accountLabel: 'Bitbucket',
  providerUsername: 'bitbucket-user',
  providerEmail: 'user@bitbucket.org',
  scopes: ['repository:read', 'pullrequest:read'],
  isDefault: true,
  repositoryCount: 3,
});

/** Invalid account (expired token) */
export const mockInvalidAccount: ProviderAccount = createMockAccount({
  id: 'account-invalid-1',
  provider: 'github',
  accountLabel: 'Expired Account',
  validationStatus: 'invalid',
  isActive: false,
  repositoryCount: 0,
});

/** Pending validation account */
export const mockPendingAccount: ProviderAccount = createMockAccount({
  id: 'account-pending-1',
  provider: 'github',
  accountLabel: 'New Account',
  validationStatus: 'pending',
  lastValidatedAt: null,
  repositoryCount: 0,
});

/** Default list of accounts */
export const mockAccounts: ProviderAccount[] = [
  mockGitHubAccount,
  mockGitLabAccount,
  mockBitbucketAccount,
];

/** Full list including secondary accounts */
export const mockAllAccounts: ProviderAccount[] = [
  mockGitHubAccount,
  mockGitHubAccount2,
  mockGitLabAccount,
  mockGitLabSelfHosted,
  mockBitbucketAccount,
];

// ============================================================================
// Validation Response Fixtures
// ============================================================================

/** Successful validation response */
export const mockValidationSuccess: ValidateAccountResponse = {
  isValid: true,
  validationStatus: 'valid',
};

/** Failed validation response */
export const mockValidationFailed: ValidateAccountResponse = {
  isValid: false,
  validationStatus: 'invalid',
  errorMessage: 'Token has expired or been revoked',
};

/** Expired token validation response */
export const mockValidationExpired: ValidateAccountResponse = {
  isValid: false,
  validationStatus: 'expired',
  errorMessage: 'Token has expired',
};

// ============================================================================
// Factory Functions
// ============================================================================

/**
 * Create an account from add request
 */
export function createAccountFromAdd(request: AddAccountRequest): ProviderAccount {
  return createMockAccount({
    provider: request.provider,
    instanceUrl: request.instanceUrl || null,
    accountLabel: request.accountLabel,
    providerUsername: request.username || 'new-user',
    validationStatus: 'pending',
    lastValidatedAt: null,
    isActive: true,
    isDefault: false,
    repositoryCount: 0,
  });
}

/**
 * Get accounts by provider
 */
export function getAccountsByProvider(provider: GitProvider): ProviderAccount[] {
  return mockAllAccounts.filter((account) => account.provider === provider);
}

/**
 * Get default account for provider
 */
export function getDefaultAccount(provider: GitProvider): ProviderAccount | undefined {
  return mockAllAccounts.find((account) => account.provider === provider && account.isDefault);
}

// ============================================================================
// Pre-built Responses
// ============================================================================

/** Successful accounts list response */
export const accountsListSuccessResponse = successResponse(mockAccounts);

/** Full accounts list response */
export const allAccountsListSuccessResponse = successResponse(mockAllAccounts);

/** Empty accounts list response */
export const emptyAccountsListResponse = successResponse<ProviderAccount[]>([]);

/** Successful single account response */
export const accountDetailSuccessResponse = successResponse(mockGitHubAccount);

/** Successful validation response */
export const validateSuccessResponse = successResponse(mockValidationSuccess);

/** Failed validation response */
export const validateFailedResponse = successResponse(mockValidationFailed);

// ============================================================================
// Error Responses
// ============================================================================

/** Account not found error */
export const accountNotFoundError = errorResponse('Account not found');

/** Account already exists error */
export const accountExistsError = errorResponse('Account already connected');

/** Invalid token error */
export const invalidTokenError = errorResponse('Invalid or expired access token');

/** Provider connection error */
export const providerConnectionError = errorResponse('Failed to connect to provider');

/** Rate limit error */
export const rateLimitError = errorResponse('API rate limit exceeded. Please try again later.');
