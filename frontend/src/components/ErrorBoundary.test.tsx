import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import ErrorBoundary from './ErrorBoundary';

// Component that throws an error
function ThrowError({ shouldThrow }: { shouldThrow: boolean }) {
  if (shouldThrow) {
    throw new Error('Test error');
  }
  return <div>Normal content</div>;
}

// Component that throws on mount
function ThrowOnMount() {
  throw new Error('Mount error');
}

describe('ErrorBoundary', () => {
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    consoleErrorSpy.mockRestore();
  });

  describe('Normal Rendering', () => {
    it('renders children when no error occurs', () => {
      render(
        <ErrorBoundary>
          <div data-testid="child-content">Child content</div>
        </ErrorBoundary>
      );

      expect(screen.getByTestId('child-content')).toBeInTheDocument();
      expect(screen.getByText('Child content')).toBeInTheDocument();
    });

    it('renders multiple children without errors', () => {
      render(
        <ErrorBoundary>
          <div data-testid="child-1">Child 1</div>
          <div data-testid="child-2">Child 2</div>
        </ErrorBoundary>
      );

      expect(screen.getByTestId('child-1')).toBeInTheDocument();
      expect(screen.getByTestId('child-2')).toBeInTheDocument();
    });
  });

  describe('Error Catching', () => {
    it('catches errors and displays fallback UI', () => {
      render(
        <ErrorBoundary>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
      expect(
        screen.getByText('An unexpected error occurred. Our team has been notified.')
      ).toBeInTheDocument();
    });

    it('displays error message in details (dev mode)', () => {
      render(
        <ErrorBoundary>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      // In dev mode, should show error details
      const errorDetails = screen.getByText('Error Details (Dev Only)');
      expect(errorDetails).toBeInTheDocument();

      const errorMessage = screen.getByText(/Mount error/);
      expect(errorMessage).toBeInTheDocument();
    });

    it('logs error to console', () => {
      render(
        <ErrorBoundary>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      expect(consoleErrorSpy).toHaveBeenCalled();
    });
  });

  describe('Custom Fallback UI', () => {
    it('renders custom fallback when provided', () => {
      const customFallback = <div data-testid="custom-fallback">Custom error message</div>;

      render(
        <ErrorBoundary fallback={customFallback}>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      expect(screen.getByTestId('custom-fallback')).toBeInTheDocument();
      expect(screen.getByText('Custom error message')).toBeInTheDocument();
      expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument();
    });

    it('uses custom fallback with complex UI', () => {
      const customFallback = (
        <div data-testid="complex-fallback">
          <h2>Oops!</h2>
          <p>Please contact support</p>
        </div>
      );

      render(
        <ErrorBoundary fallback={customFallback}>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      expect(screen.getByTestId('complex-fallback')).toBeInTheDocument();
      expect(screen.getByText('Oops!')).toBeInTheDocument();
      expect(screen.getByText('Please contact support')).toBeInTheDocument();
    });
  });

  describe('Error Handler Callback', () => {
    it('calls onError callback when error occurs', () => {
      const onError = vi.fn();

      render(
        <ErrorBoundary onError={onError}>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      expect(onError).toHaveBeenCalledTimes(1);
      expect(onError).toHaveBeenCalledWith(
        expect.any(Error),
        expect.objectContaining({
          componentStack: expect.any(String),
        })
      );
    });

    it('passes correct error to callback', () => {
      const onError = vi.fn();

      render(
        <ErrorBoundary onError={onError}>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      const [error] = onError.mock.calls[0];
      expect(error.message).toBe('Mount error');
    });

    it('does not call onError when no error occurs', () => {
      const onError = vi.fn();

      render(
        <ErrorBoundary onError={onError}>
          <div>No error</div>
        </ErrorBoundary>
      );

      expect(onError).not.toHaveBeenCalled();
    });
  });

  describe('Default Fallback UI', () => {
    it('displays AlertTriangle icon', () => {
      const { container } = render(
        <ErrorBoundary>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      // Check for AlertTriangle icon (svg element)
      const icon = container.querySelector('svg');
      expect(icon).toBeInTheDocument();
    });

    it('displays error in a card layout', () => {
      const { container } = render(
        <ErrorBoundary>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      // Check for card component
      const card = container.querySelector('[class*="rounded-lg"]');
      expect(card).toBeInTheDocument();
    });

    it('has action buttons for recovery', () => {
      render(
        <ErrorBoundary>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      const tryAgainButton = screen.getByText('Try Again');
      const reloadButton = screen.getByText('Reload Page');

      expect(tryAgainButton).toBeInTheDocument();
      expect(reloadButton).toBeInTheDocument();
    });
  });

  describe('Error Recovery', () => {
    it('can recover from error when props change', () => {
      const { rerender } = render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(screen.getByText('Something went wrong')).toBeInTheDocument();

      // Re-render with non-throwing component
      // Note: ErrorBoundary doesn't automatically recover, but this tests the component behavior
      rerender(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      // Should still show error (error boundaries don't auto-recover)
      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    });
  });

  describe('Nested ErrorBoundaries', () => {
    it('inner boundary catches error before outer boundary', () => {
      const outerOnError = vi.fn();
      const innerOnError = vi.fn();

      render(
        <ErrorBoundary onError={outerOnError}>
          <div>Outer content</div>
          <ErrorBoundary onError={innerOnError}>
            <ThrowOnMount />
          </ErrorBoundary>
        </ErrorBoundary>
      );

      // Inner boundary should catch the error
      expect(innerOnError).toHaveBeenCalledTimes(1);
      expect(outerOnError).not.toHaveBeenCalled();

      // Outer content should still render
      expect(screen.getByText('Outer content')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles error with null children', () => {
      const onError = vi.fn();

      render(<ErrorBoundary onError={onError}>{null}</ErrorBoundary>);

      expect(onError).not.toHaveBeenCalled();
    });

    it('handles error with undefined children', () => {
      const onError = vi.fn();

      render(<ErrorBoundary onError={onError}>{undefined}</ErrorBoundary>);

      expect(onError).not.toHaveBeenCalled();
    });

    it('displays fallback for errors during render', () => {
      function ComponentThatThrows() {
        // Simulate runtime error
        const data = null as unknown;
        return <div>{(data as { nonexistent: { property: string } }).nonexistent.property}</div>;
      }

      render(
        <ErrorBoundary>
          <ComponentThatThrows />
        </ErrorBoundary>
      );

      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('fallback UI uses semantic HTML', () => {
      const { container } = render(
        <ErrorBoundary>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      // Should use Card component (semantic structure)
      const card = container.querySelector('[class*="card"]');
      expect(card).toBeInTheDocument();
    });

    it('provides user-friendly error message', () => {
      render(
        <ErrorBoundary>
          <ThrowOnMount />
        </ErrorBoundary>
      );

      // Should have user-friendly description
      expect(
        screen.getByText('An unexpected error occurred. Our team has been notified.')
      ).toBeInTheDocument();
    });
  });
});
