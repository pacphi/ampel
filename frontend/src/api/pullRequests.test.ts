import { describe, expect, it, vi, beforeEach } from 'vitest';
import { pullRequestsApi } from './pullRequests';
import apiClient from './client';
import type { PullRequestWithDetails, PaginatedResponse } from '@/types';

// Mock the API client
vi.mock('./client', () => ({
  default: {
    get: vi.fn(),
    post: vi.fn(),
  },
}));

const mockedApiClient = vi.mocked(apiClient);

describe('pullRequestsApi', () => {
  const mockPullRequest: PullRequestWithDetails = {
    id: 'pr-1',
    repositoryId: 'repo-1',
    provider: 'github',
    providerId: '123',
    number: 42,
    title: 'Test PR',
    description: 'Test description',
    url: 'https://github.com/test/repo/pull/42',
    state: 'open',
    sourceBranch: 'feature/test',
    targetBranch: 'main',
    author: 'testuser',
    authorAvatarUrl: 'https://example.com/avatar.png',
    isDraft: false,
    isMergeable: true,
    hasConflicts: false,
    additions: 100,
    deletions: 50,
    changedFiles: 5,
    commitsCount: 3,
    commentsCount: 2,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-02T00:00:00Z',
    status: 'green',
    ciChecks: [],
    reviews: [],
    repositoryName: 'repo',
    repositoryOwner: 'test',
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('list', () => {
    it('should list pull requests with default pagination', async () => {
      const mockResponse: PaginatedResponse<PullRequestWithDetails> = {
        data: [mockPullRequest],
        total: 1,
        page: 1,
        perPage: 20,
        totalPages: 1,
      };

      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: mockResponse },
      });

      const result = await pullRequestsApi.list();

      expect(mockedApiClient.get).toHaveBeenCalledWith('/pull-requests', {
        params: { page: 1, perPage: 20 },
      });
      expect(result).toEqual(mockResponse);
    });

    it('should list pull requests with custom pagination', async () => {
      const mockResponse: PaginatedResponse<PullRequestWithDetails> = {
        data: [mockPullRequest],
        total: 50,
        page: 2,
        perPage: 10,
        totalPages: 5,
      };

      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: mockResponse },
      });

      const result = await pullRequestsApi.list(2, 10);

      expect(mockedApiClient.get).toHaveBeenCalledWith('/pull-requests', {
        params: { page: 2, perPage: 10 },
      });
      expect(result).toEqual(mockResponse);
    });
  });

  describe('listByRepository', () => {
    it('should list pull requests for a specific repository', async () => {
      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: [mockPullRequest] },
      });

      const result = await pullRequestsApi.listByRepository('repo-1');

      expect(mockedApiClient.get).toHaveBeenCalledWith('/repositories/repo-1/pull-requests');
      expect(result).toEqual([mockPullRequest]);
    });
  });

  describe('get', () => {
    it('should get a specific pull request', async () => {
      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: mockPullRequest },
      });

      const result = await pullRequestsApi.get('repo-1', 'pr-1');

      expect(mockedApiClient.get).toHaveBeenCalledWith('/repositories/repo-1/pull-requests/pr-1');
      expect(result).toEqual(mockPullRequest);
    });
  });

  describe('merge', () => {
    it('should merge a pull request with merge strategy', async () => {
      const mockMergeResult = {
        merged: true,
        sha: 'abc123',
        message: 'Pull request merged successfully',
      };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: mockMergeResult },
      });

      const request = {
        strategy: 'merge' as const,
        commitTitle: 'Merge PR #42',
        commitMessage: 'Merge pull request',
        deleteBranch: true,
      };

      const result = await pullRequestsApi.merge('repo-1', 'pr-1', request);

      expect(mockedApiClient.post).toHaveBeenCalledWith(
        '/repositories/repo-1/pull-requests/pr-1/merge',
        request
      );
      expect(result).toEqual(mockMergeResult);
    });

    it('should merge a pull request with squash strategy', async () => {
      const mockMergeResult = {
        merged: true,
        sha: 'def456',
        message: 'Pull request squashed and merged',
      };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: mockMergeResult },
      });

      const request = {
        strategy: 'squash' as const,
        deleteBranch: false,
      };

      const result = await pullRequestsApi.merge('repo-1', 'pr-1', request);

      expect(mockedApiClient.post).toHaveBeenCalledWith(
        '/repositories/repo-1/pull-requests/pr-1/merge',
        request
      );
      expect(result).toEqual(mockMergeResult);
    });

    it('should merge a pull request with rebase strategy', async () => {
      const mockMergeResult = {
        merged: true,
        sha: 'ghi789',
        message: 'Pull request rebased and merged',
      };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: mockMergeResult },
      });

      const request = {
        strategy: 'rebase' as const,
        deleteBranch: true,
      };

      const result = await pullRequestsApi.merge('repo-1', 'pr-1', request);

      expect(mockedApiClient.post).toHaveBeenCalledWith(
        '/repositories/repo-1/pull-requests/pr-1/merge',
        request
      );
      expect(result).toEqual(mockMergeResult);
    });
  });

  describe('refresh', () => {
    it('should refresh a pull request', async () => {
      const refreshedPr = { ...mockPullRequest, updatedAt: '2024-01-03T00:00:00Z' };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: refreshedPr },
      });

      const result = await pullRequestsApi.refresh('repo-1', 'pr-1');

      expect(mockedApiClient.post).toHaveBeenCalledWith(
        '/repositories/repo-1/pull-requests/pr-1/refresh'
      );
      expect(result).toEqual(refreshedPr);
    });
  });
});
