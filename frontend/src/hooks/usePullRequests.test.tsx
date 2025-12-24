import { describe, expect, it, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactNode } from 'react';
import {
  usePullRequests,
  useRepositoryPullRequests,
  usePullRequest,
  useMergePullRequest,
  useRefreshPullRequest,
} from './usePullRequests';
import { pullRequestsApi } from '@/api/pullRequests';
import type { PullRequestWithDetails, PaginatedResponse } from '@/types';

// Mock the API
vi.mock('@/api/pullRequests', () => ({
  pullRequestsApi: {
    list: vi.fn(),
    listByRepository: vi.fn(),
    get: vi.fn(),
    merge: vi.fn(),
    refresh: vi.fn(),
  },
}));

const mockedApi = vi.mocked(pullRequestsApi);

// Create a wrapper with React Query provider
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

describe('usePullRequests hooks', () => {
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
    isDraft: false,
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

  describe('usePullRequests', () => {
    it('should fetch pull requests with default pagination', async () => {
      const mockResponse: PaginatedResponse<PullRequestWithDetails> = {
        items: [mockPullRequest],
        total: 1,
        page: 1,
        perPage: 20,
        totalPages: 1,
      };

      mockedApi.list.mockResolvedValueOnce(mockResponse);

      const { result } = renderHook(() => usePullRequests(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockedApi.list).toHaveBeenCalledWith(1, 20);
      expect(result.current.data).toEqual(mockResponse);
    });

    it('should fetch pull requests with custom pagination', async () => {
      const mockResponse: PaginatedResponse<PullRequestWithDetails> = {
        items: [mockPullRequest],
        total: 50,
        page: 2,
        perPage: 10,
        totalPages: 5,
      };

      mockedApi.list.mockResolvedValueOnce(mockResponse);

      const { result } = renderHook(() => usePullRequests(2, 10), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockedApi.list).toHaveBeenCalledWith(2, 10);
    });
  });

  describe('useRepositoryPullRequests', () => {
    it('should fetch pull requests for a repository', async () => {
      mockedApi.listByRepository.mockResolvedValueOnce([mockPullRequest]);

      const { result } = renderHook(() => useRepositoryPullRequests('repo-1'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockedApi.listByRepository).toHaveBeenCalledWith('repo-1');
      expect(result.current.data).toEqual([mockPullRequest]);
    });

    it('should not fetch when repoId is empty', async () => {
      const { result } = renderHook(() => useRepositoryPullRequests(''), {
        wrapper: createWrapper(),
      });

      // Should be disabled and not fetch
      expect(result.current.isFetching).toBe(false);
      expect(mockedApi.listByRepository).not.toHaveBeenCalled();
    });
  });

  describe('usePullRequest', () => {
    it('should fetch a specific pull request', async () => {
      mockedApi.get.mockResolvedValueOnce(mockPullRequest);

      const { result } = renderHook(() => usePullRequest('repo-1', 'pr-1'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockedApi.get).toHaveBeenCalledWith('repo-1', 'pr-1');
      expect(result.current.data).toEqual(mockPullRequest);
    });

    it('should not fetch when repoId is empty', async () => {
      const { result } = renderHook(() => usePullRequest('', 'pr-1'), {
        wrapper: createWrapper(),
      });

      expect(result.current.isFetching).toBe(false);
      expect(mockedApi.get).not.toHaveBeenCalled();
    });

    it('should not fetch when prId is empty', async () => {
      const { result } = renderHook(() => usePullRequest('repo-1', ''), {
        wrapper: createWrapper(),
      });

      expect(result.current.isFetching).toBe(false);
      expect(mockedApi.get).not.toHaveBeenCalled();
    });
  });

  describe('useMergePullRequest', () => {
    it('should merge a pull request', async () => {
      const mockMergeResult = {
        merged: true,
        sha: 'abc123',
        message: 'Merged successfully',
      };

      mockedApi.merge.mockResolvedValueOnce(mockMergeResult);

      const { result } = renderHook(() => useMergePullRequest(), {
        wrapper: createWrapper(),
      });

      await result.current.mutateAsync({
        repoId: 'repo-1',
        prId: 'pr-1',
        request: {
          strategy: 'merge',
          deleteBranch: true,
        },
      });

      expect(mockedApi.merge).toHaveBeenCalledWith('repo-1', 'pr-1', {
        strategy: 'merge',
        deleteBranch: true,
      });
    });
  });

  describe('useRefreshPullRequest', () => {
    it('should refresh a pull request', async () => {
      const refreshedPr = { ...mockPullRequest, updatedAt: '2024-01-03T00:00:00Z' };
      mockedApi.refresh.mockResolvedValueOnce(refreshedPr);

      const { result } = renderHook(() => useRefreshPullRequest(), {
        wrapper: createWrapper(),
      });

      const data = await result.current.mutateAsync({
        repoId: 'repo-1',
        prId: 'pr-1',
      });

      expect(mockedApi.refresh).toHaveBeenCalledWith('repo-1', 'pr-1');
      expect(data).toEqual(refreshedPr);
    });
  });
});
