import apiClient from './client';
import type {
  ApiResponse,
  DiscoveredRepository,
  GitProvider,
  Repository,
  RepositoryWithStatus,
  RefreshJobResponse,
  RefreshJobStatus,
} from '@/types';

export const repositoriesApi = {
  async list(): Promise<RepositoryWithStatus[]> {
    const response = await apiClient.get<ApiResponse<RepositoryWithStatus[]>>('/repositories');
    return response.data.data!;
  },

  async get(id: string): Promise<RepositoryWithStatus> {
    const response = await apiClient.get<ApiResponse<RepositoryWithStatus>>(`/repositories/${id}`);
    return response.data.data!;
  },

  async discover(provider: GitProvider): Promise<DiscoveredRepository[]> {
    const response = await apiClient.get<ApiResponse<DiscoveredRepository[]>>(
      '/repositories/discover',
      {
        params: { provider },
      }
    );
    return response.data.data!;
  },

  async add(
    provider: GitProvider,
    owner: string,
    name: string,
    pollIntervalSeconds?: number
  ): Promise<Repository> {
    const response = await apiClient.post<ApiResponse<Repository>>('/repositories', {
      provider,
      owner,
      name,
      pollIntervalSeconds,
    });
    return response.data.data!;
  },

  async update(id: string, data: { pollIntervalSeconds?: number }): Promise<Repository> {
    const response = await apiClient.put<ApiResponse<Repository>>(`/repositories/${id}`, data);
    return response.data.data!;
  },

  async remove(id: string): Promise<void> {
    await apiClient.delete(`/repositories/${id}`);
  },

  async refreshAll(): Promise<RefreshJobResponse> {
    const response = await apiClient.post<ApiResponse<RefreshJobResponse>>(
      '/repositories/refresh-all'
    );
    return response.data.data!;
  },

  async getRefreshStatus(jobId: string): Promise<RefreshJobStatus> {
    const response = await apiClient.get<ApiResponse<RefreshJobStatus>>(
      `/repositories/refresh-status/${jobId}`
    );
    return response.data.data!;
  },
};
