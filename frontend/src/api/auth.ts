import apiClient from './client';
import type { ApiResponse, AuthTokens, User } from '@/types';

export const authApi = {
  async register(email: string, password: string, displayName?: string): Promise<AuthTokens> {
    const response = await apiClient.post<ApiResponse<AuthTokens>>('/auth/register', {
      email,
      password,
      displayName,
    });
    return response.data.data!;
  },

  async login(email: string, password: string): Promise<AuthTokens> {
    const response = await apiClient.post<ApiResponse<AuthTokens>>('/auth/login', {
      email,
      password,
    });
    return response.data.data!;
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
