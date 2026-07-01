import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { playbooksApi } from '@/api/playbooks';
import type {
  CreatePlaybookRequest,
  PlaybookPreviewRequest,
  UpdatePlaybookRequest,
} from '@/types/playbook';

export const playbookKeys = {
  all: ['remediation', 'playbooks'] as const,
  detail: (id: string) => ['remediation', 'playbooks', id] as const,
};

export function usePlaybooks() {
  return useQuery({
    queryKey: playbookKeys.all,
    queryFn: () => playbooksApi.listPlaybooks(),
  });
}

export function usePlaybook(id: string) {
  return useQuery({
    queryKey: playbookKeys.detail(id),
    queryFn: () => playbooksApi.getPlaybook(id),
    enabled: !!id,
  });
}

/**
 * On-demand fetch of the built-in default playbook (ADR-006). Modeled as a
 * mutation so it fires on a user action (the editor's "Load built-in default"),
 * not on mount.
 */
export function useLoadEmbeddedPlaybook() {
  return useMutation({
    mutationFn: () => playbooksApi.getEmbeddedPlaybook(),
  });
}

export function useCreatePlaybook() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreatePlaybookRequest) => playbooksApi.createPlaybook(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.all });
    },
  });
}

export function useUpdatePlaybook() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdatePlaybookRequest }) =>
      playbooksApi.updatePlaybook(id, data),
    onSuccess: (_data, { id }) => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.all });
      queryClient.invalidateQueries({ queryKey: playbookKeys.detail(id) });
    },
  });
}

export function useDeletePlaybook() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => playbooksApi.deletePlaybook(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.all });
    },
  });
}

/** Preview mutation — renders the assembled prompt with no model call. */
export function usePreviewPlaybook() {
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data?: PlaybookPreviewRequest }) =>
      playbooksApi.previewPlaybook(id, data),
  });
}
