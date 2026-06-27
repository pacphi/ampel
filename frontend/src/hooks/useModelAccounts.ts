import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { modelAccountsApi } from '@/api/modelAccounts';
import type { CreateModelAccountRequest, UpdateModelAccountRequest } from '@/types/modelAccount';

export const modelAccountKeys = {
  all: ['model-accounts'] as const,
  detail: (id: string) => ['model-accounts', id] as const,
};

export function useModelAccounts() {
  return useQuery({
    queryKey: modelAccountKeys.all,
    queryFn: () => modelAccountsApi.listAccounts(),
  });
}

export function useModelAccount(id: string) {
  return useQuery({
    queryKey: modelAccountKeys.detail(id),
    queryFn: () => modelAccountsApi.getAccount(id),
    enabled: !!id,
  });
}

export function useCreateModelAccount() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateModelAccountRequest) => modelAccountsApi.createAccount(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: modelAccountKeys.all });
    },
  });
}

export function useUpdateModelAccount() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateModelAccountRequest }) =>
      modelAccountsApi.updateAccount(id, data),
    onSuccess: (_data, { id }) => {
      queryClient.invalidateQueries({ queryKey: modelAccountKeys.all });
      queryClient.invalidateQueries({ queryKey: modelAccountKeys.detail(id) });
    },
  });
}

export function useDeleteModelAccount() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => modelAccountsApi.deleteAccount(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: modelAccountKeys.all });
    },
  });
}

export function useValidateModelAccount() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => modelAccountsApi.validateAccount(id),
    onSuccess: (_data, id) => {
      queryClient.invalidateQueries({ queryKey: modelAccountKeys.all });
      queryClient.invalidateQueries({ queryKey: modelAccountKeys.detail(id) });
    },
  });
}
