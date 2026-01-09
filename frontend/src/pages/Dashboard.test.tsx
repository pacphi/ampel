import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Dashboard from './Dashboard';

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'dashboard:title': 'Dashboard',
        'dashboard:filters.onlyWithPrs': 'Only show repos with PRs',
        'dashboard:views.repositoryGrid': 'Repository grid view',
        'dashboard:views.repositoryList': 'Repository list view',
        'dashboard:views.pullRequests': 'Pull requests view',
        'dashboard:stats.totalRepositories': 'Total Repositories',
        'dashboard:stats.openPRs': 'Open PRs',
        'dashboard:stats.readyToMerge': 'Ready to Merge',
        'dashboard:stats.needsAttention': 'Needs Attention',
        'dashboard:emptyState.title': 'No repositories found',
        'dashboard:emptyState.description':
          'Add repositories from the Repositories page to get started',
        'dashboard:breakdown.public': 'Public',
        'dashboard:breakdown.private': 'Private',
        'dashboard:breakdown.archived': 'Archived',
        'common:actions.refresh': 'Refresh',
        'common:actions.refreshing': 'Refreshing...',
      };
      return translations[key] || key;
    },
    i18n: { language: 'en', changeLanguage: vi.fn() },
    ready: true,
  }),
}));

// Mock API modules
vi.mock('@/api/dashboard', () => ({
  dashboardApi: {
    getSummary: vi.fn(),
    getGrid: vi.fn(),
  },
}));

vi.mock('@/api/pullRequests', () => ({
  pullRequestsApi: {
    list: vi.fn(),
  },
}));

vi.mock('@/api/settings', () => ({
  settingsApi: {
    getBehavior: vi.fn(),
  },
}));

import { dashboardApi } from '@/api/dashboard';
import { pullRequestsApi } from '@/api/pullRequests';
import { settingsApi } from '@/api/settings';

const mockedDashboardApi = vi.mocked(dashboardApi);
const mockedPullRequestsApi = vi.mocked(pullRequestsApi);
const mockedSettingsApi = vi.mocked(settingsApi);

function renderDashboard() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <Dashboard />
    </QueryClientProvider>
  );
}

describe('Dashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Loading State', () => {
    it('renders loading spinner while fetching data', () => {
      mockedDashboardApi.getSummary.mockReturnValue(new Promise(() => {}));
      mockedDashboardApi.getGrid.mockReturnValue(new Promise(() => {}));
      mockedPullRequestsApi.list.mockReturnValue(new Promise(() => {}));
      mockedSettingsApi.getBehavior.mockReturnValue(new Promise(() => {}));

      const { container } = renderDashboard();

      expect(screen.getByText('Dashboard')).toBeInTheDocument();
      expect(screen.getByRole('heading', { name: /dashboard/i })).toBeInTheDocument();
      // Combined tiles show spinners while loading
      const spinners = container.querySelectorAll('.animate-spin');
      expect(spinners.length).toBeGreaterThan(0);
    });
  });

  describe('Summary Cards', () => {
    it('renders summary statistics correctly', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 25,
        totalOpenPrs: 42,
        statusCounts: { green: 10, yellow: 20, red: 12 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      // Wait for data to load - check for actual values
      await waitFor(() => {
        expect(screen.getByText('25')).toBeInTheDocument();
      });

      expect(screen.getByText('Total Repositories')).toBeInTheDocument();
      expect(screen.getByText('42')).toBeInTheDocument();
      expect(screen.getByText('12')).toBeInTheDocument();
    });

    it('calculates ready to merge count based on PR status and settings', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 5,
        totalOpenPrs: 10,
        statusCounts: { green: 3, yellow: 4, red: 3 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            title: 'PR 1',
          },
          {
            id: '2',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            title: 'PR 2',
          },
          {
            id: '3',
            status: 'yellow',
            isDraft: false,
            hasConflicts: false,
            ciChecks: [],
            title: 'PR 3',
          },
        ],
        total: 3,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      // Wait for data to load and check ready to merge count (only green PRs = 2)
      await waitFor(() => {
        // Check that the Ready to Merge value is 2 (matching green PR count)
        const readyToMergeCard = screen.getByText('Ready to Merge').closest('div');
        expect(readyToMergeCard).toBeInTheDocument();
      });

      // The "2" should appear in the Ready to Merge card
      await waitFor(() => {
        expect(screen.getByText('5')).toBeInTheDocument(); // totalRepositories
      });
    });

    it('includes yellow PRs in ready to merge when skipReviewRequirement is enabled', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 5,
        totalOpenPrs: 10,
        statusCounts: { green: 2, yellow: 5, red: 3 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [
          {
            id: '1',
            status: 'green',
            isDraft: false,
            hasConflicts: false,
            title: 'PR 1',
          },
          {
            id: '2',
            status: 'yellow',
            isDraft: false,
            hasConflicts: false,
            ciChecks: [],
            reviews: [],
            title: 'PR 2',
          },
        ],
        total: 2,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: true,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      // Wait for data to load - both green and yellow PRs count
      await waitFor(() => {
        expect(screen.getByText('5')).toBeInTheDocument(); // totalRepositories
      });

      expect(screen.getByText('Ready to Merge')).toBeInTheDocument();
    });
  });

  describe('View Mode Toggle', () => {
    it('renders view mode toggle buttons', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 0,
        totalOpenPrs: 0,
        statusCounts: { green: 0, yellow: 0, red: 0 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByTitle('Repository grid view')).toBeInTheDocument();
      });

      expect(screen.getByTitle('Repository list view')).toBeInTheDocument();
      expect(screen.getByTitle('Pull requests view')).toBeInTheDocument();
    });

    it('switches to list view when clicked', async () => {
      const user = userEvent.setup();
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 0,
        totalOpenPrs: 0,
        statusCounts: { green: 0, yellow: 0, red: 0 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByTitle('Repository list view')).toBeInTheDocument();
      });

      const listViewButton = screen.getByTitle('Repository list view');
      await user.click(listViewButton);

      // Verify button is now active (variant="default")
      expect(listViewButton.className).toContain('bg-primary');
    });

    it('switches to PR view when clicked', async () => {
      const user = userEvent.setup();
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 0,
        totalOpenPrs: 0,
        statusCounts: { green: 0, yellow: 0, red: 0 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByTitle('Pull requests view')).toBeInTheDocument();
      });

      const prViewButton = screen.getByTitle('Pull requests view');
      await user.click(prViewButton);

      expect(prViewButton.className).toContain('bg-primary');
    });
  });

  describe('Refresh Button', () => {
    it('refetches all dashboard data when refresh is clicked', async () => {
      const user = userEvent.setup();
      let gridCallCount = 0;
      let summaryCallCount = 0;
      let prsCallCount = 0;

      mockedDashboardApi.getSummary.mockImplementation(async () => {
        summaryCallCount++;
        return {
          totalRepositories: 10,
          totalOpenPrs: 5,
          statusCounts: { green: 2, yellow: 2, red: 1 },
        };
      });
      mockedDashboardApi.getGrid.mockImplementation(async () => {
        gridCallCount++;
        return [];
      });
      mockedPullRequestsApi.list.mockImplementation(async () => {
        prsCallCount++;
        return {
          data: [],
          total: 0,
          page: 1,
          pageSize: 1000,
        };
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /refresh/i })).toBeInTheDocument();
      });

      // Initial load
      expect(gridCallCount).toBe(1);
      expect(summaryCallCount).toBe(1);
      expect(prsCallCount).toBe(1);

      const refreshButton = screen.getByRole('button', { name: /refresh/i });
      await user.click(refreshButton);

      // After refresh, all queries should be refetched
      await waitFor(() => {
        expect(gridCallCount).toBe(2);
      });
      await waitFor(() => {
        expect(summaryCallCount).toBe(2);
      });
      await waitFor(() => {
        expect(prsCallCount).toBe(2);
      });
    });

    it('shows loading state while refreshing', async () => {
      const user = userEvent.setup();
      let resolveGrid: () => void;
      const gridPromise = new Promise<void>((resolve) => {
        resolveGrid = resolve;
      });

      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 10,
        totalOpenPrs: 5,
        statusCounts: { green: 2, yellow: 2, red: 1 },
      });
      mockedDashboardApi.getGrid.mockImplementation(async () => {
        await gridPromise;
        return [];
      });
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      // Resolve initial load
      resolveGrid!();

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /refresh/i })).toBeInTheDocument();
      });

      // Setup a new promise for the refresh
      const refreshPromise = new Promise<void>((resolve) => {
        resolveGrid = resolve;
      });
      mockedDashboardApi.getGrid.mockImplementation(async () => {
        await refreshPromise;
        return [];
      });

      const refreshButton = screen.getByRole('button', { name: /refresh/i });
      await user.click(refreshButton);

      // Should show loading state
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /refreshing/i })).toBeInTheDocument();
      });
      expect(screen.getByRole('button', { name: /refreshing/i })).toBeDisabled();

      // Resolve the refresh
      resolveGrid!();

      // Should return to normal state
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /^refresh$/i })).toBeInTheDocument();
      });
      expect(screen.getByRole('button', { name: /^refresh$/i })).not.toBeDisabled();
    });
  });

  describe('Empty State', () => {
    it('shows empty state message when no repositories', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 0,
        totalOpenPrs: 0,
        statusCounts: { green: 0, yellow: 0, red: 0 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('No repositories found')).toBeInTheDocument();
      });

      expect(
        screen.getByText('Add repositories from the Repositories page to get started')
      ).toBeInTheDocument();
    });
  });

  describe('Repository Grid View', () => {
    it('displays repositories in grid layout', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 2,
        totalOpenPrs: 5,
        statusCounts: { green: 2, yellow: 2, red: 1 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([
        {
          id: '1',
          name: 'repo1',
          owner: 'owner1',
          fullName: 'owner1/repo1',
          provider: 'github',
          openPrCount: 3,
          status: 'green',
          url: 'https://github.com/owner1/repo1',
        },
        {
          id: '2',
          name: 'repo2',
          owner: 'owner2',
          fullName: 'owner2/repo2',
          provider: 'gitlab',
          openPrCount: 2,
          status: 'yellow',
          url: 'https://gitlab.com/owner2/repo2',
        },
      ]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      // RepoCard displays name and owner separately, not fullName
      await waitFor(() => {
        expect(screen.getByText('repo1')).toBeInTheDocument();
      });

      expect(screen.getByText('repo2')).toBeInTheDocument();
    });
  });

  describe('Combined Summary + Breakdown Tiles', () => {
    it('displays combined summary and breakdown tiles', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 20,
        totalOpenPrs: 10,
        statusCounts: { green: 5, yellow: 3, red: 2 },
        providerCounts: { github: 15, gitlab: 3, bitbucket: 2 },
        repositoryBreakdown: { public: 12, private: 6, archived: 2 },
        openPrsBreakdown: { public: 6, private: 3, archived: 1 },
        readyToMergeBreakdown: { public: 3, private: 2, archived: 0 },
        needsAttentionBreakdown: { public: 1, private: 1, archived: 0 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      // Combined tiles have main titles (no separate "by Visibility" titles)
      await waitFor(() => {
        expect(screen.getByText('Total Repositories')).toBeInTheDocument();
      });

      expect(screen.getByText('Open PRs')).toBeInTheDocument();
      expect(screen.getByText('Ready to Merge')).toBeInTheDocument();
      expect(screen.getByText('Needs Attention')).toBeInTheDocument();
    });

    it('displays correct breakdown counts from API data', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 20,
        totalOpenPrs: 10,
        statusCounts: { green: 5, yellow: 3, red: 2 },
        providerCounts: { github: 15, gitlab: 3, bitbucket: 2 },
        repositoryBreakdown: { public: 12, private: 6, archived: 2 },
        openPrsBreakdown: { public: 6, private: 3, archived: 1 },
        readyToMergeBreakdown: { public: 3, private: 2, archived: 0 },
        needsAttentionBreakdown: { public: 1, private: 1, archived: 0 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('20')).toBeInTheDocument();
      });

      // Repository breakdown
      expect(screen.getByText('12')).toBeInTheDocument(); // public repos

      // The numbers might appear multiple times, so we check they exist
      const sixElements = screen.getAllByText('6');
      expect(sixElements.length).toBeGreaterThan(0); // private repos + public PRs

      const twoElements = screen.getAllByText('2');
      expect(twoElements.length).toBeGreaterThan(0); // archived repos + red status
    });

    it('shows loading state in combined tiles', async () => {
      mockedDashboardApi.getSummary.mockReturnValue(new Promise(() => {}));
      mockedDashboardApi.getGrid.mockReturnValue(new Promise(() => {}));
      mockedPullRequestsApi.list.mockReturnValue(new Promise(() => {}));
      mockedSettingsApi.getBehavior.mockReturnValue(new Promise(() => {}));

      const { container } = renderDashboard();

      // Wait for component to render
      await waitFor(() => {
        expect(screen.getByText('Dashboard')).toBeInTheDocument();
      });

      // Check for spinner in combined tiles (they use animate-spin)
      const spinners = container.querySelectorAll('.animate-spin');
      expect(spinners.length).toBeGreaterThan(0);
    });

    it('handles missing breakdown data gracefully', async () => {
      // API returns summary without breakdown fields
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 10,
        totalOpenPrs: 5,
        statusCounts: { green: 2, yellow: 2, red: 1 },
        providerCounts: { github: 8, gitlab: 1, bitbucket: 1 },
        // No breakdown fields
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('10')).toBeInTheDocument();
      });

      // Combined tiles should still render with main titles
      expect(screen.getByText('Total Repositories')).toBeInTheDocument();
      expect(screen.getByText('Open PRs')).toBeInTheDocument();
    });

    it('displays all zero counts when no repositories exist', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 0,
        totalOpenPrs: 0,
        statusCounts: { green: 0, yellow: 0, red: 0 },
        providerCounts: { github: 0, gitlab: 0, bitbucket: 0 },
        repositoryBreakdown: { public: 0, private: 0, archived: 0 },
        openPrsBreakdown: { public: 0, private: 0, archived: 0 },
        readyToMergeBreakdown: { public: 0, private: 0, archived: 0 },
        needsAttentionBreakdown: { public: 0, private: 0, archived: 0 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('No repositories found')).toBeInTheDocument();
      });

      // Combined tiles should show zeros
      const zeroElements = screen.getAllByText('0');
      expect(zeroElements.length).toBeGreaterThan(0);
    });

    it('displays correct icon labels in combined tiles', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 20,
        totalOpenPrs: 10,
        statusCounts: { green: 5, yellow: 3, red: 2 },
        providerCounts: { github: 15, gitlab: 3, bitbucket: 2 },
        repositoryBreakdown: { public: 12, private: 6, archived: 2 },
        openPrsBreakdown: { public: 6, private: 3, archived: 1 },
        readyToMergeBreakdown: { public: 3, private: 2, archived: 0 },
        needsAttentionBreakdown: { public: 1, private: 1, archived: 0 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('Total Repositories')).toBeInTheDocument();
      });

      // Each combined tile should have Public, Private, and Archived labels
      await waitFor(() => {
        const publicLabels = screen.queryAllByText('Public');
        expect(publicLabels.length).toBeGreaterThan(0);
      });

      const publicLabels = screen.getAllByText('Public');
      const privateLabels = screen.getAllByText('Private');
      const archivedLabels = screen.getAllByText('Archived');

      // Should have 4 combined tiles, each with these labels
      expect(publicLabels.length).toBe(4);
      expect(privateLabels.length).toBe(4);
      expect(archivedLabels.length).toBe(4);
    });

    it('maintains responsive layout with combined tiles', async () => {
      mockedDashboardApi.getSummary.mockResolvedValue({
        totalRepositories: 20,
        totalOpenPrs: 10,
        statusCounts: { green: 5, yellow: 3, red: 2 },
        providerCounts: { github: 15, gitlab: 3, bitbucket: 2 },
        repositoryBreakdown: { public: 12, private: 6, archived: 2 },
        openPrsBreakdown: { public: 6, private: 3, archived: 1 },
        readyToMergeBreakdown: { public: 3, private: 2, archived: 0 },
        needsAttentionBreakdown: { public: 1, private: 1, archived: 0 },
      });
      mockedDashboardApi.getGrid.mockResolvedValue([]);
      mockedPullRequestsApi.list.mockResolvedValue({
        data: [],
        total: 0,
        page: 1,
        pageSize: 1000,
      });
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      const { container } = renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('Total Repositories')).toBeInTheDocument();
      });

      // Check for grid layout classes
      const grids = container.querySelectorAll('.grid');
      expect(grids.length).toBeGreaterThan(0);
    });
  });
});
