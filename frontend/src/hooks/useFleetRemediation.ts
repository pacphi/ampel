import { useMutation, useQuery } from '@tanstack/react-query';
import { remediationApi } from '@/api/remediation';

export const fleetRemediationKeys = {
  fleet: ['remediation', 'fleet'] as const,
};

/** Default poll interval (ms) for live fleet eligibility updates. */
export const FLEET_POLL_INTERVAL_MS = 15000;

/**
 * Fleet overview query. Uses polling (`refetchInterval`) for live updates —
 * deliberately NOT SSE in Phase 1.
 */
export function useFleetRemediation(pollIntervalMs: number = FLEET_POLL_INTERVAL_MS) {
  return useQuery({
    queryKey: fleetRemediationKeys.fleet,
    queryFn: () => remediationApi.getFleet(),
    refetchInterval: pollIntervalMs,
  });
}

/** Read-only dry-run preview for a single repository. */
export function usePreviewRepository() {
  return useMutation({
    mutationFn: (repoId: string) => remediationApi.previewRepository(repoId),
  });
}
