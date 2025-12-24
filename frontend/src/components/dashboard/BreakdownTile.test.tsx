import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import BreakdownTile from './BreakdownTile';
import { Boxes } from 'lucide-react';
import type { VisibilityBreakdown } from '@/types';

describe('BreakdownTile', () => {
  const mockBreakdown: VisibilityBreakdown = {
    public: 10,
    private: 5,
    archived: 2,
  };

  describe('Component Rendering', () => {
    it('renders title correctly', () => {
      render(<BreakdownTile title="Test Breakdown" breakdown={mockBreakdown} icon={Boxes} />);
      expect(screen.getByText('Test Breakdown')).toBeInTheDocument();
    });

    it('renders with custom title', () => {
      render(
        <BreakdownTile title="Repositories by Visibility" breakdown={mockBreakdown} icon={Boxes} />
      );
      expect(screen.getByText('Repositories by Visibility')).toBeInTheDocument();
    });
  });

  describe('Visibility Counts Display', () => {
    it('displays all visibility counts', () => {
      render(<BreakdownTile title="Repositories" breakdown={mockBreakdown} icon={Boxes} />);

      expect(screen.getByText('10')).toBeInTheDocument(); // Public
      expect(screen.getByText('5')).toBeInTheDocument(); // Private
      expect(screen.getByText('2')).toBeInTheDocument(); // Archived
    });

    it('displays zero counts correctly', () => {
      const zeroBreakdown: VisibilityBreakdown = {
        public: 0,
        private: 0,
        archived: 0,
      };

      render(<BreakdownTile title="Empty Breakdown" breakdown={zeroBreakdown} icon={Boxes} />);

      const zeroCounts = screen.getAllByText('0');
      expect(zeroCounts).toHaveLength(3);
    });

    it('displays large numbers correctly', () => {
      const largeBreakdown: VisibilityBreakdown = {
        public: 999,
        private: 456,
        archived: 123,
      };

      render(<BreakdownTile title="Large Numbers" breakdown={largeBreakdown} icon={Boxes} />);

      expect(screen.getByText('999')).toBeInTheDocument();
      expect(screen.getByText('456')).toBeInTheDocument();
      expect(screen.getByText('123')).toBeInTheDocument();
    });
  });

  describe('Loading State', () => {
    it('shows loading spinner when isLoading is true', () => {
      const { container } = render(
        <BreakdownTile
          title="Loading Test"
          breakdown={mockBreakdown}
          icon={Boxes}
          isLoading={true}
        />
      );

      // Check for spinner element
      const spinner = container.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });

    it('does not show counts when loading', () => {
      render(
        <BreakdownTile
          title="Loading Test"
          breakdown={mockBreakdown}
          icon={Boxes}
          isLoading={true}
        />
      );

      // Should not show any counts
      expect(screen.queryByText('10')).not.toBeInTheDocument();
      expect(screen.queryByText('5')).not.toBeInTheDocument();
      expect(screen.queryByText('2')).not.toBeInTheDocument();
    });

    it('shows counts when not loading', () => {
      render(
        <BreakdownTile
          title="Not Loading"
          breakdown={mockBreakdown}
          icon={Boxes}
          isLoading={false}
        />
      );

      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('5')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument();
    });

    it('defaults to not loading when isLoading is undefined', () => {
      render(<BreakdownTile title="Default State" breakdown={mockBreakdown} icon={Boxes} />);

      expect(screen.getByText('10')).toBeInTheDocument();
    });
  });

  describe('Icon Labels', () => {
    it('displays correct icon labels', () => {
      render(<BreakdownTile title="Repos" breakdown={mockBreakdown} icon={Boxes} />);

      expect(screen.getByText('Public')).toBeInTheDocument();
      expect(screen.getByText('Private')).toBeInTheDocument();
      expect(screen.getByText('Archived')).toBeInTheDocument();
    });

    it('labels are case-sensitive', () => {
      render(<BreakdownTile title="Repos" breakdown={mockBreakdown} icon={Boxes} />);

      // Should be "Public" not "public"
      expect(screen.queryByText('public')).not.toBeInTheDocument();
      expect(screen.getByText('Public')).toBeInTheDocument();
    });
  });

  describe('Icon Rendering', () => {
    it('renders visibility icons with correct styles', () => {
      const { container } = render(
        <BreakdownTile title="Icons Test" breakdown={mockBreakdown} icon={Boxes} />
      );

      // Check for Globe icon (public) with green color
      const globeIcon = container.querySelector('.text-green-600');
      expect(globeIcon).toBeInTheDocument();

      // Check for Lock icon (private) with amber color
      const lockIcon = container.querySelector('.text-amber-600');
      expect(lockIcon).toBeInTheDocument();

      // Check for Archive icon (archived) with gray color
      const archiveIcon = container.querySelector('.text-gray-500');
      expect(archiveIcon).toBeInTheDocument();
    });

    it('renders header icon', () => {
      const { container } = render(
        <BreakdownTile title="Header Icon Test" breakdown={mockBreakdown} icon={Boxes} />
      );

      // Header icon should have muted foreground color
      const headerIcon = container.querySelector('.text-muted-foreground');
      expect(headerIcon).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has semantic card structure', () => {
      const { container } = render(
        <BreakdownTile title="Accessibility Test" breakdown={mockBreakdown} icon={Boxes} />
      );

      // Check for Card component
      const card = container.querySelector('[class*="card"]');
      expect(card).toBeInTheDocument();
    });

    it('uses semantic font weights for labels and values', () => {
      const { container } = render(
        <BreakdownTile title="Font Test" breakdown={mockBreakdown} icon={Boxes} />
      );

      // Values should be semibold
      const values = container.querySelectorAll('.font-semibold');
      expect(values.length).toBeGreaterThan(0);
    });

    it('has proper text hierarchy', () => {
      render(<BreakdownTile title="Hierarchy Test" breakdown={mockBreakdown} icon={Boxes} />);

      // Title should be rendered
      expect(screen.getByText('Hierarchy Test')).toBeInTheDocument();

      // Labels should be present
      expect(screen.getByText('Public')).toBeInTheDocument();
      expect(screen.getByText('Private')).toBeInTheDocument();
      expect(screen.getByText('Archived')).toBeInTheDocument();
    });
  });

  describe('Layout and Spacing', () => {
    it('arranges breakdown items vertically', () => {
      const { container } = render(
        <BreakdownTile title="Layout Test" breakdown={mockBreakdown} icon={Boxes} />
      );

      // Should have space-y-2 for vertical spacing
      const spacedContainer = container.querySelector('.space-y-2');
      expect(spacedContainer).toBeInTheDocument();
    });

    it('has proper spacing between icon and text', () => {
      const { container } = render(
        <BreakdownTile title="Spacing Test" breakdown={mockBreakdown} icon={Boxes} />
      );

      // Check for gap-2 between icon and label
      const gapContainer = container.querySelector('.gap-2');
      expect(gapContainer).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles single visibility type with all others zero', () => {
      const singleTypeBreakdown: VisibilityBreakdown = {
        public: 42,
        private: 0,
        archived: 0,
      };

      render(<BreakdownTile title="Single Type" breakdown={singleTypeBreakdown} icon={Boxes} />);

      expect(screen.getByText('42')).toBeInTheDocument();
      const zeroCounts = screen.getAllByText('0');
      expect(zeroCounts).toHaveLength(2);
    });

    it('handles all archived repositories', () => {
      const allArchivedBreakdown: VisibilityBreakdown = {
        public: 0,
        private: 0,
        archived: 100,
      };

      render(<BreakdownTile title="All Archived" breakdown={allArchivedBreakdown} icon={Boxes} />);

      expect(screen.getByText('100')).toBeInTheDocument();
    });

    it('renders correctly with very long title', () => {
      const longTitle = 'This is a very long title for testing text overflow and wrapping behavior';

      render(<BreakdownTile title={longTitle} breakdown={mockBreakdown} icon={Boxes} />);

      expect(screen.getByText(longTitle)).toBeInTheDocument();
    });
  });

  describe('Component Props Validation', () => {
    it('accepts and renders different icon components', () => {
      const { container } = render(
        <BreakdownTile title="Different Icon" breakdown={mockBreakdown} icon={Boxes} />
      );

      // Icon should be rendered
      expect(container.querySelector('svg')).toBeInTheDocument();
    });

    it('maintains consistent layout across different breakdown values', () => {
      const { container: container1 } = render(
        <BreakdownTile
          title="Small Numbers"
          breakdown={{ public: 1, private: 2, archived: 3 }}
          icon={Boxes}
        />
      );

      const { container: container2 } = render(
        <BreakdownTile
          title="Large Numbers"
          breakdown={{ public: 999, private: 888, archived: 777 }}
          icon={Boxes}
        />
      );

      // Both should have the same structural elements
      expect(container1.querySelector('.space-y-2')).toBeInTheDocument();
      expect(container2.querySelector('.space-y-2')).toBeInTheDocument();
    });
  });

  describe('Visual Consistency', () => {
    it('maintains color scheme consistency', () => {
      const { container } = render(
        <BreakdownTile title="Color Scheme" breakdown={mockBreakdown} icon={Boxes} />
      );

      // Check for consistent color classes
      expect(container.querySelector('.text-green-600')).toBeInTheDocument();
      expect(container.querySelector('.text-amber-600')).toBeInTheDocument();
      expect(container.querySelector('.text-gray-500')).toBeInTheDocument();
    });

    it('uses consistent text styles for labels', () => {
      const { container } = render(
        <BreakdownTile title="Text Styles" breakdown={mockBreakdown} icon={Boxes} />
      );

      // Labels should be muted
      const mutedLabels = container.querySelectorAll('.text-muted-foreground');
      // Should have at least the labels (Public, Private, Archived)
      expect(mutedLabels.length).toBeGreaterThanOrEqual(3);
    });
  });
});
