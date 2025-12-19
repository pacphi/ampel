import apiClient from './client';
import type { ApiResponse } from '@/types';
import type {
  ProviderAccount,
  AddAccountRequest,
  UpdateAccountRequest,
  ValidateAccountResponse,
} from '@/types/account';

export const accountsApi = {
  async listAccounts(): Promise<ProviderAccount[]> {
    const response = await apiClient.get<ApiResponse<ProviderAccount[]>>('/accounts');
    return response.data.data!;
  },

  async getAccount(id: string): Promise<ProviderAccount> {
    const response = await apiClient.get<ApiResponse<ProviderAccount>>(`/accounts/${id}`);
    return response.data.data!;
  },

  async addAccount(data: AddAccountRequest): Promise<ProviderAccount> {
    const response = await apiClient.post<ApiResponse<ProviderAccount>>('/accounts', data);
    return response.data.data!;
  },

  async updateAccount(id: string, data: UpdateAccountRequest): Promise<ProviderAccount> {
    const response = await apiClient.patch<ApiResponse<ProviderAccount>>(`/accounts/${id}`, data);
    return response.data.data!;
  },

  async deleteAccount(id: string): Promise<void> {
    await apiClient.delete(`/accounts/${id}`);
  },

  async validateAccount(id: string): Promise<ValidateAccountResponse> {
    const response = await apiClient.post<ApiResponse<ValidateAccountResponse>>(
      `/accounts/${id}/validate`
    );
    return response.data.data!;
  },

  async setDefaultAccount(id: string): Promise<ProviderAccount> {
    const response = await apiClient.post<ApiResponse<ProviderAccount>>(
      `/accounts/${id}/set-default`
    );
    return response.data.data!;
  },
};
