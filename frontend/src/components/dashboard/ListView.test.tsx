import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import ListView from './ListView';
import type { RepositoryWithStatus } from '@/types';

function renderListView(repositories: Partial<RepositoryWithStatus>[]) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <ListView repositories={repositories} />
    </QueryClientProvider>
  );
}

describe('ListView', () => {
  describe('Empty State', () => {
    it('shows empty state when no repositories', () => {
      renderListView([]);

      expect(screen.getByText('No repositories found')).toBeInTheDocument();
      expect(
        screen.getByText('Add repositories from the Repositories page to get started')
      ).toBeInTheDocument();
    });
  });

  describe('Repository Display', () => {
    it('renders repositories in list layout', () => {
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

      renderListView(repositories);

      expect(screen.getByText('owner1/repo1')).toBeInTheDocument();
      expect(screen.getByText('owner2/repo2')).toBeInTheDocument();
    });

    it('displays table headers', () => {
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

      renderListView(repositories);

      expect(screen.getByText('Status')).toBeInTheDocument();
      expect(screen.getByText('Repository')).toBeInTheDocument();
      expect(screen.getByText('Provider')).toBeInTheDocument();
      expect(screen.getByText('PRs')).toBeInTheDocument();
    });

    it('renders correct number of rows', () => {
      const repositories = Array.from({ length: 5 }, (_, i) => ({
        id: `${i + 1}`,
        name: `repo${i + 1}`,
        owner: `owner${i + 1}`,
        fullName: `owner${i + 1}/repo${i + 1}`,
        provider: 'github',
        openPrCount: i,
        status: 'green',
        url: `https://github.com/owner${i + 1}/repo${i + 1}`,
      }));

      const { container } = renderListView(repositories);

      const rows = container.querySelectorAll('tbody tr');
      expect(rows.length).toBe(5);
    });

    it('displays PR counts', () => {
      const repositories = [
        {
          id: '1',
          name: 'repo1',
          owner: 'owner1',
          fullName: 'owner1/repo1',
          provider: 'github',
          openPrCount: 12,
          status: 'green',
          url: 'https://github.com/owner1/repo1',
        },
      ];

      renderListView(repositories);

      expect(screen.getByText('12')).toBeInTheDocument();
    });

    it('displays provider names capitalized', () => {
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

      const { container } = renderListView(repositories);

      const cells = container.querySelectorAll('td');
      const providerCells = Array.from(cells).filter((cell) =>
        cell.className.includes('capitalize')
      );

      expect(providerCells.length).toBeGreaterThan(0);
    });
  });

  describe('External Links', () => {
    it('includes links to repositories', () => {
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

      renderListView(repositories);

      const links = screen.getAllByRole('link');
      const repoLink = links.find((link) =>
        link.getAttribute('href')?.includes('github.com/owner1/repo1')
      );
      expect(repoLink).toBeInTheDocument();
      expect(repoLink?.getAttribute('target')).toBe('_blank');
      expect(repoLink?.getAttribute('rel')).toBe('noopener noreferrer');
    });
  });

  describe('Status Badges', () => {
    it('renders status badges for each repository', () => {
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
          provider: 'github',
          openPrCount: 3,
          status: 'red',
          url: 'https://github.com/owner2/repo2',
        },
      ];

      const { container } = renderListView(repositories);

      const greenBadge = container.querySelector('[data-status="green"]');
      const redBadge = container.querySelector('[data-status="red"]');

      expect(greenBadge).toBeInTheDocument();
      expect(redBadge).toBeInTheDocument();
    });
  });
});
