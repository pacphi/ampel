import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { remediationApi } from '@/api/remediation';
import type { ListRunsFilters } from '@/types/remediation';

export const remediationRunKeys = {
  all: ['remediation', 'runs'] as const,
  list: (filters: ListRunsFilters) => ['remediation', 'runs', 'list', filters] as const,
  detail: (id: string) => ['remediation', 'runs', 'detail', id] as const,
};

/** Default poll interval (ms) for the run list while no SSE stream is open. */
export const RUNS_POLL_INTERVAL_MS = 15000;

/** Run history (newest first), optionally filtered. */
export function useRemediationRuns(
  filters: ListRunsFilters = {},
  pollIntervalMs: number = RUNS_POLL_INTERVAL_MS
) {
  return useQuery({
    queryKey: remediationRunKeys.list(filters),
    queryFn: () => remediationApi.listRuns(filters),
    refetchInterval: pollIntervalMs,
  });
}

/** Run detail (dispositions + CI matrix + conflict report). */
export function useRemediationRun(id: string) {
  return useQuery({
    queryKey: remediationRunKeys.detail(id),
    queryFn: () => remediationApi.getRun(id),
    enabled: !!id,
  });
}

export function useApproveRun() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => remediationApi.approveRun(id),
    onSuccess: (_data, id) => {
      queryClient.invalidateQueries({ queryKey: remediationRunKeys.detail(id) });
      queryClient.invalidateQueries({ queryKey: remediationRunKeys.all });
    },
  });
}

export function useCancelRun() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => remediationApi.cancelRun(id),
    onSuccess: (_data, id) => {
      queryClient.invalidateQueries({ queryKey: remediationRunKeys.detail(id) });
      queryClient.invalidateQueries({ queryKey: remediationRunKeys.all });
    },
  });
}

export function useTriggerRun() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (repoId: string) => remediationApi.triggerRun(repoId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: remediationRunKeys.all });
      queryClient.invalidateQueries({ queryKey: ['remediation', 'fleet'] });
    },
  });
}
