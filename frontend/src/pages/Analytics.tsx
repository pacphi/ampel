import { useQuery } from '@tanstack/react-query';
import { analyticsApi, type RepositoryHealth } from '@/api/analytics';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  TrendingUp,
  TrendingDown,
  Minus,
  Clock,
  GitPullRequest,
  AlertTriangle,
  Activity,
} from 'lucide-react';

function formatHours(hours: number): string {
  if (hours < 1) {
    return `${Math.round(hours * 60)}m`;
  } else if (hours < 24) {
    return `${hours.toFixed(1)}h`;
  } else {
    return `${(hours / 24).toFixed(1)}d`;
  }
}

function getScoreColor(score: number): string {
  if (score >= 80) return 'text-ampel-green';
  if (score >= 50) return 'text-ampel-yellow';
  return 'text-ampel-red';
}

function getTrendIcon(trend: string) {
  switch (trend) {
    case 'up':
      return <TrendingUp className="h-4 w-4 text-ampel-green" />;
    case 'down':
      return <TrendingDown className="h-4 w-4 text-ampel-red" />;
    default:
      return <Minus className="h-4 w-4 text-muted-foreground" />;
  }
}

function HealthCard({ health }: { health: RepositoryHealth }) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base truncate">{health.repositoryName}</CardTitle>
          <div className="flex items-center gap-1">
            {getTrendIcon(health.trend)}
            <span className={`text-2xl font-bold ${getScoreColor(health.currentScore)}`}>
              {health.currentScore}
            </span>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="flex items-center gap-2">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <div>
              <p className="text-muted-foreground">Merge Time</p>
              <p className="font-medium">{formatHours(health.metrics.avgTimeToMergeHours)}</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <GitPullRequest className="h-4 w-4 text-muted-foreground" />
            <div>
              <p className="text-muted-foreground">Throughput</p>
              <p className="font-medium">{health.metrics.prThroughputPerWeek}/week</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Activity className="h-4 w-4 text-muted-foreground" />
            <div>
              <p className="text-muted-foreground">Review Time</p>
              <p className="font-medium">{formatHours(health.metrics.avgReviewTimeHours)}</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
            <div>
              <p className="text-muted-foreground">Stale PRs</p>
              <p className="font-medium">{health.metrics.stalePrCount}</p>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export default function Analytics() {
  const { data: summary, isLoading: summaryLoading } = useQuery({
    queryKey: ['analytics', 'summary'],
    queryFn: () => analyticsApi.getSummary(30),
  });

  const { data: healthData, isLoading: healthLoading } = useQuery({
    queryKey: ['analytics', 'health'],
    queryFn: () => analyticsApi.getHealthOverview(),
  });

  const isLoading = summaryLoading || healthLoading;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Analytics</h1>

      {/* Summary Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">PRs Merged (30d)</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {isLoading ? '-' : summary?.totalPrsMerged || 0}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Avg Merge Time</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {isLoading ? '-' : formatHours(summary?.avgTimeToMergeHours || 0)}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Avg Review Time</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {isLoading ? '-' : formatHours(summary?.avgReviewTimeHours || 0)}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Bot PRs</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {isLoading ? '-' : `${(summary?.botPrPercentage || 0).toFixed(0)}%`}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Health Overview */}
      <div>
        <h2 className="text-lg font-semibold mb-4">Repository Health Scores</h2>
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          </div>
        ) : healthData && healthData.length > 0 ? (
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {healthData.map((health) => (
              <HealthCard key={health.repositoryId} health={health} />
            ))}
          </div>
        ) : (
          <div className="text-center py-12">
            <p className="text-muted-foreground">No health data available yet</p>
            <p className="text-sm text-muted-foreground mt-1">
              Health scores are calculated hourly based on PR metrics
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
