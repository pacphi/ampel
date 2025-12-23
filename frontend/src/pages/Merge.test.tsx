import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Merge from './Merge';

vi.mock('@/api/pullRequests', () => ({
  pullRequestsApi: {
    list: vi.fn(),
  },
}));

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

vi.mock('@/components/ui/use-toast', () => ({
  useToast: vi.fn(() => ({ toast: vi.fn(), dismiss: vi.fn(), toasts: [] })),
}));

import { pullRequestsApi } from '@/api/pullRequests';
import { mergeApi } from '@/api/merge';
import { settingsApi } from '@/api/settings';
import { useToast } from '@/components/ui/use-toast';

const mockedPullRequestsApi = vi.mocked(pullRequestsApi);
const mockedMergeApi = vi.mocked(mergeApi);
const mockedSettingsApi = vi.mocked(settingsApi);
const mockedUseToast = vi.mocked(useToast);

function renderMerge() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <Merge />
    </QueryClientProvider>
  );
}

describe('Merge', () => {
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
      mockedPullRequestsApi.list.mockReturnValue(new Promise(() => {}));
      mockedSettingsApi.getBehavior.mockReturnValue(new Promise(() => {}));

      const { container } = renderMerge();

      const spinner = container.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });
  });

  describe('Page Header', () => {
    it('renders page title and description', async () => {
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('Bulk Merge')).toBeInTheDocument();
      });

      expect(
        screen.getByText('Select and merge multiple pull requests at once')
      ).toBeInTheDocument();
    });
  });

  describe('Merge Options', () => {
    it('renders merge strategy selector', async () => {
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('Merge Strategy')).toBeInTheDocument();
      });

      expect(screen.getByText('Squash and merge')).toBeInTheDocument();
    });

    it('renders delete branch toggle', async () => {
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('Delete branches after merge')).toBeInTheDocument();
      });
    });

    it('applies settings defaults', async () => {
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'rebase',
        deleteBranchesDefault: true,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('Rebase and merge')).toBeInTheDocument();
      });

      const deleteToggle = screen.getByRole('switch');
      expect(deleteToggle).toBeChecked();
    });
  });

  describe('Mergeable PRs', () => {
    it('shows empty state when no mergeable PRs', async () => {
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            number: 123,
            title: 'Non-mergeable PR',
            status: 'red',
            isDraft: false,
            hasConflicts: true,
            url: 'https://github.com/test/repo/pull/123',
          },
        ],
        total: 1,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('No PRs are ready to merge. PRs must have:')).toBeInTheDocument();
      });

      expect(screen.getByText('All CI checks passing')).toBeInTheDocument();
      expect(screen.getByText('Required approvals')).toBeInTheDocument();
      expect(screen.getByText('No merge conflicts')).toBeInTheDocument();
    });

    it('displays mergeable PRs grouped by repository', async () => {
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            number: 123,
            title: 'PR 1',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            repositoryOwner: 'acme',
            repositoryName: 'app',
            sourceBranch: 'feature',
            targetBranch: 'main',
            url: 'https://github.com/acme/app/pull/123',
          },
          {
            id: '2',
            number: 124,
            title: 'PR 2',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            repositoryOwner: 'acme',
            repositoryName: 'app',
            sourceBranch: 'bugfix',
            targetBranch: 'main',
            url: 'https://github.com/acme/app/pull/124',
          },
        ],
        total: 2,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('acme/app')).toBeInTheDocument();
      });

      expect(screen.getByText('PR 1')).toBeInTheDocument();
      expect(screen.getByText('PR 2')).toBeInTheDocument();
      expect(screen.getByText('2 PRs')).toBeInTheDocument();
    });

    it('allows yellow PRs when skipReviewRequirement is enabled', async () => {
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            number: 123,
            title: 'Yellow PR without CI issues',
            status: 'yellow',
            isDraft: false,
            hasConflicts: false,
            ciChecks: [],
            reviews: [],
            repositoryOwner: 'test',
            repositoryName: 'repo',
            sourceBranch: 'feature',
            targetBranch: 'main',
            url: 'https://github.com/test/repo/pull/123',
          },
        ],
        total: 1,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: true,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('Ready to Merge')).toBeInTheDocument();
      });

      expect(screen.getByText('Yellow PR without CI issues')).toBeInTheDocument();
    });
  });

  describe('PR Selection', () => {
    it('selects individual PR when clicked', async () => {
      const user = userEvent.setup();
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            number: 123,
            title: 'Test PR',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            repositoryOwner: 'test',
            repositoryName: 'repo',
            sourceBranch: 'feature',
            targetBranch: 'main',
            url: 'https://github.com/test/repo/pull/123',
          },
        ],
        total: 1,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('Test PR')).toBeInTheDocument();
      });

      // Find the PR row by looking for the container with the checkbox
      // The title is nested several divs deep, so we need to find the row container
      const prTitle = screen.getByText('Test PR');
      // Go up until we find the div with the checkbox input
      let prRow = prTitle.closest('.flex.items-center.gap-4.p-3');
      expect(prRow).toBeInTheDocument();

      const checkbox = prRow?.querySelector('input[type="checkbox"]');
      expect(checkbox).toBeInTheDocument();

      if (checkbox) {
        await user.click(checkbox);
        expect(checkbox).toBeChecked();
      }
    });

    it('selects all PRs when select all is clicked', async () => {
      const user = userEvent.setup();
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            number: 123,
            title: 'PR 1',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            repositoryOwner: 'test',
            repositoryName: 'repo',
            sourceBranch: 'f1',
            targetBranch: 'main',
            url: 'https://github.com/test/repo/pull/123',
          },
          {
            id: '2',
            number: 124,
            title: 'PR 2',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            repositoryOwner: 'test',
            repositoryName: 'repo',
            sourceBranch: 'f2',
            targetBranch: 'main',
            url: 'https://github.com/test/repo/pull/124',
          },
        ],
        total: 2,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('PR 1')).toBeInTheDocument();
      });

      const selectAllButton = screen.getByRole('button', { name: /select all/i });
      await user.click(selectAllButton);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /merge 2 pr/i })).toBeInTheDocument();
      });
    });
  });

  describe('Bulk Merge Execution', () => {
    it('merges selected PRs successfully', async () => {
      const user = userEvent.setup();
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            number: 123,
            title: 'Test PR',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            repositoryOwner: 'test',
            repositoryName: 'repo',
            sourceBranch: 'feature',
            targetBranch: 'main',
            url: 'https://github.com/test/repo/pull/123',
          },
        ],
        total: 1,
        page: 1,
        pageSize: 100,
      });
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

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('Test PR')).toBeInTheDocument();
      });

      const selectAllButton = screen.getByRole('button', { name: /select all/i });
      await user.click(selectAllButton);

      const mergeButton = await screen.findByRole('button', { name: /merge 1 pr$/i });
      await user.click(mergeButton);

      await waitFor(() => {
        expect(mockedMergeApi.bulkMerge).toHaveBeenCalledWith({
          pullRequestIds: ['1'],
          strategy: 'squash',
          deleteBranch: false,
        });
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Merge successful',
        description: 'Successfully merged 1 PR(s)',
      });
    });

    it('shows error toast on merge failure', async () => {
      const user = userEvent.setup();
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            number: 123,
            title: 'Test PR',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            repositoryOwner: 'test',
            repositoryName: 'repo',
            sourceBranch: 'feature',
            targetBranch: 'main',
            url: 'https://github.com/test/repo/pull/123',
          },
        ],
        total: 1,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });
      mockedMergeApi.bulkMerge.mockRejectedValue({
        response: { data: { error: 'Merge conflict detected' } },
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('Test PR')).toBeInTheDocument();
      });

      const selectAllButton = screen.getByRole('button', { name: /select all/i });
      await user.click(selectAllButton);

      const mergeButton = await screen.findByRole('button', { name: /merge 1 pr$/i });
      await user.click(mergeButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          variant: 'destructive',
          title: 'Merge failed',
          description: 'Merge conflict detected',
        });
      });
    });
  });

  describe('Blockers Display', () => {
    it('shows blockers for non-mergeable PRs', async () => {
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            number: 123,
            title: 'Blocked PR',
            status: 'red',
            isDraft: true,
            hasConflicts: true,
            ciChecks: [
              {
                name: 'CI',
                status: 'completed',
                conclusion: 'failure',
              },
            ],
            reviews: [{ state: 'changes_requested', author: 'reviewer' }],
            repositoryOwner: 'test',
            repositoryName: 'repo',
            sourceBranch: 'feature',
            targetBranch: 'main',
            url: 'https://github.com/test/repo/pull/123',
          },
        ],
        total: 1,
        page: 1,
        pageSize: 100,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderMerge();

      await waitFor(() => {
        expect(screen.getByText('Blocked PR')).toBeInTheDocument();
      });

      expect(screen.getByText('Draft')).toBeInTheDocument();
      expect(screen.getByText('Conflicts')).toBeInTheDocument();
      expect(screen.getByText('CI failed')).toBeInTheDocument();
      expect(screen.getByText('Changes requested')).toBeInTheDocument();
    });
  });
});
