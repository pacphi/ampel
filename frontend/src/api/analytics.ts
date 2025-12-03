import apiClient from './client';
import type { ApiResponse } from '@/types';

export interface AnalyticsSummary {
  totalPrsMerged: number;
  avgTimeToMergeHours: number;
  avgReviewTimeHours: number;
  botPrPercentage: number;
  topContributors: ContributorStats[];
}

export interface ContributorStats {
  author: string;
  prCount: number;
  avgMergeTimeHours: number;
}

export interface RepositoryHealth {
  repositoryId: string;
  repositoryName: string;
  currentScore: number;
  trend: 'up' | 'down' | 'stable';
  metrics: HealthMetrics;
  history: HealthScorePoint[];
}

export interface HealthMetrics {
  avgTimeToMergeHours: number;
  avgReviewTimeHours: number;
  stalePrCount: number;
  failedCheckRate: number;
  prThroughputPerWeek: number;
}

export interface HealthScorePoint {
  date: string;
  score: number;
}

export const analyticsApi = {
  async getSummary(days?: number): Promise<AnalyticsSummary> {
    const response = await apiClient.get<ApiResponse<AnalyticsSummary>>('/analytics/summary', {
      params: { days },
    });
    return response.data.data!;
  },

  async getHealthOverview(): Promise<RepositoryHealth[]> {
    const response = await apiClient.get<ApiResponse<RepositoryHealth[]>>('/analytics/health');
    return response.data.data!;
  },

  async getRepositoryHealth(repoId: string): Promise<RepositoryHealth> {
    const response = await apiClient.get<ApiResponse<RepositoryHealth>>(
      `/repositories/${repoId}/health`
    );
    return response.data.data!;
  },
};
