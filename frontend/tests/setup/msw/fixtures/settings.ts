/**
 * Settings API fixtures for MSW handlers
 *
 * Provides typed mock data for settings endpoints.
 * Includes user behavior settings and notification preferences.
 */

import type {
  UserSettings,
  NotificationPreferences,
  UpdateUserSettingsRequest,
  UpdateNotificationPreferencesRequest,
} from '@/api/settings';
import { successResponse, errorResponse } from './auth';

// Re-export utilities
export { successResponse, errorResponse };

// ============================================================================
// User Settings Fixtures
// ============================================================================

/** Default user behavior settings */
export const mockUserSettings: UserSettings = {
  mergeDelaySeconds: 30,
  requireApproval: true,
  deleteBranchesDefault: true,
  defaultMergeStrategy: 'squash',
  skipReviewRequirement: false,
};

/** Conservative settings (more restrictions) */
export const mockConservativeSettings: UserSettings = {
  mergeDelaySeconds: 120,
  requireApproval: true,
  deleteBranchesDefault: false,
  defaultMergeStrategy: 'merge',
  skipReviewRequirement: false,
};

/** Aggressive settings (fewer restrictions) */
export const mockAggressiveSettings: UserSettings = {
  mergeDelaySeconds: 0,
  requireApproval: false,
  deleteBranchesDefault: true,
  defaultMergeStrategy: 'squash',
  skipReviewRequirement: true,
};

// ============================================================================
// Notification Preferences Fixtures
// ============================================================================

/** Default notification preferences */
export const mockNotificationPreferences: NotificationPreferences = {
  emailEnabled: true,
  slackEnabled: false,
  slackWebhookUrl: null,
  pushEnabled: true,
  notifyOnPrReady: true,
  notifyOnPrFailed: true,
  notifyOnReviewRequested: true,
  digestFrequency: 'daily',
  smtpHost: null,
  smtpPort: null,
  smtpUsername: null,
  smtpFromEmail: null,
  smtpToEmails: null,
  smtpUseTls: true,
  notifyOnMergeSuccess: true,
  notifyOnMergeFailure: true,
  slackChannel: null,
};

/** Fully configured notification preferences */
export const mockFullNotificationPreferences: NotificationPreferences = {
  emailEnabled: true,
  slackEnabled: true,
  slackWebhookUrl: 'https://hooks.slack.com/services/xxx/yyy/zzz',
  pushEnabled: true,
  notifyOnPrReady: true,
  notifyOnPrFailed: true,
  notifyOnReviewRequested: true,
  digestFrequency: 'realtime',
  smtpHost: 'smtp.example.com',
  smtpPort: 587,
  smtpUsername: 'notifications@example.com',
  smtpFromEmail: 'ampel@example.com',
  smtpToEmails: ['team@example.com', 'alerts@example.com'],
  smtpUseTls: true,
  notifyOnMergeSuccess: true,
  notifyOnMergeFailure: true,
  slackChannel: '#pr-notifications',
};

/** Minimal notification preferences (all disabled) */
export const mockDisabledNotificationPreferences: NotificationPreferences = {
  emailEnabled: false,
  slackEnabled: false,
  slackWebhookUrl: null,
  pushEnabled: false,
  notifyOnPrReady: false,
  notifyOnPrFailed: false,
  notifyOnReviewRequested: false,
  digestFrequency: 'never',
  smtpHost: null,
  smtpPort: null,
  smtpUsername: null,
  smtpFromEmail: null,
  smtpToEmails: null,
  smtpUseTls: true,
  notifyOnMergeSuccess: false,
  notifyOnMergeFailure: false,
  slackChannel: null,
};

// ============================================================================
// Factory Functions
// ============================================================================

/**
 * Create user settings with custom values
 */
export function createUserSettings(overrides: Partial<UserSettings> = {}): UserSettings {
  return {
    ...mockUserSettings,
    ...overrides,
  };
}

/**
 * Create notification preferences with custom values
 */
export function createNotificationPreferences(
  overrides: Partial<NotificationPreferences> = {}
): NotificationPreferences {
  return {
    ...mockNotificationPreferences,
    ...overrides,
  };
}

/**
 * Apply updates to user settings (simulates PATCH behavior)
 */
export function applyUserSettingsUpdate(
  current: UserSettings,
  updates: UpdateUserSettingsRequest
): UserSettings {
  return {
    ...current,
    ...(updates.mergeDelaySeconds !== undefined && {
      mergeDelaySeconds: updates.mergeDelaySeconds,
    }),
    ...(updates.requireApproval !== undefined && { requireApproval: updates.requireApproval }),
    ...(updates.deleteBranchesDefault !== undefined && {
      deleteBranchesDefault: updates.deleteBranchesDefault,
    }),
    ...(updates.defaultMergeStrategy !== undefined && {
      defaultMergeStrategy: updates.defaultMergeStrategy as UserSettings['defaultMergeStrategy'],
    }),
    ...(updates.skipReviewRequirement !== undefined && {
      skipReviewRequirement: updates.skipReviewRequirement,
    }),
  };
}

/**
 * Apply updates to notification preferences (simulates PATCH behavior)
 */
export function applyNotificationUpdate(
  current: NotificationPreferences,
  updates: UpdateNotificationPreferencesRequest
): NotificationPreferences {
  return {
    ...current,
    ...updates,
  };
}

// ============================================================================
// Pre-built Responses
// ============================================================================

/** Successful behavior settings response */
export const behaviorSettingsSuccessResponse = successResponse(mockUserSettings);

/** Successful notification preferences response */
export const notificationPreferencesSuccessResponse = successResponse(mockNotificationPreferences);

/** Successful test slack response */
export const testSlackSuccessResponse = successResponse(true);

/** Failed test slack response */
export const testSlackFailedResponse = successResponse(false);

/** Successful test email response */
export const testEmailSuccessResponse = successResponse(true);

/** Failed test email response */
export const testEmailFailedResponse = successResponse(false);

// ============================================================================
// Error Responses
// ============================================================================

/** Invalid settings error */
export const invalidSettingsError = errorResponse('Invalid settings configuration');

/** Slack webhook error */
export const slackWebhookError = errorResponse('Invalid Slack webhook URL');

/** SMTP configuration error */
export const smtpConfigError = errorResponse('Failed to connect to SMTP server');
