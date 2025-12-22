import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { NotificationsSettings } from './NotificationsSettings';

vi.mock('@/api/settings', () => ({
  settingsApi: {
    getNotifications: vi.fn(),
    updateNotifications: vi.fn(),
    testSlack: vi.fn(),
    testEmail: vi.fn(),
  },
}));

vi.mock('@/components/ui/use-toast', () => ({
  useToast: vi.fn(() => ({ toast: vi.fn(), dismiss: vi.fn(), toasts: [] })),
}));

import { settingsApi } from '@/api/settings';
import { useToast } from '@/components/ui/use-toast';

const mockedSettingsApi = vi.mocked(settingsApi);
const mockedUseToast = vi.mocked(useToast);

const defaultNotificationPrefs = {
  emailEnabled: false,
  slackEnabled: false,
  slackWebhookUrl: null,
  pushEnabled: false,
  notifyOnPrReady: false,
  notifyOnPrFailed: false,
  notifyOnReviewRequested: false,
  digestFrequency: 'daily',
  smtpHost: null,
  smtpPort: null,
  smtpUsername: null,
  smtpFromEmail: null,
  smtpToEmails: null,
  smtpUseTls: true,
  notifyOnMergeSuccess: true,
  notifyOnMergeFailure: true,
  slackChannel: null,
};

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
    it('renders slack notifications card', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue(defaultNotificationPrefs);

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getByText('Slack Notifications')).toBeInTheDocument();
      });
    });

    it('renders email notifications card', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue(defaultNotificationPrefs);

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getByText('Email Notifications')).toBeInTheDocument();
      });
    });

    it('renders merge notifications card', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue(defaultNotificationPrefs);

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getByText('Merge Notifications')).toBeInTheDocument();
      });
    });

    it('displays enable slack notifications toggle', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue(defaultNotificationPrefs);

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getByText(/enable slack notifications/i)).toBeInTheDocument();
      });
    });

    it('displays enable email notifications toggle', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue(defaultNotificationPrefs);

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getByText(/enable email notifications/i)).toBeInTheDocument();
      });
    });
  });

  describe('Settings Updates', () => {
    it('toggles slack notifications', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getNotifications.mockResolvedValue(defaultNotificationPrefs);
      mockedSettingsApi.updateNotifications.mockResolvedValue({
        ...defaultNotificationPrefs,
        slackEnabled: true,
      });

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getAllByRole('switch').length).toBeGreaterThan(0);
      });

      const switches = screen.getAllByRole('switch');
      // First switch is the Slack enabled toggle
      await user.click(switches[0]);

      await waitFor(() => {
        expect(mockedSettingsApi.updateNotifications).toHaveBeenCalledWith({
          slackEnabled: true,
        });
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Settings updated',
        description: 'Your notification settings have been saved.',
      });
    });

    it('reflects current notification preferences', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue({
        ...defaultNotificationPrefs,
        slackEnabled: true,
        emailEnabled: true,
      });

      renderNotificationsSettings();

      await waitFor(() => {
        const switches = screen.getAllByRole('switch');
        expect(switches[0]).toBeChecked(); // slackEnabled
        expect(switches[1]).toBeChecked(); // emailEnabled
      });
    });

    it('shows error toast on update failure', async () => {
      const user = userEvent.setup();
      mockedSettingsApi.getNotifications.mockResolvedValue(defaultNotificationPrefs);
      mockedSettingsApi.updateNotifications.mockRejectedValue({
        response: { data: { error: 'Failed to update' } },
      });

      renderNotificationsSettings();

      await waitFor(() => {
        expect(screen.getAllByRole('switch').length).toBeGreaterThan(0);
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

  describe('Test Buttons', () => {
    it('test slack button is disabled when no webhook URL', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue(defaultNotificationPrefs);

      renderNotificationsSettings();

      await waitFor(() => {
        const testSlackButton = screen.getByRole('button', { name: /send test message/i });
        expect(testSlackButton).toBeDisabled();
      });
    });

    it('test slack button is enabled when webhook URL is set', async () => {
      mockedSettingsApi.getNotifications.mockResolvedValue({
        ...defaultNotificationPrefs,
        slackWebhookUrl: 'https://hooks.slack.com/services/test',
      });

      renderNotificationsSettings();

      await waitFor(() => {
        const testSlackButton = screen.getByRole('button', { name: /send test message/i });
        expect(testSlackButton).toBeEnabled();
      });
    });
  });
});
