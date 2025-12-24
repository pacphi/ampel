import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import PRCard from './PRCard';
import type { PullRequestWithDetails } from '@/types';

function renderPRCard(
  pr: Partial<PullRequestWithDetails>,
  showRepo = true,
  skipReviewRequirement = false
) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <PRCard pr={pr} showRepo={showRepo} skipReviewRequirement={skipReviewRequirement} />
    </QueryClientProvider>
  );
}

describe('PRCard', () => {
  describe('Basic Display', () => {
    it('renders PR title and number', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Fix authentication bug',
        status: 'green',
        url: 'https://github.com/test/repo/pull/123',
        author: 'john-doe',
        sourceBranch: 'fix/auth',
        targetBranch: 'main',
        additions: 50,
        deletions: 20,
        commentsCount: 0,
      };

      renderPRCard(pr);

      expect(screen.getByText('#123 Fix authentication bug')).toBeInTheDocument();
    });

    it('displays repository when showRepo is true', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        url: 'https://github.com/acme/app/pull/123',
        repositoryOwner: 'acme',
        repositoryName: 'app',
      };

      renderPRCard(pr, true);

      expect(screen.getByText('acme/app')).toBeInTheDocument();
    });

    it('hides repository when showRepo is false', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        url: 'https://github.com/acme/app/pull/123',
        repositoryOwner: 'acme',
        repositoryName: 'app',
      };

      renderPRCard(pr, false);

      expect(screen.queryByText('acme/app')).not.toBeInTheDocument();
    });

    it('renders author information', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        url: 'https://github.com/test/repo/pull/123',
        author: 'jane-smith',
        sourceBranch: 'feature',
        targetBranch: 'main',
        additions: 100,
        deletions: 50,
        commentsCount: 0,
      };

      renderPRCard(pr);

      expect(screen.getByText('jane-smith')).toBeInTheDocument();
    });

    it('renders branch information', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        url: 'https://github.com/test/repo/pull/123',
        author: 'dev',
        sourceBranch: 'feature/new-feature',
        targetBranch: 'develop',
        additions: 100,
        deletions: 50,
        commentsCount: 0,
      };

      renderPRCard(pr);

      // Branch info is rendered as a single span: "sourceBranch → targetBranch"
      expect(screen.getByText(/feature\/new-feature → develop/)).toBeInTheDocument();
    });

    it('renders code changes statistics', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        url: 'https://github.com/test/repo/pull/123',
        author: 'dev',
        sourceBranch: 'feature',
        targetBranch: 'main',
        additions: 250,
        deletions: 100,
        commentsCount: 0,
      };

      renderPRCard(pr);

      expect(screen.getByText('+250')).toBeInTheDocument();
      expect(screen.getByText('-100')).toBeInTheDocument();
    });

    it('renders comments count when present', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        url: 'https://github.com/test/repo/pull/123',
        author: 'dev',
        sourceBranch: 'feature',
        targetBranch: 'main',
        additions: 50,
        deletions: 20,
        commentsCount: 5,
      };

      renderPRCard(pr);

      expect(screen.getByText('5')).toBeInTheDocument();
    });

    it('hides comments when count is zero', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        url: 'https://github.com/test/repo/pull/123',
        author: 'dev',
        sourceBranch: 'feature',
        targetBranch: 'main',
        additions: 50,
        deletions: 20,
        commentsCount: 0,
      };

      const { container } = renderPRCard(pr);

      const commentIcons = container.querySelectorAll('.lucide-message-square');
      expect(commentIcons.length).toBe(0);
    });
  });

  describe('Status Badge', () => {
    it('renders green status badge', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        url: 'https://github.com/test/repo/pull/123',
      };

      const { container } = renderPRCard(pr);

      // StatusBadge uses bg-ampel-* classes instead of data-status attribute
      const statusBadge = container.querySelector('span.bg-ampel-green');
      expect(statusBadge).toBeTruthy();
    });

    it('renders yellow status badge', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'yellow',
        url: 'https://github.com/test/repo/pull/123',
      };

      const { container } = renderPRCard(pr);

      // StatusBadge uses bg-ampel-* classes instead of data-status attribute
      const statusBadge = container.querySelector('span.bg-ampel-yellow');
      expect(statusBadge).toBeTruthy();
    });

    it('renders red status badge', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'red',
        url: 'https://github.com/test/repo/pull/123',
      };

      const { container } = renderPRCard(pr);

      // StatusBadge uses bg-ampel-* classes instead of data-status attribute
      const statusBadge = container.querySelector('span.bg-ampel-red');
      expect(statusBadge).toBeTruthy();
    });
  });

  describe('Merge Button', () => {
    it('shows merge button for mergeable PRs', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        isMergeable: true,
        hasConflicts: false,
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.getByRole('button', { name: /merge/i })).toBeInTheDocument();
    });

    it('hides merge button for non-mergeable PRs', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'red',
        isMergeable: false,
        hasConflicts: true,
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.queryByRole('button', { name: /merge/i })).not.toBeInTheDocument();
    });

    it('hides merge button when PR has conflicts', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        isMergeable: true,
        hasConflicts: true,
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.queryByRole('button', { name: /merge/i })).not.toBeInTheDocument();
    });
  });

  describe('Blockers Display', () => {
    it('shows draft blocker', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Draft PR',
        status: 'yellow',
        isDraft: true,
        hasConflicts: false,
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.getByText('Draft')).toBeInTheDocument();
    });

    it('shows conflicts blocker', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Conflicted PR',
        status: 'red',
        isDraft: false,
        hasConflicts: true,
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.getByText('Conflicts')).toBeInTheDocument();
    });

    it('shows CI failed blocker', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Failed CI PR',
        status: 'red',
        isDraft: false,
        hasConflicts: false,
        ciChecks: [
          {
            name: 'Tests',
            status: 'completed',
            conclusion: 'failure',
          },
        ],
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.getByText('CI failed')).toBeInTheDocument();
    });

    it('shows CI pending blocker', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Pending CI PR',
        status: 'yellow',
        isDraft: false,
        hasConflicts: false,
        ciChecks: [
          {
            name: 'Tests',
            status: 'in_progress',
            conclusion: null,
          },
        ],
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.getByText('CI pending')).toBeInTheDocument();
    });

    it('shows changes requested blocker', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Changes Requested PR',
        status: 'red',
        isDraft: false,
        hasConflicts: false,
        ciChecks: [],
        reviews: [
          {
            state: 'changes_requested',
            author: 'reviewer',
          },
        ],
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.getByText('Changes requested')).toBeInTheDocument();
    });

    it('shows awaiting review blocker', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Awaiting Review PR',
        status: 'yellow',
        isDraft: false,
        hasConflicts: false,
        ciChecks: [],
        reviews: [
          {
            state: 'commented',
            author: 'reviewer',
          },
        ],
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.getByText('Awaiting review')).toBeInTheDocument();
    });

    it('shows needs review blocker when no reviews', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'No Reviews PR',
        status: 'yellow',
        isDraft: false,
        hasConflicts: false,
        ciChecks: [],
        reviews: [],
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.getByText('Needs review')).toBeInTheDocument();
    });

    it('hides review blockers when skipReviewRequirement is true', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'No Review Required PR',
        status: 'yellow',
        isDraft: false,
        hasConflicts: false,
        ciChecks: [],
        reviews: [],
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr, true, true);

      expect(screen.queryByText('Needs review')).not.toBeInTheDocument();
      expect(screen.queryByText('Awaiting review')).not.toBeInTheDocument();
    });

    it('shows multiple blockers', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Multiple Blockers PR',
        status: 'red',
        isDraft: true,
        hasConflicts: true,
        ciChecks: [
          {
            name: 'Tests',
            status: 'completed',
            conclusion: 'failure',
          },
        ],
        reviews: [
          {
            state: 'changes_requested',
            author: 'reviewer',
          },
        ],
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.getByText('Draft')).toBeInTheDocument();
      expect(screen.getByText('Conflicts')).toBeInTheDocument();
      expect(screen.getByText('CI failed')).toBeInTheDocument();
      expect(screen.getByText('Changes requested')).toBeInTheDocument();
    });

    it('hides blockers for green PRs', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Green PR',
        status: 'green',
        isDraft: false,
        hasConflicts: false,
        ciChecks: [],
        reviews: [{ state: 'approved', author: 'reviewer' }],
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      expect(screen.queryByText('Draft')).not.toBeInTheDocument();
      expect(screen.queryByText('Conflicts')).not.toBeInTheDocument();
      expect(screen.queryByText('CI failed')).not.toBeInTheDocument();
    });
  });

  describe('External Link', () => {
    it('renders external link to PR', () => {
      const pr = {
        id: '1',
        number: 123,
        title: 'Test PR',
        status: 'green',
        url: 'https://github.com/test/repo/pull/123',
      };

      renderPRCard(pr);

      const links = screen.getAllByRole('link');
      const externalLink = links.find((link) => link.getAttribute('href') === pr.url);
      expect(externalLink).toBeInTheDocument();
      expect(externalLink?.getAttribute('target')).toBe('_blank');
      expect(externalLink?.getAttribute('rel')).toBe('noopener noreferrer');
    });
  });
});
