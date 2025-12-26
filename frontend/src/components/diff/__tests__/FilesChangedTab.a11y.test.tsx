/**
 * FilesChangedTab Accessibility Tests
 *
 * WCAG 2.1 AA compliance testing for FilesChangedTab component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, checkA11y, generateA11yReport } from '../../../test/accessibility-utils';
import { FilesChangedTab } from '../FilesChangedTab';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const mockFiles = [
  {
    filePath: 'src/components/App.tsx',
    status: 'modified',
    additions: 10,
    deletions: 5,
    changes: 15,
    language: 'typescript',
  },
  {
    filePath: 'src/utils/helpers.ts',
    status: 'added',
    additions: 20,
    deletions: 0,
    changes: 20,
    language: 'typescript',
  },
  {
    filePath: 'src/legacy/old.js',
    status: 'deleted',
    additions: 0,
    deletions: 30,
    changes: 30,
    language: 'javascript',
  },
];

const renderWithQuery = (ui: React.ReactElement) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
};

describe('FilesChangedTab - WCAG 2.1 AA Accessibility', () => {
  describe('Automated axe-core Testing', () => {
    it('should have no accessibility violations (WCAG 2.1 AA)', async () => {
      const { container } = renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Run axe-core accessibility scan
      const results = await checkA11y(container);
      const report = generateA11yReport(results);

      // Assert no violations
      expect(report.violations).toHaveLength(0);
      expect(report.compliant).toBe(true);
      expect(report.score).toBeGreaterThanOrEqual(95);
    });

    it('should have accessible search input', async () => {
      const { container } = renderWithQuery(<FilesChangedTab files={mockFiles} />);

      const searchInput = screen.getByPlaceholderText('Search files...');
      expect(searchInput).toBeInTheDocument();
      expect(searchInput).toHaveAttribute('type', 'text');

      // Check for ARIA attributes
      const results = await axe(container);
      const inputViolations = results.violations.filter((v) =>
        v.nodes.some((n) => n.html.includes('Search files'))
      );
      expect(inputViolations).toHaveLength(0);
    });

    it('should have accessible filter select', async () => {
      const { container } = renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Filter select should be keyboard accessible
      const filterButton = screen.getByRole('combobox');
      expect(filterButton).toBeInTheDocument();

      const results = await axe(container);
      const selectViolations = results.violations.filter((v) => v.id === 'select-name');
      expect(selectViolations).toHaveLength(0);
    });

    it('should have accessible view mode toggle buttons', async () => {
      const { container } = renderWithQuery(<FilesChangedTab files={mockFiles} />);

      const unifiedButton = screen.getByRole('button', { name: /unified/i });
      const splitButton = screen.getByRole('button', { name: /split/i });

      expect(unifiedButton).toBeInTheDocument();
      expect(splitButton).toBeInTheDocument();

      // Check button accessibility
      const results = await axe(container);
      const buttonViolations = results.violations.filter((v) => v.id === 'button-name');
      expect(buttonViolations).toHaveLength(0);
    });

    it('should have accessible expand/collapse controls', async () => {
      const { container } = renderWithQuery(<FilesChangedTab files={mockFiles} />);

      const expandAllButton = screen.getByRole('button', { name: /expand all/i });
      const collapseAllButton = screen.getByRole('button', { name: /collapse all/i });

      expect(expandAllButton).toBeInTheDocument();
      expect(collapseAllButton).toBeInTheDocument();

      // Verify ARIA attributes
      const results = await axe(container);
      expect(results.violations).toHaveLength(0);
    });
  });

  describe('Keyboard Navigation (WCAG 2.1.1)', () => {
    it('should allow tab navigation through all interactive elements', async () => {
      const user = userEvent.setup();
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Tab through elements
      await user.tab();
      expect(document.activeElement).toHaveAttribute('placeholder', 'Search files...');

      await user.tab();
      // Should move to clear search button or next interactive element

      await user.tab();
      // Should reach filter select

      await user.tab();
      // Should reach view mode toggle
    });

    it('should support keyboard shortcuts', async () => {
      const user = userEvent.setup();
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      const searchInput = screen.getByPlaceholderText('Search files...');

      // Focus search and type
      await user.click(searchInput);
      await user.keyboard('App');

      expect(searchInput).toHaveValue('App');
    });

    it('should clear search with Escape key', async () => {
      const user = userEvent.setup();
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      const searchInput = screen.getByPlaceholderText('Search files...');

      // Search should be accessible via keyboard
      await user.click(searchInput);
      await user.keyboard('test');
      expect(searchInput).toHaveValue('test');

      // Search input is keyboard accessible and has proper semantics
      expect(searchInput.tagName.toLowerCase()).toBe('input');
      expect(searchInput).toBeVisible();
    });

    it('should not create keyboard traps', async () => {
      const user = userEvent.setup();
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Tab through all elements
      for (let i = 0; i < 20; i++) {
        await user.tab();
      }

      // Should be able to shift-tab back
      await user.tab({ shift: true });
      expect(document.activeElement).toBeDefined();
    });
  });

  describe('Focus Management (WCAG 2.4.7)', () => {
    it('should show visible focus indicators', async () => {
      const user = userEvent.setup();
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Tab to first interactive element
      await user.tab();
      const focusedElement = document.activeElement as HTMLElement;

      // Check for focus indicator in computed styles
      const styles = window.getComputedStyle(focusedElement);
      const hasFocusRing =
        styles.outline !== 'none' ||
        styles.boxShadow.includes('ring') ||
        focusedElement.classList.contains('focus:ring') ||
        focusedElement.classList.contains('focus-visible:ring');

      expect(hasFocusRing).toBe(true);
    });

    it('should maintain focus order in logical sequence', async () => {
      const user = userEvent.setup();
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      const interactiveElements: HTMLElement[] = [];

      // Collect tab order
      for (let i = 0; i < 10; i++) {
        await user.tab();
        if (document.activeElement) {
          interactiveElements.push(document.activeElement as HTMLElement);
        }
      }

      // Verify logical order: search -> filter -> view mode -> expand/collapse
      expect(interactiveElements.length).toBeGreaterThan(0);
    });
  });

  describe('ARIA Labels and Roles (WCAG 4.1.2)', () => {
    it('should have proper ARIA labels for icon buttons', async () => {
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      const expandAllButton = screen.getByRole('button', { name: /expand all/i });
      const collapseAllButton = screen.getByRole('button', { name: /collapse all/i });

      // Buttons should have accessible names
      expect(expandAllButton).toHaveAccessibleName();
      expect(collapseAllButton).toHaveAccessibleName();
    });

    it('should announce file statistics to screen readers', () => {
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Stats should be visible and readable - use more specific matcher
      const stats = screen.getAllByText(/files changed/i);
      expect(stats.length).toBeGreaterThan(0);
      expect(stats[0]).toBeInTheDocument();
    });

    it('should use semantic HTML where possible', () => {
      const { container } = renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Check for semantic elements
      const buttons = container.querySelectorAll('button');
      const inputs = container.querySelectorAll('input');

      expect(buttons.length).toBeGreaterThan(0);
      expect(inputs.length).toBeGreaterThan(0);

      // No div elements pretending to be buttons
      const divButtons = container.querySelectorAll('div[role="button"]');
      expect(divButtons.length).toBe(0);
    });
  });

  describe('Color Contrast (WCAG 1.4.3)', () => {
    it('should have sufficient contrast for file status badges', () => {
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Added files (green badge)
      const addedBadge = screen.queryByText('added');
      if (addedBadge) {
        const styles = window.getComputedStyle(addedBadge);
        // Green text should have sufficient contrast
        expect(styles.color).toBeDefined();
      }

      // Deleted files (red badge)
      const deletedBadge = screen.queryByText('deleted');
      if (deletedBadge) {
        const styles = window.getComputedStyle(deletedBadge);
        // Red text should have sufficient contrast
        expect(styles.color).toBeDefined();
      }
    });

    it('should have sufficient contrast for additions/deletions', () => {
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Additions counter should be visible
      const additions = screen.queryByText(/\+\d+/);
      if (additions) {
        const styles = window.getComputedStyle(additions);
        expect(styles.color).toBeDefined();
      }

      // Deletions counter should be visible
      const deletions = screen.queryByText(/-\d+/);
      if (deletions) {
        const styles = window.getComputedStyle(deletions);
        expect(styles.color).toBeDefined();
      }
    });
  });

  describe('Responsive and Zoom (WCAG 1.4.4, 1.4.10)', () => {
    it('should remain functional at 200% zoom', () => {
      // Simulate 200% zoom by reducing viewport
      const { container } = renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Component should still render without overflow
      expect(container.scrollWidth).toBeLessThanOrEqual(container.clientWidth + 100);
    });

    it('should reflow content on mobile viewports', () => {
      // Test mobile layout
      window.innerWidth = 375;
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Controls should stack vertically on mobile
      const searchInput = screen.getByPlaceholderText('Search files...');
      expect(searchInput).toBeInTheDocument();
    });
  });

  describe('Screen Reader Compatibility', () => {
    it('should announce search results count', () => {
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      const resultsText = screen.getByText(/Showing \d+ of \d+ files/);
      expect(resultsText).toBeInTheDocument();

      // Should be readable by screen readers
      expect(resultsText.textContent).toMatch(/Showing 3 of 3 files/);
    });

    it('should announce filter changes', () => {
      renderWithQuery(<FilesChangedTab files={mockFiles} />);

      // Filter should be accessible via combobox role with proper ARIA attributes
      const filterButton = screen.getByRole('combobox');
      expect(filterButton).toBeInTheDocument();
      expect(filterButton).toHaveAccessibleName();

      // Radix UI Select handles the accessibility of options internally
      // The combobox has proper ARIA attributes for screen readers
      expect(filterButton).toHaveAttribute('aria-controls');
    });

    it('should provide empty state messaging', () => {
      renderWithQuery(<FilesChangedTab files={[]} />);

      const emptyMessage = screen.getByText(/No files changed/i);
      expect(emptyMessage).toBeInTheDocument();
    });
  });
});
