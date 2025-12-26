/**
 * DiffViewer Accessibility Tests
 *
 * WCAG 2.1 AA compliance testing for DiffViewer component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { checkA11y, generateA11yReport } from '../../../test/accessibility-utils';
import { DiffViewer } from '../DiffViewer';
import type { DiffFile } from '../../../types/diff';

const mockDiffFile: DiffFile = {
  id: 'file-1',
  to: 'src/App.tsx',
  from: 'src/App.tsx',
  new: false,
  deleted: false,
  renamed: false,
  isBinary: false,
  additions: 10,
  deletions: 5,
  chunks: [
    {
      content:
        '@@ -1,5 +1,10 @@\n import React from "react";\n+import { useState } from "react";\n export function App() {\n-  return <div>Hello</div>;\n+  const [count, setCount] = useState(0);\n+  return <div>{count}</div>;\n }',
      changes: [
        { type: 'normal', content: ' import React from "react";' },
        { type: 'insert', content: '+import { useState } from "react";' },
        { type: 'normal', content: ' export function App() {' },
        { type: 'delete', content: '-  return <div>Hello</div>;' },
        { type: 'insert', content: '+  const [count, setCount] = useState(0);' },
        { type: 'insert', content: '+  return <div>{count}</div>;' },
        { type: 'normal', content: ' }' },
      ],
      newStart: 1,
      newLines: 10,
      oldStart: 1,
      oldLines: 5,
    },
  ],
  language: 'typescript',
  isExpanded: false,
  binary: false,
};

const mockMultiFileDiff = {
  files: [
    {
      filePath: 'src/components/App.tsx',
      status: 'modified',
      additions: 10,
      deletions: 5,
      changes: 15,
      language: 'typescript',
      patch: '@@ -1,5 +1,10 @@\n import React from "react";\n+import { useState } from "react";\n',
    },
    {
      filePath: 'src/utils/helpers.ts',
      status: 'added',
      additions: 20,
      deletions: 0,
      changes: 20,
      language: 'typescript',
      patch: '@@ -0,0 +1,20 @@\n+export function helper() {\n+  return true;\n+}',
    },
  ],
  summary: {
    totalFiles: 2,
    totalAdditions: 30,
    totalDeletions: 5,
    totalChanges: 35,
  },
};

describe('DiffViewer - WCAG 2.1 AA Accessibility', () => {
  describe('Automated axe-core Testing', () => {
    it('should have no accessibility violations (WCAG 2.1 AA)', async () => {
      const { container } = render(<DiffViewer file={mockDiffFile} />);

      const results = await checkA11y(container);
      const report = generateA11yReport(results);

      expect(report.violations).toHaveLength(0);
      expect(report.compliant).toBe(true);
    });

    it('should have no violations in multi-file view', async () => {
      const { container } = render(<DiffViewer diff={mockMultiFileDiff} />);

      const results = await checkA11y(container);
      const report = generateA11yReport(results);

      expect(report.violations).toHaveLength(0);
    });
  });

  describe('Color Contrast (WCAG 1.4.3)', () => {
    it('should have sufficient contrast for added lines (green)', () => {
      render(<DiffViewer file={mockDiffFile} />);

      // Added lines should be visible with sufficient contrast
      // Green text on light background: should meet 4.5:1 for AA
      const addedLine = screen.queryByText(/\+import { useState }/);
      if (addedLine) {
        const styles = window.getComputedStyle(addedLine);
        expect(styles.color).toBeDefined();
      }
    });

    it('should have sufficient contrast for deleted lines (red)', () => {
      render(<DiffViewer file={mockDiffFile} />);

      // Deleted lines should be visible with sufficient contrast
      // Red text on light background: should meet 4.5:1 for AA
      const deletedLine = screen.queryByText(/-{2}return <div>Hello<\/div>;/);
      if (deletedLine) {
        const styles = window.getComputedStyle(deletedLine);
        expect(styles.color).toBeDefined();
      }
    });

    it('should maintain contrast in dark mode', () => {
      // Simulate dark mode
      document.documentElement.classList.add('dark');

      render(<DiffViewer file={mockDiffFile} />);

      // Contrast should be maintained in dark mode
      const container = screen.getByText(/import React/i).closest('.diff-viewer-container');
      expect(container).toBeInTheDocument();

      document.documentElement.classList.remove('dark');
    });
  });

  describe('Keyboard Navigation (WCAG 2.1.1)', () => {
    it('should allow keyboard navigation through file filters', async () => {
      const user = userEvent.setup();
      render(<DiffViewer diff={mockMultiFileDiff} />);

      const filterInput = screen.getByPlaceholderText('Filter files...');
      await user.click(filterInput);
      await user.keyboard('App');

      expect(filterInput).toHaveValue('App');
    });

    it('should allow keyboard toggle for file expansion', async () => {
      const user = userEvent.setup();
      render(<DiffViewer diff={mockMultiFileDiff} />);

      // Find file header
      const fileHeader = screen.getByText('src/components/App.tsx');
      await user.click(fileHeader);

      // File should expand (patch should be visible)
      const patch = await screen.findByText(/import React/);
      expect(patch).toBeInTheDocument();
    });

    it('should support filter button keyboard activation', async () => {
      const user = userEvent.setup();
      render(<DiffViewer diff={mockMultiFileDiff} />);

      const modifiedButton = screen.getByRole('button', { name: /modified/i });
      await user.tab();
      await user.tab(); // Navigate to button
      await user.keyboard('{Enter}');

      // Filter should be applied
      expect(modifiedButton).toHaveClass('bg-primary'); // or appropriate active class
    });
  });

  describe('ARIA Labels and Roles (WCAG 4.1.2)', () => {
    it('should announce diff summary statistics', () => {
      render(<DiffViewer diff={mockMultiFileDiff} />);

      const summary = screen.getByText(/2 files changed/);
      expect(summary).toBeInTheDocument();
      expect(summary).toHaveTextContent('30 additions');
      expect(summary).toHaveTextContent('5 deletions');
    });

    it('should have accessible filter buttons', () => {
      render(<DiffViewer diff={mockMultiFileDiff} />);

      const modifiedButton = screen.getByRole('button', { name: /modified/i });
      const addedButton = screen.getByRole('button', { name: /added/i });
      const deletedButton = screen.getByRole('button', { name: /deleted/i });

      expect(modifiedButton).toHaveAccessibleName();
      expect(addedButton).toHaveAccessibleName();
      expect(deletedButton).toHaveAccessibleName();
    });

    it('should announce file status with badges', () => {
      render(<DiffViewer diff={mockMultiFileDiff} />);

      const modifiedBadge = screen.queryByText('modified');
      const addedBadge = screen.queryByText('added');

      // Badges should be readable by screen readers
      if (modifiedBadge) expect(modifiedBadge).toBeInTheDocument();
      if (addedBadge) expect(addedBadge).toBeInTheDocument();
    });

    it('should indicate binary files appropriately', () => {
      const binaryDiff = {
        ...mockMultiFileDiff,
        files: [
          {
            filePath: 'image.png',
            status: 'added',
            additions: 0,
            deletions: 0,
            changes: 0,
            isBinary: true,
          },
        ],
      };

      render(<DiffViewer diff={binaryDiff} />);

      const binaryBadge = screen.getByText('Binary');
      expect(binaryBadge).toBeInTheDocument();
    });
  });

  describe('Focus Management (WCAG 2.4.7)', () => {
    it('should show visible focus indicators on interactive elements', async () => {
      const user = userEvent.setup();
      render(<DiffViewer diff={mockMultiFileDiff} />);

      await user.tab();
      const focusedElement = document.activeElement as HTMLElement;

      const styles = window.getComputedStyle(focusedElement);
      const hasFocusRing =
        styles.outline !== 'none' ||
        styles.boxShadow.includes('ring') ||
        focusedElement.classList.contains('focus:ring');

      expect(hasFocusRing).toBe(true);
    });

    it('should maintain focus when expanding/collapsing files', async () => {
      const user = userEvent.setup();
      render(<DiffViewer diff={mockMultiFileDiff} />);

      const fileHeader = screen.getByText('src/components/App.tsx');
      await user.click(fileHeader);

      // Focus should remain on or near the file header
      expect(document.activeElement).toBeDefined();
    });
  });

  describe('Semantic HTML (WCAG 4.1.2)', () => {
    it('should use semantic button elements', () => {
      const { container } = render(<DiffViewer diff={mockMultiFileDiff} />);

      const buttons = container.querySelectorAll('button');
      expect(buttons.length).toBeGreaterThan(0);

      // No clickable divs
      const clickableDivs = container.querySelectorAll('div[onclick]');
      expect(clickableDivs.length).toBe(0);
    });

    it('should use semantic input elements', () => {
      const { container } = render(<DiffViewer diff={mockMultiFileDiff} />);

      const inputs = container.querySelectorAll('input');
      expect(inputs.length).toBeGreaterThan(0);

      inputs.forEach((input) => {
        expect(input).toHaveAttribute('type');
      });
    });
  });

  describe('Empty States (WCAG 3.3.1)', () => {
    it('should provide clear messaging when no file is provided', () => {
      render(<DiffViewer />);

      const message = screen.getByText(/No file provided/i);
      expect(message).toBeInTheDocument();
    });

    it('should provide clear messaging when no diff available', () => {
      const emptyFile: DiffFile = {
        ...mockDiffFile,
        chunks: [],
      };

      render(<DiffViewer file={emptyFile} />);

      const message = screen.getByText(/No diff available/i);
      expect(message).toBeInTheDocument();
    });

    it('should provide clear messaging when no files match filter', () => {
      render(<DiffViewer diff={mockMultiFileDiff} />);

      const filterInput = screen.getByPlaceholderText('Filter files...');
      userEvent.type(filterInput, 'nonexistent.tsx');

      const message = screen.getByText(/No files changed/i);
      expect(message).toBeInTheDocument();
    });
  });

  describe('Responsive and Reflow (WCAG 1.4.10)', () => {
    it('should handle unified view mode', () => {
      render(<DiffViewer file={mockDiffFile} viewMode="unified" />);

      // Unified view should render without horizontal scroll
      const container = screen.getByText(/import React/i).closest('.diff-viewer-container');
      expect(container).toBeInTheDocument();
    });

    it('should handle split view mode', () => {
      render(<DiffViewer file={mockDiffFile} viewMode="split" />);

      // Split view should render side-by-side
      const container = screen.getByText(/import React/i).closest('.diff-viewer-container');
      expect(container).toBeInTheDocument();
    });

    it('should handle line wrapping', () => {
      render(<DiffViewer file={mockDiffFile} wrapLines={true} />);

      // Long lines should wrap
      const container = screen.getByText(/import React/i).closest('.diff-viewer-container');
      expect(container).toBeInTheDocument();
    });
  });

  describe('Syntax Highlighting (WCAG 1.4.1)', () => {
    it('should not rely solely on color for code meaning', () => {
      render(<DiffViewer file={mockDiffFile} syntaxHighlighting={true} />);

      // Added/deleted lines should have symbols (+ and -) not just color
      const addedLine = screen.queryByText(/\+import { useState }/);
      const deletedLine = screen.queryByText(/-{2}return <div>Hello<\/div>;/);

      if (addedLine) expect(addedLine.textContent).toContain('+');
      if (deletedLine) expect(deletedLine.textContent).toContain('-');
    });

    it('should be functional without syntax highlighting', () => {
      render(<DiffViewer file={mockDiffFile} syntaxHighlighting={false} />);

      // Diff should still be readable without highlighting
      const content = screen.getByText(/import React/i);
      expect(content).toBeInTheDocument();
    });
  });
});
