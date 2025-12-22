import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import StatusBadge from './StatusBadge';

describe('StatusBadge', () => {
  describe('Visual Appearance', () => {
    it('renders green status with correct color', () => {
      const { container } = render(<StatusBadge status="green" />);

      const badge = container.firstChild;
      expect(badge).toHaveAttribute('data-status', 'green');
      expect(badge).toHaveClass('bg-ampel-green');
    });

    it('renders yellow status with correct color', () => {
      const { container } = render(<StatusBadge status="yellow" />);

      const badge = container.firstChild;
      expect(badge).toHaveAttribute('data-status', 'yellow');
      expect(badge).toHaveClass('bg-ampel-yellow');
    });

    it('renders red status with correct color', () => {
      const { container } = render(<StatusBadge status="red" />);

      const badge = container.firstChild;
      expect(badge).toHaveAttribute('data-status', 'red');
      expect(badge).toHaveClass('bg-ampel-red');
    });
  });

  describe('Size Variants', () => {
    it('renders small size by default', () => {
      const { container } = render(<StatusBadge status="green" />);

      const badge = container.firstChild;
      expect(badge).toHaveClass('h-3');
      expect(badge).toHaveClass('w-3');
    });

    it('renders large size when specified', () => {
      const { container } = render(<StatusBadge status="green" size="lg" />);

      const badge = container.firstChild;
      expect(badge).toHaveClass('h-4');
      expect(badge).toHaveClass('w-4');
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
    it('is a circular shape', () => {
      const { container } = render(<StatusBadge status="green" />);

      const badge = container.firstChild;
      expect(badge).toHaveClass('rounded-full');
    });

    it('has proper semantic structure with label', () => {
      render(<StatusBadge status="green" showLabel />);

      const container = screen.getByText('Ready').closest('div');
      expect(container).toBeInTheDocument();
    });
  });
});
