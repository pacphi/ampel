import apiClient from './client';
import type { ApiResponse } from '@/types';
import type {
  ConsolidationPlan,
  CreatePolicyRequest,
  FleetRow,
  RemediationPolicy,
  UpdatePolicyRequest,
} from '@/types/remediation';

/**
 * Typed client for the Fleet Remediation API (base path `/api`, all authed).
 */
export const remediationApi = {
  async listPolicies(): Promise<RemediationPolicy[]> {
    const response = await apiClient.get<ApiResponse<RemediationPolicy[]>>('/remediation/policies');
    return response.data.data!;
  },

  async getPolicy(id: string): Promise<RemediationPolicy> {
    const response = await apiClient.get<ApiResponse<RemediationPolicy>>(
      `/remediation/policies/${id}`
    );
    return response.data.data!;
  },

  async createPolicy(data: CreatePolicyRequest): Promise<RemediationPolicy> {
    const response = await apiClient.post<ApiResponse<RemediationPolicy>>(
      '/remediation/policies',
      data
    );
    return response.data.data!;
  },

  async updatePolicy(id: string, data: UpdatePolicyRequest): Promise<RemediationPolicy> {
    const response = await apiClient.patch<ApiResponse<RemediationPolicy>>(
      `/remediation/policies/${id}`,
      data
    );
    return response.data.data!;
  },

  async deletePolicy(id: string): Promise<void> {
    await apiClient.delete(`/remediation/policies/${id}`);
  },

  async togglePolicy(id: string): Promise<RemediationPolicy> {
    const response = await apiClient.post<ApiResponse<RemediationPolicy>>(
      `/remediation/policies/${id}/toggle`
    );
    return response.data.data!;
  },

  async previewRepository(repoId: string): Promise<ConsolidationPlan> {
    const response = await apiClient.post<ApiResponse<ConsolidationPlan>>(
      `/remediation/repositories/${repoId}/preview`
    );
    return response.data.data!;
  },

  async getFleet(): Promise<FleetRow[]> {
    const response = await apiClient.get<ApiResponse<FleetRow[]>>('/remediation/fleet');
    return response.data.data!;
  },
};
