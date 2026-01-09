/// <reference types="vitest/globals" />
import { afterAll, afterEach, beforeAll, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import '@testing-library/jest-dom';

// MSW server imports
import { server, resetHandlers } from './setup/msw/server';

// ============================================================================
// MSW Server Lifecycle
// ============================================================================

/**
 * Start MSW server before all tests.
 * Intercepts all HTTP requests and returns mock responses.
 */
beforeAll(() => {
  server.listen({
    onUnhandledRequest: 'warn',
  });
});

/**
 * Reset handlers and cleanup after each test.
 * Ensures tests don't affect each other.
 */
afterEach(() => {
  // Reset MSW handlers to defaults
  resetHandlers();

  // Cleanup React Testing Library
  cleanup();

  // Clear localStorage
  localStorage.clear();
});

/**
 * Stop MSW server after all tests complete.
 */
afterAll(() => {
  server.close();
});

// Mock window.matchMedia - must be defined before any tests run
const mockMatchMedia = vi.fn().mockImplementation((query: string) => ({
  matches: false,
  media: query,
  onchange: null,
  addListener: vi.fn(),
  removeListener: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  dispatchEvent: vi.fn(),
}));

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  configurable: true,
  value: mockMatchMedia,
});

// Ensure matchMedia is available globally
globalThis.matchMedia = mockMatchMedia as unknown as typeof globalThis.matchMedia;

// Mock IntersectionObserver
class MockIntersectionObserver implements IntersectionObserver {
  readonly root: Element | Document | null = null;
  readonly rootMargin: string = '';
  readonly thresholds: ReadonlyArray<number> = [];
  disconnect(): void {}
  observe(): void {}
  takeRecords(): IntersectionObserverEntry[] {
    return [];
  }
  unobserve(): void {}
}
globalThis.IntersectionObserver = MockIntersectionObserver;

// Mock ResizeObserver
class MockResizeObserver implements ResizeObserver {
  disconnect(): void {}
  observe(): void {}
  unobserve(): void {}
}
globalThis.ResizeObserver = MockResizeObserver;

// Mock PointerCapture methods (required for Radix UI Select)
if (typeof HTMLElement !== 'undefined') {
  HTMLElement.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  HTMLElement.prototype.setPointerCapture = vi.fn();
  HTMLElement.prototype.releasePointerCapture = vi.fn();
}

// Mock scrollIntoView (required for Radix UI Select)
Element.prototype.scrollIntoView = vi.fn();

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
    get length() {
      return Object.keys(store).length;
    },
    key: (index: number) => {
      const keys = Object.keys(store);
      return keys[index] || null;
    },
  };
})();

Object.defineProperty(globalThis, 'localStorage', {
  value: localStorageMock,
  writable: true,
});

// Environment detection
export const isCI = (): boolean => {
  return import.meta.env.CI === 'true' || import.meta.env.GITHUB_ACTIONS === 'true';
};

// Test utilities
export const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export const flushPromises = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));
