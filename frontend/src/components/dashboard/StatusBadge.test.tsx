import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import StatusBadge from './StatusBadge';

describe('StatusBadge', () => {
  describe('Visual Appearance', () => {
    it('renders green status with correct color', () => {
      const { container } = render(<StatusBadge status="green" />);

      // The badge is wrapped in a flex container, the actual dot is the span inside
      const dot = container.querySelector('span.rounded-full');
      expect(dot).toBeInTheDocument();
      expect(dot).toHaveClass('bg-ampel-green');
    });

    it('renders yellow status with correct color', () => {
      const { container } = render(<StatusBadge status="yellow" />);

      const dot = container.querySelector('span.rounded-full');
      expect(dot).toBeInTheDocument();
      expect(dot).toHaveClass('bg-ampel-yellow');
    });

    it('renders red status with correct color', () => {
      const { container } = render(<StatusBadge status="red" />);

      const dot = container.querySelector('span.rounded-full');
      expect(dot).toBeInTheDocument();
      expect(dot).toHaveClass('bg-ampel-red');
    });

    it('returns null for none status', () => {
      const { container } = render(<StatusBadge status="none" />);
      expect(container.firstChild).toBeNull();
    });
  });

  describe('Size Variants', () => {
    it('renders medium size by default', () => {
      const { container } = render(<StatusBadge status="green" />);

      const dot = container.querySelector('span.rounded-full');
      expect(dot).toHaveClass('h-3');
      expect(dot).toHaveClass('w-3');
    });

    it('renders small size when specified', () => {
      const { container } = render(<StatusBadge status="green" size="sm" />);

      const dot = container.querySelector('span.rounded-full');
      expect(dot).toHaveClass('h-2');
      expect(dot).toHaveClass('w-2');
    });

    it('renders large size when specified', () => {
      const { container } = render(<StatusBadge status="green" size="lg" />);

      const dot = container.querySelector('span.rounded-full');
      expect(dot).toHaveClass('h-4');
      expect(dot).toHaveClass('w-4');
    });
  });

  describe('Label Display', () => {
    it('hides label by default', () => {
      render(<StatusBadge status="green" />);

      expect(screen.queryByText('Ready')).not.toBeInTheDocument();
    });

    it('shows label when showLabel is true', () => {
      render(<StatusBadge status="green" showLabel />);

      expect(screen.getByText('Ready')).toBeInTheDocument();
    });

    it('displays correct label for green status', () => {
      render(<StatusBadge status="green" showLabel />);

      expect(screen.getByText('Ready')).toBeInTheDocument();
    });

    it('displays correct label for yellow status', () => {
      render(<StatusBadge status="yellow" showLabel />);

      expect(screen.getByText('Pending')).toBeInTheDocument();
    });

    it('displays correct label for red status', () => {
      render(<StatusBadge status="red" showLabel />);

      expect(screen.getByText('Blocked')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('dot is a circular shape', () => {
      const { container } = render(<StatusBadge status="green" />);

      const dot = container.querySelector('span.rounded-full');
      expect(dot).toBeInTheDocument();
      expect(dot).toHaveClass('rounded-full');
    });

    it('has proper semantic structure with label', () => {
      render(<StatusBadge status="green" showLabel />);

      const container = screen.getByText('Ready').closest('div');
      expect(container).toBeInTheDocument();
    });

    it('has tooltip with description', () => {
      const { container } = render(<StatusBadge status="green" />);

      const wrapper = container.firstChild;
      expect(wrapper).toHaveAttribute(
        'title',
        'Ready to merge - CI passed, approved, no conflicts'
      );
    });
  });
});
