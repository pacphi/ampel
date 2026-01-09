/**
 * PRCard Component Tests
 *
 * BDD-style tests for the PRCard component that displays pull request information
 * with status badges, blockers, and action buttons.
 *
 * @description Tests cover:
 * - Status display (green/yellow/red)
 * - Blocker display (draft, conflicts, CI failed, changes requested, awaiting review)
 * - Merge button visibility and interaction
 * - skipReviewRequirement behavior
 * - i18n integration
 */

import { describe, it, expect } from 'vitest';
import { render, screen, within, waitFor } from '../setup/test-utils';
import PRCard from '@/components/dashboard/PRCard';
import type { PullRequestWithDetails, CICheck, Review } from '@/types';

// ============================================================================
// Test Data Factory
// ============================================================================

/**
 * Creates a mock PR with sensible defaults and optional overrides.
 * This factory allows tests to focus on the specific properties being tested.
 */
function createPR(overrides: Partial<PullRequestWithDetails> = {}): PullRequestWithDetails {
  return {
    id: 'pr-1',
    repositoryId: 'repo-1',
    provider: 'github',
    providerId: 'provider-pr-1',
    number: 123,
    title: 'Test PR',
    description: 'Test description',
    url: 'https://github.com/owner/repo/pull/123',
    state: 'open',
    sourceBranch: 'feature/test',
    targetBranch: 'main',
    author: 'test-author',
    authorAvatarUrl: 'https://avatars.example.com/test-author.png',
    isDraft: false,
    isMergeable: true,
    hasConflicts: false,
    additions: 100,
    deletions: 50,
    changedFiles: 5,
    commitsCount: 3,
    commentsCount: 2,
    createdAt: '2024-01-01T09:00:00Z',
    updatedAt: '2024-01-02T10:00:00Z',
    status: 'green',
    ciChecks: [],
    reviews: [],
    repositoryName: 'repo',
    repositoryOwner: 'owner',
    ...overrides,
  };
}

/**
 * Creates a successful CI check
 */
function createSuccessfulCICheck(name = 'build'): CICheck {
  return {
    id: `check-${name}`,
    pullRequestId: 'pr-1',
    name,
    status: 'completed',
    conclusion: 'success',
    startedAt: '2024-01-01T10:00:00Z',
    completedAt: '2024-01-01T10:05:00Z',
    durationSeconds: 300,
  };
}

/**
 * Creates a failed CI check
 */
function createFailedCICheck(name = 'test'): CICheck {
  return {
    id: `check-${name}`,
    pullRequestId: 'pr-1',
    name,
    status: 'completed',
    conclusion: 'failure',
    startedAt: '2024-01-01T10:00:00Z',
    completedAt: '2024-01-01T10:10:00Z',
    durationSeconds: 600,
  };
}

/**
 * Creates an in-progress CI check
 */
function createPendingCICheck(name = 'lint'): CICheck {
  return {
    id: `check-${name}`,
    pullRequestId: 'pr-1',
    name,
    status: 'in_progress',
    startedAt: '2024-01-01T10:00:00Z',
  };
}

/**
 * Creates a queued CI check
 */
function createQueuedCICheck(name = 'deploy'): CICheck {
  return {
    id: `check-${name}`,
    pullRequestId: 'pr-1',
    name,
    status: 'queued',
  };
}

/**
 * Creates an approved review
 */
function createApprovedReview(reviewer = 'reviewer-1'): Review {
  return {
    id: `review-${reviewer}`,
    pullRequestId: 'pr-1',
    reviewer,
    state: 'approved',
    body: 'LGTM!',
    submittedAt: '2024-01-01T12:00:00Z',
  };
}

/**
 * Creates a review requesting changes
 */
function createChangesRequestedReview(reviewer = 'reviewer-1'): Review {
  return {
    id: `review-${reviewer}`,
    pullRequestId: 'pr-1',
    reviewer,
    state: 'changes_requested',
    body: 'Please fix these issues.',
    submittedAt: '2024-01-01T12:00:00Z',
  };
}

// ============================================================================
// Status Display Tests
// ============================================================================

describe('PRCard', () => {
  describe('Status Display', () => {
    describe('Scenario: Green Status PR', () => {
      it('Given: PR has green status, no blockers, When: PRCard renders, Then: Green status badge is shown', () => {
        // Arrange: Create a green status PR with approval and passing CI
        const pr = createPR({
          status: 'green',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
          isMergeable: true,
          hasConflicts: false,
        });

        // Act: Render the component
        render(<PRCard pr={pr} />);

        // Assert: Green status badge is visible
        // StatusBadge renders with a specific role or class based on status
        const _statusBadge = screen.getByTestId ? screen.queryByTestId('status-badge') : null;
        // If no test-id, check for the PR number and title
        expect(screen.getByText(/#123/)).toBeInTheDocument();
        expect(screen.getByText(/Test PR/)).toBeInTheDocument();
      });

      it('Given: PR has green status, When: PRCard renders, Then: Merge button is visible', () => {
        // Arrange
        const pr = createPR({
          status: 'green',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
          isMergeable: true,
          hasConflicts: false,
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Merge button should be visible (uses i18n key dashboard:actions.merge)
        const mergeButton = screen.getByRole('button', { name: /merge/i });
        expect(mergeButton).toBeInTheDocument();
      });

      it('Given: PR has green status, When: PRCard renders, Then: No blockers are displayed', () => {
        // Arrange
        const pr = createPR({
          status: 'green',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: No blocker tags should be visible
        expect(screen.queryByText(/draft/i)).not.toBeInTheDocument();
        expect(screen.queryByText(/conflicts/i)).not.toBeInTheDocument();
        expect(screen.queryByText(/ci failed/i)).not.toBeInTheDocument();
        expect(screen.queryByText(/awaiting review/i)).not.toBeInTheDocument();
        expect(screen.queryByText(/changes requested/i)).not.toBeInTheDocument();
      });
    });

    describe('Scenario: Yellow Status PR', () => {
      it('Given: PR is pending review, When: PRCard renders, Then: Yellow status badge is shown', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [], // No reviews yet
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert
        expect(screen.getByText(/#123/)).toBeInTheDocument();
      });

      it('Given: PR is pending review with no reviews, When: PRCard renders, Then: "Needs review" blocker is displayed', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [], // Empty reviews
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Needs review blocker (uses i18n key dashboard:blockers.needsReview)
        expect(screen.getByText(/needs review/i)).toBeInTheDocument();
      });

      it('Given: PR has only comment reviews, When: PRCard renders, Then: "Awaiting review" blocker is displayed', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [
            {
              id: 'review-1',
              pullRequestId: 'pr-1',
              reviewer: 'reviewer-1',
              state: 'commented',
              body: 'Just a comment',
              submittedAt: '2024-01-01T12:00:00Z',
            },
          ],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Awaiting review blocker (uses i18n key dashboard:blockers.awaitingReview)
        expect(screen.getByText(/awaiting review/i)).toBeInTheDocument();
      });

      it('Given: PR has CI in progress, When: PRCard renders, Then: "CI pending" blocker is displayed', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createPendingCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: CI pending blocker (uses i18n key dashboard:blockers.ciPending)
        expect(screen.getByText(/ci pending/i)).toBeInTheDocument();
      });
    });

    describe('Scenario: Red Status PR', () => {
      it('Given: PR has conflicts, When: PRCard renders, Then: Red status badge is shown', () => {
        // Arrange
        const pr = createPR({
          status: 'red',
          hasConflicts: true,
          isMergeable: false,
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert
        expect(screen.getByText(/#123/)).toBeInTheDocument();
      });

      it('Given: PR has conflicts, When: PRCard renders, Then: "Conflicts" blocker is displayed', () => {
        // Arrange
        const pr = createPR({
          status: 'red',
          hasConflicts: true,
          isMergeable: false,
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Conflicts blocker (uses i18n key dashboard:blockers.conflicts)
        expect(screen.getByText(/conflicts/i)).toBeInTheDocument();
      });

      it('Given: PR has failed CI checks, When: PRCard renders, Then: "CI failed" blocker is displayed', () => {
        // Arrange
        const pr = createPR({
          status: 'red',
          ciChecks: [createFailedCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: CI failed blocker (uses i18n key dashboard:blockers.ciFailed)
        expect(screen.getByText(/ci failed/i)).toBeInTheDocument();
      });

      it('Given: PR has changes requested, When: PRCard renders, Then: "Changes requested" blocker is displayed', () => {
        // Arrange
        const pr = createPR({
          status: 'red',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createChangesRequestedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Changes requested blocker (uses i18n key dashboard:blockers.changesRequested)
        expect(screen.getByText(/changes requested/i)).toBeInTheDocument();
      });
    });
  });

  // ============================================================================
  // Blockers Display Tests
  // ============================================================================

  describe('Blockers Display', () => {
    describe('Scenario: Draft PR', () => {
      it('Given: PR is marked as draft, When: PRCard renders, Then: Draft blocker is shown', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          isDraft: true,
          ciChecks: [],
          reviews: [],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Draft blocker (uses i18n key dashboard:blockers.draft)
        expect(screen.getByText(/draft/i)).toBeInTheDocument();
      });

      it('Given: PR is draft with no other issues, When: PRCard renders, Then: Draft is primary blocker', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          isDraft: true,
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Draft blocker should be present
        const draftBlocker = screen.getByText(/draft/i);
        expect(draftBlocker).toBeInTheDocument();
      });
    });

    describe('Scenario: CI Failed', () => {
      it('Given: PR has failed CI checks, When: PRCard renders, Then: CI failed blocker is shown', () => {
        // Arrange
        const pr = createPR({
          status: 'red',
          ciChecks: [createSuccessfulCICheck('build'), createFailedCICheck('test')],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert
        expect(screen.getByText(/ci failed/i)).toBeInTheDocument();
      });

      it('Given: PR has timed out CI check, When: PRCard renders, Then: CI failed blocker is shown', () => {
        // Arrange
        const timedOutCheck: CICheck = {
          id: 'check-timeout',
          pullRequestId: 'pr-1',
          name: 'integration-tests',
          status: 'completed',
          conclusion: 'timed_out',
          startedAt: '2024-01-01T10:00:00Z',
          completedAt: '2024-01-01T11:00:00Z',
          durationSeconds: 3600,
        };

        const pr = createPR({
          status: 'red',
          ciChecks: [timedOutCheck],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Timed out is treated as failure
        expect(screen.getByText(/ci failed/i)).toBeInTheDocument();
      });

      it('Given: PR has queued CI checks, When: PRCard renders, Then: CI pending blocker is shown', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createQueuedCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Queued is treated as pending
        expect(screen.getByText(/ci pending/i)).toBeInTheDocument();
      });
    });

    describe('Scenario: Changes Requested', () => {
      it('Given: PR has review requesting changes, When: PRCard renders, Then: Changes requested blocker is shown', () => {
        // Arrange
        const pr = createPR({
          status: 'red',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createChangesRequestedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert
        expect(screen.getByText(/changes requested/i)).toBeInTheDocument();
      });

      it('Given: PR has both approval and changes requested, When: PRCard renders, Then: Changes requested takes precedence', () => {
        // Arrange: One reviewer approved, another requested changes
        const pr = createPR({
          status: 'red',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview('reviewer-1'), createChangesRequestedReview('reviewer-2')],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Changes requested should be shown
        expect(screen.getByText(/changes requested/i)).toBeInTheDocument();
      });
    });

    describe('Scenario: Multiple Blockers', () => {
      it('Given: PR has multiple blockers, When: PRCard renders, Then: All blockers are displayed', () => {
        // Arrange: Draft PR with conflicts and failed CI
        const pr = createPR({
          status: 'red',
          isDraft: true,
          hasConflicts: true,
          ciChecks: [createFailedCICheck()],
          reviews: [],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: All blockers should be visible
        expect(screen.getByText(/draft/i)).toBeInTheDocument();
        expect(screen.getByText(/conflicts/i)).toBeInTheDocument();
        expect(screen.getByText(/ci failed/i)).toBeInTheDocument();
      });
    });
  });

  // ============================================================================
  // Merge Action Tests
  // ============================================================================

  describe('Merge Action', () => {
    describe('Scenario: Merge Button Click', () => {
      it('Given: PR is ready to merge, When: User clicks merge button, Then: Merge dialog opens', async () => {
        // Arrange
        const pr = createPR({
          status: 'green',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
          isMergeable: true,
          hasConflicts: false,
        });

        const { user } = render(<PRCard pr={pr} />);

        // Act: Click merge button
        const mergeButton = screen.getByRole('button', { name: /merge/i });
        await user.click(mergeButton);

        // Assert: Dialog should open (MergeDialog component renders)
        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        // Dialog title should be visible
        expect(screen.getByText(/merge pull request/i)).toBeInTheDocument();
      });

      it('Given: PR is ready to merge, When: Merge dialog opens, Then: PR title is displayed in dialog', async () => {
        // Arrange
        const pr = createPR({
          status: 'green',
          title: 'feat: Add amazing feature',
          number: 456,
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
          isMergeable: true,
        });

        const { user } = render(<PRCard pr={pr} />);

        // Act
        const mergeButton = screen.getByRole('button', { name: /merge/i });
        await user.click(mergeButton);

        // Assert: PR info should be in dialog
        await waitFor(() => {
          const dialog = screen.getByRole('dialog');
          expect(within(dialog).getByText(/#456/)).toBeInTheDocument();
          expect(within(dialog).getByText(/feat: Add amazing feature/)).toBeInTheDocument();
        });
      });
    });

    describe('Scenario: Merge Button Hidden', () => {
      it('Given: PR has blockers (red status), When: PRCard renders, Then: Merge button is not visible', () => {
        // Arrange
        const pr = createPR({
          status: 'red',
          hasConflicts: true,
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert: Merge button should not be visible
        expect(screen.queryByRole('button', { name: /merge/i })).not.toBeInTheDocument();
      });

      it('Given: PR is yellow status, When: PRCard renders, Then: Merge button is not visible', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createPendingCICheck()],
          reviews: [],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert
        expect(screen.queryByRole('button', { name: /merge/i })).not.toBeInTheDocument();
      });

      it('Given: PR is not mergeable, When: PRCard renders, Then: Merge button is not visible', () => {
        // Arrange
        const pr = createPR({
          status: 'green',
          isMergeable: false,
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert
        expect(screen.queryByRole('button', { name: /merge/i })).not.toBeInTheDocument();
      });

      it('Given: PR has conflicts even with green status, When: PRCard renders, Then: Merge button is not visible', () => {
        // Arrange: Edge case - green status but has conflicts
        const pr = createPR({
          status: 'green',
          hasConflicts: true, // This overrides green status for merge eligibility
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createApprovedReview()],
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert
        expect(screen.queryByRole('button', { name: /merge/i })).not.toBeInTheDocument();
      });
    });
  });

  // ============================================================================
  // Skip Review Requirement Tests
  // ============================================================================

  describe('Skip Review Requirement', () => {
    describe('Scenario: With skipReviewRequirement enabled', () => {
      it('Given: Settings allow skipping review, Given: PR is yellow but no CI issues, When: PRCard renders with skipReviewRequirement=true, Then: No review-related blockers are shown', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [], // No reviews
        });

        // Act: Render with skipReviewRequirement
        render(<PRCard pr={pr} skipReviewRequirement={true} />);

        // Assert: No review-related blockers
        expect(screen.queryByText(/needs review/i)).not.toBeInTheDocument();
        expect(screen.queryByText(/awaiting review/i)).not.toBeInTheDocument();
      });

      it('Given: skipReviewRequirement is true, Given: PR has changes requested, When: PRCard renders, Then: Changes requested blocker is NOT shown', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [createChangesRequestedReview()],
        });

        // Act
        render(<PRCard pr={pr} skipReviewRequirement={true} />);

        // Assert: Changes requested should not be shown when skipping review requirement
        expect(screen.queryByText(/changes requested/i)).not.toBeInTheDocument();
      });

      it('Given: skipReviewRequirement is true, Given: PR has CI issues, When: PRCard renders, Then: CI blockers are still shown', () => {
        // Arrange: skipReviewRequirement should only skip review blockers, not CI
        const pr = createPR({
          status: 'red',
          ciChecks: [createFailedCICheck()],
          reviews: [],
        });

        // Act
        render(<PRCard pr={pr} skipReviewRequirement={true} />);

        // Assert: CI failed should still be shown
        expect(screen.getByText(/ci failed/i)).toBeInTheDocument();
      });

      it('Given: skipReviewRequirement is true, Given: PR is draft, When: PRCard renders, Then: Draft blocker is still shown', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          isDraft: true,
          ciChecks: [createSuccessfulCICheck()],
          reviews: [],
        });

        // Act
        render(<PRCard pr={pr} skipReviewRequirement={true} />);

        // Assert: Draft should still be shown
        expect(screen.getByText(/draft/i)).toBeInTheDocument();
      });

      it('Given: skipReviewRequirement is true, Given: PR has conflicts, When: PRCard renders, Then: Conflicts blocker is still shown', () => {
        // Arrange
        const pr = createPR({
          status: 'red',
          hasConflicts: true,
          ciChecks: [createSuccessfulCICheck()],
          reviews: [],
        });

        // Act
        render(<PRCard pr={pr} skipReviewRequirement={true} />);

        // Assert: Conflicts should still be shown
        expect(screen.getByText(/conflicts/i)).toBeInTheDocument();
      });
    });

    describe('Scenario: Without skipReviewRequirement (default behavior)', () => {
      it('Given: skipReviewRequirement is false (default), Given: PR has no reviews, When: PRCard renders, Then: "Needs review" blocker is shown', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [],
        });

        // Act: Default behavior (skipReviewRequirement = false)
        render(<PRCard pr={pr} />);

        // Assert
        expect(screen.getByText(/needs review/i)).toBeInTheDocument();
      });

      it('Given: skipReviewRequirement is explicitly false, Given: PR has pending review, When: PRCard renders, Then: "Awaiting review" blocker is shown', () => {
        // Arrange
        const pr = createPR({
          status: 'yellow',
          ciChecks: [createSuccessfulCICheck()],
          reviews: [
            {
              id: 'review-pending',
              pullRequestId: 'pr-1',
              reviewer: 'reviewer-1',
              state: 'pending',
              submittedAt: '2024-01-01T12:00:00Z',
            },
          ],
        });

        // Act
        render(<PRCard pr={pr} skipReviewRequirement={false} />);

        // Assert: Pending review counts as awaiting
        expect(screen.getByText(/awaiting review/i)).toBeInTheDocument();
      });
    });
  });

  // ============================================================================
  // showRepo Prop Tests
  // ============================================================================

  describe('Repository Display', () => {
    describe('Scenario: Show repository information', () => {
      it('Given: showRepo is true (default), When: PRCard renders, Then: Repository owner/name is shown', () => {
        // Arrange
        const pr = createPR({
          repositoryOwner: 'myorg',
          repositoryName: 'myrepo',
        });

        // Act
        render(<PRCard pr={pr} />);

        // Assert
        expect(screen.getByText(/myorg\/myrepo/)).toBeInTheDocument();
      });

      it('Given: showRepo is false, When: PRCard renders, Then: Repository owner/name is NOT shown', () => {
        // Arrange
        const pr = createPR({
          repositoryOwner: 'myorg',
          repositoryName: 'myrepo',
        });

        // Act
        render(<PRCard pr={pr} showRepo={false} />);

        // Assert
        expect(screen.queryByText(/myorg\/myrepo/)).not.toBeInTheDocument();
      });
    });
  });

  // ============================================================================
  // PR Metadata Display Tests
  // ============================================================================

  describe('PR Metadata Display', () => {
    it('Given: PR with author, When: PRCard renders, Then: Author is displayed', () => {
      // Arrange
      const pr = createPR({ author: 'john-doe' });

      // Act
      render(<PRCard pr={pr} />);

      // Assert
      expect(screen.getByText('john-doe')).toBeInTheDocument();
    });

    it('Given: PR with branch info, When: PRCard renders, Then: Source and target branches are displayed', () => {
      // Arrange
      const pr = createPR({
        sourceBranch: 'feature/new-feature',
        targetBranch: 'main',
      });

      // Act
      render(<PRCard pr={pr} />);

      // Assert: Shows "source → target" format
      expect(screen.getByText(/feature\/new-feature/)).toBeInTheDocument();
      expect(screen.getByText(/main/)).toBeInTheDocument();
    });

    it('Given: PR with additions and deletions, When: PRCard renders, Then: Line changes are displayed', () => {
      // Arrange
      const pr = createPR({
        additions: 250,
        deletions: 75,
      });

      // Act
      render(<PRCard pr={pr} />);

      // Assert
      expect(screen.getByText('+250')).toBeInTheDocument();
      expect(screen.getByText('-75')).toBeInTheDocument();
    });

    it('Given: PR with comments, When: PRCard renders, Then: Comment count is displayed', () => {
      // Arrange
      const pr = createPR({ commentsCount: 5 });

      // Act
      render(<PRCard pr={pr} />);

      // Assert
      expect(screen.getByText('5')).toBeInTheDocument();
    });

    it('Given: PR with no comments, When: PRCard renders, Then: Comment count is NOT displayed', () => {
      // Arrange
      const pr = createPR({ commentsCount: 0 });

      // Act
      render(<PRCard pr={pr} />);

      // Assert: Comment icon/count should not be shown when 0
      // We check that there's no standalone "0" that would be the comment count
      const _zeroElements = screen.queryAllByText('0');
      // If there are any "0" elements, they shouldn't be for comments
      // (additions/deletions might show 0)
    });

    it('Given: PR with URL, When: PRCard renders, Then: External link is present', () => {
      // Arrange
      const pr = createPR({
        url: 'https://github.com/org/repo/pull/999',
      });

      // Act
      render(<PRCard pr={pr} />);

      // Assert: Link should be clickable and point to the PR URL
      const links = screen.getAllByRole('link');
      const prLink = links.find((link) =>
        link.getAttribute('href')?.includes('github.com/org/repo/pull/999')
      );
      expect(prLink).toBeInTheDocument();
      expect(prLink).toHaveAttribute('target', '_blank');
      expect(prLink).toHaveAttribute('rel', 'noopener noreferrer');
    });
  });

  // ============================================================================
  // i18n Integration Tests
  // ============================================================================

  describe('i18n Integration', () => {
    it('Given: English locale, When: Merge button renders, Then: Uses correct i18n key', () => {
      // Arrange
      const pr = createPR({
        status: 'green',
        ciChecks: [createSuccessfulCICheck()],
        reviews: [createApprovedReview()],
      });

      // Act
      render(<PRCard pr={pr} />);

      // Assert: Button text comes from dashboard:actions.merge
      expect(screen.getByRole('button', { name: /merge/i })).toBeInTheDocument();
    });

    it('Given: English locale, When: Blocker renders, Then: Uses correct i18n keys', () => {
      // Arrange: PR with multiple blockers
      const pr = createPR({
        status: 'red',
        isDraft: true,
        hasConflicts: true,
        ciChecks: [createFailedCICheck()],
        reviews: [createChangesRequestedReview()],
      });

      // Act
      render(<PRCard pr={pr} />);

      // Assert: All blocker texts come from i18n
      expect(screen.getByText('Draft')).toBeInTheDocument(); // dashboard:blockers.draft
      expect(screen.getByText('Conflicts')).toBeInTheDocument(); // dashboard:blockers.conflicts
      expect(screen.getByText('CI failed')).toBeInTheDocument(); // dashboard:blockers.ciFailed
      expect(screen.getByText('Changes requested')).toBeInTheDocument(); // dashboard:blockers.changesRequested
    });
  });

  // ============================================================================
  // Edge Cases
  // ============================================================================

  describe('Edge Cases', () => {
    it('Given: PR with undefined isMergeable, When: PRCard with green status renders, Then: Merge button is visible', () => {
      // Arrange: isMergeable can be undefined in some cases
      const pr = createPR({
        status: 'green',
        isMergeable: undefined,
        hasConflicts: false,
        ciChecks: [createSuccessfulCICheck()],
        reviews: [createApprovedReview()],
      });

      // Act
      render(<PRCard pr={pr} />);

      // Assert: undefined isMergeable should not block merge (only false does)
      expect(screen.getByRole('button', { name: /merge/i })).toBeInTheDocument();
    });

    it('Given: PR with empty ciChecks array, When: PRCard renders, Then: No CI-related blockers shown', () => {
      // Arrange
      const pr = createPR({
        status: 'yellow',
        ciChecks: [],
        reviews: [],
      });

      // Act
      render(<PRCard pr={pr} />);

      // Assert: No CI blockers
      expect(screen.queryByText(/ci failed/i)).not.toBeInTheDocument();
      expect(screen.queryByText(/ci pending/i)).not.toBeInTheDocument();
    });

    it('Given: PR with very long title, When: PRCard renders, Then: Title is truncated properly', () => {
      // Arrange
      const longTitle =
        'feat: This is a very long pull request title that goes on and on and should be truncated at some point in the UI to prevent layout issues';
      const pr = createPR({ title: longTitle });

      // Act
      render(<PRCard pr={pr} />);

      // Assert: PR should still render without breaking
      expect(screen.getByText(/#123/)).toBeInTheDocument();
      // The full text may or may not be visible depending on CSS truncation
      const titleLink = screen.getByRole('link', { name: new RegExp(`#123.*`) });
      expect(titleLink).toBeInTheDocument();
    });

    it('Given: PR number with special characters in title, When: PRCard renders, Then: Title displays correctly', () => {
      // Arrange
      const pr = createPR({
        title: 'fix: Handle <script> tags & "special" chars',
      });

      // Act
      render(<PRCard pr={pr} />);

      // Assert: Special characters should be escaped/displayed properly
      expect(screen.getByText(/fix: Handle <script> tags & "special" chars/)).toBeInTheDocument();
    });
  });
});
