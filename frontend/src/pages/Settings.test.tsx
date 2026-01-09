import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import Settings from './Settings';

// Mock react-i18next with translations
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        // Settings namespace - tabs
        'settings:tabs.profile': 'Profile',
        'settings:tabs.accounts': 'Accounts',
        'settings:tabs.prFilters': 'PR Filters',
        'settings:tabs.repositoryFilters': 'Repository Filters',
        'settings:tabs.notifications': 'Notifications',
        'settings:tabs.behavior': 'Behavior',
        // Settings namespace - title
        'settings:title': 'Settings',
        // Settings namespace - account section
        'settings:account.profile': 'Profile',
        'settings:account.yourAccountInfo': 'Your account information',
        'settings:account.email': 'Email',
        'settings:account.emailPlaceholder': 'Enter your email',
        'settings:account.displayName': 'Display name',
        'settings:account.displayNamePlaceholder': 'Enter your display name',
        'settings:account.memberSince': 'Member Since',
        'settings:account.notSet': 'Not set',
        'settings:account.unknown': 'Unknown',
        // Settings namespace - profile messages
        'settings:profileUpdated': 'Profile updated',
        'settings:profileUpdateSuccess': 'Your profile has been saved successfully.',
        'settings:profileUpdateFailed': 'Failed to update profile',
        // Settings namespace - prFilters section
        'settings:prFilters.title': 'PR Filters',
        'settings:prFilters.description':
          'Configure global filters for pull request processing. These apply to auto-merge rules across all repositories.',
        'settings:prFilters.allowedActors': 'Allowed Actors (Bots/Users)',
        'settings:prFilters.allowedActorsDescription':
          'Only process PRs from these trusted actors. Typically bots like dependabot, renovate, etc.',
        'settings:prFilters.allowedActorsPlaceholder': 'e.g., dependabot[bot]',
        'settings:prFilters.skipLabels': 'Skip Labels',
        'settings:prFilters.skipLabelsDescription':
          'PRs with these labels will be skipped during auto-merge processing.',
        'settings:prFilters.skipLabelsPlaceholder': 'e.g., do-not-merge',
        'settings:prFilters.maxAgeDays': 'Max PR Age (Days)',
        'settings:prFilters.maxAgeDaysDescription':
          'Skip PRs older than this many days. Leave empty for no limit.',
        'settings:prFilters.maxAgeDaysPlaceholder': 'e.g., 30',
        // Settings namespace - filter messages
        'settings:filtersUpdated': 'Filters updated',
        'settings:filtersUpdateSuccess': 'Your PR filter settings have been saved.',
        'settings:filtersReset': 'Filters reset',
        'settings:filtersResetSuccess': 'Your PR filter settings have been reset to defaults.',
        'settings:filtersUpdateFailed': 'Failed to update filters',
        // Settings namespace - language
        'settings:language.title': 'Language Preferences',
        'settings:language.description': 'Select your preferred language for the application',
        // Common namespace
        'common:app.edit': 'Edit',
        'common:app.save': 'Save',
        'common:app.cancel': 'Cancel',
        'common:app.add': 'Add',
        'common:app.error': 'An error occurred',
        'common:actions.saving': 'Saving...',
        'common:actions.reset': 'Reset to Defaults',
        'common:actions.showEmail': 'Show email',
        'common:actions.hideEmail': 'Hide email',
      };
      return translations[key] || key;
    },
    i18n: { language: 'en', changeLanguage: vi.fn() },
    ready: true,
  }),
  Trans: ({ children }: { children: React.ReactNode }) => children,
  initReactI18next: { type: '3rdParty', init: vi.fn() },
}));

// Mock the i18n hooks used by LanguageSwitcher
vi.mock('@/i18n/hooks', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en', changeLanguage: vi.fn() },
    ready: true,
  }),
}));

// Mock LanguageSwitcher component to avoid complex dependencies
vi.mock('@/components/LanguageSwitcher', () => ({
  default: () => <div data-testid="language-switcher">Language Switcher</div>,
}));

vi.mock('@/hooks/useAuth', () => ({
  useAuth: vi.fn(),
}));

vi.mock('@/api/auth', () => ({
  authApi: {
    updateProfile: vi.fn(),
  },
}));

vi.mock('@/api/prFilters', () => ({
  prFiltersApi: {
    get: vi.fn(),
    update: vi.fn(),
    reset: vi.fn(),
  },
}));

vi.mock('@/api/accounts', () => ({
  accountsApi: {
    listAccounts: vi.fn(),
  },
}));

vi.mock('@/components/ui/use-toast', () => ({
  useToast: vi.fn(() => ({ toast: vi.fn(), dismiss: vi.fn(), toasts: [] })),
}));

import { useAuth } from '@/hooks/useAuth';
import { authApi } from '@/api/auth';
import { prFiltersApi } from '@/api/prFilters';
import { useToast } from '@/components/ui/use-toast';

const mockedUseAuth = vi.mocked(useAuth);
const mockedAuthApi = vi.mocked(authApi);
const mockedPrFiltersApi = vi.mocked(prFiltersApi);
const mockedUseToast = vi.mocked(useToast);

function renderSettings(initialRoute = '/settings') {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[initialRoute]}>
        <Routes>
          <Route path="/settings/*" element={<Settings />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe('Settings', () => {
  const mockToast = vi.fn();
  const mockRefreshUser = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockedUseToast.mockReturnValue({
      toast: mockToast,
      dismiss: vi.fn(),
      toasts: [],
    });
    mockedUseAuth.mockReturnValue({
      user: {
        id: '1',
        email: 'test@example.com',
        displayName: 'Test User',
        createdAt: '2024-01-01T00:00:00Z',
      },
      isLoading: false,
      isAuthenticated: true,
      login: vi.fn(),
      register: vi.fn(),
      logout: vi.fn(),
      refreshUser: mockRefreshUser,
    });
  });

  describe('Navigation', () => {
    it('renders settings navigation', async () => {
      renderSettings();

      await waitFor(() => {
        // Profile appears in both nav and card title, so use getAllByText
        const profileElements = screen.getAllByText('Profile');
        expect(profileElements.length).toBeGreaterThanOrEqual(1);
      });
      expect(screen.getByText('Accounts')).toBeInTheDocument();
      expect(screen.getByText('PR Filters')).toBeInTheDocument();
      expect(screen.getByText('Repository Filters')).toBeInTheDocument();
      expect(screen.getByText('Notifications')).toBeInTheDocument();
      expect(screen.getByText('Behavior')).toBeInTheDocument();
    });

    it('highlights active nav item', async () => {
      renderSettings('/settings');

      await waitFor(() => {
        const profileLink = screen.getByRole('link', { name: /profile/i });
        expect(profileLink).toBeInTheDocument();
        expect(profileLink.className).toContain('bg-primary');
      });
    });

    it('navigates to PR filters page', async () => {
      const user = userEvent.setup();
      mockedPrFiltersApi.get.mockResolvedValue({
        allowedActors: ['dependabot[bot]'],
        skipLabels: ['do-not-merge'],
        maxAgeDays: null,
      });

      renderSettings();

      const filtersLink = screen.getByText('PR Filters');
      await user.click(filtersLink);

      await waitFor(() => {
        // Card title should show 'PR Filters'
        expect(screen.getByRole('heading', { name: 'PR Filters' })).toBeInTheDocument();
      });
    });
  });

  describe('Profile Settings', () => {
    it('displays user profile information', () => {
      renderSettings();

      // Email is masked by default, display name is shown
      expect(screen.getByText('Test User')).toBeInTheDocument();
      // Email should be masked: "test" -> t + ** + t = t**t
      expect(screen.getByText(/t\*\*t@example\.com/)).toBeInTheDocument();
    });

    it('masks email by default', () => {
      renderSettings();

      // The email is masked using the maskEmail function
      // For test@example.com, local part is "test" (4 chars) -> t + **(length-2) + t = t**t
      expect(screen.getByText(/t\*\*t@example\.com/)).toBeInTheDocument();
    });

    it('shows email when eye button is clicked', async () => {
      const user = userEvent.setup();
      renderSettings();

      const eyeButton = screen.getByTitle('Show email');
      await user.click(eyeButton);

      await waitFor(() => {
        expect(screen.getByText('test@example.com')).toBeInTheDocument();
      });
    });

    it('enters edit mode when edit button is clicked', async () => {
      const user = userEvent.setup();
      renderSettings();

      const editButton = screen.getByRole('button', { name: /edit/i });
      await user.click(editButton);

      await waitFor(() => {
        expect(screen.getByPlaceholderText('Enter your email')).toBeInTheDocument();
      });

      expect(screen.getByPlaceholderText('Enter your display name')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /save/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument();
    });

    it('updates profile when save is clicked', async () => {
      const user = userEvent.setup();
      mockedAuthApi.updateProfile.mockResolvedValue({});

      renderSettings();

      const editButton = screen.getByRole('button', { name: /edit/i });
      await user.click(editButton);

      const emailInput = await screen.findByPlaceholderText('Enter your email');
      await user.clear(emailInput);
      await user.type(emailInput, 'newemail@example.com');

      const saveButton = screen.getByRole('button', { name: /save/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockedAuthApi.updateProfile).toHaveBeenCalledWith({
          email: 'newemail@example.com',
        });
      });

      expect(mockRefreshUser).toHaveBeenCalled();
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Profile updated',
        description: 'Your profile has been saved successfully.',
      });
    });

    it('cancels edit when cancel button is clicked', async () => {
      const user = userEvent.setup();
      renderSettings();

      const editButton = screen.getByRole('button', { name: /edit/i });
      await user.click(editButton);

      const emailInput = await screen.findByPlaceholderText('Enter your email');
      await user.clear(emailInput);
      await user.type(emailInput, 'changed@example.com');

      const cancelButton = screen.getByRole('button', { name: /cancel/i });
      await user.click(cancelButton);

      await waitFor(() => {
        expect(screen.queryByPlaceholderText('Enter your email')).not.toBeInTheDocument();
      });

      // Original masked email should be displayed
      // For test@example.com, local part is "test" -> "t**t"
      expect(screen.getByText(/t\*\*t@example\.com/)).toBeInTheDocument();
    });

    it('shows error toast on update failure', async () => {
      const user = userEvent.setup();
      mockedAuthApi.updateProfile.mockRejectedValue({
        response: { data: { error: 'Email already exists' } },
      });

      renderSettings();

      const editButton = screen.getByRole('button', { name: /edit/i });
      await user.click(editButton);

      const emailInput = await screen.findByPlaceholderText('Enter your email');
      await user.clear(emailInput);
      await user.type(emailInput, 'duplicate@example.com');

      const saveButton = screen.getByRole('button', { name: /save/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          variant: 'destructive',
          title: 'Failed to update profile',
          description: 'Email already exists',
        });
      });
    });
  });

  describe('Filters Settings', () => {
    it('displays filter settings', async () => {
      const user = userEvent.setup();
      mockedPrFiltersApi.get.mockResolvedValue({
        allowedActors: ['dependabot[bot]', 'renovate[bot]'],
        skipLabels: ['do-not-merge', 'wip'],
        maxAgeDays: 30,
      });

      renderSettings();

      const filtersLink = screen.getByText('PR Filters');
      await user.click(filtersLink);

      await waitFor(() => {
        expect(screen.getByText('dependabot[bot]')).toBeInTheDocument();
      });

      expect(screen.getByText('renovate[bot]')).toBeInTheDocument();
      expect(screen.getByText('do-not-merge')).toBeInTheDocument();
      expect(screen.getByText('wip')).toBeInTheDocument();
      expect(screen.getByDisplayValue('30')).toBeInTheDocument();
    });

    it('adds new allowed actor', async () => {
      const user = userEvent.setup();
      mockedPrFiltersApi.get.mockResolvedValue({
        allowedActors: ['dependabot[bot]'],
        skipLabels: [],
        maxAgeDays: null,
      });
      mockedPrFiltersApi.update.mockResolvedValue({});

      renderSettings();

      const filtersLink = screen.getByText('PR Filters');
      await user.click(filtersLink);

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/e\.g\., dependabot/i)).toBeInTheDocument();
      });

      const actorInput = screen.getByPlaceholderText(/e\.g\., dependabot/i);
      await user.type(actorInput, 'renovate-bot');

      const addButton = screen.getAllByRole('button', { name: /add/i })[0];
      await user.click(addButton);

      await waitFor(() => {
        expect(mockedPrFiltersApi.update).toHaveBeenCalledWith({
          allowedActors: ['dependabot[bot]', 'renovate-bot'],
        });
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Filters updated',
        description: 'Your PR filter settings have been saved.',
      });
    });

    it('removes allowed actor', async () => {
      const user = userEvent.setup();
      mockedPrFiltersApi.get.mockResolvedValue({
        allowedActors: ['dependabot[bot]', 'renovate[bot]'],
        skipLabels: [],
        maxAgeDays: null,
      });
      mockedPrFiltersApi.update.mockResolvedValue({});

      renderSettings();

      const filtersLink = screen.getByText('PR Filters');
      await user.click(filtersLink);

      await waitFor(() => {
        expect(screen.getByText('dependabot[bot]')).toBeInTheDocument();
      });

      const removeButtons = screen.getAllByRole('button', { hidden: true });
      const firstRemoveButton = removeButtons.find((btn) => btn.querySelector('svg.lucide-x'));

      if (firstRemoveButton) {
        await user.click(firstRemoveButton);

        await waitFor(() => {
          expect(mockedPrFiltersApi.update).toHaveBeenCalled();
        });
      }
    });

    it('resets filters to defaults', async () => {
      const user = userEvent.setup();
      mockedPrFiltersApi.get.mockResolvedValue({
        allowedActors: ['custombot[bot]'],
        skipLabels: ['custom-label'],
        maxAgeDays: 60,
      });
      mockedPrFiltersApi.reset.mockResolvedValue({});

      renderSettings();

      const filtersLink = screen.getByText('PR Filters');
      await user.click(filtersLink);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /reset to defaults/i })).toBeInTheDocument();
      });

      const resetButton = screen.getByRole('button', { name: /reset to defaults/i });
      await user.click(resetButton);

      await waitFor(() => {
        expect(mockedPrFiltersApi.reset).toHaveBeenCalled();
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Filters reset',
        description: 'Your PR filter settings have been reset to defaults.',
      });
    });

    it('updates max age days', async () => {
      const user = userEvent.setup();
      mockedPrFiltersApi.get.mockResolvedValue({
        allowedActors: [],
        skipLabels: [],
        maxAgeDays: null,
      });
      mockedPrFiltersApi.update.mockResolvedValue({});

      renderSettings();

      const filtersLink = screen.getByText('PR Filters');
      await user.click(filtersLink);

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/e\.g\., 30/i)).toBeInTheDocument();
      });

      const maxAgeInput = screen.getByPlaceholderText(/e\.g\., 30/i);
      // Use fireEvent.change to set value directly (avoids controlled input reset issues)
      fireEvent.change(maxAgeInput, { target: { value: '45' } });

      await waitFor(() => {
        expect(mockedPrFiltersApi.update).toHaveBeenCalledWith({
          maxAgeDays: 45,
        });
      });
    });
  });

  describe('Member Since', () => {
    it('formats member since date correctly', () => {
      renderSettings();

      expect(screen.getByText('January 1, 2024')).toBeInTheDocument();
    });

    it('shows unknown when created date is missing', () => {
      mockedUseAuth.mockReturnValue({
        user: {
          id: '1',
          email: 'test@example.com',
          displayName: 'Test User',
          createdAt: undefined,
        },
        isLoading: false,
        isAuthenticated: true,
        login: vi.fn(),
        register: vi.fn(),
        logout: vi.fn(),
        refreshUser: vi.fn(),
      });

      renderSettings();

      expect(screen.getByText('Unknown')).toBeInTheDocument();
    });
  });
});
