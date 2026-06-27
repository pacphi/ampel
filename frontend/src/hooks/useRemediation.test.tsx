import { describe, expect, it, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactNode } from 'react';
import {
  useRemediationPolicies,
  useCreateRemediationPolicy,
  useToggleRemediationPolicy,
  useDeleteRemediationPolicy,
} from './useRemediationPolicies';
import { useFleetRemediation, usePreviewRepository } from './useFleetRemediation';
import { remediationApi } from '@/api/remediation';
import type { ConsolidationPlan, FleetRow, RemediationPolicy } from '@/types/remediation';

vi.mock('@/api/remediation', () => ({
  remediationApi: {
    listPolicies: vi.fn(),
    getPolicy: vi.fn(),
    createPolicy: vi.fn(),
    updatePolicy: vi.fn(),
    deletePolicy: vi.fn(),
    togglePolicy: vi.fn(),
    previewRepository: vi.fn(),
    getFleet: vi.fn(),
  },
}));

const mockedApi = vi.mocked(remediationApi);

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

const mockPolicy: RemediationPolicy = {
  id: 'policy-1',
  scopeType: 'repository',
  scopeId: 'repo-1',
  enabled: true,
  minOpenPrs: 2,
  prSelection: 'all_open',
  autonomyLevel: 'dry_run_only',
  remediationTier: 'consolidate_only',
  maxPrsPerRun: 10,
  allowedTargets: ['main'],
  skipDraft: true,
  requireGreenBeforeMerge: true,
  airGapped: false,
  autoMergeEnabled: false,
  autoMergeRule: null,
  requireHumanApproval: false,
  agentBudget: null,
  notificationConfig: null,
  playbookRef: null,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
};

const mockFleetRow: FleetRow = {
  repositoryId: 'repo-1',
  name: 'acme/app',
  openPrCount: 5,
  eligible: true,
  policyState: 'dry_run',
  airGapped: false,
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe('useRemediationPolicies', () => {
  it('should_returnPolicies_when_listSucceeds', async () => {
    mockedApi.listPolicies.mockResolvedValue([mockPolicy]);

    const { result } = renderHook(() => useRemediationPolicies(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual([mockPolicy]);
  });
});

describe('useCreateRemediationPolicy', () => {
  it('should_callCreateApi_when_mutationInvoked', async () => {
    mockedApi.createPolicy.mockResolvedValue(mockPolicy);

    const { result } = renderHook(() => useCreateRemediationPolicy(), { wrapper: createWrapper() });

    result.current.mutate({
      scopeType: 'repository',
      scopeId: 'repo-1',
      minOpenPrs: 2,
      autonomyLevel: 'dry_run_only',
      maxPrsPerRun: 10,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.createPolicy).toHaveBeenCalledWith(
      expect.objectContaining({ scopeId: 'repo-1', autonomyLevel: 'dry_run_only' })
    );
  });
});

describe('useToggleRemediationPolicy', () => {
  it('should_callToggleApi_when_mutationInvoked', async () => {
    mockedApi.togglePolicy.mockResolvedValue({ ...mockPolicy, enabled: false });

    const { result } = renderHook(() => useToggleRemediationPolicy(), { wrapper: createWrapper() });

    result.current.mutate('policy-1');

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.togglePolicy).toHaveBeenCalledWith('policy-1');
  });
});

describe('useDeleteRemediationPolicy', () => {
  it('should_callDeleteApi_when_mutationInvoked', async () => {
    mockedApi.deletePolicy.mockResolvedValue();

    const { result } = renderHook(() => useDeleteRemediationPolicy(), { wrapper: createWrapper() });

    result.current.mutate('policy-1');

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.deletePolicy).toHaveBeenCalledWith('policy-1');
  });
});

describe('useFleetRemediation', () => {
  it('should_returnFleetRows_when_querySucceeds', async () => {
    mockedApi.getFleet.mockResolvedValue([mockFleetRow]);

    const { result } = renderHook(() => useFleetRemediation(false as unknown as number), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual([mockFleetRow]);
  });
});

describe('usePreviewRepository', () => {
  it('should_returnPlan_when_previewMutationSucceeds', async () => {
    const plan: ConsolidationPlan = {
      would_select: [{ number: 1, title: 'PR 1', branch: 'feat' }],
      pr_count: 1,
      predicted_conflicts: [],
      estimated_duration_secs: 45,
      air_gapped: false,
      blocked_by_air_gap: false,
    };
    mockedApi.previewRepository.mockResolvedValue(plan);

    const { result } = renderHook(() => usePreviewRepository(), { wrapper: createWrapper() });

    result.current.mutate('repo-1');

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.previewRepository).toHaveBeenCalledWith('repo-1');
    expect(result.current.data).toEqual(plan);
  });
});
