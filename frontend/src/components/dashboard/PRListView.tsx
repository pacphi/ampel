import { useState, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { mergeApi, type BulkMergeRequest } from '@/api/merge';
import { settingsApi } from '@/api/settings';
import { useInfinitePullRequests } from '@/hooks/usePullRequests';
import PRCard from './PRCard';
import { Button } from '@/components/ui/button';
import { useToast } from '@/components/ui/use-toast';
import type { AmpelStatus } from '@/types';
import { GitMerge, Loader2, CheckSquare, Square, Filter, ChevronDown } from 'lucide-react';
import { MergeResultsDialog } from '@/components/merge/MergeResultsDialog';
import type { BulkMergeResponse } from '@/api/merge';

interface PRListViewProps {
  filterStatus?: AmpelStatus;
}

export default function PRListView({ filterStatus }: PRListViewProps) {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [selectedPrs, setSelectedPrs] = useState<Set<string>>(new Set());
  const [statusFilter, setStatusFilter] = useState<AmpelStatus | 'all'>(filterStatus || 'all');
  const [mergeResults, setMergeResults] = useState<BulkMergeResponse | null>(null);
  const [showResults, setShowResults] = useState(false);

  // Get user settings for merge defaults
  const { data: settings } = useQuery({
    queryKey: ['user-settings', 'behavior'],
    queryFn: () => settingsApi.getBehavior(),
    staleTime: 60000,
  });

  const {
    data: prsData,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfinitePullRequests(20);

  // Flatten all pages into a single array
  const prs = useMemo(() => {
    return prsData?.pages.flatMap((page) => page.items) || [];
  }, [prsData]);

  // Get total count from the first page
  const totalPrs = prsData?.pages[0]?.total || 0;

  const filteredPrs = statusFilter === 'all' ? prs : prs.filter((pr) => pr.status === statusFilter);

  const mergeablePrs = filteredPrs.filter(
    (pr) => pr.status === 'green' && pr.isMergeable !== false && !pr.hasConflicts
  );

  const selectedMergeablePrs = mergeablePrs.filter((pr) => selectedPrs.has(pr.id));

  const bulkMergeMutation = useMutation({
    mutationFn: (request: BulkMergeRequest) => mergeApi.bulkMerge(request),
    onSuccess: (data) => {
      setMergeResults(data);
      setShowResults(true);
      setSelectedPrs(new Set());
      queryClient.invalidateQueries({ queryKey: ['dashboard'] });
      queryClient.invalidateQueries({ queryKey: ['pull-requests'] });

      if (data.failed === 0) {
        toast({
          title: 'Bulk merge complete',
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
        title: 'Bulk merge failed',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const toggleSelect = (prId: string) => {
    const newSelected = new Set(selectedPrs);
    if (newSelected.has(prId)) {
      newSelected.delete(prId);
    } else {
      newSelected.add(prId);
    }
    setSelectedPrs(newSelected);
  };

  const selectAllMergeable = () => {
    if (selectedPrs.size === mergeablePrs.length) {
      setSelectedPrs(new Set());
    } else {
      setSelectedPrs(new Set(mergeablePrs.map((pr) => pr.id)));
    }
  };

  const bulkMerge = () => {
    if (selectedMergeablePrs.length === 0) return;

    bulkMergeMutation.mutate({
      pullRequestIds: selectedMergeablePrs.map((pr) => pr.id),
      strategy: settings?.defaultMergeStrategy || 'squash',
      deleteBranch: settings?.deleteBranchesDefault || false,
    });
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Toolbar */}
      <div className="flex items-center justify-between gap-4 flex-wrap">
        <div className="flex items-center gap-2">
          <Filter className="h-4 w-4 text-muted-foreground" />
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as AmpelStatus | 'all')}
            className="text-sm border rounded-md px-2 py-1 bg-background"
          >
            <option value="all">All PRs ({totalPrs})</option>
            <option value="green">Ready ({prs.filter((p) => p.status === 'green').length})</option>
            <option value="yellow">
              Pending ({prs.filter((p) => p.status === 'yellow').length})
            </option>
            <option value="red">Blocked ({prs.filter((p) => p.status === 'red').length})</option>
          </select>
        </div>

        {mergeablePrs.length > 0 && (
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={selectAllMergeable}>
              {selectedPrs.size === mergeablePrs.length ? (
                <CheckSquare className="h-4 w-4 mr-1" />
              ) : (
                <Square className="h-4 w-4 mr-1" />
              )}
              Select all mergeable ({mergeablePrs.length})
            </Button>
            {selectedMergeablePrs.length > 0 && (
              <Button
                size="sm"
                onClick={bulkMerge}
                disabled={bulkMergeMutation.isPending}
                className="bg-ampel-green hover:bg-ampel-green/90"
              >
                {bulkMergeMutation.isPending ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                    Merging...
                  </>
                ) : (
                  <>
                    <GitMerge className="h-4 w-4 mr-1" />
                    Merge selected ({selectedMergeablePrs.length})
                  </>
                )}
              </Button>
            )}
          </div>
        )}
      </div>

      {/* PR List */}
      {filteredPrs.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-muted-foreground">No pull requests found</p>
        </div>
      ) : (
        <>
          <div className="space-y-2">
            {filteredPrs.map((pr) => {
              const isMergeable =
                pr.status === 'green' && pr.isMergeable !== false && !pr.hasConflicts;
              return (
                <div key={pr.id} className="flex items-start gap-2">
                  {isMergeable && (
                    <button
                      onClick={() => toggleSelect(pr.id)}
                      className="mt-4 text-muted-foreground hover:text-foreground"
                    >
                      {selectedPrs.has(pr.id) ? (
                        <CheckSquare className="h-5 w-5" />
                      ) : (
                        <Square className="h-5 w-5" />
                      )}
                    </button>
                  )}
                  <div className={isMergeable ? 'flex-1' : 'flex-1 ml-7'}>
                    <PRCard pr={pr} skipReviewRequirement={settings?.skipReviewRequirement} />
                  </div>
                </div>
              );
            })}
          </div>

          {/* Load More Button */}
          {hasNextPage && (
            <div className="flex justify-center pt-4">
              <Button
                variant="outline"
                onClick={() => fetchNextPage()}
                disabled={isFetchingNextPage}
              >
                {isFetchingNextPage ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Loading...
                  </>
                ) : (
                  <>
                    <ChevronDown className="h-4 w-4 mr-2" />
                    Load more PRs ({totalPrs - prs.length} remaining)
                  </>
                )}
              </Button>
            </div>
          )}

          {/* Showing all message */}
          {!hasNextPage && prs.length > 0 && (
            <div className="text-center pt-4">
              <p className="text-sm text-muted-foreground">Showing all {totalPrs} PRs</p>
            </div>
          )}
        </>
      )}

      {/* Merge Results Dialog */}
      <MergeResultsDialog open={showResults} onOpenChange={setShowResults} results={mergeResults} />
    </div>
  );
}
