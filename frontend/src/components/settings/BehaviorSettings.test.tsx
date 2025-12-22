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
    it('renders behavior settings card', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText('Behavior Settings')).toBeInTheDocument();
      });
    });

    it('displays skip review requirement toggle', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText(/skip review requirement/i)).toBeInTheDocument();
      });
    });

    it('displays default merge strategy selector', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText('Default Merge Strategy')).toBeInTheDocument();
      });
    });

    it('displays delete branches toggle', async () => {
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText(/delete branches by default/i)).toBeInTheDocument();
      });
    });
  });

  describe('Settings Updates', () => {
    it('toggles skip review requirement', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });
      mockedSettingsApi.updateBehavior.mockResolvedValue({});

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByRole('switch')).toBeInTheDocument();
      });

      const switches = screen.getAllByRole('switch');
      const skipReviewSwitch = switches[0];
      await user.click(skipReviewSwitch);

      await waitFor(() => {
        expect(mockedSettingsApi.updateBehavior).toHaveBeenCalledWith({
          skipReviewRequirement: true,
        });
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Settings updated',
        description: 'Your behavior settings have been saved.',
      });
    });

    it('changes merge strategy', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });
      mockedSettingsApi.updateBehavior.mockResolvedValue({});

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText('Squash and merge')).toBeInTheDocument();
      });

      const strategyButton = screen.getByRole('combobox');
      await user.click(strategyButton);

      const rebaseOption = await screen.findByText('Rebase and merge');
      await user.click(rebaseOption);

      await waitFor(() => {
        expect(mockedSettingsApi.updateBehavior).toHaveBeenCalledWith({
          defaultMergeStrategy: 'rebase',
        });
      });
    });

    it('toggles delete branches default', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });
      mockedSettingsApi.updateBehavior.mockResolvedValue({});

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getAllByRole('switch')).toHaveLength(2);
      });

      const switches = screen.getAllByRole('switch');
      const deleteBranchSwitch = switches[1];
      await user.click(deleteBranchSwitch);

      await waitFor(() => {
        expect(mockedSettingsApi.updateBehavior).toHaveBeenCalledWith({
          deleteBranchesDefault: true,
        });
      });
    });

    it('shows error toast on update failure', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getBehavior.mockResolvedValue({
        skipReviewRequirement: false,
        defaultMergeStrategy: 'squash',
        deleteBranchesDefault: false,
      });
      mockedSettingsApi.updateBehavior.mockRejectedValue({
        response: { data: { error: 'Update failed' } },
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByRole('switch')).toBeInTheDocument();
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
        skipReviewRequirement: true,
        defaultMergeStrategy: 'rebase',
        deleteBranchesDefault: true,
      });

      renderBehaviorSettings();

      await waitFor(() => {
        expect(screen.getByText('Rebase and merge')).toBeInTheDocument();
      });

      const switches = screen.getAllByRole('switch');
      expect(switches[0]).toBeChecked();
      expect(switches[1]).toBeChecked();
    });
  });
});
