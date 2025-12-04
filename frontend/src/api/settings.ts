import apiClient from './client';
import type { ApiResponse } from '@/types';

export interface UserSettings {
  mergeDelaySeconds: number;
  requireApproval: boolean;
  deleteBranchesDefault: boolean;
  defaultMergeStrategy: 'merge' | 'squash' | 'rebase';
  skipReviewRequirement: boolean;
}

export interface UpdateUserSettingsRequest {
  mergeDelaySeconds?: number;
  requireApproval?: boolean;
  deleteBranchesDefault?: boolean;
  defaultMergeStrategy?: string;
  skipReviewRequirement?: boolean;
}

export interface NotificationPreferences {
  emailEnabled: boolean;
  slackEnabled: boolean;
  slackWebhookUrl: string | null;
  pushEnabled: boolean;
  notifyOnPrReady: boolean;
  notifyOnPrFailed: boolean;
  notifyOnReviewRequested: boolean;
  digestFrequency: string;
  // Email SMTP settings
  smtpHost: string | null;
  smtpPort: number | null;
  smtpUsername: string | null;
  smtpFromEmail: string | null;
  smtpToEmails: string[] | null;
  smtpUseTls: boolean;
  // Merge notification settings
  notifyOnMergeSuccess: boolean;
  notifyOnMergeFailure: boolean;
  slackChannel: string | null;
}

export interface UpdateNotificationPreferencesRequest {
  emailEnabled?: boolean;
  slackEnabled?: boolean;
  slackWebhookUrl?: string;
  pushEnabled?: boolean;
  notifyOnPrReady?: boolean;
  notifyOnPrFailed?: boolean;
  notifyOnReviewRequested?: boolean;
  digestFrequency?: string;
  // Email SMTP settings
  smtpHost?: string;
  smtpPort?: number;
  smtpUsername?: string;
  smtpPassword?: string;
  smtpFromEmail?: string;
  smtpToEmails?: string[];
  smtpUseTls?: boolean;
  // Merge notification settings
  notifyOnMergeSuccess?: boolean;
  notifyOnMergeFailure?: boolean;
  slackChannel?: string;
}

export const settingsApi = {
  // Behavior settings
  async getBehavior(): Promise<UserSettings> {
    const response = await apiClient.get<ApiResponse<UserSettings>>('/settings/behavior');
    return response.data.data!;
  },

  async updateBehavior(data: UpdateUserSettingsRequest): Promise<UserSettings> {
    const response = await apiClient.put<ApiResponse<UserSettings>>('/settings/behavior', data);
    return response.data.data!;
  },

  // Notification settings
  async getNotifications(): Promise<NotificationPreferences> {
    const response = await apiClient.get<ApiResponse<NotificationPreferences>>(
      '/notifications/preferences'
    );
    return response.data.data!;
  },

  async updateNotifications(
    data: UpdateNotificationPreferencesRequest
  ): Promise<NotificationPreferences> {
    const response = await apiClient.put<ApiResponse<NotificationPreferences>>(
      '/notifications/preferences',
      data
    );
    return response.data.data!;
  },

  async testSlack(): Promise<boolean> {
    const response = await apiClient.post<ApiResponse<boolean>>('/notifications/test-slack');
    return response.data.data!;
  },

  async testEmail(): Promise<boolean> {
    const response = await apiClient.post<ApiResponse<boolean>>('/notifications/test-email');
    return response.data.data!;
  },
};
