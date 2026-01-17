import apiClient from './client';
import type { ApiResponse, PaginatedResponse, PullRequestWithDetails } from '@/types';

export interface MergeRequest {
  strategy: 'merge' | 'squash' | 'rebase';
  commitTitle?: string;
  commitMessage?: string;
  deleteBranch: boolean;
}

export interface MergeResult {
  merged: boolean;
  sha?: string;
  message: string;
}

export const pullRequestsApi = {
  async list(page = 1, perPage = 20): Promise<PaginatedResponse<PullRequestWithDetails>> {
    const response = await apiClient.get<ApiResponse<PaginatedResponse<PullRequestWithDetails>>>(
      '/pull-requests',
      {
        params: { page, perPage },
      }
    );
    return response.data.data!;
  },

  async listByRepository(repoId: string): Promise<PullRequestWithDetails[]> {
    const response = await apiClient.get<ApiResponse<PullRequestWithDetails[]>>(
      `/repositories/${repoId}/pull-requests`
    );
    return response.data.data!;
  },

  async get(repoId: string, prId: string): Promise<PullRequestWithDetails> {
    const response = await apiClient.get<ApiResponse<PullRequestWithDetails>>(
      `/repositories/${repoId}/pull-requests/${prId}`
    );
    return response.data.data!;
  },

  async merge(repoId: string, prId: string, request: MergeRequest): Promise<MergeResult> {
    const response = await apiClient.post<ApiResponse<MergeResult>>(
      `/repositories/${repoId}/pull-requests/${prId}/merge`,
      request
    );
    return response.data.data!;
  },

  async refresh(repoId: string, prId: string): Promise<PullRequestWithDetails> {
    const response = await apiClient.post<ApiResponse<PullRequestWithDetails>>(
      `/repositories/${repoId}/pull-requests/${prId}/refresh`
    );
    return response.data.data!;
  },

  async getDiff(repoId: string, prId: string): Promise<PrDiffResponse> {
    const response = await apiClient.get<ApiResponse<PrDiffResponse>>(
      `/repositories/${repoId}/pull-requests/${prId}/diff`
    );
    return response.data.data!;
  },
};

export interface PrDiffFile {
  filename: string;
  status: 'added' | 'modified' | 'removed' | 'renamed';
  additions: number;
  deletions: number;
  changes: number;
  patch?: string;
  previous_filename?: string;
}

export interface PrDiffResponse {
  files: PrDiffFile[];
  total_additions: number;
  total_deletions: number;
  total_files: number;
}
