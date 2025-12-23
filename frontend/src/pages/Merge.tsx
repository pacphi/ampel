import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { pullRequestsApi } from '@/api/pullRequests';
import { mergeApi, type BulkMergeRequest, type BulkMergeResponse } from '@/api/merge';
import { settingsApi } from '@/api/settings';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useToast } from '@/components/ui/use-toast';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import {
  GitMerge,
  Check,
  X,
  Clock,
  AlertCircle,
  ExternalLink,
  Eye,
  MessageSquare,
  CircleDot,
  GitBranch,
  ChevronRight,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import type { PullRequestWithDetails } from '@/types';

// Group PRs by repository
function groupByRepo(prs: PullRequestWithDetails[]) {
  const groups: Record<string, { owner: string; name: string; prs: PullRequestWithDetails[] }> = {};

  for (const pr of prs) {
    const key = `${pr.repositoryOwner}/${pr.repositoryName}`;
    if (!groups[key]) {
      groups[key] = {
        owner: pr.repositoryOwner,
        name: pr.repositoryName,
        prs: [],
      };
    }
    groups[key].prs.push(pr);
  }

  // Sort by repo name
  return Object.values(groups).sort((a, b) =>
    `${a.owner}/${a.name}`.localeCompare(`${b.owner}/${b.name}`)
  );
}
import { MergeResultsDialog } from '@/components/merge/MergeResultsDialog';

// Get blockers explaining why a PR isn't ready to merge
function getBlockers(pr: PullRequestWithDetails, skipReviewRequirement = false) {
  const blockers: { label: string; type: 'warning' | 'error'; icon: React.ReactNode }[] = [];

  // Check for draft
  if (pr.isDraft) {
    blockers.push({
      label: 'Draft',
      type: 'warning',
      icon: <CircleDot className="h-3 w-3" />,
    });
  }

  // Check for conflicts
  if (pr.hasConflicts) {
    blockers.push({
      label: 'Conflicts',
      type: 'error',
      icon: <AlertCircle className="h-3 w-3" />,
    });
  }

  // Check CI status
  if (pr.ciChecks && pr.ciChecks.length > 0) {
    const hasFailed = pr.ciChecks.some(
      (c) =>
        c.status === 'completed' && (c.conclusion === 'failure' || c.conclusion === 'timed_out')
    );
    const hasPending = pr.ciChecks.some((c) => c.status === 'queued' || c.status === 'in_progress');

    if (hasFailed) {
      blockers.push({
        label: 'CI failed',
        type: 'error',
        icon: <X className="h-3 w-3" />,
      });
    } else if (hasPending) {
      blockers.push({
        label: 'CI pending',
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
          label: 'Changes requested',
          type: 'error',
          icon: <MessageSquare className="h-3 w-3" />,
        });
      } else if (!hasApproval) {
        blockers.push({
          label: 'Awaiting review',
          type: 'warning',
          icon: <Eye className="h-3 w-3" />,
        });
      }
    } else {
      // No reviews at all
      blockers.push({
        label: 'Needs review',
        type: 'warning',
        icon: <Eye className="h-3 w-3" />,
      });
    }
  }

  return blockers;
}

export default function Merge() {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [mergeResults, setMergeResults] = useState<BulkMergeResponse | null>(null);
  const [showResults, setShowResults] = useState(false);

  // Fetch user settings for defaults
  const { data: settings } = useQuery({
    queryKey: ['user-settings', 'behavior'],
    queryFn: () => settingsApi.getBehavior(),
    staleTime: 60000,
  });

  // State with defaults - initialized with fallbacks, updated via user actions
  const [strategy, setStrategy] = useState<'merge' | 'squash' | 'rebase' | null>(null);
  const [deleteBranch, setDeleteBranch] = useState<boolean | null>(null);

  // Derive effective values: use user selection if set, otherwise use settings defaults
  const effectiveStrategy = strategy ?? settings?.defaultMergeStrategy ?? 'squash';
  const effectiveDeleteBranch = deleteBranch ?? settings?.deleteBranchesDefault ?? false;

  // Fetch all PRs
  const { data: prsResponse, isLoading } = useQuery({
    queryKey: ['pull-requests'],
    queryFn: () => pullRequestsApi.list(1, 100),
  });

  const prs = prsResponse?.items || [];
  const skipReviewRequirement = settings?.skipReviewRequirement || false;

  // Filter to only mergeable PRs
  // If skipReviewRequirement is enabled, we allow PRs that would be green except for reviews
  const mergeablePrs = prs.filter((pr) => {
    // Must not be draft and have no conflicts
    if (pr.isDraft || pr.hasConflicts) return false;

    // If green status, always mergeable
    if (pr.status === 'green') return true;

    // If skipReviewRequirement is enabled, check if the only blocker is review-related
    if (skipReviewRequirement && pr.status === 'yellow') {
      const blockers = getBlockers(pr, true); // Get blockers without review requirement
      return blockers.length === 0; // Mergeable if no other blockers
    }

    return false;
  });

  const bulkMergeMutation = useMutation({
    mutationFn: (request: BulkMergeRequest) => mergeApi.bulkMerge(request),
    onSuccess: (data) => {
      setMergeResults(data);
      setShowResults(true);
      queryClient.invalidateQueries({ queryKey: ['pull-requests'] });
      setSelectedIds(new Set());

      if (data.failed === 0) {
        toast({
          title: 'Merge successful',
          description: `Successfully merged ${data.success} PR(s)`,
        });
      } else {
        toast({
          variant: 'destructive',
          title: 'Some merges failed',
          description: `${data.success} merged, ${data.failed} failed`,
        });
      }
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Merge failed',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const toggleSelection = (id: string) => {
    const newSelected = new Set(selectedIds);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    setSelectedIds(newSelected);
  };

  const selectAll = () => {
    if (selectedIds.size === mergeablePrs.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(mergeablePrs.map((pr) => pr.id)));
    }
  };

  const handleMerge = () => {
    if (selectedIds.size === 0) return;

    bulkMergeMutation.mutate({
      pullRequestIds: Array.from(selectedIds),
      strategy: effectiveStrategy,
      deleteBranch: effectiveDeleteBranch,
    });
  };

  const getStatusIcon = (pr: PullRequestWithDetails) => {
    if (pr.status === 'green') return <Check className="h-4 w-4 text-green-500" />;
    if (pr.status === 'yellow') return <Clock className="h-4 w-4 text-yellow-500" />;
    if (pr.status === 'red') return <X className="h-4 w-4 text-red-500" />;
    return <AlertCircle className="h-4 w-4 text-muted-foreground" />;
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <GitMerge className="h-6 w-6" />
            Bulk Merge
          </h1>
          <p className="text-muted-foreground">Select and merge multiple pull requests at once</p>
        </div>
      </div>

      {/* Merge Controls */}
      <Card>
        <CardHeader>
          <CardTitle>Merge Options</CardTitle>
          <CardDescription>Configure how selected PRs will be merged</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap gap-6">
            <div className="space-y-2">
              <Label>Merge Strategy</Label>
              <Select
                value={effectiveStrategy}
                onValueChange={(v) => setStrategy(v as 'merge' | 'squash' | 'rebase')}
              >
                <SelectTrigger className="w-[200px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="squash">Squash and merge</SelectItem>
                  <SelectItem value="merge">Create a merge commit</SelectItem>
                  <SelectItem value="rebase">Rebase and merge</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center gap-3 pt-6">
              <Switch
                id="delete-branch"
                checked={effectiveDeleteBranch}
                onCheckedChange={setDeleteBranch}
              />
              <Label htmlFor="delete-branch">Delete branches after merge</Label>
            </div>
          </div>

          <div className="flex items-center justify-between pt-4 border-t">
            <div className="text-sm text-muted-foreground">
              {selectedIds.size} of {mergeablePrs.length} PRs selected
            </div>
            <div className="flex gap-2">
              <Button variant="outline" onClick={selectAll}>
                {selectedIds.size === mergeablePrs.length ? 'Deselect All' : 'Select All'}
              </Button>
              <Button
                onClick={handleMerge}
                disabled={selectedIds.size === 0 || bulkMergeMutation.isPending}
              >
                <GitMerge className="h-4 w-4 mr-2" />
                {bulkMergeMutation.isPending
                  ? 'Merging...'
                  : `Merge ${selectedIds.size} PR${selectedIds.size !== 1 ? 's' : ''}`}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* PR List */}
      <Card>
        <CardHeader>
          <CardTitle>Ready to Merge</CardTitle>
          <CardDescription>PRs that have passed all checks and are ready to merge</CardDescription>
        </CardHeader>
        <CardContent>
          {mergeablePrs.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No PRs are ready to merge. PRs must have:
              <ul className="mt-2 text-sm">
                <li>All CI checks passing</li>
                <li>Required approvals</li>
                <li>No merge conflicts</li>
              </ul>
            </div>
          ) : (
            <div className="space-y-6">
              {groupByRepo(mergeablePrs).map((group) => (
                <div key={`${group.owner}/${group.name}`}>
                  {/* Repository Header */}
                  <div className="flex items-center gap-2 mb-3 pb-2 border-b">
                    <GitBranch className="h-4 w-4 text-muted-foreground" />
                    <span className="font-medium text-sm">
                      {group.owner}/{group.name}
                    </span>
                    <span className="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
                      {group.prs.length} PR{group.prs.length !== 1 ? 's' : ''}
                    </span>
                  </div>
                  {/* PRs in this repo */}
                  <div className="space-y-2 pl-2">
                    {group.prs.map((pr) => (
                      <div
                        key={pr.id}
                        className={cn(
                          'flex items-center gap-4 p-3 rounded-lg border cursor-pointer transition-colors',
                          selectedIds.has(pr.id) ? 'bg-primary/5 border-primary' : 'hover:bg-accent'
                        )}
                        onClick={() => toggleSelection(pr.id)}
                      >
                        <input
                          type="checkbox"
                          checked={selectedIds.has(pr.id)}
                          onChange={() => toggleSelection(pr.id)}
                          className="h-4 w-4 rounded border-gray-300"
                        />
                        {getStatusIcon(pr)}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="font-medium truncate">{pr.title}</span>
                            <span className="text-sm text-muted-foreground shrink-0">
                              #{pr.number}
                            </span>
                          </div>
                          <div className="flex items-center gap-1 text-sm text-muted-foreground">
                            <span className="truncate">{pr.sourceBranch}</span>
                            <ChevronRight className="h-3 w-3 shrink-0" />
                            <span className="truncate">{pr.targetBranch}</span>
                          </div>
                        </div>
                        <a
                          href={pr.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          onClick={(e) => e.stopPropagation()}
                          className="text-muted-foreground hover:text-foreground shrink-0"
                        >
                          <ExternalLink className="h-4 w-4" />
                        </a>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* All PRs (non-mergeable) */}
      {prs.length > mergeablePrs.length && (
        <Card>
          <CardHeader>
            <CardTitle>Not Ready</CardTitle>
            <CardDescription>PRs that need attention before they can be merged</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-6">
              {groupByRepo(prs.filter((pr) => !mergeablePrs.includes(pr))).map((group) => (
                <div key={`${group.owner}/${group.name}`}>
                  {/* Repository Header */}
                  <div className="flex items-center gap-2 mb-3 pb-2 border-b">
                    <GitBranch className="h-4 w-4 text-muted-foreground" />
                    <span className="font-medium text-sm">
                      {group.owner}/{group.name}
                    </span>
                    <span className="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
                      {group.prs.length} PR{group.prs.length !== 1 ? 's' : ''}
                    </span>
                  </div>
                  {/* PRs in this repo */}
                  <div className="space-y-2 pl-2">
                    {group.prs.map((pr) => {
                      const blockers = getBlockers(pr, skipReviewRequirement);
                      return (
                        <div key={pr.id} className="flex items-center gap-4 p-3 rounded-lg border">
                          {getStatusIcon(pr)}
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2">
                              <span className="font-medium truncate">{pr.title}</span>
                              <span className="text-sm text-muted-foreground shrink-0">
                                #{pr.number}
                              </span>
                            </div>
                            <div className="flex items-center gap-1 text-sm text-muted-foreground">
                              <span className="truncate">{pr.sourceBranch}</span>
                              <ChevronRight className="h-3 w-3 shrink-0" />
                              <span className="truncate">{pr.targetBranch}</span>
                            </div>
                            {blockers.length > 0 && (
                              <div className="flex items-center gap-1.5 mt-1.5 flex-wrap">
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
                          <a
                            href={pr.url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-muted-foreground hover:text-foreground shrink-0"
                          >
                            <ExternalLink className="h-4 w-4" />
                          </a>
                        </div>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Results Dialog */}
      <MergeResultsDialog open={showResults} onOpenChange={setShowResults} results={mergeResults} />
    </div>
  );
}
