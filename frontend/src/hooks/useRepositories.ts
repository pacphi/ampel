import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { repositoriesApi } from '@/api/repositories';
import type { GitProvider } from '@/types';

export function useRepositories() {
  return useQuery({
    queryKey: ['repositories'],
    queryFn: () => repositoriesApi.list(),
  });
}

export function useRepository(id: string) {
  return useQuery({
    queryKey: ['repositories', id],
    queryFn: () => repositoriesApi.get(id),
    enabled: !!id,
  });
}

export function useDiscoverRepositories(provider: GitProvider | null, page = 1) {
  return useQuery({
    queryKey: ['discover-repositories', provider, page],
    queryFn: () => repositoriesApi.discover(provider!, page),
    enabled: !!provider,
  });
}

export function useAddRepository() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      provider,
      owner,
      name,
      pollIntervalSeconds,
    }: {
      provider: GitProvider;
      owner: string;
      name: string;
      pollIntervalSeconds?: number;
    }) => repositoriesApi.add(provider, owner, name, pollIntervalSeconds),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard'] });
    },
  });
}

export function useRemoveRepository() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => repositoriesApi.remove(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard'] });
    },
  });
}

export function useUpdateRepository() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: { pollIntervalSeconds?: number } }) =>
      repositoriesApi.update(id, data),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['repositories', id] });
    },
  });
}
