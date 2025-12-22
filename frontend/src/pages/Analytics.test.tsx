import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Analytics from './Analytics';

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
      renderAnalytics();

      expect(screen.getByText('Analytics')).toBeInTheDocument();
    });

    it('displays coming soon message', () => {
      renderAnalytics();

      expect(screen.getByText(/coming soon/i)).toBeInTheDocument();
    });

    it('shows placeholder content', async () => {
      renderAnalytics();

      await waitFor(() => {
        expect(screen.getByText(/analytics dashboard/i)).toBeInTheDocument();
      });
    });
  });

  describe('Layout', () => {
    it('has proper spacing', () => {
      const { container } = renderAnalytics();

      const mainContainer = container.querySelector('.space-y-6');
      expect(mainContainer).toBeInTheDocument();
    });
  });
});
