/**
 * Pull Request API fixtures for MSW handlers
 *
 * Provides typed mock data for pull request endpoints.
 * Includes PRs with various statuses, CI checks, and reviews.
 */

import type {
  PullRequestWithDetails,
  PullRequest,
  CICheck,
  Review,
  PaginatedResponse,
} from '@/types';
import type { MergeResult } from '@/api/pullRequests';
import { successResponse, errorResponse } from './auth';

// Re-export utilities
export { successResponse, errorResponse };

// ============================================================================
// CI Check Fixtures
// ============================================================================

/** Successful CI check */
export const mockCICheckSuccess: CICheck = {
  id: 'check-1',
  pullRequestId: 'pr-1',
  name: 'build',
  status: 'completed',
  conclusion: 'success',
  url: 'https://github.com/org/repo/actions/runs/123',
  startedAt: '2024-01-01T10:00:00Z',
  completedAt: '2024-01-01T10:05:00Z',
  durationSeconds: 300,
};

/** Failed CI check */
export const mockCICheckFailed: CICheck = {
  id: 'check-2',
  pullRequestId: 'pr-1',
  name: 'test',
  status: 'completed',
  conclusion: 'failure',
  url: 'https://github.com/org/repo/actions/runs/124',
  startedAt: '2024-01-01T10:00:00Z',
  completedAt: '2024-01-01T10:10:00Z',
  durationSeconds: 600,
};

/** In-progress CI check */
export const mockCICheckInProgress: CICheck = {
  id: 'check-3',
  pullRequestId: 'pr-1',
  name: 'lint',
  status: 'in_progress',
  url: 'https://github.com/org/repo/actions/runs/125',
  startedAt: '2024-01-01T10:00:00Z',
};

/** Queued CI check */
export const mockCICheckQueued: CICheck = {
  id: 'check-4',
  pullRequestId: 'pr-1',
  name: 'deploy-preview',
  status: 'queued',
};

// ============================================================================
// Review Fixtures
// ============================================================================

/** Approved review */
export const mockReviewApproved: Review = {
  id: 'review-1',
  pullRequestId: 'pr-1',
  reviewer: 'reviewer-1',
  reviewerAvatarUrl: 'https://avatars.example.com/reviewer-1.png',
  state: 'approved',
  body: 'LGTM!',
  submittedAt: '2024-01-01T12:00:00Z',
};

/** Changes requested review */
export const mockReviewChangesRequested: Review = {
  id: 'review-2',
  pullRequestId: 'pr-1',
  reviewer: 'reviewer-2',
  reviewerAvatarUrl: 'https://avatars.example.com/reviewer-2.png',
  state: 'changes_requested',
  body: 'Please fix the typos in the documentation.',
  submittedAt: '2024-01-01T11:00:00Z',
};

/** Comment review */
export const mockReviewComment: Review = {
  id: 'review-3',
  pullRequestId: 'pr-1',
  reviewer: 'reviewer-3',
  state: 'commented',
  body: 'Have you considered using a different approach?',
  submittedAt: '2024-01-01T10:30:00Z',
};

/** Pending review */
export const mockReviewPending: Review = {
  id: 'review-4',
  pullRequestId: 'pr-1',
  reviewer: 'reviewer-4',
  state: 'pending',
  submittedAt: '2024-01-01T13:00:00Z',
};

// ============================================================================
// Pull Request Fixtures
// ============================================================================

/** Create a base pull request */
export function createBasePullRequest(overrides: Partial<PullRequest> = {}): PullRequest {
  const id = overrides.id || `pr-${Math.random().toString(36).substring(7)}`;
  const number = overrides.number || Math.floor(Math.random() * 1000);

  return {
    id,
    repositoryId: 'repo-1',
    provider: 'github',
    providerId: `provider-pr-${id}`,
    number,
    title: `Feature: Add new functionality #${number}`,
    description: 'This PR adds new functionality to the application.',
    url: `https://github.com/org/repo/pull/${number}`,
    state: 'open',
    sourceBranch: 'feature/new-feature',
    targetBranch: 'main',
    author: 'test-author',
    authorAvatarUrl: 'https://avatars.example.com/test-author.png',
    isDraft: false,
    isMergeable: true,
    hasConflicts: false,
    additions: 150,
    deletions: 50,
    changedFiles: 5,
    commitsCount: 3,
    commentsCount: 2,
    createdAt: '2024-01-01T09:00:00Z',
    updatedAt: new Date().toISOString(),
    ...overrides,
  };
}

/** Create a full pull request with details */
export function createMockPullRequest(
  overrides: Partial<PullRequestWithDetails> = {}
): PullRequestWithDetails {
  const base = createBasePullRequest(overrides);

  return {
    ...base,
    status: 'green',
    ciChecks: [mockCICheckSuccess],
    reviews: [mockReviewApproved],
    repositoryName: 'test-repo',
    repositoryOwner: 'test-owner',
    ...overrides,
  };
}

/** PR ready to merge (green status) */
export const mockPRGreen: PullRequestWithDetails = createMockPullRequest({
  id: 'pr-green-1',
  number: 101,
  title: 'feat: Add user authentication',
  status: 'green',
  ciChecks: [mockCICheckSuccess, { ...mockCICheckSuccess, id: 'check-lint', name: 'lint' }],
  reviews: [mockReviewApproved, { ...mockReviewApproved, id: 'review-2', reviewer: 'reviewer-2' }],
});

/** PR in progress (yellow status) */
export const mockPRYellow: PullRequestWithDetails = createMockPullRequest({
  id: 'pr-yellow-1',
  number: 102,
  title: 'feat: Implement dashboard filters',
  status: 'yellow',
  ciChecks: [mockCICheckSuccess, mockCICheckInProgress],
  reviews: [],
});

/** PR blocked (red status) - CI failed */
export const mockPRRedCIFailed: PullRequestWithDetails = createMockPullRequest({
  id: 'pr-red-ci-1',
  number: 103,
  title: 'fix: Fix memory leak in worker',
  status: 'red',
  ciChecks: [mockCICheckFailed, mockCICheckSuccess],
  reviews: [mockReviewApproved],
});

/** PR blocked (red status) - Changes requested */
export const mockPRRedChangesRequested: PullRequestWithDetails = createMockPullRequest({
  id: 'pr-red-changes-1',
  number: 104,
  title: 'docs: Update README',
  status: 'red',
  ciChecks: [mockCICheckSuccess],
  reviews: [mockReviewChangesRequested],
});

/** PR blocked (red status) - Has conflicts */
export const mockPRRedConflicts: PullRequestWithDetails = createMockPullRequest({
  id: 'pr-red-conflicts-1',
  number: 105,
  title: 'refactor: Reorganize project structure',
  status: 'red',
  hasConflicts: true,
  isMergeable: false,
  ciChecks: [mockCICheckSuccess],
  reviews: [mockReviewApproved],
});

/** Draft PR */
export const mockPRDraft: PullRequestWithDetails = createMockPullRequest({
  id: 'pr-draft-1',
  number: 106,
  title: '[WIP] feat: New feature in progress',
  status: 'yellow',
  isDraft: true,
  ciChecks: [],
  reviews: [],
});

/** Default list of mock PRs */
export const mockPullRequests: PullRequestWithDetails[] = [
  mockPRGreen,
  mockPRYellow,
  mockPRRedCIFailed,
  mockPRRedChangesRequested,
  mockPRDraft,
];

// ============================================================================
// Paginated Response Fixtures
// ============================================================================

/** Create a paginated response */
export function createPaginatedResponse<T>(
  items: T[],
  page = 1,
  perPage = 20
): PaginatedResponse<T> {
  const total = items.length;
  const totalPages = Math.ceil(total / perPage);
  const startIndex = (page - 1) * perPage;
  const paginatedItems = items.slice(startIndex, startIndex + perPage);

  return {
    items: paginatedItems,
    total,
    page,
    perPage,
    totalPages,
  };
}

/** Default paginated PR response */
export const mockPaginatedPRs = createPaginatedResponse(mockPullRequests);

// ============================================================================
// Merge Result Fixtures
// ============================================================================

/** Successful merge result */
export const mockMergeSuccess: MergeResult = {
  merged: true,
  sha: 'abc123def456',
  message: 'Pull request merged successfully',
};

/** Failed merge result - conflicts */
export const mockMergeFailedConflicts: MergeResult = {
  merged: false,
  message: 'Pull request has conflicts that must be resolved',
};

/** Failed merge result - CI not passing */
export const mockMergeFailedCI: MergeResult = {
  merged: false,
  message: 'Required status checks have not passed',
};

// ============================================================================
// Pre-built Responses
// ============================================================================

/** Successful PR list response */
export const prListSuccessResponse = successResponse(mockPaginatedPRs);

/** Empty PR list response */
export const emptyPRListResponse = successResponse(
  createPaginatedResponse<PullRequestWithDetails>([])
);

/** Successful PR detail response */
export const prDetailSuccessResponse = successResponse(mockPRGreen);

/** Successful merge response */
export const mergeSuccessResponse = successResponse(mockMergeSuccess);

/** Failed merge response */
export const mergeFailedResponse = successResponse(mockMergeFailedConflicts);

// ============================================================================
// Error Responses
// ============================================================================

/** PR not found error */
export const prNotFoundError = errorResponse('Pull request not found');

/** Repository not found error */
export const repoNotFoundError = errorResponse('Repository not found');

/** Merge not allowed error */
export const mergeNotAllowedError = errorResponse('Merge is not allowed for this pull request');
