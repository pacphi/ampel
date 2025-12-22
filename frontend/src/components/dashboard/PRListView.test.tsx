import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import PRListView from './PRListView';
import type { PaginatedResponse, PullRequestWithDetails } from '@/types';
import type { UseInfiniteQueryResult } from '@tanstack/react-query';

vi.mock('@/api/merge', () => ({
  mergeApi: {
    bulkMerge: vi.fn(),
  },
}));

vi.mock('@/api/settings', () => ({
  settingsApi: {
    getBehavior: vi.fn(),
  },
}));

vi.mock('@/hooks/usePullRequests', () => ({
  useInfinitePullRequests: vi.fn(),
}));

vi.mock('@/components/ui/use-toast', () => ({
  useToast: vi.fn(() => ({ toast: vi.fn(), dismiss: vi.fn(), toasts: [] })),
}));

import { mergeApi } from '@/api/merge';
import { settingsApi } from '@/api/settings';
import { useInfinitePullRequests } from '@/hooks/usePullRequests';
import { useToast } from '@/components/ui/use-toast';

const mockedMergeApi = vi.mocked(mergeApi);
const mockedSettingsApi = vi.mocked(settingsApi);
const mockedUseInfinitePullRequests = vi.mocked(useInfinitePullRequests);
const mockedUseToast = vi.mocked(useToast);

function renderPRListView(filterStatus?: 'green' | 'yellow' | 'red') {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <PRListView filterStatus={filterStatus} />
    </QueryClientProvider>
  );
}

describe('PRListView', () => {
  const mockToast = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockedUseToast.mockReturnValue({
      toast: mockToast,
      dismiss: vi.fn(),
      toasts: [],
    });
  });

  describe('Loading State', () => {
    it('renders loading spinner while fetching data', () => {
      mockedUseInfinitePullRequests.mockReturnValue({
        data: undefined,
        isLoading: true,
        isFetchingNextPage: false,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      const { container } = renderPRListView();

      const spinner = container.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });
  });

  describe('Empty State', () => {
    it('shows empty state when no PRs', async () => {
      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: [], total: 0 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText('No pull requests found')).toBeInTheDocument();
      });
    });
  });

  describe('PR List Display', () => {
    it('renders pull requests', async () => {
      const mockPRs = [
        {
          id: '1',
          number: 123,
          title: 'Fix bug in authentication',
          status: 'green',
          isMergeable: true,
          hasConflicts: false,
          isDraft: false,
          author: 'john-doe',
          sourceBranch: 'fix/auth',
          targetBranch: 'main',
          repositoryOwner: 'acme',
          repositoryName: 'app',
          url: 'https://github.com/acme/app/pull/123',
          additions: 50,
          deletions: 20,
          commentsCount: 3,
          createdAt: '2024-01-01T00:00:00Z',
        },
        {
          id: '2',
          number: 124,
          title: 'Add new feature',
          status: 'yellow',
          isMergeable: true,
          hasConflicts: false,
          isDraft: false,
          author: 'jane-smith',
          sourceBranch: 'feat/new-feature',
          targetBranch: 'main',
          repositoryOwner: 'acme',
          repositoryName: 'app',
          url: 'https://github.com/acme/app/pull/124',
          additions: 100,
          deletions: 10,
          commentsCount: 0,
          createdAt: '2024-01-02T00:00:00Z',
        },
      ];

      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: mockPRs, total: 2 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText('Fix bug in authentication')).toBeInTheDocument();
      });

      expect(screen.getByText('Add new feature')).toBeInTheDocument();
    });

    it('shows checkbox for mergeable PRs only', async () => {
      const mockPRs = [
        {
          id: '1',
          number: 123,
          title: 'Mergeable PR',
          status: 'green',
          isMergeable: true,
          hasConflicts: false,
          isDraft: false,
          url: 'https://github.com/test/repo/pull/123',
        },
        {
          id: '2',
          number: 124,
          title: 'Non-mergeable PR',
          status: 'red',
          isMergeable: false,
          hasConflicts: true,
          isDraft: false,
          url: 'https://github.com/test/repo/pull/124',
        },
      ];

      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: mockPRs, total: 2 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText('Mergeable PR')).toBeInTheDocument();
      });

      // Only one checkbox should be visible (for mergeable PR)
      const checkboxes = screen.queryAllByRole('button', { name: /square/i });
      expect(checkboxes.length).toBe(1);
    });
  });

  describe('Status Filter', () => {
    it('filters PRs by status', async () => {
      const user = userEvent.setup();
      const mockPRs = [
        {
          id: '1',
          title: 'Green PR',
          status: 'green',
          url: 'https://github.com/test/repo/pull/1',
        },
        {
          id: '2',
          title: 'Yellow PR',
          status: 'yellow',
          url: 'https://github.com/test/repo/pull/2',
        },
        {
          id: '3',
          title: 'Red PR',
          status: 'red',
          url: 'https://github.com/test/repo/pull/3',
        },
      ];

      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: mockPRs, total: 3 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText('Green PR')).toBeInTheDocument();
      });

      const filterSelect = screen.getByRole('combobox');
      await user.selectOptions(filterSelect, 'green');

      // After filtering, only green PR should be visible
      expect(screen.getByText('Green PR')).toBeInTheDocument();
      expect(screen.queryByText('Yellow PR')).not.toBeInTheDocument();
      expect(screen.queryByText('Red PR')).not.toBeInTheDocument();
    });
  });

  describe('Bulk Selection', () => {
    it('toggles individual PR selection', async () => {
      const user = userEvent.setup();
      const mockPRs = [
        {
          id: '1',
          number: 123,
          title: 'PR 1',
          status: 'green',
          isMergeable: true,
          hasConflicts: false,
          isDraft: false,
          url: 'https://github.com/test/repo/pull/123',
        },
      ];

      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: mockPRs, total: 1 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText('PR 1')).toBeInTheDocument();
      });

      const checkbox = screen.getByRole('button', { name: /square/i });
      await user.click(checkbox);

      // Should show merge button
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /merge selected/i })).toBeInTheDocument();
      });
    });

    it('selects all mergeable PRs', async () => {
      const user = userEvent.setup();
      const mockPRs = [
        {
          id: '1',
          title: 'PR 1',
          status: 'green',
          isMergeable: true,
          hasConflicts: false,
          isDraft: false,
          url: 'https://github.com/test/repo/pull/1',
        },
        {
          id: '2',
          title: 'PR 2',
          status: 'green',
          isMergeable: true,
          hasConflicts: false,
          isDraft: false,
          url: 'https://github.com/test/repo/pull/2',
        },
      ];

      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: mockPRs, total: 2 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText(/select all mergeable/i)).toBeInTheDocument();
      });

      const selectAllButton = screen.getByRole('button', { name: /select all mergeable/i });
      await user.click(selectAllButton);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /merge selected \(2\)/i })).toBeInTheDocument();
      });
    });
  });

  describe('Bulk Merge', () => {
    it('merges selected PRs successfully', async () => {
      const user = userEvent.setup();
      const mockPRs = [
        {
          id: '1',
          title: 'PR 1',
          status: 'green',
          isMergeable: true,
          hasConflicts: false,
          isDraft: false,
          url: 'https://github.com/test/repo/pull/1',
        },
      ];

      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: mockPRs, total: 1 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });
      mockedMergeApi.bulkMerge.mockResolvedValue({
        success: 1,
        failed: 0,
        results: [{ pullRequestId: '1', success: true }],
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText('PR 1')).toBeInTheDocument();
      });

      const checkbox = screen.getByRole('button', { name: /square/i });
      await user.click(checkbox);

      const mergeButton = await screen.findByRole('button', { name: /merge selected/i });
      await user.click(mergeButton);

      await waitFor(() => {
        expect(mockedMergeApi.bulkMerge).toHaveBeenCalledWith({
          pullRequestIds: ['1'],
          strategy: 'squash',
          deleteBranch: false,
        });
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Bulk merge complete',
        description: 'Successfully merged 1 PR(s)',
      });
    });

    it('shows error toast on merge failure', async () => {
      const user = userEvent.setup();
      const mockPRs = [
        {
          id: '1',
          title: 'PR 1',
          status: 'green',
          isMergeable: true,
          hasConflicts: false,
          isDraft: false,
          url: 'https://github.com/test/repo/pull/1',
        },
      ];

      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: mockPRs, total: 1 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });
      mockedMergeApi.bulkMerge.mockRejectedValue({
        response: { data: { error: 'Merge failed' } },
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText('PR 1')).toBeInTheDocument();
      });

      const checkbox = screen.getByRole('button', { name: /square/i });
      await user.click(checkbox);

      const mergeButton = await screen.findByRole('button', { name: /merge selected/i });
      await user.click(mergeButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          variant: 'destructive',
          title: 'Bulk merge failed',
          description: 'Merge failed',
        });
      });
    });
  });

  describe('Pagination', () => {
    it('shows load more button when more pages available', async () => {
      const mockPRs = [
        {
          id: '1',
          title: 'PR 1',
          url: 'https://github.com/test/repo/pull/1',
        },
      ];

      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: mockPRs, total: 50 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: true,
        fetchNextPage: vi.fn(),
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText(/load more prs/i)).toBeInTheDocument();
      });
    });

    it('fetches next page when load more is clicked', async () => {
      const user = userEvent.setup();
      const fetchNextPage = vi.fn();
      const mockPRs = [
        {
          id: '1',
          title: 'PR 1',
          url: 'https://github.com/test/repo/pull/1',
        },
      ];

      mockedUseInfinitePullRequests.mockReturnValue({
        data: { pages: [{ data: mockPRs, total: 50 }], pageParams: [] },
        isLoading: false,
        isFetchingNextPage: false,
        hasNextPage: true,
        fetchNextPage,
      } as UseInfiniteQueryResult<PaginatedResponse<PullRequestWithDetails>, Error>);
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderPRListView();

      await waitFor(() => {
        expect(screen.getByText(/load more prs/i)).toBeInTheDocument();
      });

      const loadMoreButton = screen.getByRole('button', { name: /load more prs/i });
      await user.click(loadMoreButton);

      expect(fetchNextPage).toHaveBeenCalled();
    });
  });
});
