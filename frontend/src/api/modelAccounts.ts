import apiClient from './client';
import type { ApiResponse } from '@/types';
import type {
  CreateModelAccountRequest,
  ModelAccount,
  ModelValidationResult,
  UpdateModelAccountRequest,
} from '@/types/modelAccount';

/**
 * Typed client for the model-provider account API (base path `/api`, all authed).
 *
 * The API key is write-only: it is sent on create/update but never returned, so
 * no response type carries a credential field.
 */
export const modelAccountsApi = {
  async listAccounts(): Promise<ModelAccount[]> {
    const response = await apiClient.get<ApiResponse<ModelAccount[]>>('/model-accounts');
    return response.data.data!;
  },

  async getAccount(id: string): Promise<ModelAccount> {
    const response = await apiClient.get<ApiResponse<ModelAccount>>(`/model-accounts/${id}`);
    return response.data.data!;
  },

  async createAccount(data: CreateModelAccountRequest): Promise<ModelAccount> {
    const response = await apiClient.post<ApiResponse<ModelAccount>>('/model-accounts', data);
    return response.data.data!;
  },

  async updateAccount(id: string, data: UpdateModelAccountRequest): Promise<ModelAccount> {
    const response = await apiClient.patch<ApiResponse<ModelAccount>>(
      `/model-accounts/${id}`,
      data
    );
    return response.data.data!;
  },

  async deleteAccount(id: string): Promise<void> {
    await apiClient.delete(`/model-accounts/${id}`);
  },

  async validateAccount(id: string): Promise<ModelValidationResult> {
    const response = await apiClient.post<ApiResponse<ModelValidationResult>>(
      `/model-accounts/${id}/validate`
    );
    return response.data.data!;
  },
};
