import { useState, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { dashboardApi } from '@/api/dashboard';
import { pullRequestsApi } from '@/api/pullRequests';
import { settingsApi } from '@/api/settings';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import GridView from '@/components/dashboard/GridView';
import ListView from '@/components/dashboard/ListView';
import PRListView from '@/components/dashboard/PRListView';
import BreakdownTile from '@/components/dashboard/BreakdownTile';
import ErrorBoundary from '@/components/ErrorBoundary';
import { Grid, List, RefreshCw, GitPullRequest, Boxes } from 'lucide-react';
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
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const { filterRepositories } = useRepositoryFilters();

  const { data: summary, isLoading: summaryLoading } = useQuery({
    queryKey: ['dashboard', 'summary'],
    queryFn: () => dashboardApi.getSummary(),
  });

  const {
    data: repositories,
    isLoading: reposLoading,
    refetch,
  } = useQuery({
    queryKey: ['dashboard', 'grid'],
    queryFn: () => dashboardApi.getGrid(),
  });

  // Fetch PRs and settings to calculate accurate "Ready to Merge" count
  // Using a high limit to ensure we get all PRs for accurate counting
  const { data: prsData } = useQuery({
    queryKey: ['pull-requests'],
    queryFn: () => pullRequestsApi.list(1, 1000),
  });

  const { data: settings } = useQuery({
    queryKey: ['user-settings', 'behavior'],
    queryFn: () => settingsApi.getBehavior(),
    staleTime: 60000,
  });

  // Calculate "Ready to Merge" count based on user's skipReviewRequirement setting
  const readyToMergeCount = useMemo(() => {
    const prs = prsData?.items || [];
    const skipReviewRequirement = settings?.skipReviewRequirement || false;
    return prs.filter((pr) => isReadyToMerge(pr, skipReviewRequirement)).length;
  }, [prsData, settings]);

  const isLoading = summaryLoading || reposLoading;

  // Apply visibility filters to repositories
  const filteredRepositories = useMemo(() => {
    return filterRepositories(repositories || []);
  }, [repositories, filterRepositories]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Dashboard</h1>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={() => refetch()}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
          <div className="flex border rounded-md">
            <Button
              variant={viewMode === 'grid' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('grid')}
              title="Repository grid view"
            >
              <Grid className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === 'list' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('list')}
              title="Repository list view"
            >
              <List className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === 'prs' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('prs')}
              title="Pull requests view"
            >
              <GitPullRequest className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Total Repositories</CardTitle>
            <Boxes className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{isLoading ? '-' : summary?.totalRepositories}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Open PRs</CardTitle>
            <GitPullRequest className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{isLoading ? '-' : summary?.totalOpenPrs}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Ready to Merge</CardTitle>
            <span className="h-3 w-3 rounded-full bg-ampel-green" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-ampel-green">
              {isLoading ? '-' : readyToMergeCount}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Needs Attention</CardTitle>
            <span className="h-3 w-3 rounded-full bg-ampel-red" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-ampel-red">
              {isLoading ? '-' : summary?.statusCounts.red}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Visibility Breakdown Tiles */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <ErrorBoundary>
          <BreakdownTile
            title="Repositories by Visibility"
            breakdown={summary?.repositoryBreakdown || { public: 0, private: 0, archived: 0 }}
            icon={Boxes}
            isLoading={isLoading}
          />
        </ErrorBoundary>
        <ErrorBoundary>
          <BreakdownTile
            title="Open PRs by Visibility"
            breakdown={summary?.openPrsBreakdown || { public: 0, private: 0, archived: 0 }}
            icon={GitPullRequest}
            isLoading={isLoading}
          />
        </ErrorBoundary>
        <ErrorBoundary>
          <BreakdownTile
            title="Ready to Merge by Visibility"
            breakdown={summary?.readyToMergeBreakdown || { public: 0, private: 0, archived: 0 }}
            icon={GreenStatusIcon}
            isLoading={isLoading}
          />
        </ErrorBoundary>
        <ErrorBoundary>
          <BreakdownTile
            title="Needs Attention by Visibility"
            breakdown={summary?.needsAttentionBreakdown || { public: 0, private: 0, archived: 0 }}
            icon={RedStatusIcon}
            isLoading={isLoading}
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
