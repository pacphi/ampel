import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { pullRequestsApi, type MergeRequest } from '@/api/pullRequests';

export function usePullRequests(page = 1, perPage = 20) {
  return useQuery({
    queryKey: ['pull-requests', page, perPage],
    queryFn: () => pullRequestsApi.list(page, perPage),
  });
}

export function useRepositoryPullRequests(repoId: string) {
  return useQuery({
    queryKey: ['pull-requests', 'repository', repoId],
    queryFn: () => pullRequestsApi.listByRepository(repoId),
    enabled: !!repoId,
  });
}

export function usePullRequest(repoId: string, prId: string) {
  return useQuery({
    queryKey: ['pull-requests', repoId, prId],
    queryFn: () => pullRequestsApi.get(repoId, prId),
    enabled: !!repoId && !!prId,
  });
}

export function useMergePullRequest() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      repoId,
      prId,
      request,
    }: {
      repoId: string;
      prId: string;
      request: MergeRequest;
    }) => pullRequestsApi.merge(repoId, prId, request),
    onSuccess: (_, { repoId, prId }) => {
      queryClient.invalidateQueries({ queryKey: ['pull-requests'] });
      queryClient.invalidateQueries({ queryKey: ['pull-requests', repoId, prId] });
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard'] });
    },
  });
}

export function useRefreshPullRequest() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ repoId, prId }: { repoId: string; prId: string }) =>
      pullRequestsApi.refresh(repoId, prId),
    onSuccess: (data, { repoId, prId }) => {
      queryClient.setQueryData(['pull-requests', repoId, prId], data);
      queryClient.invalidateQueries({ queryKey: ['pull-requests'] });
    },
  });
}
