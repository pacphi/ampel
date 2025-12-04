import apiClient from './client';
import type { ApiResponse } from '@/types';

export interface PrFilter {
  id: string;
  userId: string;
  allowedActors: string[];
  skipLabels: string[];
  maxAgeDays: number | null;
}

export interface UpdatePrFilterRequest {
  allowedActors?: string[];
  skipLabels?: string[];
  maxAgeDays?: number | null;
}

export const prFiltersApi = {
  async get(): Promise<PrFilter> {
    const response = await apiClient.get<ApiResponse<PrFilter>>('/pr-filters');
    return response.data.data!;
  },

  async update(data: UpdatePrFilterRequest): Promise<PrFilter> {
    const response = await apiClient.put<ApiResponse<PrFilter>>('/pr-filters', data);
    return response.data.data!;
  },

  async reset(): Promise<PrFilter> {
    const response = await apiClient.post<ApiResponse<PrFilter>>('/pr-filters/reset');
    return response.data.data!;
  },
};
