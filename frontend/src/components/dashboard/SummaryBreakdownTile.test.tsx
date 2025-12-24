import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import SummaryBreakdownTile from './SummaryBreakdownTile';
import { Boxes } from 'lucide-react';

describe('SummaryBreakdownTile', () => {
  const defaultProps = {
    title: 'Test Title',
    count: 100,
    breakdown: { public: 50, private: 30, archived: 20 },
    icon: Boxes,
  };

  describe('Component Rendering', () => {
    it('renders title correctly', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      expect(screen.getByText('Test Title')).toBeInTheDocument();
    });

    it('renders the main count', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      expect(screen.getByText('100')).toBeInTheDocument();
    });

    it('renders with custom title', () => {
      render(<SummaryBreakdownTile {...defaultProps} title="Custom Title" />);
      expect(screen.getByText('Custom Title')).toBeInTheDocument();
    });
  });

  describe('Visibility Counts Display', () => {
    it('displays all visibility counts', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      expect(screen.getByText('50')).toBeInTheDocument();
      expect(screen.getByText('30')).toBeInTheDocument();
      expect(screen.getByText('20')).toBeInTheDocument();
    });

    it('displays zero counts correctly', () => {
      render(
        <SummaryBreakdownTile
          {...defaultProps}
          count={0}
          breakdown={{ public: 0, private: 0, archived: 0 }}
        />
      );
      // Count of zeros - main count plus three breakdown zeros
      const zeros = screen.getAllByText('0');
      expect(zeros.length).toBeGreaterThanOrEqual(4);
    });

    it('displays large numbers correctly', () => {
      render(
        <SummaryBreakdownTile
          {...defaultProps}
          count={999999}
          breakdown={{ public: 10000, private: 5000, archived: 1000 }}
        />
      );
      expect(screen.getByText('999999')).toBeInTheDocument();
      expect(screen.getByText('10000')).toBeInTheDocument();
    });
  });

  describe('Loading State', () => {
    it('shows loading spinner when isLoading is true', () => {
      render(<SummaryBreakdownTile {...defaultProps} isLoading={true} />);
      const spinner = document.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });

    it('does not show count when loading', () => {
      render(<SummaryBreakdownTile {...defaultProps} isLoading={true} />);
      expect(screen.queryByText('100')).not.toBeInTheDocument();
    });

    it('shows count when not loading', () => {
      render(<SummaryBreakdownTile {...defaultProps} isLoading={false} />);
      expect(screen.getByText('100')).toBeInTheDocument();
    });

    it('defaults to not loading when isLoading is undefined', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      expect(screen.getByText('100')).toBeInTheDocument();
    });
  });

  describe('Count Color', () => {
    it('applies custom count color', () => {
      render(<SummaryBreakdownTile {...defaultProps} countColor="text-ampel-green" />);
      const countElement = screen.getByRole('status');
      expect(countElement).toHaveClass('text-ampel-green');
    });

    it('works without count color', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      const countElement = screen.getByRole('status');
      expect(countElement).not.toHaveClass('text-ampel-green');
    });
  });

  describe('Icon Labels', () => {
    it('displays correct icon labels', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      expect(screen.getByText('Public')).toBeInTheDocument();
      expect(screen.getByText('Private')).toBeInTheDocument();
      expect(screen.getByText('Archived')).toBeInTheDocument();
    });
  });

  describe('Icon Rendering', () => {
    it('renders visibility icons with correct styles', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      // Check for Globe icon (green-600)
      const globeIcon = document.querySelector('.text-green-600');
      expect(globeIcon).toBeInTheDocument();

      // Check for Lock icon (amber-600)
      const lockIcon = document.querySelector('.text-amber-600');
      expect(lockIcon).toBeInTheDocument();

      // Check for Archive icon (gray-500)
      const archiveIcon = document.querySelector('.text-gray-500');
      expect(archiveIcon).toBeInTheDocument();
    });

    it('renders header icon', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      // The header icon should have text-muted-foreground class
      const headerIcon = document.querySelector('.text-muted-foreground');
      expect(headerIcon).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has semantic card structure', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      expect(screen.getByRole('region')).toBeInTheDocument();
    });

    it('has proper aria labels', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      expect(
        screen.getByRole('region', { name: 'Test Title summary with visibility breakdown' })
      ).toBeInTheDocument();
      expect(screen.getByRole('status', { name: 'Test Title: 100' })).toBeInTheDocument();
      expect(screen.getByRole('list', { name: 'Visibility breakdown' })).toBeInTheDocument();
    });

    it('has listitem roles for breakdown items', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      const listItems = screen.getAllByRole('listitem');
      expect(listItems).toHaveLength(3);
    });
  });

  describe('Edge Cases', () => {
    it('handles single visibility type with all others zero', () => {
      render(
        <SummaryBreakdownTile
          {...defaultProps}
          count={100}
          breakdown={{ public: 100, private: 0, archived: 0 }}
        />
      );
      // Use getAllByText since "100" appears in both main count and public breakdown
      const elements = screen.getAllByText('100');
      expect(elements.length).toBe(2);
    });

    it('handles all archived repositories', () => {
      render(
        <SummaryBreakdownTile
          {...defaultProps}
          count={50}
          breakdown={{ public: 0, private: 0, archived: 50 }}
        />
      );
      // Use getAllByText since "50" appears in both main count and archived breakdown
      const elements = screen.getAllByText('50');
      expect(elements.length).toBe(2);
    });

    it('renders correctly with very long title', () => {
      render(
        <SummaryBreakdownTile
          {...defaultProps}
          title="This is a very long title that might cause layout issues"
        />
      );
      expect(
        screen.getByText('This is a very long title that might cause layout issues')
      ).toBeInTheDocument();
    });
  });

  describe('Component Props Validation', () => {
    it('accepts and renders different icon components', () => {
      const CustomIcon = () => <span data-testid="custom-icon">📦</span>;
      render(<SummaryBreakdownTile {...defaultProps} icon={CustomIcon} />);
      expect(screen.getByTestId('custom-icon')).toBeInTheDocument();
    });

    it('maintains consistent layout across different breakdown values', () => {
      const { rerender } = render(
        <SummaryBreakdownTile
          {...defaultProps}
          breakdown={{ public: 1, private: 1, archived: 1 }}
        />
      );
      expect(screen.getByText('Public')).toBeInTheDocument();

      rerender(
        <SummaryBreakdownTile
          {...defaultProps}
          breakdown={{ public: 999, private: 999, archived: 999 }}
        />
      );
      expect(screen.getByText('Public')).toBeInTheDocument();
    });
  });

  describe('Visual Consistency', () => {
    it('maintains color scheme consistency', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      const publicIcon = document.querySelector('.text-green-600');
      const privateIcon = document.querySelector('.text-amber-600');
      const archivedIcon = document.querySelector('.text-gray-500');

      expect(publicIcon).toBeInTheDocument();
      expect(privateIcon).toBeInTheDocument();
      expect(archivedIcon).toBeInTheDocument();
    });

    it('uses consistent text styles for labels', () => {
      render(<SummaryBreakdownTile {...defaultProps} />);
      const mutedTexts = document.querySelectorAll('.text-muted-foreground');
      expect(mutedTexts.length).toBeGreaterThan(0);
    });
  });
});
