import apiClient from './client';
import type { ApiResponse, GitProvider, ProviderConnection } from '@/types';

export const oauthApi = {
  async getOAuthUrl(provider: GitProvider): Promise<string> {
    const response = await apiClient.get<ApiResponse<{ url: string }>>(`/oauth/${provider}/url`);
    return response.data.data!.url;
  },

  async listConnections(): Promise<ProviderConnection[]> {
    const response = await apiClient.get<ApiResponse<ProviderConnection[]>>('/oauth/connections');
    return response.data.data!;
  },

  async disconnect(provider: GitProvider): Promise<void> {
    await apiClient.delete(`/oauth/connections/${provider}`);
  },
};
