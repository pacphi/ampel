import { useState, useMemo, useEffect } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { dashboardApi } from '@/api/dashboard';
import { pullRequestsApi } from '@/api/pullRequests';
import { repositoriesApi } from '@/api/repositories';
import { settingsApi } from '@/api/settings';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Card } from '@/components/ui/card';
import GridView from '@/components/dashboard/GridView';
import ListView from '@/components/dashboard/ListView';
import PRListView from '@/components/dashboard/PRListView';
import SummaryBreakdownTile from '@/components/dashboard/SummaryBreakdownTile';
import RefreshProgress from '@/components/dashboard/RefreshProgress';
import ErrorBoundary from '@/components/ErrorBoundary';
import { Grid, List, RefreshCw, GitPullRequest, Boxes, X } from 'lucide-react';
import type { PullRequestWithDetails } from '@/types';
import { useRepositoryFilters } from '@/hooks/useRepositoryFilters';

type ViewMode = 'grid' | 'list' | 'prs';

// Custom icon components for status indicators
const GreenStatusIcon = () => <span className="h-3 w-3 rounded-full bg-ampel-green" />;
const RedStatusIcon = () => <span className="h-3 w-3 rounded-full bg-ampel-red" />;

// Calculate if a PR is ready to merge (same logic as Merge page)
function isReadyToMerge(pr: PullRequestWithDetails, skipReviewRequirement: boolean): boolean {
  // Must not be draft and have no conflicts
  if (pr.isDraft || pr.hasConflicts) return false;

  // If green status, always ready
  if (pr.status === 'green') return true;

  // If skipReviewRequirement is enabled and status is yellow, check if review is the only blocker
  if (skipReviewRequirement && pr.status === 'yellow') {
    // Check for non-review blockers
    const hasCIFailed =
      pr.ciChecks?.some(
        (c) =>
          c.status === 'completed' && (c.conclusion === 'failure' || c.conclusion === 'timed_out')
      ) || false;
    const hasCIPending =
      pr.ciChecks?.some((c) => c.status === 'queued' || c.status === 'in_progress') || false;

    // If no CI issues, then the only blocker must be review-related, so it's ready
    return !hasCIFailed && !hasCIPending;
  }

  return false;
}

export default function Dashboard() {
  const { t } = useTranslation(['dashboard', 'common']);
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const { filters, setFilters, filterRepositories } = useRepositoryFilters();
  const queryClient = useQueryClient();
  const [refreshJobId, setRefreshJobId] = useState<string | null>(null);
  const [showRefreshProgress, setShowRefreshProgress] = useState(false);

  const { data: summary, isLoading: summaryLoading } = useQuery({
    queryKey: ['dashboard', 'summary'],
    queryFn: () => dashboardApi.getSummary(),
  });

  const { data: repositories, isLoading: reposLoading } = useQuery({
    queryKey: ['dashboard', 'grid'],
    queryFn: () => dashboardApi.getGrid(),
  });

  // Fetch PRs and settings to calculate accurate "Ready to Merge" count
  // Using a high limit to ensure we get all PRs for accurate counting
  const { data: prsData } = useQuery({
    queryKey: ['pull-requests'],
    queryFn: () => pullRequestsApi.list(1, 1000),
  });

  // Refresh all dashboard data
  const handleRefresh = async () => {
    setShowRefreshProgress(true);
    try {
      const { jobId } = await repositoriesApi.refreshAll();
      setRefreshJobId(jobId);
    } catch (error) {
      console.error('Failed to start refresh:', error);
      setShowRefreshProgress(false);
      setRefreshJobId(null);
    }
  };

  // Poll for refresh status
  const { data: refreshStatus } = useQuery({
    queryKey: ['refresh-status', refreshJobId],
    queryFn: () => repositoriesApi.getRefreshStatus(refreshJobId!),
    enabled: !!refreshJobId,
    refetchInterval: refreshJobId ? 1000 : false,
  });

  // Derive refreshing state from query
  const isRefreshing = !!refreshJobId && !refreshStatus?.isComplete;

  // Handle refresh completion
  useEffect(() => {
    if (refreshStatus?.isComplete) {
      // Stop polling by clearing the job ID
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setRefreshJobId(null);
      // Invalidate queries to show fresh data
      queryClient.invalidateQueries({ queryKey: ['dashboard'] });
      queryClient.invalidateQueries({ queryKey: ['pull-requests'] });
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    }
  }, [refreshStatus?.isComplete, queryClient]);

  const { data: settings } = useQuery({
    queryKey: ['user-settings', 'behavior'],
    queryFn: () => settingsApi.getBehavior(),
    staleTime: 60000,
  });

  // Create a map of repository ID to repository for visibility lookups
  const repositoryMap = useMemo(() => {
    const map = new Map<string, { isPrivate: boolean; isArchived: boolean }>();
    for (const repo of repositories || []) {
      map.set(repo.id, { isPrivate: repo.isPrivate, isArchived: repo.isArchived });
    }
    return map;
  }, [repositories]);

  // Calculate "Ready to Merge" count and breakdown based on user's skipReviewRequirement setting
  const { readyToMergeCount, readyToMergeBreakdown } = useMemo(() => {
    const prs = prsData?.items || [];
    const skipReviewRequirement = settings?.skipReviewRequirement || false;

    let count = 0;
    const breakdown = { public: 0, private: 0, archived: 0 };

    for (const pr of prs) {
      if (isReadyToMerge(pr, skipReviewRequirement)) {
        count++;
        const repo = repositoryMap.get(pr.repositoryId);
        if (repo) {
          if (repo.isArchived) {
            breakdown.archived++;
          } else if (repo.isPrivate) {
            breakdown.private++;
          } else {
            breakdown.public++;
          }
        }
      }
    }

    return { readyToMergeCount: count, readyToMergeBreakdown: breakdown };
  }, [prsData, settings, repositoryMap]);

  // Calculate "Needs Attention" breakdown (red status PRs)
  const needsAttentionBreakdown = useMemo(() => {
    const prs = prsData?.items || [];
    const breakdown = { public: 0, private: 0, archived: 0 };

    for (const pr of prs) {
      if (pr.status === 'red') {
        const repo = repositoryMap.get(pr.repositoryId);
        if (repo) {
          if (repo.isArchived) {
            breakdown.archived++;
          } else if (repo.isPrivate) {
            breakdown.private++;
          } else {
            breakdown.public++;
          }
        }
      }
    }

    return breakdown;
  }, [prsData, repositoryMap]);

  const isLoading = summaryLoading || reposLoading;

  // Apply visibility filters to repositories
  const filteredRepositories = useMemo(() => {
    return filterRepositories(repositories || []);
  }, [repositories, filterRepositories]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">{t('dashboard:title')}</h1>
        <div className="flex items-center gap-4">
          {viewMode !== 'prs' && (
            <div className="flex items-center gap-2">
              <Checkbox
                id="only-with-prs"
                checked={filters.onlyWithPrs}
                onCheckedChange={(checked) =>
                  setFilters({ ...filters, onlyWithPrs: checked === true })
                }
              />
              <Label htmlFor="only-with-prs" className="text-sm cursor-pointer">
                {t('dashboard:filters.onlyWithPrs')}
              </Label>
            </div>
          )}
          <Button variant="outline" size="sm" onClick={handleRefresh} disabled={isRefreshing}>
            <RefreshCw className={`h-4 w-4 mr-2 ${isRefreshing ? 'animate-spin' : ''}`} />
            {isRefreshing ? t('common:actions.refreshing') : t('common:actions.refresh')}
          </Button>
          <div className="flex border rounded-md">
            <Button
              variant={viewMode === 'grid' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('grid')}
              title={t('dashboard:views.repositoryGrid')}
            >
              <Grid className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === 'list' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('list')}
              title={t('dashboard:views.repositoryList')}
            >
              <List className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === 'prs' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('prs')}
              title={t('dashboard:views.pullRequests')}
            >
              <GitPullRequest className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>

      {/* Refresh Progress Banner */}
      {showRefreshProgress && (
        <Card className="relative border-primary/50 bg-primary/5">
          <Button
            variant="ghost"
            size="sm"
            className="absolute top-2 right-2 h-6 w-6 p-0"
            onClick={() => setShowRefreshProgress(false)}
            aria-label="Close"
          >
            <X className="h-4 w-4" />
          </Button>
          <div className="p-4 pr-12">
            {refreshStatus ? (
              <RefreshProgress status={refreshStatus} />
            ) : (
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-sm">
                  <RefreshCw className="h-4 w-4 animate-spin text-primary" />
                  <span className="text-muted-foreground">
                    Starting refresh of {summary?.totalRepositories || 0} repositories...
                  </span>
                </div>
              </div>
            )}
          </div>
        </Card>
      )}

      {/* Combined Summary + Breakdown Tiles */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <ErrorBoundary>
          <SummaryBreakdownTile
            title={t('dashboard:stats.totalRepositories')}
            count={summary?.totalRepositories || 0}
            breakdown={summary?.repositoryBreakdown || { public: 0, private: 0, archived: 0 }}
            icon={Boxes}
            isLoading={isLoading}
          />
        </ErrorBoundary>
        <ErrorBoundary>
          <SummaryBreakdownTile
            title={t('dashboard:stats.openPRs')}
            count={summary?.totalOpenPrs || 0}
            breakdown={summary?.openPrsBreakdown || { public: 0, private: 0, archived: 0 }}
            icon={GitPullRequest}
            isLoading={isLoading}
          />
        </ErrorBoundary>
        <ErrorBoundary>
          <SummaryBreakdownTile
            title={t('dashboard:stats.readyToMerge')}
            count={readyToMergeCount}
            breakdown={readyToMergeBreakdown}
            icon={GreenStatusIcon}
            isLoading={isLoading}
            countColor="text-ampel-green"
          />
        </ErrorBoundary>
        <ErrorBoundary>
          <SummaryBreakdownTile
            title={t('dashboard:stats.needsAttention')}
            count={summary?.statusCounts.red || 0}
            breakdown={needsAttentionBreakdown}
            icon={RedStatusIcon}
            isLoading={isLoading}
            countColor="text-ampel-red"
          />
        </ErrorBoundary>
      </div>

      {/* Repository/PR View */}
      {viewMode === 'prs' ? (
        <PRListView />
      ) : isLoading ? (
        <div className="flex items-center justify-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        </div>
      ) : viewMode === 'grid' ? (
        <GridView repositories={filteredRepositories} />
      ) : (
        <ListView repositories={filteredRepositories} />
      )}
    </div>
  );
}
