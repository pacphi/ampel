import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { RepositoryWithStatus } from '@/types';

// Mock react-i18next - GridView doesn't use translations but imports components that do
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      language: 'en',
      changeLanguage: vi.fn(),
    },
    ready: true,
  }),
}));

import GridView from './GridView';

function renderGridView(repositories: Partial<RepositoryWithStatus>[]) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <GridView repositories={repositories} />
    </QueryClientProvider>
  );
}

describe('GridView', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Empty State', () => {
    it('shows empty state when no repositories', () => {
      renderGridView([]);

      expect(screen.getByText('No repositories found')).toBeInTheDocument();
      expect(
        screen.getByText('Add repositories from the Repositories page to get started')
      ).toBeInTheDocument();
    });
  });

  describe('Repository Display', () => {
    it('renders repositories in grid layout', () => {
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
        {
          id: '3',
          name: 'repo3',
          owner: 'owner3',
          fullName: 'owner3/repo3',
          provider: 'bitbucket',
          openPrCount: 0,
          status: 'red',
          url: 'https://bitbucket.org/owner3/repo3',
        },
      ];

      renderGridView(repositories);

      // RepoCard displays name and owner separately, not fullName
      expect(screen.getByText('repo1')).toBeInTheDocument();
      expect(screen.getByText('repo2')).toBeInTheDocument();
      expect(screen.getByText('repo3')).toBeInTheDocument();
    });

    it('applies grid layout classes', () => {
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

      const { container } = renderGridView(repositories);

      const gridContainer = container.querySelector('.grid');
      expect(gridContainer).toBeInTheDocument();
      expect(gridContainer?.className).toContain('gap-4');
    });

    it('renders correct number of repository cards', () => {
      const repositories = Array.from({ length: 10 }, (_, i) => ({
        id: `${i + 1}`,
        name: `repo${i + 1}`,
        owner: `owner${i + 1}`,
        fullName: `owner${i + 1}/repo${i + 1}`,
        provider: 'github',
        openPrCount: i,
        status: 'green',
        url: `https://github.com/owner${i + 1}/repo${i + 1}`,
      }));

      renderGridView(repositories);

      // Check that we have all 10 repositories - RepoCard displays name separately
      repositories.forEach((repo) => {
        expect(screen.getByText(repo.name)).toBeInTheDocument();
      });
    });
  });
});
