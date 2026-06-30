import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { remediationApi } from '@/api/remediation';
import type { CreatePolicyRequest, UpdatePolicyRequest } from '@/types/remediation';

export const remediationPolicyKeys = {
  all: ['remediation', 'policies'] as const,
  detail: (id: string) => ['remediation', 'policies', id] as const,
  scopes: ['remediation', 'scopes'] as const,
};

/** The scopes the caller may attach a policy to, for the editor's pickers. */
export function useRemediationScopes() {
  return useQuery({
    queryKey: remediationPolicyKeys.scopes,
    queryFn: () => remediationApi.listScopes(),
  });
}

export function useRemediationPolicies() {
  return useQuery({
    queryKey: remediationPolicyKeys.all,
    queryFn: () => remediationApi.listPolicies(),
  });
}

export function useRemediationPolicy(id: string) {
  return useQuery({
    queryKey: remediationPolicyKeys.detail(id),
    queryFn: () => remediationApi.getPolicy(id),
    enabled: !!id,
  });
}

export function useCreateRemediationPolicy() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreatePolicyRequest) => remediationApi.createPolicy(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: remediationPolicyKeys.all });
      queryClient.invalidateQueries({ queryKey: ['remediation', 'fleet'] });
    },
  });
}

export function useUpdateRemediationPolicy() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdatePolicyRequest }) =>
      remediationApi.updatePolicy(id, data),
    onSuccess: (_data, { id }) => {
      queryClient.invalidateQueries({ queryKey: remediationPolicyKeys.all });
      queryClient.invalidateQueries({ queryKey: remediationPolicyKeys.detail(id) });
      queryClient.invalidateQueries({ queryKey: ['remediation', 'fleet'] });
    },
  });
}

export function useDeleteRemediationPolicy() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => remediationApi.deletePolicy(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: remediationPolicyKeys.all });
      queryClient.invalidateQueries({ queryKey: ['remediation', 'fleet'] });
    },
  });
}

export function useToggleRemediationPolicy() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => remediationApi.togglePolicy(id),
    onSuccess: (_data, id) => {
      queryClient.invalidateQueries({ queryKey: remediationPolicyKeys.all });
      queryClient.invalidateQueries({ queryKey: remediationPolicyKeys.detail(id) });
      queryClient.invalidateQueries({ queryKey: ['remediation', 'fleet'] });
    },
  });
}
