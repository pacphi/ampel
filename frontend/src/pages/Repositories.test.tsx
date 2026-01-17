import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Repositories from './Repositories';
import type { RepositoryWithStatus, DiscoveredRepository } from '@/types';
import type { UseQueryResult, UseMutationResult } from '@tanstack/react-query';

// Mock react-i18next with translations
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        // repositories namespace
        'repositories:title': 'Repositories',
        'repositories:providers.title': 'Connect a Provider',
        'repositories:providers.connected': '(Connected)',
        'repositories:addFrom.title': `Add from ${options?.provider ?? '{{provider}}'}`,
        'repositories:addFrom.addAll': `Add All (${options?.count ?? '{{count}}'})`,
        'repositories:addFrom.close': 'Close',
        'repositories:addFrom.empty': 'No new repositories found to add',
        'repositories:search.placeholder': 'Search repositories...',
        'repositories:table.status': 'Status',
        'repositories:table.repository': 'Repository',
        'repositories:table.provider': 'Provider',
        'repositories:table.prs': 'Open PRs',
        'repositories:table.actions': 'Actions',
        'repositories:empty.title': 'No repositories found',
        'repositories:empty.description':
          'Add repositories from your connected providers to get started',
        'repositories:toast.added': 'Repository Added',
        'repositories:toast.addedDescription': `${options?.name ?? '{{name}}'} has been added to your dashboard`,
        'repositories:toast.addFailed': 'Failed to Add Repository',
        'repositories:toast.removed': 'Repository Removed',
        'repositories:toast.removedDescription': `${options?.name ?? '{{name}}'} has been removed from your dashboard`,
        'repositories:toast.removeFailed': 'Failed to Remove Repository',
        'repositories:toast.bulkAdded': 'Repositories Added',
        'repositories:toast.bulkAddedDescription': `Successfully added ${options?.count ?? '{{count}}'} repositories${options?.failed ?? ''}`,
        'repositories:toast.bulkAddedFailed': ` (${options?.count ?? '{{count}}'} failed)`,
        'repositories:toast.bulkAddFailed': 'Failed to Add Repositories',
        'repositories:toast.bulkAddFailedDescription':
          'Could not add any of the selected repositories',
        // common namespace
        'common:providers.github': 'GitHub',
        'common:providers.gitlab': 'GitLab',
        'common:providers.bitbucket': 'Bitbucket',
        // errors namespace
        'errors:general.unknownError': 'An unknown error occurred',
      };
      return translations[key] ?? key;
    },
    i18n: { language: 'en' },
  }),
}));

vi.mock('@/hooks/useRepositories', () => ({
  useRepositories: vi.fn(),
  useAddRepository: vi.fn(),
  useRemoveRepository: vi.fn(),
  useDiscoverRepositories: vi.fn(),
}));

vi.mock('@/api/accounts', () => ({
  accountsApi: {
    listAccounts: vi.fn(),
  },
}));

vi.mock('@/components/ui/use-toast', () => ({
  useToast: vi.fn(() => ({ toast: vi.fn(), dismiss: vi.fn(), toasts: [] })),
}));

import {
  useRepositories,
  useAddRepository,
  useRemoveRepository,
  useDiscoverRepositories,
} from '@/hooks/useRepositories';
import { accountsApi } from '@/api/accounts';
import { useToast } from '@/components/ui/use-toast';

type AddRepositoryParams = {
  provider: string;
  owner: string;
  name: string;
  pollIntervalSeconds?: number;
};

const mockedUseRepositories = vi.mocked(useRepositories);
const mockedUseAddRepository = vi.mocked(useAddRepository);
const mockedUseRemoveRepository = vi.mocked(useRemoveRepository);
const mockedUseDiscoverRepositories = vi.mocked(useDiscoverRepositories);
const mockedAccountsApi = vi.mocked(accountsApi);
const mockedUseToast = vi.mocked(useToast);

function renderRepositories() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <Repositories />
    </QueryClientProvider>
  );
}

describe('Repositories', () => {
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
    it('renders loading spinner while fetching repositories', () => {
      mockedUseRepositories.mockReturnValue({
        data: undefined,
        isLoading: true,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: undefined,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([]);

      const { container } = renderRepositories();

      const spinner = container.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });
  });

  describe('Provider Connection', () => {
    it('renders provider connection buttons', async () => {
      mockedUseRepositories.mockReturnValue({
        data: [],
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: undefined,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /github/i })).toBeInTheDocument();
      });

      expect(screen.getByRole('button', { name: /gitlab/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /bitbucket/i })).toBeInTheDocument();
    });

    it('shows connected status for active providers', async () => {
      mockedUseRepositories.mockReturnValue({
        data: [],
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: undefined,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([
        {
          id: '1',
          provider: 'github',
          username: 'testuser',
          isActive: true,
        },
      ]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /github \(connected\)/i })).toBeInTheDocument();
      });
    });
  });

  describe('Repository List', () => {
    it('displays repository table with data', async () => {
      const repositories = [
        {
          id: '1',
          name: 'repo1',
          owner: 'owner1',
          fullName: 'owner1/repo1',
          provider: 'github',
          openPrCount: 5,
          status: 'green',
          url: 'https://github.com/owner1/repo1',
        },
        {
          id: '2',
          name: 'repo2',
          owner: 'owner2',
          fullName: 'owner2/repo2',
          provider: 'gitlab',
          openPrCount: 3,
          status: 'yellow',
          url: 'https://gitlab.com/owner2/repo2',
        },
      ];

      mockedUseRepositories.mockReturnValue({
        data: repositories,
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: undefined,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByText('repo1')).toBeInTheDocument();
      });

      expect(screen.getByText('repo2')).toBeInTheDocument();
      expect(screen.getByText('5')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
    });

    it('shows empty state when no repositories', async () => {
      mockedUseRepositories.mockReturnValue({
        data: [],
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: undefined,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByText('No repositories found')).toBeInTheDocument();
      });

      expect(
        screen.getByText('Add repositories from your connected providers to get started')
      ).toBeInTheDocument();
    });
  });

  describe('Search Functionality', () => {
    it('filters repositories by search query', async () => {
      const user = userEvent.setup();
      const repositories = [
        {
          id: '1',
          name: 'frontend',
          owner: 'acme',
          fullName: 'acme/frontend',
          provider: 'github',
          openPrCount: 5,
          status: 'green',
          url: 'https://github.com/acme/frontend',
        },
        {
          id: '2',
          name: 'backend',
          owner: 'acme',
          fullName: 'acme/backend',
          provider: 'github',
          openPrCount: 3,
          status: 'yellow',
          url: 'https://github.com/acme/backend',
        },
      ];

      mockedUseRepositories.mockReturnValue({
        data: repositories,
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: undefined,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByText('frontend')).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText('Search repositories...');
      await user.type(searchInput, 'frontend');

      await waitFor(() => {
        expect(screen.getByText('frontend')).toBeInTheDocument();
        expect(screen.queryByText('backend')).not.toBeInTheDocument();
      });
    });
  });

  describe('Add Repository', () => {
    it('shows discovered repositories when provider is selected', async () => {
      const user = userEvent.setup();
      const discoveredRepos = [
        {
          providerId: 'gh-1',
          provider: 'github',
          owner: 'testorg',
          name: 'newrepo',
          fullName: 'testorg/newrepo',
          description: 'A new repository',
        },
      ];

      mockedUseRepositories.mockReturnValue({
        data: [],
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: discoveredRepos,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([
        { id: '1', provider: 'github', username: 'testuser', isActive: true },
      ]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /github \(connected\)/i })).toBeInTheDocument();
      });

      const githubButton = screen.getByRole('button', { name: /github \(connected\)/i });
      await user.click(githubButton);

      await waitFor(() => {
        expect(screen.getByText('testorg/newrepo')).toBeInTheDocument();
      });

      expect(screen.getByText('A new repository')).toBeInTheDocument();
    });

    it('adds repository when add button is clicked', async () => {
      const user = userEvent.setup();
      const mutateAsync = vi.fn().mockResolvedValue({});
      const discoveredRepos = [
        {
          providerId: 'gh-1',
          provider: 'github',
          owner: 'testorg',
          name: 'newrepo',
          fullName: 'testorg/newrepo',
        },
      ];

      mockedUseRepositories.mockReturnValue({
        data: [],
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync,
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: discoveredRepos,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([
        { id: '1', provider: 'github', username: 'testuser', isActive: true },
      ]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /github \(connected\)/i })).toBeInTheDocument();
      });

      const githubButton = screen.getByRole('button', { name: /github \(connected\)/i });
      await user.click(githubButton);

      await waitFor(() => {
        expect(screen.getByText('testorg/newrepo')).toBeInTheDocument();
      });

      const addButtons = screen.getAllByRole('button');
      const addButton = addButtons.find((btn) =>
        btn.querySelector('svg')?.classList.contains('lucide-plus')
      );
      expect(addButton).toBeInTheDocument();

      if (addButton) {
        await user.click(addButton);

        await waitFor(() => {
          expect(mutateAsync).toHaveBeenCalledWith({
            provider: 'github',
            owner: 'testorg',
            name: 'newrepo',
          });
        });

        expect(mockToast).toHaveBeenCalledWith({
          title: 'Repository Added',
          description: 'testorg/newrepo has been added to your dashboard',
        });
      }
    });

    it('shows error toast when add fails', async () => {
      const user = userEvent.setup();
      const mutateAsync = vi
        .fn()
        .mockRejectedValue({ response: { data: { error: 'Already exists' } } });
      const discoveredRepos = [
        {
          providerId: 'gh-1',
          provider: 'github',
          owner: 'testorg',
          name: 'newrepo',
          fullName: 'testorg/newrepo',
        },
      ];

      mockedUseRepositories.mockReturnValue({
        data: [],
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync,
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: discoveredRepos,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([
        { id: '1', provider: 'github', username: 'testuser', isActive: true },
      ]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /github \(connected\)/i })).toBeInTheDocument();
      });

      const githubButton = screen.getByRole('button', { name: /github \(connected\)/i });
      await user.click(githubButton);

      await waitFor(() => {
        expect(screen.getByText('testorg/newrepo')).toBeInTheDocument();
      });

      const addButtons = screen.getAllByRole('button');
      const addButton = addButtons.find((btn) =>
        btn.querySelector('svg')?.classList.contains('lucide-plus')
      );

      if (addButton) {
        await user.click(addButton);

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            variant: 'destructive',
            title: 'Failed to Add Repository',
            description: 'Already exists',
          });
        });
      }
    });
  });

  describe('Remove Repository', () => {
    it('removes repository when delete button is clicked', async () => {
      const user = userEvent.setup();
      const mutateAsync = vi.fn().mockResolvedValue({});
      const repositories = [
        {
          id: '1',
          name: 'repo1',
          owner: 'owner1',
          fullName: 'owner1/repo1',
          provider: 'github',
          openPrCount: 5,
          status: 'green',
          url: 'https://github.com/owner1/repo1',
        },
      ];

      mockedUseRepositories.mockReturnValue({
        data: repositories,
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync,
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: undefined,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByText('repo1')).toBeInTheDocument();
      });

      const deleteButtons = screen.getAllByRole('button');
      const deleteButton = deleteButtons.find((btn) => btn.querySelector('svg.lucide-trash-2'));
      expect(deleteButton).toBeInTheDocument();

      if (deleteButton) {
        await user.click(deleteButton);

        await waitFor(() => {
          expect(mutateAsync).toHaveBeenCalledWith('1');
        });

        expect(mockToast).toHaveBeenCalledWith({
          title: 'Repository Removed',
          description: 'owner1/repo1 has been removed from your dashboard',
        });
      }
    });
  });

  describe('Add All Repositories', () => {
    it('adds all discovered repositories when add all is clicked', async () => {
      const user = userEvent.setup();
      const mutateAsync = vi.fn().mockResolvedValue({});
      const discoveredRepos = [
        {
          providerId: 'gh-1',
          provider: 'github',
          owner: 'test',
          name: 'repo1',
          fullName: 'test/repo1',
        },
        {
          providerId: 'gh-2',
          provider: 'github',
          owner: 'test',
          name: 'repo2',
          fullName: 'test/repo2',
        },
      ];

      mockedUseRepositories.mockReturnValue({
        data: [],
        isLoading: false,
      } as UseQueryResult<RepositoryWithStatus[], Error>);
      mockedUseAddRepository.mockReturnValue({
        mutateAsync,
        isPending: false,
      } as UseMutationResult<RepositoryWithStatus, Error, AddRepositoryParams>);
      mockedUseRemoveRepository.mockReturnValue({
        mutateAsync: vi.fn(),
        isPending: false,
      } as UseMutationResult<void, Error, string>);
      mockedUseDiscoverRepositories.mockReturnValue({
        data: discoveredRepos,
        isLoading: false,
      } as UseQueryResult<DiscoveredRepository[], Error>);
      mockedAccountsApi.listAccounts.mockResolvedValue([
        { id: '1', provider: 'github', username: 'testuser', isActive: true },
      ]);

      renderRepositories();

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /github \(connected\)/i })).toBeInTheDocument();
      });

      const githubButton = screen.getByRole('button', { name: /github \(connected\)/i });
      await user.click(githubButton);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /add all \(2\)/i })).toBeInTheDocument();
      });

      const addAllButton = screen.getByRole('button', { name: /add all \(2\)/i });
      await user.click(addAllButton);

      await waitFor(() => {
        expect(mutateAsync).toHaveBeenCalledTimes(2);
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Repositories Added',
        description: 'Successfully added 2 repositories',
      });
    });
  });
});
