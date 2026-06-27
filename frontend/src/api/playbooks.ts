import apiClient from './client';
import type { ApiResponse } from '@/types';
import type {
  CreatePlaybookRequest,
  Playbook,
  PlaybookPreviewRequest,
  PlaybookPreviewResponse,
  UpdatePlaybookRequest,
} from '@/types/playbook';

/**
 * Typed client for the remediation playbook API (base path `/api`, all authed).
 */
export const playbooksApi = {
  async listPlaybooks(): Promise<Playbook[]> {
    const response = await apiClient.get<ApiResponse<Playbook[]>>('/remediation/playbooks');
    return response.data.data!;
  },

  async getPlaybook(id: string): Promise<Playbook> {
    const response = await apiClient.get<ApiResponse<Playbook>>(`/remediation/playbooks/${id}`);
    return response.data.data!;
  },

  async createPlaybook(data: CreatePlaybookRequest): Promise<Playbook> {
    const response = await apiClient.post<ApiResponse<Playbook>>('/remediation/playbooks', data);
    return response.data.data!;
  },

  async updatePlaybook(id: string, data: UpdatePlaybookRequest): Promise<Playbook> {
    const response = await apiClient.patch<ApiResponse<Playbook>>(
      `/remediation/playbooks/${id}`,
      data
    );
    return response.data.data!;
  },

  async deletePlaybook(id: string): Promise<void> {
    await apiClient.delete(`/remediation/playbooks/${id}`);
  },

  /** Render the assembled prompt with NO model call (lint a playbook safely). */
  async previewPlaybook(
    id: string,
    data: PlaybookPreviewRequest = {}
  ): Promise<PlaybookPreviewResponse> {
    const response = await apiClient.post<ApiResponse<PlaybookPreviewResponse>>(
      `/remediation/playbooks/${id}/preview`,
      data
    );
    return response.data.data!;
  },
};
