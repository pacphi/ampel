/// <reference types="vitest/globals" />
import { vi } from 'vitest';
import type { TFunction } from 'i18next';

/**
 * Mock translation function for tests
 * Returns the key passed to it, which allows tests to verify the correct translation key is used
 */
export function createMockT(): TFunction {
  const mockT = vi.fn((key: string, options?: Record<string, unknown>) => {
    // Handle interpolation for testing
    if (options && typeof options === 'object') {
      let result = key;
      Object.keys(options).forEach((optionKey) => {
        if (optionKey !== 'defaultValue' && optionKey !== 'count') {
          result = result.replace(`{{${optionKey}}}`, options[optionKey]);
        }
      });
      return result;
    }
    return key;
  }) as TFunction;

  return mockT;
}

/**
 * Create a mock useTranslation hook that returns translation keys
 * This allows tests to verify that the correct i18n keys are being used
 */
export function createMockUseTranslation() {
  const mockT = createMockT();

  return vi.fn(() => ({
    t: mockT,
    i18n: {
      language: 'en',
      changeLanguage: vi.fn(),
      isInitialized: true,
    },
    ready: true,
  }));
}

/**
 * Translation mappings for tests that need actual translated strings
 * This mimics what the actual translation files contain
 */
export const mockTranslations = {
  'dashboard:blockers.draft': 'Draft',
  'dashboard:blockers.conflicts': 'Conflicts',
  'dashboard:blockers.ciFailed': 'CI failed',
  'dashboard:blockers.ciPending': 'CI pending',
  'dashboard:blockers.changesRequested': 'Changes requested',
  'dashboard:blockers.awaitingReview': 'Awaiting review',
  'dashboard:blockers.needsReview': 'Needs review',
  'dashboard:actions.merge': 'Merge',
  'dashboard:empty.title': 'No repositories found',
  'dashboard:empty.description': 'Add repositories from the Repositories page to get started',
  'dashboard:views.repositoryList': 'Repository list view',
  'common:visibility.public': 'Public',
  'common:visibility.private': 'Private',
  'common:visibility.archived': 'Archived',
  'common:actions.viewOnProvider': 'View on {{provider}}',
};

/**
 * Create a translation function that returns actual strings
 * Use this when tests need to check for actual translated content
 */
export function createMockTWithTranslations(): TFunction {
  const mockT = vi.fn((key: string, options?: Record<string, unknown>) => {
    // Check if we have a translation for this key
    const translation = mockTranslations[key as keyof typeof mockTranslations];

    if (translation && options && typeof options === 'object') {
      // Handle interpolation
      let result = translation;
      Object.keys(options).forEach((optionKey) => {
        if (optionKey !== 'defaultValue' && optionKey !== 'count') {
          result = result.replace(`{{${optionKey}}}`, options[optionKey]);
        }
      });
      return result;
    }

    return translation || key;
  }) as TFunction;

  return mockT;
}

/**
 * Create useTranslation hook that returns actual translated strings
 */
export function createMockUseTranslationWithStrings() {
  const mockT = createMockTWithTranslations();

  return vi.fn(() => ({
    t: mockT,
    i18n: {
      language: 'en',
      changeLanguage: vi.fn(),
      isInitialized: true,
    },
    ready: true,
  }));
}
