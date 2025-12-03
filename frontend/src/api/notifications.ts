import apiClient from './client';
import type { ApiResponse } from '@/types';

export interface NotificationPreferences {
  emailEnabled: boolean;
  slackEnabled: boolean;
  slackWebhookUrl?: string;
  pushEnabled: boolean;
  notifyOnPrReady: boolean;
  notifyOnPrFailed: boolean;
  notifyOnReviewRequested: boolean;
  digestFrequency: 'none' | 'daily' | 'weekly';
}

export interface UpdateNotificationPreferencesRequest {
  emailEnabled?: boolean;
  slackEnabled?: boolean;
  slackWebhookUrl?: string;
  pushEnabled?: boolean;
  notifyOnPrReady?: boolean;
  notifyOnPrFailed?: boolean;
  notifyOnReviewRequested?: boolean;
  digestFrequency?: 'none' | 'daily' | 'weekly';
}

export const notificationsApi = {
  async getPreferences(): Promise<NotificationPreferences> {
    const response = await apiClient.get<ApiResponse<NotificationPreferences>>(
      '/notifications/preferences'
    );
    return response.data.data!;
  },

  async updatePreferences(
    data: UpdateNotificationPreferencesRequest
  ): Promise<NotificationPreferences> {
    const response = await apiClient.put<ApiResponse<NotificationPreferences>>(
      '/notifications/preferences',
      data
    );
    return response.data.data!;
  },

  async testSlackWebhook(): Promise<boolean> {
    const response = await apiClient.post<ApiResponse<boolean>>('/notifications/test-slack');
    return response.data.data!;
  },
};
