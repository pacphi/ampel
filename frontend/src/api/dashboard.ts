import apiClient from './client';
import type { ApiResponse, DashboardSummary, RepositoryWithStatus } from '@/types';

export const dashboardApi = {
  async getSummary(): Promise<DashboardSummary> {
    const response = await apiClient.get<ApiResponse<DashboardSummary>>('/dashboard/summary');
    return response.data.data!;
  },

  async getGrid(): Promise<RepositoryWithStatus[]> {
    const response = await apiClient.get<ApiResponse<RepositoryWithStatus[]>>('/dashboard/grid');
    return response.data.data!;
  },
};
