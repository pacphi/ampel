import apiClient from './client';
import type { ApiResponse } from '@/types';
import type {
  ModelCatalog,
  OllamaPullResponse,
  OllamaPullStatusResponse,
  OllamaTagsResponse,
} from '@/types/remediation';

/**
 * Typed client for the model-catalog API (base path `/api`, all authed).
 *
 * The catalog is metadata-only (no credentials). Ollama discovery/pull operate
 * on an existing saved Ollama account, identified by `accountId`.
 */
export const modelCatalogApi = {
  async getCatalog(organizationId?: string): Promise<ModelCatalog> {
    const response = await apiClient.get<ApiResponse<ModelCatalog>>('/model-catalog', {
      params: organizationId ? { organizationId } : undefined,
    });
    return response.data.data!;
  },

  async getOllamaTags(accountId: string): Promise<OllamaTagsResponse> {
    const response = await apiClient.get<ApiResponse<OllamaTagsResponse>>(
      '/model-catalog/ollama/tags',
      { params: { accountId } }
    );
    return response.data.data!;
  },

  async pullOllamaModel(accountId: string, model: string): Promise<OllamaPullResponse> {
    const response = await apiClient.post<ApiResponse<OllamaPullResponse>>(
      '/model-catalog/ollama/pull',
      { accountId, model }
    );
    return response.data.data!;
  },

  async getPullStatus(jobId: string): Promise<OllamaPullStatusResponse> {
    const response = await apiClient.get<ApiResponse<OllamaPullStatusResponse>>(
      `/model-catalog/ollama/pull/${jobId}/status`
    );
    return response.data.data!;
  },
};
