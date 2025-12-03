import apiClient from './client';
import type { ApiResponse, ProviderConnection, AddConnectionRequest } from '@/types';

export const connectionsApi = {
  async list(): Promise<ProviderConnection[]> {
    const response = await apiClient.get<ApiResponse<ProviderConnection[]>>('/connections');
    return response.data.data || [];
  },

  async get(id: string): Promise<ProviderConnection> {
    const response = await apiClient.get<ApiResponse<ProviderConnection>>(`/connections/${id}`);
    return response.data.data!;
  },

  async add(request: AddConnectionRequest): Promise<ProviderConnection> {
    const response = await apiClient.post<ApiResponse<ProviderConnection>>('/connections', request);
    return response.data.data!;
  },

  async update(
    id: string,
    data: { connectionName?: string; accessToken?: string }
  ): Promise<ProviderConnection> {
    const response = await apiClient.put<ApiResponse<ProviderConnection>>(
      `/connections/${id}`,
      data
    );
    return response.data.data!;
  },

  async delete(id: string): Promise<void> {
    await apiClient.delete(`/connections/${id}`);
  },

  async validate(id: string): Promise<ProviderConnection> {
    const response = await apiClient.post<ApiResponse<ProviderConnection>>(
      `/connections/${id}/validate`
    );
    return response.data.data!;
  },
};
