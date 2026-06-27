import apiClient from './client';
import type { ApiResponse } from '@/types';
import type {
  ConsolidationPlan,
  CreatePolicyRequest,
  FleetRow,
  ListRunsFilters,
  RemediationPolicy,
  RemediationRun,
  RunDetail,
  SseToken,
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

  // --- Phase 3: Runs (history/detail), approvals, manual trigger, SSE token ---

  async listRuns(filters: ListRunsFilters = {}): Promise<RemediationRun[]> {
    const response = await apiClient.get<ApiResponse<RemediationRun[]>>('/remediation/runs', {
      params: filters,
    });
    return response.data.data!;
  },

  async getRun(id: string): Promise<RunDetail> {
    const response = await apiClient.get<ApiResponse<RunDetail>>(`/remediation/runs/${id}`);
    return response.data.data!;
  },

  async approveRun(id: string): Promise<RemediationRun> {
    const response = await apiClient.post<ApiResponse<RemediationRun>>(
      `/remediation/runs/${id}/approve`
    );
    return response.data.data!;
  },

  async cancelRun(id: string): Promise<RemediationRun> {
    const response = await apiClient.post<ApiResponse<RemediationRun>>(
      `/remediation/runs/${id}/cancel`
    );
    return response.data.data!;
  },

  async triggerRun(repoId: string): Promise<RemediationRun> {
    const response = await apiClient.post<ApiResponse<RemediationRun>>(
      `/remediation/repositories/${repoId}/run`
    );
    return response.data.data!;
  },

  /**
   * Fetch a short-lived SSE token. `EventSource` cannot send an Authorization
   * header, so the stream is authenticated via a `?token=` query parameter.
   */
  async fetchSseToken(): Promise<SseToken> {
    const response = await apiClient.post<ApiResponse<SseToken>>('/remediation/sse-token');
    return response.data.data!;
  },
};

/**
 * Build the authenticated EventSource URL for a run's event stream. Mirrors the
 * axios `baseURL` so the path includes the `/api` prefix in every environment.
 */
export function buildRunEventsUrl(runId: string, token: string): string {
  const baseURL = apiClient.defaults.baseURL ?? '/api';
  return `${baseURL}/remediation/runs/${runId}/events?token=${encodeURIComponent(token)}`;
}
