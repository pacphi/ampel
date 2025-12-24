import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BehaviorSettings } from './BehaviorSettings';

vi.mock('@/api/settings', () => ({
  settingsApi: {
    getBehavior: vi.fn(),
    updateBehavior: vi.fn(),
  },
}));

vi.mock('@/components/ui/use-toast', () => ({
  useToast: vi.fn(() => ({ toast: vi.fn(), dismiss: vi.fn(), toasts: [] })),
}));

import { settingsApi } from '@/api/settings';
import { useToast } from '@/components/ui/use-toast';

const mockedSettingsApi = vi.mocked(settingsApi);
const mockedUseToast = vi.mocked(useToast);

const defaultBehaviorSettings = {
  mergeDelaySeconds: 5,
  requireApproval: false,
  deleteBranchesDefault: false,
  defaultMergeStrategy: 'squash' as const,
  skipReviewRequirement: false,
};

function renderBehaviorSettings() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <BehaviorSettings />
    </QueryClientProvider>
  );
}

describe('BehaviorSettings', () => {
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
    it('shows loading spinner while fetching settings', () => {
      mockedSettingsApi.getBehavior.mockReturnValue(new Promise(() => {}));

      const { container } = renderBehaviorSettings();

      const spinner = container.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });
  });

  describe('Settings Display', () => {
    it('renders merge behavior settings card', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText('Merge Behavior')).toBeInTheDocument();
      });
    });

    it('displays default merge strategy selector', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText('Default Merge Strategy')).toBeInTheDocument();
      });
    });

    it('displays delete branches toggle', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText(/delete branches after merge/i)).toBeInTheDocument();
      });
    });

    it('displays require approval toggle', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText(/require approval before merge/i)).toBeInTheDocument();
      });
    });

    it('displays allow merge without reviews toggle', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText(/allow merge without reviews/i)).toBeInTheDocument();
      });
    });
  });

  describe('Settings Updates', () => {
    it('toggles delete branches setting', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);
      mockedSettingsApi.updateBehavior.mockResolvedValue({
        ...defaultBehaviorSettings,
        deleteBranchesDefault: true,
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getAllByRole('switch').length).toBe(3);
      });

      const switches = screen.getAllByRole('switch');
      // First switch is deleteBranchesDefault
      await user.click(switches[0]);

      await waitFor(() => {
        expect(mockedSettingsApi.updateBehavior).toHaveBeenCalledWith({
          deleteBranchesDefault: true,
        });
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Settings updated',
        description: 'Your behavior settings have been saved.',
      });
    });

    it('changes merge strategy', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);
      mockedSettingsApi.updateBehavior.mockResolvedValue({
        ...defaultBehaviorSettings,
        defaultMergeStrategy: 'rebase',
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText('Squash and merge')).toBeInTheDocument();
      });

      const strategyButton = screen.getByRole('combobox');
      await user.click(strategyButton);

      const rebaseOption = await screen.findByRole('option', { name: /rebase and merge/i });
      await user.click(rebaseOption);

      await waitFor(() => {
        expect(mockedSettingsApi.updateBehavior).toHaveBeenCalledWith({
          defaultMergeStrategy: 'rebase',
        });
      });
    });

    it('toggles require approval setting', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);
      mockedSettingsApi.updateBehavior.mockResolvedValue({
        ...defaultBehaviorSettings,
        requireApproval: true,
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getAllByRole('switch').length).toBe(3);
      });

      const switches = screen.getAllByRole('switch');
      // Second switch is requireApproval
      await user.click(switches[1]);

      await waitFor(() => {
        expect(mockedSettingsApi.updateBehavior).toHaveBeenCalledWith({
          requireApproval: true,
        });
      });
    });

    it('toggles skip review requirement setting', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);
      mockedSettingsApi.updateBehavior.mockResolvedValue({
        ...defaultBehaviorSettings,
        skipReviewRequirement: true,
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getAllByRole('switch').length).toBe(3);
      });

      const switches = screen.getAllByRole('switch');
      // Third switch is skipReviewRequirement
      await user.click(switches[2]);

      await waitFor(() => {
        expect(mockedSettingsApi.updateBehavior).toHaveBeenCalledWith({
          skipReviewRequirement: true,
        });
      });
    });

    it('shows error toast on update failure', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getBehavior.mockResolvedValue(defaultBehaviorSettings);
      mockedSettingsApi.updateBehavior.mockRejectedValue({
        response: { data: { error: 'Update failed' } },
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getAllByRole('switch').length).toBe(3);
      });

      const switches = screen.getAllByRole('switch');
      await user.click(switches[0]);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          variant: 'destructive',
          title: 'Failed to update settings',
          description: 'Update failed',
        });
      });
    });
  });

  describe('Setting Values', () => {
    it('reflects current settings state', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue({
        ...defaultBehaviorSettings,
        deleteBranchesDefault: true,
        requireApproval: true,
        skipReviewRequirement: true,
        defaultMergeStrategy: 'rebase',
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText('Rebase and merge')).toBeInTheDocument();
      });

      const switches = screen.getAllByRole('switch');
      expect(switches[0]).toBeChecked(); // deleteBranchesDefault
      expect(switches[1]).toBeChecked(); // requireApproval
      expect(switches[2]).toBeChecked(); // skipReviewRequirement
    });
  });
});
