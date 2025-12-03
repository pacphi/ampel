import apiClient from './client';
import type { ApiResponse, AuthTokens, User, OAuthUrlResponse } from '@/types';

export type OAuthProvider = 'github' | 'google';

export const authApi = {
  // Social login methods
  async getOAuthUrl(provider: OAuthProvider): Promise<string> {
    const response = await apiClient.post<ApiResponse<OAuthUrlResponse>>(`/auth/${provider}/url`);
    return response.data.data!.url;
  },

  async refresh(refreshToken: string): Promise<AuthTokens> {
    const response = await apiClient.post<ApiResponse<AuthTokens>>('/auth/refresh', {
      refreshToken,
    });
    return response.data.data!;
  },

  async me(): Promise<User> {
    const response = await apiClient.get<ApiResponse<User>>('/auth/me');
    return response.data.data!;
  },

  async logout(): Promise<void> {
    await apiClient.post('/auth/logout');
  },
};
