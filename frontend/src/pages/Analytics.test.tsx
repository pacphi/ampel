import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Analytics from './Analytics';

vi.mock('@/api/analytics', () => ({
  analyticsApi: {
    getSummary: vi.fn(),
    getHealthOverview: vi.fn(),
  },
}));

import { analyticsApi } from '@/api/analytics';

const mockedAnalyticsApi = vi.mocked(analyticsApi);

function renderAnalytics() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <Analytics />
    </QueryClientProvider>
  );
}

describe('Analytics', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Page Structure', () => {
    it('renders analytics page title', () => {
      mockedAnalyticsApi.getSummary.mockReturnValue(new Promise(() => {}));
      mockedAnalyticsApi.getHealthOverview.mockReturnValue(new Promise(() => {}));

      renderAnalytics();

      expect(screen.getByText('Analytics')).toBeInTheDocument();
    });

    it('displays summary cards with PRs merged', async () => {
      mockedAnalyticsApi.getSummary.mockResolvedValue({
        totalPrsMerged: 42,
        avgTimeToMergeHours: 4.5,
        avgReviewTimeHours: 2.0,
        botPrPercentage: 30,
      });
      mockedAnalyticsApi.getHealthOverview.mockResolvedValue([]);

      renderAnalytics();

      // Wait for data to load - check for actual value instead of just header
      await waitFor(() => {
        expect(screen.getByText('42')).toBeInTheDocument();
      });

      expect(screen.getByText('PRs Merged (30d)')).toBeInTheDocument();
    });

    it('shows repository health scores section', async () => {
      mockedAnalyticsApi.getSummary.mockResolvedValue({
        totalPrsMerged: 10,
        avgTimeToMergeHours: 2.0,
        avgReviewTimeHours: 1.0,
        botPrPercentage: 20,
      });
      mockedAnalyticsApi.getHealthOverview.mockResolvedValue([]);

      renderAnalytics();

      await waitFor(() => {
        expect(screen.getByText('Repository Health Scores')).toBeInTheDocument();
      });
    });
  });

  describe('Loading State', () => {
    it('shows loading spinner while fetching data', () => {
      mockedAnalyticsApi.getSummary.mockReturnValue(new Promise(() => {}));
      mockedAnalyticsApi.getHealthOverview.mockReturnValue(new Promise(() => {}));

      renderAnalytics();

      // Summary cards show "-" while loading
      expect(screen.getAllByText('-').length).toBeGreaterThan(0);
    });
  });

  describe('Empty State', () => {
    it('shows no data message when health data is empty', async () => {
      mockedAnalyticsApi.getSummary.mockResolvedValue({
        totalPrsMerged: 0,
        avgTimeToMergeHours: 0,
        avgReviewTimeHours: 0,
        botPrPercentage: 0,
      });
      mockedAnalyticsApi.getHealthOverview.mockResolvedValue([]);

      renderAnalytics();

      await waitFor(() => {
        expect(screen.getByText('No health data available yet')).toBeInTheDocument();
      });
    });
  });

  describe('Layout', () => {
    it('has proper spacing', () => {
      mockedAnalyticsApi.getSummary.mockReturnValue(new Promise(() => {}));
      mockedAnalyticsApi.getHealthOverview.mockReturnValue(new Promise(() => {}));

      const { container } = renderAnalytics();

      const mainContainer = container.querySelector('.space-y-6');
      expect(mainContainer).toBeInTheDocument();
    });
  });

  describe('Health Data Display', () => {
    it('displays repository health cards', async () => {
      mockedAnalyticsApi.getSummary.mockResolvedValue({
        totalPrsMerged: 50,
        avgTimeToMergeHours: 6.0,
        avgReviewTimeHours: 3.0,
        botPrPercentage: 25,
      });
      mockedAnalyticsApi.getHealthOverview.mockResolvedValue([
        {
          repositoryId: '1',
          repositoryName: 'owner/repo1',
          currentScore: 85,
          trend: 'up',
          metrics: {
            avgTimeToMergeHours: 4.0,
            prThroughputPerWeek: 12,
            avgReviewTimeHours: 2.0,
            stalePrCount: 1,
          },
        },
      ]);

      renderAnalytics();

      await waitFor(() => {
        expect(screen.getByText('owner/repo1')).toBeInTheDocument();
      });

      expect(screen.getByText('85')).toBeInTheDocument();
    });
  });
});
