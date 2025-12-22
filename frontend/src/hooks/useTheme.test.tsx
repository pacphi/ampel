import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ThemeProvider, useTheme } from './useTheme';

// Mock matchMedia with a controllable implementation
const mockMatchMedia = vi.fn();

// Test component that uses useTheme hook
function TestComponent() {
  const { theme, setTheme, resolvedTheme } = useTheme();

  return (
    <div>
      <div data-testid="theme">{theme}</div>
      <div data-testid="resolved-theme">{resolvedTheme}</div>
      <button onClick={() => setTheme('light')}>Set Light</button>
      <button onClick={() => setTheme('dark')}>Set Dark</button>
      <button onClick={() => setTheme('system')}>Set System</button>
    </div>
  );
}

// Test component that doesn't use provider
function ComponentWithoutProvider() {
  try {
    useTheme();
    return <div>no-error</div>;
  } catch (e) {
    return <div data-testid="error">{(e as Error).message}</div>;
  }
}

describe('useTheme', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    // Reset document classes
    document.documentElement.classList.remove('light', 'dark');

    // Setup matchMedia mock for each test
    mockMatchMedia.mockImplementation((query: string) => ({
      matches: query === '(prefers-color-scheme: dark)' ? false : false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }));
    window.matchMedia = mockMatchMedia;
  });

  afterEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove('light', 'dark');
  });

  it('throws error when useTheme is used outside of ThemeProvider', () => {
    render(<ComponentWithoutProvider />);
    expect(screen.getByTestId('error')).toHaveTextContent(
      'useTheme must be used within a ThemeProvider'
    );
  });

  it('defaults to system theme when no stored theme', async () => {
    render(
      <ThemeProvider>
        <TestComponent />
      </ThemeProvider>
    );

    expect(screen.getByTestId('theme')).toHaveTextContent('system');
  });

  it('loads stored theme from localStorage', async () => {
    localStorage.setItem('theme', 'dark');

    render(
      <ThemeProvider>
        <TestComponent />
      </ThemeProvider>
    );

    expect(screen.getByTestId('theme')).toHaveTextContent('dark');
  });

  it('sets light theme correctly', async () => {
    const user = userEvent.setup();

    render(
      <ThemeProvider>
        <TestComponent />
      </ThemeProvider>
    );

    await user.click(screen.getByText('Set Light'));

    await waitFor(() => {
      expect(screen.getByTestId('theme')).toHaveTextContent('light');
      expect(screen.getByTestId('resolved-theme')).toHaveTextContent('light');
      expect(localStorage.getItem('theme')).toBe('light');
      expect(document.documentElement.classList.contains('light')).toBe(true);
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });
  });

  it('sets dark theme correctly', async () => {
    const user = userEvent.setup();

    render(
      <ThemeProvider>
        <TestComponent />
      </ThemeProvider>
    );

    await user.click(screen.getByText('Set Dark'));

    await waitFor(() => {
      expect(screen.getByTestId('theme')).toHaveTextContent('dark');
      expect(screen.getByTestId('resolved-theme')).toHaveTextContent('dark');
      expect(localStorage.getItem('theme')).toBe('dark');
      expect(document.documentElement.classList.contains('dark')).toBe(true);
      expect(document.documentElement.classList.contains('light')).toBe(false);
    });
  });

  it('sets system theme correctly', async () => {
    const user = userEvent.setup();
    localStorage.setItem('theme', 'dark');

    render(
      <ThemeProvider>
        <TestComponent />
      </ThemeProvider>
    );

    await user.click(screen.getByText('Set System'));

    await waitFor(() => {
      expect(screen.getByTestId('theme')).toHaveTextContent('system');
      expect(localStorage.getItem('theme')).toBe('system');
    });
  });

  it('resolves system theme to light when prefers-color-scheme is light', async () => {
    // The mock in setup.ts returns matches: false (light mode)
    render(
      <ThemeProvider>
        <TestComponent />
      </ThemeProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('resolved-theme')).toHaveTextContent('light');
    });
  });

  it('removes previous theme class when changing themes', async () => {
    const user = userEvent.setup();

    render(
      <ThemeProvider>
        <TestComponent />
      </ThemeProvider>
    );

    await user.click(screen.getByText('Set Dark'));
    await waitFor(() => {
      expect(document.documentElement.classList.contains('dark')).toBe(true);
    });

    await user.click(screen.getByText('Set Light'));
    await waitFor(() => {
      expect(document.documentElement.classList.contains('light')).toBe(true);
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });
  });
});
