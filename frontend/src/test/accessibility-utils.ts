/**
 * Accessibility Testing Utilities
 *
 * Utilities for WCAG 2.1 AA compliance testing
 */

import { configureAxe } from 'vitest-axe';

/**
 * Configure axe-core for WCAG 2.1 Level AA testing
 */
export const axe = configureAxe({
  rules: {
    // Enable all WCAG 2.1 Level A and AA rules
    'color-contrast': { enabled: true },
    label: { enabled: true },
    'button-name': { enabled: true },
    'link-name': { enabled: true },
    'image-alt': { enabled: true },
    'input-button-name': { enabled: true },
    'aria-valid-attr-value': { enabled: true },
    'aria-valid-attr': { enabled: true },
    'aria-required-attr': { enabled: true },
    'aria-hidden-focus': { enabled: true },
    'focus-order-semantics': { enabled: true },
    'landmark-one-main': { enabled: true },
    'page-has-heading-one': { enabled: true },
  },
});

/**
 * WCAG 2.1 AA compliance checker
 */
export const checkA11y = async (container: HTMLElement) => {
  const results = await axe(container);
  return results;
};

/**
 * Color contrast checker
 * WCAG 2.1 AA requires:
 * - Normal text: 4.5:1
 * - Large text (18pt+): 3:1
 * - UI components: 3:1
 */
export const checkColorContrast = (
  foreground: string,
  background: string
): {
  ratio: number;
  passesAA: boolean;
  passesAAA: boolean;
} => {
  const getLuminance = (hex: string): number => {
    const rgb = parseInt(hex.slice(1), 16);
    const r = (rgb >> 16) & 0xff;
    const g = (rgb >> 8) & 0xff;
    const b = (rgb >> 0) & 0xff;

    const [rs, gs, bs] = [r, g, b].map((c) => {
      const s = c / 255;
      return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
    });

    return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
  };

  const l1 = getLuminance(foreground);
  const l2 = getLuminance(background);
  const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);

  return {
    ratio: Math.round(ratio * 100) / 100,
    passesAA: ratio >= 4.5,
    passesAAA: ratio >= 7,
  };
};

/**
 * Keyboard navigation test helpers
 */
export const simulateKeyPress = (key: string, target?: HTMLElement) => {
  const element = target || document.activeElement;
  if (!element) return;

  const event = new KeyboardEvent('keydown', {
    key,
    bubbles: true,
    cancelable: true,
  });

  element.dispatchEvent(event);
};

export const simulateTab = (shift = false) => {
  simulateKeyPress(shift ? 'Shift+Tab' : 'Tab');
};

export const simulateEnter = (target?: HTMLElement) => {
  simulateKeyPress('Enter', target);
};

export const simulateEscape = (target?: HTMLElement) => {
  simulateKeyPress('Escape', target);
};

export const simulateArrowDown = (target?: HTMLElement) => {
  simulateKeyPress('ArrowDown', target);
};

export const simulateArrowUp = (target?: HTMLElement) => {
  simulateKeyPress('ArrowUp', target);
};

/**
 * Focus indicator visibility checker
 */
export const hasFocusIndicator = (element: HTMLElement): boolean => {
  const styles = window.getComputedStyle(element);
  const outline = styles.outline;
  const boxShadow = styles.boxShadow;

  // Check if element has visible focus indicator
  return (
    (outline !== 'none' && outline !== '0px') ||
    (boxShadow !== 'none' && boxShadow.includes('ring'))
  );
};

/**
 * ARIA label checker
 */
export const hasAccessibleName = (element: HTMLElement): boolean => {
  const ariaLabel = element.getAttribute('aria-label');
  const ariaLabelledBy = element.getAttribute('aria-labelledby');
  const title = element.getAttribute('title');
  const textContent = element.textContent?.trim();

  return !!(ariaLabel || ariaLabelledBy || title || textContent);
};

/**
 * Screen reader announcement simulator
 */
export const getScreenReaderText = (element: HTMLElement): string => {
  // Get aria-label first
  const ariaLabel = element.getAttribute('aria-label');
  if (ariaLabel) return ariaLabel;

  // Get aria-labelledby
  const ariaLabelledBy = element.getAttribute('aria-labelledby');
  if (ariaLabelledBy) {
    const labelElement = document.getElementById(ariaLabelledBy);
    if (labelElement) return labelElement.textContent || '';
  }

  // Get text content
  const textContent = element.textContent?.trim();
  if (textContent) return textContent;

  // Get title
  const title = element.getAttribute('title');
  if (title) return title;

  return '';
};

/**
 * Semantic HTML validator
 */
export const isSemanticElement = (element: HTMLElement): boolean => {
  const semanticTags = [
    'button',
    'a',
    'input',
    'select',
    'textarea',
    'nav',
    'header',
    'footer',
    'main',
    'section',
    'article',
    'aside',
  ];

  return semanticTags.includes(element.tagName.toLowerCase());
};

/**
 * WCAG 2.1 AA compliance report generator
 */
export interface A11yViolation {
  id: string;
  impact: 'critical' | 'serious' | 'moderate' | 'minor';
  description: string;
  help: string;
  helpUrl: string;
  nodes: Array<{
    html: string;
    target: string[];
  }>;
}

export interface A11yReport {
  violations: A11yViolation[];
  passes: number;
  incomplete: number;
  compliant: boolean;
  score: number;
}

interface AxeResults {
  violations: Array<{
    id: string;
    impact?: 'critical' | 'serious' | 'moderate' | 'minor' | null;
    description: string;
    help: string;
    helpUrl: string;
    nodes: Array<{
      html: string;
      target: string[];
    }>;
  }>;
  passes: unknown[];
  incomplete: unknown[];
}

export const generateA11yReport = (results: AxeResults): A11yReport => {
  const violations: A11yViolation[] = results.violations.map((v) => ({
    id: v.id,
    impact: (v.impact || 'moderate') as 'critical' | 'serious' | 'moderate' | 'minor',
    description: v.description,
    help: v.help,
    helpUrl: v.helpUrl,
    nodes: v.nodes.map((n) => ({
      html: n.html,
      target: n.target,
    })),
  }));

  const totalTests = results.passes.length + violations.length + results.incomplete.length;
  const score = totalTests > 0 ? Math.round((results.passes.length / totalTests) * 100) : 100;

  return {
    violations,
    passes: results.passes.length,
    incomplete: results.incomplete.length,
    compliant: violations.length === 0,
    score,
  };
};
