import apiClient from './client';
import type { ApiResponse } from '@/types';

export interface BulkMergeRequest {
  pullRequestIds: string[];
  strategy?: 'merge' | 'squash' | 'rebase';
  deleteBranch?: boolean;
}

export interface MergeItemResult {
  pullRequestId: string;
  repositoryName: string;
  prNumber: number;
  prTitle: string;
  status: 'pending' | 'success' | 'failed' | 'skipped';
  errorMessage: string | null;
  mergeSha: string | null;
}

export interface BulkMergeResponse {
  operationId: string;
  status: 'in_progress' | 'completed' | 'failed';
  total: number;
  success: number;
  failed: number;
  skipped: number;
  results: MergeItemResult[];
}

export const mergeApi = {
  async bulkMerge(request: BulkMergeRequest): Promise<BulkMergeResponse> {
    const response = await apiClient.post<ApiResponse<BulkMergeResponse>>('/merge/bulk', request);
    return response.data.data!;
  },

  async getOperation(operationId: string): Promise<BulkMergeResponse> {
    const response = await apiClient.get<ApiResponse<BulkMergeResponse>>(
      `/merge/operations/${operationId}`
    );
    return response.data.data!;
  },

  async listOperations(): Promise<BulkMergeResponse[]> {
    const response = await apiClient.get<ApiResponse<BulkMergeResponse[]>>('/merge/operations');
    return response.data.data!;
  },
};
