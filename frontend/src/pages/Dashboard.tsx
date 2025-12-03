import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { dashboardApi } from '@/api/dashboard';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import GridView from '@/components/dashboard/GridView';
import ListView from '@/components/dashboard/ListView';
import { Grid, List, RefreshCw, GitPullRequest, Boxes } from 'lucide-react';

type ViewMode = 'grid' | 'list';

export default function Dashboard() {
  const [viewMode, setViewMode] = useState<ViewMode>('grid');

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

  const isLoading = summaryLoading || reposLoading;

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
            >
              <Grid className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === 'list' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('list')}
            >
              <List className="h-4 w-4" />
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
              {isLoading ? '-' : summary?.statusCounts.green}
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

      {/* Repository View */}
      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        </div>
      ) : viewMode === 'grid' ? (
        <GridView repositories={repositories || []} />
      ) : (
        <ListView repositories={repositories || []} />
      )}
    </div>
  );
}
