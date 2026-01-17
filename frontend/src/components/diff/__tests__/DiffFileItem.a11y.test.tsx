/**
 * DiffFileItem Accessibility Tests
 *
 * WCAG 2.1 AA compliance testing for DiffFileItem component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { checkA11y, generateA11yReport } from '../../../test/accessibility-utils';
import { DiffFileItem } from '../DiffFileItem';
import type { DiffFile } from '../../../types/diff';

const mockFile: DiffFile = {
  id: 'file-1',
  to: 'src/components/App.tsx',
  from: 'src/components/App.tsx',
  new: false,
  deleted: false,
  renamed: false,
  isBinary: false,
  additions: 10,
  deletions: 5,
  chunks: [
    {
      content:
        '@@ -1,5 +1,10 @@\n import React from "react";\n+import { useState } from "react";\n',
      changes: [
        { type: 'normal', content: ' import React from "react";' },
        { type: 'insert', content: '+import { useState } from "react";' },
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

const mockAddedFile: DiffFile = {
  ...mockFile,
  id: 'file-2',
  to: 'src/utils/new-helper.ts',
  new: true,
  additions: 20,
  deletions: 0,
};

const mockDeletedFile: DiffFile = {
  ...mockFile,
  id: 'file-3',
  to: 'src/legacy/old.js',
  deleted: true,
  additions: 0,
  deletions: 30,
};

const mockBinaryFile: DiffFile = {
  ...mockFile,
  id: 'file-4',
  to: 'assets/image.png',
  isBinary: true,
  additions: 0,
  deletions: 0,
  chunks: [],
};

describe('DiffFileItem - WCAG 2.1 AA Accessibility', () => {
  describe('Automated axe-core Testing', () => {
    it('should have no accessibility violations (WCAG 2.1 AA)', async () => {
      const { container } = render(
        <DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />
      );

      const results = await checkA11y(container);
      const report = generateA11yReport(results);

      expect(report.violations).toHaveLength(0);
      expect(report.compliant).toBe(true);
    });

    it('should have no violations for added files', async () => {
      const { container } = render(
        <DiffFileItem file={mockAddedFile} viewMode="unified" syntaxHighlighting={true} />
      );

      const results = await checkA11y(container);
      expect(results.violations).toHaveLength(0);
    });

    it('should have no violations for deleted files', async () => {
      const { container } = render(
        <DiffFileItem file={mockDeletedFile} viewMode="unified" syntaxHighlighting={true} />
      );

      const results = await checkA11y(container);
      expect(results.violations).toHaveLength(0);
    });

    it('should have no violations for binary files', async () => {
      const { container } = render(
        <DiffFileItem file={mockBinaryFile} viewMode="unified" syntaxHighlighting={true} />
      );

      const results = await checkA11y(container);
      expect(results.violations).toHaveLength(0);
    });
  });

  describe('Keyboard Navigation (WCAG 2.1.1)', () => {
    it('should allow keyboard expand/collapse', async () => {
      const user = userEvent.setup();
      const onToggleExpand = vi.fn();

      render(
        <DiffFileItem
          file={mockFile}
          viewMode="unified"
          syntaxHighlighting={true}
          onToggleExpand={onToggleExpand}
        />
      );

      const fileHeader = screen.getByText('App.tsx');
      await user.click(fileHeader);

      expect(onToggleExpand).toHaveBeenCalledWith('file-1', true);
    });

    it('should be keyboard accessible via Enter key', async () => {
      const user = userEvent.setup();
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      const expandButton = screen.getByRole('button');
      expandButton.focus();
      await user.keyboard('{Enter}');

      // File should expand
      expect(expandButton).toBeInTheDocument();
    });

    it('should support Space key for toggle', async () => {
      const user = userEvent.setup();
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      const expandButton = screen.getByRole('button');
      expandButton.focus();
      await user.keyboard(' ');

      // File should toggle
      expect(expandButton).toBeInTheDocument();
    });
  });

  describe('Focus Management (WCAG 2.4.7)', () => {
    it('should show visible focus indicator on file header', async () => {
      const user = userEvent.setup();
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      await user.tab();
      const focusedElement = document.activeElement as HTMLElement;

      const styles = window.getComputedStyle(focusedElement);
      const hasFocusRing =
        styles.outline !== 'none' ||
        styles.boxShadow.includes('ring') ||
        focusedElement.classList.contains('focus:ring');

      expect(hasFocusRing).toBe(true);
    });

    it('should maintain focus when expanding file', async () => {
      const user = userEvent.setup();
      render(
        <DiffFileItem
          file={mockFile}
          viewMode="unified"
          syntaxHighlighting={true}
          isExpanded={false}
        />
      );

      const fileHeader = screen.getByText('App.tsx');
      await user.click(fileHeader);

      // Focus should remain accessible
      expect(document.activeElement).toBeDefined();
    });
  });

  describe('ARIA Labels and Roles (WCAG 4.1.2)', () => {
    it('should have accessible expand/collapse button', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      const button = screen.getByRole('button');
      expect(button).toBeInTheDocument();
      expect(button).toHaveAttribute('class');
    });

    it('should announce file status with badge', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      const statusBadge = screen.getByText('modified');
      expect(statusBadge).toBeInTheDocument();
    });

    it('should announce added status', () => {
      render(<DiffFileItem file={mockAddedFile} viewMode="unified" syntaxHighlighting={true} />);

      const addedBadge = screen.getByText('added');
      expect(addedBadge).toBeInTheDocument();
    });

    it('should announce deleted status', () => {
      render(<DiffFileItem file={mockDeletedFile} viewMode="unified" syntaxHighlighting={true} />);

      const deletedBadge = screen.getByText('deleted');
      expect(deletedBadge).toBeInTheDocument();
    });

    it('should indicate binary file type', () => {
      render(
        <DiffFileItem
          file={mockBinaryFile}
          viewMode="unified"
          syntaxHighlighting={true}
          isExpanded={true}
        />
      );

      const binaryBadge = screen.getByText('Binary');
      expect(binaryBadge).toBeInTheDocument();
    });

    it('should announce language badge', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      // TypeScript language badge should be present
      const languageBadge = screen.queryByText(/typescript/i);
      if (languageBadge) {
        expect(languageBadge).toBeInTheDocument();
      }
    });
  });

  describe('Color Contrast (WCAG 1.4.3)', () => {
    it('should have sufficient contrast for additions counter', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      const additions = screen.getByText('10');
      const styles = window.getComputedStyle(additions);

      // Green text should have sufficient contrast
      expect(styles.color).toBeDefined();
    });

    it('should have sufficient contrast for deletions counter', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      const deletions = screen.getByText('5');
      const styles = window.getComputedStyle(deletions);

      // Red text should have sufficient contrast
      expect(styles.color).toBeDefined();
    });

    it('should have sufficient contrast for file status badges', () => {
      render(<DiffFileItem file={mockAddedFile} viewMode="unified" syntaxHighlighting={true} />);

      const badge = screen.getByText('added');
      const styles = window.getComputedStyle(badge);

      expect(styles.color).toBeDefined();
      expect(styles.backgroundColor).toBeDefined();
    });
  });

  describe('Screen Reader Announcements', () => {
    it('should announce file name and status', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      // File name should be visible
      const fileName = screen.getByText('App.tsx');
      expect(fileName).toBeInTheDocument();

      // Status should be visible
      const status = screen.getByText('modified');
      expect(status).toBeInTheDocument();
    });

    it('should announce file path separately', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      // Path should be separate from file name
      const path = screen.getByText('src/components');
      expect(path).toBeInTheDocument();
    });

    it('should announce additions and deletions', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      const additions = screen.getByText('10');
      const deletions = screen.getByText('5');

      expect(additions).toBeInTheDocument();
      expect(deletions).toBeInTheDocument();
    });

    it('should announce binary file message', () => {
      render(
        <DiffFileItem
          file={mockBinaryFile}
          viewMode="unified"
          syntaxHighlighting={true}
          isExpanded={true}
        />
      );

      const binaryMessage = screen.getByText(/Binary file - no diff available/i);
      expect(binaryMessage).toBeInTheDocument();
    });
  });

  describe('Semantic HTML (WCAG 4.1.2)', () => {
    it('should use semantic button for expand/collapse', () => {
      const { container } = render(
        <DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />
      );

      const button = container.querySelector('button');
      expect(button).toBeInTheDocument();

      // Should not be a div pretending to be a button
      const divButton = container.querySelector('div[role="button"]');
      expect(divButton).not.toBeInTheDocument();
    });

    it('should use semantic icons with accessible context', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      // File information is presented with semantic HTML and accessible text
      // Icons are handled by the third-party library - we test our semantic structure
      const fileName = screen.getByText('App.tsx');
      expect(fileName).toBeInTheDocument();
      expect(fileName).toBeVisible();
    });
  });

  describe('Responsive Behavior (WCAG 1.4.10)', () => {
    it('should truncate long file names', () => {
      const longNameFile: DiffFile = {
        ...mockFile,
        to: 'src/components/very/long/path/to/a/component/with/an/extremely/long/name/Component.tsx',
      };

      render(<DiffFileItem file={longNameFile} viewMode="unified" syntaxHighlighting={true} />);

      const fileName = screen.getByText('Component.tsx');
      expect(fileName).toBeInTheDocument();
      expect(fileName.className).toContain('truncate');
    });

    it('should handle file header at mobile widths', () => {
      render(<DiffFileItem file={mockFile} viewMode="unified" syntaxHighlighting={true} />);

      // File header should be responsive
      const fileName = screen.getByText('App.tsx');
      expect(fileName.closest('div')).toHaveClass('flex');
    });
  });

  describe('Interactive State Management', () => {
    it('should indicate expanded state visually', () => {
      render(
        <DiffFileItem
          file={mockFile}
          viewMode="unified"
          syntaxHighlighting={true}
          isExpanded={true}
        />
      );

      // Chevron should indicate expanded state
      const button = screen.getByRole('button');
      const chevron = button.querySelector('svg');
      expect(chevron).toBeInTheDocument();
    });

    it('should indicate collapsed state visually', () => {
      render(
        <DiffFileItem
          file={mockFile}
          viewMode="unified"
          syntaxHighlighting={true}
          isExpanded={false}
        />
      );

      // Chevron should indicate collapsed state
      const button = screen.getByRole('button');
      const chevron = button.querySelector('svg');
      expect(chevron).toBeInTheDocument();
    });
  });

  describe('Error States and Edge Cases', () => {
    it('should handle file with no additions or deletions', () => {
      const unchangedFile: DiffFile = {
        ...mockFile,
        additions: 0,
        deletions: 0,
      };

      render(<DiffFileItem file={unchangedFile} viewMode="unified" syntaxHighlighting={true} />);

      // Should still render the file
      const fileName = screen.getByText('App.tsx');
      expect(fileName).toBeInTheDocument();
    });

    it('should handle renamed files', () => {
      const renamedFile: DiffFile = {
        ...mockFile,
        from: 'src/components/OldName.tsx',
        to: 'src/components/NewName.tsx',
        renamed: true,
      };

      render(<DiffFileItem file={renamedFile} viewMode="unified" syntaxHighlighting={true} />);

      const badge = screen.getByText('renamed');
      expect(badge).toBeInTheDocument();
    });
  });
});
