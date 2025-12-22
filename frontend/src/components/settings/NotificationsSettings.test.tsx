import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { NotificationsSettings } from './NotificationsSettings';

vi.mock('@/api/settings', () => ({
  settingsApi: {
    getNotifications: vi.fn(),
    updateNotifications: vi.fn(),
  },
}));

vi.mock('@/components/ui/use-toast', () => ({
  useToast: vi.fn(() => ({ toast: vi.fn(), dismiss: vi.fn(), toasts: [] })),
}));

import { settingsApi } from '@/api/settings';
import { useToast } from '@/components/ui/use-toast';

const mockedSettingsApi = vi.mocked(settingsApi);
const mockedUseToast = vi.mocked(useToast);

function renderNotificationsSettings() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <NotificationsSettings />
    </QueryClientProvider>
  );
}

describe('NotificationsSettings', () => {
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
      mockedSettingsApi.getNotifications.mockReturnValue(new Promise(() => {}));

      const { container } = renderNotificationsSettings();

      const spinner = container.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });
  });

  describe('Settings Display', () => {
    it('renders notifications settings card', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue({
        emailOnMerge: false,
        emailOnConflict: false,
        emailOnReview: false,
      });

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getByText('Notification Settings')).toBeInTheDocument();
      });
    });

    it('displays email on merge toggle', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue({
        emailOnMerge: false,
        emailOnConflict: false,
        emailOnReview: false,
      });

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getByText(/email on successful merge/i)).toBeInTheDocument();
      });
    });

    it('displays email on conflict toggle', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue({
        emailOnMerge: false,
        emailOnConflict: false,
        emailOnReview: false,
      });

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getByText(/email on merge conflict/i)).toBeInTheDocument();
      });
    });

    it('displays email on review toggle', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue({
        emailOnMerge: false,
        emailOnConflict: false,
        emailOnReview: false,
      });

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getByText(/email on review request/i)).toBeInTheDocument();
      });
    });
  });

  describe('Settings Updates', () => {
    it('toggles email on merge', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getNotifications.mockResolvedValue({
        emailOnMerge: false,
        emailOnConflict: false,
        emailOnReview: false,
      });
      mockedSettingsApi.updateNotifications.mockResolvedValue({});

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getAllByRole('switch')).toHaveLength(3);
      });

      const switches = screen.getAllByRole('switch');
      await user.click(switches[0]);

      await waitFor(() => {
        expect(mockedSettingsApi.updateNotifications).toHaveBeenCalledWith({
          emailOnMerge: true,
        });
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Settings updated',
        description: 'Your notification settings have been saved.',
      });
    });

    it('reflects current notification preferences', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue({
        emailOnMerge: true,
        emailOnConflict: true,
        emailOnReview: false,
      });

      renderNotificationsSettings();

      await waitFor(() => {
        const switches = screen.getAllByRole('switch');
        expect(switches[0]).toBeChecked();
        expect(switches[1]).toBeChecked();
        expect(switches[2]).not.toBeChecked();
      });
    });

    it('shows error toast on update failure', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getNotifications.mockResolvedValue({
        emailOnMerge: false,
        emailOnConflict: false,
        emailOnReview: false,
      });
      mockedSettingsApi.updateNotifications.mockRejectedValue({
        response: { data: { error: 'Failed to update' } },
      });

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getAllByRole('switch')).toHaveLength(3);
      });

      const switches = screen.getAllByRole('switch');
      await user.click(switches[0]);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          variant: 'destructive',
          title: 'Failed to update settings',
          description: 'Failed to update',
        });
      });
    });
  });
});
