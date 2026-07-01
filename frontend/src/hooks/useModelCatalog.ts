import { useMutation, useQuery } from '@tanstack/react-query';
import { modelCatalogApi } from '@/api/modelCatalog';
import type { OllamaPullStatusResponse } from '@/types/remediation';

export const modelCatalogKeys = {
  catalog: (organizationId?: string) => ['model-catalog', organizationId ?? null] as const,
  ollamaTags: (accountId: string) => ['model-catalog', 'ollama-tags', accountId] as const,
  pullStatus: (jobId: string) => ['model-catalog', 'ollama-pull', jobId] as const,
};

/** The metadata-rich provider/model catalog for the model-account picker. */
export function useModelCatalog(organizationId?: string) {
  return useQuery({
    queryKey: modelCatalogKeys.catalog(organizationId),
    queryFn: () => modelCatalogApi.getCatalog(organizationId),
  });
}

/** Tags discovered on a saved Ollama account's server. */
export function useOllamaTags(accountId?: string) {
  return useQuery({
    queryKey: modelCatalogKeys.ollamaTags(accountId ?? ''),
    queryFn: () => modelCatalogApi.getOllamaTags(accountId!),
    enabled: !!accountId,
  });
}

/** Kick off an Ollama model pull on a saved account. */
export function usePullOllamaModel() {
  return useMutation({
    mutationFn: ({ accountId, model }: { accountId: string; model: string }) =>
      modelCatalogApi.pullOllamaModel(accountId, model),
  });
}

/** Poll a pull job until it reaches a terminal state (`ready` or `error`). */
export function usePullStatus(jobId?: string) {
  return useQuery({
    queryKey: modelCatalogKeys.pullStatus(jobId ?? ''),
    queryFn: () => modelCatalogApi.getPullStatus(jobId!),
    enabled: !!jobId,
    refetchInterval: (query) => {
      const status = (query.state.data as OllamaPullStatusResponse | undefined)?.status;
      return status === 'ready' || status === 'error' ? false : 1500;
    },
  });
}
