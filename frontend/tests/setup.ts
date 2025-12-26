/**
 * Vitest Setup File
 *
 * Global test configuration and mocks
 *
 * Best Practice: Mock third-party UI libraries to focus on testing our code,
 * not the library's internals. We test the integration, not the library itself.
 *
 * References:
 * - https://gist.github.com/jakubdrozdek/5ab10c334ca03d323fc0a95c82422ffd
 * - https://memo.d.foundation/playground/frontend/react/testing-strategies/
 */

import '@testing-library/jest-dom';
import { vi } from 'vitest';
import React from 'react';

/**
 * Mock @git-diff-view/react library
 *
 * The library uses Canvas API and complex DOM manipulation that doesn't work in JSDOM.
 * We mock it to render basic diff content so our accessibility tests can query it.
 *
 * We DON'T test the library's rendering - that's the library's job.
 * We DO test our accessibility features: ARIA labels, keyboard navigation, etc.
 */
vi.mock('@git-diff-view/react', () => ({
  DiffView: ({ diffText }: { diffText?: string }) => {
    if (!diffText) return null;

    // Parse diff lines for test queryability
    const lines = diffText.split('\n');

    return React.createElement(
      'div',
      { className: 'diff-view-mock', 'data-testid': 'diff-view' },
      lines.map((line, index) =>
        React.createElement('div', { key: index, className: 'diff-line' }, line)
      )
    );
  },
}));

/**
 * Mock Canvas API
 *
 * Some components may use canvas indirectly. Provide a minimal mock
 * to prevent "not implemented" errors in JSDOM.
 */
HTMLCanvasElement.prototype.getContext = vi.fn(
  () => null
) as unknown as typeof HTMLCanvasElement.prototype.getContext;
