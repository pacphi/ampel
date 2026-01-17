import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import StatusBadge from './StatusBadge';
import MergeDialog from './MergeDialog';
import { Button } from '@/components/ui/button';
import { formatRelativeTime } from '@/lib/utils';
import type { PullRequestWithDetails } from '@/types';
import {
  ExternalLink,
  GitMerge,
  User,
  GitBranch,
  FileCode,
  MessageSquare,
  Eye,
  Clock,
  X,
  AlertCircle,
  CircleDot,
} from 'lucide-react';
import { cn } from '@/lib/utils';

interface PRCardProps {
  pr: PullRequestWithDetails;
  showRepo?: boolean;
  skipReviewRequirement?: boolean;
}

// Get blockers explaining why a PR isn't ready to merge
function getBlockers(
  pr: PullRequestWithDetails,
  skipReviewRequirement = false,
  t: (key: string) => string
) {
  const blockers: { label: string; type: 'warning' | 'error'; icon: React.ReactNode }[] = [];

  if (pr.isDraft) {
    blockers.push({
      label: t('dashboard:blockers.draft'),
      type: 'warning',
      icon: <CircleDot className="h-3 w-3" />,
    });
  }

  if (pr.hasConflicts) {
    blockers.push({
      label: t('dashboard:blockers.conflicts'),
      type: 'error',
      icon: <AlertCircle className="h-3 w-3" />,
    });
  }

  if (pr.ciChecks && pr.ciChecks.length > 0) {
    const hasFailed = pr.ciChecks.some(
      (c) =>
        c.status === 'completed' && (c.conclusion === 'failure' || c.conclusion === 'timed_out')
    );
    const hasPending = pr.ciChecks.some((c) => c.status === 'queued' || c.status === 'in_progress');

    if (hasFailed) {
      blockers.push({
        label: t('dashboard:blockers.ciFailed'),
        type: 'error',
        icon: <X className="h-3 w-3" />,
      });
    } else if (hasPending) {
      blockers.push({
        label: t('dashboard:blockers.ciPending'),
        type: 'warning',
        icon: <Clock className="h-3 w-3" />,
      });
    }
  }

  // Check review status (skip if user has enabled skipReviewRequirement)
  if (!skipReviewRequirement) {
    if (pr.reviews && pr.reviews.length > 0) {
      const hasChangesRequested = pr.reviews.some((r) => r.state === 'changes_requested');
      const hasApproval = pr.reviews.some((r) => r.state === 'approved');

      if (hasChangesRequested) {
        blockers.push({
          label: t('dashboard:blockers.changesRequested'),
          type: 'error',
          icon: <MessageSquare className="h-3 w-3" />,
        });
      } else if (!hasApproval) {
        blockers.push({
          label: t('dashboard:blockers.awaitingReview'),
          type: 'warning',
          icon: <Eye className="h-3 w-3" />,
        });
      }
    } else {
      blockers.push({
        label: t('dashboard:blockers.needsReview'),
        type: 'warning',
        icon: <Eye className="h-3 w-3" />,
      });
    }
  }

  return blockers;
}

export default function PRCard({
  pr,
  showRepo = true,
  skipReviewRequirement = false,
}: PRCardProps) {
  const { t } = useTranslation(['dashboard', 'common']);
  const [mergeOpen, setMergeOpen] = useState(false);
  const canMerge = pr.status === 'green' && pr.isMergeable !== false && !pr.hasConflicts;
  const blockers = pr.status !== 'green' ? getBlockers(pr, skipReviewRequirement, t) : [];

  return (
    <>
      <div className="flex items-start gap-4 p-4 rounded-lg border hover:bg-muted/50 transition-colors">
        <StatusBadge status={pr.status} size="lg" />

        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0">
              <a
                href={pr.url}
                target="_blank"
                rel="noopener noreferrer"
                className="font-medium hover:underline truncate block"
              >
                #{pr.number} {pr.title}
              </a>
              {showRepo && (
                <p className="text-sm text-muted-foreground">
                  {pr.repositoryOwner}/{pr.repositoryName}
                </p>
              )}
            </div>
            <div className="flex items-center gap-2 shrink-0">
              {canMerge && (
                <Button
                  size="sm"
                  onClick={() => setMergeOpen(true)}
                  className="bg-ampel-green hover:bg-ampel-green/90"
                >
                  <GitMerge className="h-4 w-4 mr-1" />
                  {t('dashboard:actions.merge')}
                </Button>
              )}
              <a
                href={pr.url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-muted-foreground hover:text-foreground"
              >
                <ExternalLink className="h-4 w-4" />
              </a>
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-4 mt-2 text-sm text-muted-foreground">
            <span className="flex items-center gap-1">
              <User className="h-3 w-3" />
              {pr.author}
            </span>
            <span className="flex items-center gap-1">
              <GitBranch className="h-3 w-3" />
              {pr.sourceBranch} → {pr.targetBranch}
            </span>
            <span className="flex items-center gap-1">
              <FileCode className="h-3 w-3" />
              <span className="text-green-500">+{pr.additions}</span>
              <span className="text-red-500">-{pr.deletions}</span>
            </span>
            {pr.commentsCount > 0 && (
              <span className="flex items-center gap-1">
                <MessageSquare className="h-3 w-3" />
                {pr.commentsCount}
              </span>
            )}
            {pr.createdAt && <span>{formatRelativeTime(pr.createdAt)}</span>}
          </div>

          {blockers.length > 0 && (
            <div className="flex items-center gap-1.5 mt-2 flex-wrap">
              {blockers.map((blocker, idx) => (
                <span
                  key={idx}
                  className={cn(
                    'inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full',
                    blocker.type === 'error'
                      ? 'bg-destructive/10 text-destructive'
                      : 'bg-yellow-500/10 text-yellow-600 dark:text-yellow-400'
                  )}
                >
                  {blocker.icon}
                  {blocker.label}
                </span>
              ))}
            </div>
          )}
        </div>
      </div>

      <MergeDialog pr={pr} open={mergeOpen} onOpenChange={setMergeOpen} />
    </>
  );
}
