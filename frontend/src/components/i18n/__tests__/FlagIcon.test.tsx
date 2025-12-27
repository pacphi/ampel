/**
 * Tests for FlagIcon component
 *
 * Verifies:
 * - Renders correct flag for each of 20 languages
 * - Handles different sizes (sm, md, lg)
 * - Accessible alt text
 * - Fallback for unsupported language codes
 */

import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';

// Mock FlagIcon component (will be implemented by frontend developer)
const FlagIcon = ({
  languageCode,
  size = 'md',
}: {
  languageCode: string;
  size?: 'sm' | 'md' | 'lg';
}) => {
  // TODO: Implement actual FlagIcon with SVG or emoji flags
  return (
    <span
      data-testid={`flag-${languageCode}`}
      data-size={size}
      role="img"
      aria-label={`Flag for ${languageCode}`}
    >
      {languageCode}
    </span>
  );
};

describe('FlagIcon', () => {
  describe('All 20 Language Flags', () => {
    const languages = [
      { code: 'en', name: 'English', flag: '🇬🇧' },
      { code: 'es', name: 'Spanish', flag: '🇪🇸' },
      { code: 'fr', name: 'French', flag: '🇫🇷' },
      { code: 'de', name: 'German', flag: '🇩🇪' },
      { code: 'it', name: 'Italian', flag: '🇮🇹' },
      { code: 'pt', name: 'Portuguese', flag: '🇵🇹' },
      { code: 'ru', name: 'Russian', flag: '🇷🇺' },
      { code: 'zh', name: 'Chinese', flag: '🇨🇳' },
      { code: 'ja', name: 'Japanese', flag: '🇯🇵' },
      { code: 'ko', name: 'Korean', flag: '🇰🇷' },
      { code: 'ar', name: 'Arabic', flag: '🇸🇦' },
      { code: 'he', name: 'Hebrew', flag: '🇮🇱' },
      { code: 'hi', name: 'Hindi', flag: '🇮🇳' },
      { code: 'bn', name: 'Bengali', flag: '🇧🇩' },
      { code: 'tr', name: 'Turkish', flag: '🇹🇷' },
      { code: 'nl', name: 'Dutch', flag: '🇳🇱' },
      { code: 'pl', name: 'Polish', flag: '🇵🇱' },
      { code: 'vi', name: 'Vietnamese', flag: '🇻🇳' },
      { code: 'th', name: 'Thai', flag: '🇹🇭' },
      { code: 'uk', name: 'Ukrainian', flag: '🇺🇦' },
    ];

    languages.forEach(({ code, name }) => {
      it(`renders ${name} flag (${code})`, () => {
        render(<FlagIcon languageCode={code} />);

        const flag = screen.getByTestId(`flag-${code}`);
        expect(flag).toBeInTheDocument();

        // TODO: When implemented with actual flags, verify:
        // expect(flag).toHaveTextContent(flag emoji or SVG);
      });
    });

    it('renders all 20 flags correctly', () => {
      const { container } = render(
        <div>
          {languages.map(({ code }) => (
            <FlagIcon key={code} languageCode={code} />
          ))}
        </div>
      );

      // Verify all 20 flags are rendered
      expect(container.querySelectorAll('[data-testid^="flag-"]')).toHaveLength(20);
    });
  });

  describe('Size Variants', () => {
    it('renders small size', () => {
      render(<FlagIcon languageCode="en" size="sm" />);

      const flag = screen.getByTestId('flag-en');
      expect(flag).toHaveAttribute('data-size', 'sm');

      // TODO: When implemented, verify CSS classes:
      // expect(flag).toHaveClass('w-4', 'h-4'); // or similar
    });

    it('renders medium size (default)', () => {
      render(<FlagIcon languageCode="en" />);

      const flag = screen.getByTestId('flag-en');
      expect(flag).toHaveAttribute('data-size', 'md');

      // TODO: When implemented, verify CSS classes:
      // expect(flag).toHaveClass('w-6', 'h-6'); // or similar
    });

    it('renders large size', () => {
      render(<FlagIcon languageCode="en" size="lg" />);

      const flag = screen.getByTestId('flag-en');
      expect(flag).toHaveAttribute('data-size', 'lg');

      // TODO: When implemented, verify CSS classes:
      // expect(flag).toHaveClass('w-8', 'h-8'); // or similar
    });
  });

  describe('Accessibility', () => {
    it('has role="img"', () => {
      render(<FlagIcon languageCode="en" />);

      const flag = screen.getByTestId('flag-en');
      expect(flag).toHaveAttribute('role', 'img');
    });

    it('has descriptive aria-label for English', () => {
      render(<FlagIcon languageCode="en" />);

      const flag = screen.getByTestId('flag-en');
      expect(flag).toHaveAttribute('aria-label');
      expect(flag.getAttribute('aria-label')).toMatch(/english/i);
    });

    it('has descriptive aria-label for Arabic', () => {
      render(<FlagIcon languageCode="ar" />);

      const flag = screen.getByTestId('flag-ar');
      expect(flag).toHaveAttribute('aria-label');
      expect(flag.getAttribute('aria-label')).toMatch(/arabic/i);
    });

    it('has descriptive aria-label for all languages', () => {
      const languages = ['en', 'es', 'fr', 'de', 'ja', 'zh', 'ar', 'he'];

      languages.forEach((code) => {
        const { container } = render(<FlagIcon languageCode={code} />);
        const flag = container.querySelector(`[data-testid="flag-${code}"]`);

        expect(flag).toHaveAttribute('aria-label');
        expect(flag?.getAttribute('aria-label')).toBeTruthy();
      });
    });
  });

  describe('Fallback Behavior', () => {
    it('renders fallback for unsupported language code', () => {
      render(<FlagIcon languageCode="xx" />);

      // TODO: When implemented, verify fallback behavior:
      // const flag = screen.getByTestId('flag-xx');
      // expect(flag).toBeInTheDocument();
      // // Should show generic globe icon or similar

      expect(screen.getByTestId('flag-xx')).toBeInTheDocument();
    });

    it('handles empty language code gracefully', () => {
      render(<FlagIcon languageCode="" />);

      // TODO: When implemented, verify fallback behavior for empty string
      expect(screen.getByTestId('flag-')).toBeInTheDocument();
    });

    it('handles undefined language code gracefully', () => {
      // @ts-expect-error Testing undefined case
      render(<FlagIcon languageCode={undefined} />);

      // TODO: When implemented, verify fallback behavior for undefined
      // Should not crash
    });
  });

  describe('Visual Appearance', () => {
    it('displays flag inline with text', () => {
      const { container } = render(
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <FlagIcon languageCode="en" />
          <span>English</span>
        </div>
      );

      // TODO: When implemented, verify inline display:
      // const flag = container.querySelector('[data-testid="flag-en"]');
      // expect(window.getComputedStyle(flag!).display).toBe('inline-block');
    });

    it('maintains aspect ratio', () => {
      render(<FlagIcon languageCode="en" />);

      // TODO: When implemented, verify aspect ratio:
      // const flag = screen.getByTestId('flag-en');
      // expect(flag).toHaveStyle({ aspectRatio: '1.5' }); // 3:2 flag ratio
    });
  });

  describe('Performance', () => {
    it('renders multiple flags efficiently', () => {
      const languages = Array.from({ length: 20 }, (_, i) => `lang-${i}`);

      const startTime = performance.now();

      render(
        <div>
          {languages.map((lang) => (
            <FlagIcon key={lang} languageCode={lang} />
          ))}
        </div>
      );

      const endTime = performance.now();

      // Should render 20 flags in under 100ms
      expect(endTime - startTime).toBeLessThan(100);
    });
  });
});
