/**
 * Vitest Global Setup
 *
 * Configures the test environment with all necessary mocks, providers, and MSW.
 * This file is loaded before all tests run.
 *
 * Key features:
 * - MSW server lifecycle management
 * - DOM API mocks (matchMedia, IntersectionObserver, etc.)
 * - localStorage mock
 * - Testing Library cleanup
 */

/// <reference types="vitest/globals" />
import { afterAll, afterEach, beforeAll, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import '@testing-library/jest-dom';

// MSW server imports
import { server, resetHandlers } from './msw/server';

// ============================================================================
// react-i18next Mock
// ============================================================================

/**
 * Mock react-i18next for components that use useTranslation hook
 * outside of the custom render wrapper from test-utils.
 *
 * This uses a dynamic import to ensure i18n is initialized before use.
 */
vi.mock('react-i18next', async () => {
  const actual = await vi.importActual<typeof import('react-i18next')>('react-i18next');
  // Dynamic import to ensure testI18n is properly initialized
  const { default: testI18n } = await import('./i18n-test-config');

  return {
    ...actual,
    useTranslation: (ns?: string | string[]) => ({
      t: (key: string, options?: Record<string, unknown>) => {
        // If the key already includes the namespace (e.g., 'notifications:slack.title'),
        // don't pass the ns parameter to avoid conflicts
        if (key.includes(':')) {
          return testI18n.t(key, options);
        }
        // For keys without namespace, use the provided namespace
        return testI18n.t(key, { ns, ...options });
      },
      i18n: testI18n,
      ready: true,
    }),
    Trans: ({ children }: { children: React.ReactNode }) => children,
    initReactI18next: actual.initReactI18next,
  };
});

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

// ============================================================================
// DOM API Mocks
// ============================================================================

/**
 * Mock window.matchMedia
 * Required for responsive components and CSS media queries
 */
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

globalThis.matchMedia = mockMatchMedia as unknown as typeof globalThis.matchMedia;

/**
 * Helper to set matchMedia to match a specific query
 */
export const setMatchMedia = (matches: boolean, query = '(prefers-color-scheme: dark)') => {
  mockMatchMedia.mockImplementation((q: string) => ({
    matches: q === query ? matches : false,
    media: q,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }));
};

/**
 * Mock IntersectionObserver
 * Required for lazy loading, infinite scroll, and visibility detection
 */
class MockIntersectionObserver implements IntersectionObserver {
  readonly root: Element | Document | null = null;
  readonly rootMargin: string = '';
  readonly thresholds: ReadonlyArray<number> = [];

  constructor(
    private callback: IntersectionObserverCallback,
    private options?: IntersectionObserverInit
  ) {}

  disconnect(): void {}
  observe(): void {}
  takeRecords(): IntersectionObserverEntry[] {
    return [];
  }
  unobserve(): void {}

  /**
   * Trigger intersection callback manually in tests
   */
  triggerIntersection(entries: Partial<IntersectionObserverEntry>[]): void {
    const fullEntries = entries.map((entry) => ({
      boundingClientRect: {} as DOMRectReadOnly,
      intersectionRatio: 1,
      intersectionRect: {} as DOMRectReadOnly,
      isIntersecting: true,
      rootBounds: null,
      target: document.createElement('div'),
      time: Date.now(),
      ...entry,
    }));
    this.callback(fullEntries as IntersectionObserverEntry[], this);
  }
}

globalThis.IntersectionObserver =
  MockIntersectionObserver as unknown as typeof IntersectionObserver;

/**
 * Mock ResizeObserver
 * Required for components that respond to size changes
 */
class MockResizeObserver implements ResizeObserver {
  constructor(private callback: ResizeObserverCallback) {}

  disconnect(): void {}
  observe(): void {}
  unobserve(): void {}

  /**
   * Trigger resize callback manually in tests
   */
  triggerResize(entries: Partial<ResizeObserverEntry>[]): void {
    const fullEntries = entries.map((entry) => ({
      borderBoxSize: [{ blockSize: 100, inlineSize: 100 }],
      contentBoxSize: [{ blockSize: 100, inlineSize: 100 }],
      contentRect: {} as DOMRectReadOnly,
      devicePixelContentBoxSize: [{ blockSize: 100, inlineSize: 100 }],
      target: document.createElement('div'),
      ...entry,
    }));
    this.callback(fullEntries as ResizeObserverEntry[], this);
  }
}

globalThis.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

/**
 * Mock PointerCapture methods
 * Required for Radix UI components (Select, Dialog, etc.)
 */
if (typeof HTMLElement !== 'undefined') {
  HTMLElement.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  HTMLElement.prototype.setPointerCapture = vi.fn();
  HTMLElement.prototype.releasePointerCapture = vi.fn();
}

/**
 * Mock scrollIntoView
 * Required for components that scroll elements into view
 */
Element.prototype.scrollIntoView = vi.fn();

/**
 * Mock scrollTo
 */
window.scrollTo = vi.fn();

// ============================================================================
// LocalStorage Mock
// ============================================================================

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

// ============================================================================
// URL and Navigation Mocks
// ============================================================================

/**
 * Mock window.location
 */
const locationMock = {
  href: 'http://localhost:3000',
  origin: 'http://localhost:3000',
  pathname: '/',
  search: '',
  hash: '',
  assign: vi.fn(),
  replace: vi.fn(),
  reload: vi.fn(),
};

Object.defineProperty(window, 'location', {
  value: locationMock,
  writable: true,
});

// ============================================================================
// Console Suppression (Optional)
// ============================================================================

/**
 * Suppress specific console warnings during tests.
 * Uncomment to hide noisy warnings that don't affect test results.
 */
// const originalWarn = console.warn;
// console.warn = (...args: unknown[]) => {
//   const message = args[0]?.toString() || '';
//   // Suppress React 18 act() warnings
//   if (message.includes('act(...)')) return;
//   // Suppress i18n backend warnings
//   if (message.includes('i18next')) return;
//   originalWarn(...args);
// };

// ============================================================================
// Test Utilities
// ============================================================================

/**
 * Check if running in CI environment
 */
export const isCI = (): boolean => {
  return import.meta.env.CI === 'true' || import.meta.env.GITHUB_ACTIONS === 'true';
};

/**
 * Wait for a specified duration
 */
export const wait = (ms: number): Promise<void> =>
  new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Flush all pending promises
 */
export const flushPromises = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

/**
 * Create a deferred promise for testing async behavior
 */
export function createDeferred<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (reason: unknown) => void;
} {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;

  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });

  return { promise, resolve, reject };
}
