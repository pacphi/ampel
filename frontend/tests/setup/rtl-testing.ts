/**
 * RTL (Right-to-Left) Testing Utilities
 *
 * Provides comprehensive testing patterns for RTL layout support:
 * 1. Document direction verification
 * 2. CSS logical property validation
 * 3. Element-level direction testing
 * 4. RTL/LTR switching verification
 *
 * Key concepts:
 * - RTL languages: Arabic (ar), Hebrew (he)
 * - Document attributes: dir="rtl", lang attribute, rtl class
 * - CSS logical properties: margin-inline-start, padding-inline-end, etc.
 */

import { expect } from 'vitest';

// ============================================================================
// Types
// ============================================================================

export type TextDirection = 'ltr' | 'rtl';

export interface RTLState {
  documentDir: string;
  documentLang: string;
  hasRTLClass: boolean;
  bodyDir: string;
  bodyHasRTLClass: boolean;
}

export interface LogicalPropertyCheck {
  property: string;
  hasViolation: boolean;
  violatingValue?: string;
}

// ============================================================================
// RTL Document State
// ============================================================================

/**
 * Get the current RTL state of the document
 */
export function getRTLState(): RTLState {
  return {
    documentDir: document.documentElement.getAttribute('dir') || 'ltr',
    documentLang: document.documentElement.getAttribute('lang') || '',
    hasRTLClass: document.documentElement.classList.contains('rtl'),
    bodyDir: document.body.getAttribute('dir') || '',
    bodyHasRTLClass: document.body.classList.contains('rtl'),
  };
}

/**
 * Reset document RTL state to defaults (LTR)
 */
export function resetRTLState(): void {
  document.documentElement.setAttribute('dir', 'ltr');
  document.documentElement.setAttribute('lang', 'en');
  document.documentElement.classList.remove('rtl');
  document.body.removeAttribute('dir');
  document.body.classList.remove('rtl');

  // Remove meta tags
  const dirMeta = document.querySelector('meta[name="direction"]');
  const langMeta = document.querySelector('meta[http-equiv="content-language"]');
  dirMeta?.remove();
  langMeta?.remove();
}

/**
 * Manually set document to RTL state (for testing without full i18n)
 */
export function setRTLState(language: string): void {
  const isRTL = ['ar', 'he'].includes(language);
  const dir: TextDirection = isRTL ? 'rtl' : 'ltr';

  document.documentElement.setAttribute('dir', dir);
  document.documentElement.setAttribute('lang', language);

  if (isRTL) {
    document.documentElement.classList.add('rtl');
    document.body.classList.add('rtl');
    document.body.setAttribute('dir', 'rtl');
  } else {
    document.documentElement.classList.remove('rtl');
    document.body.classList.remove('rtl');
    document.body.removeAttribute('dir');
  }
}

// ============================================================================
// RTL Assertions
// ============================================================================

/**
 * Assert that the document is in RTL mode
 *
 * Checks:
 * - document.documentElement.dir === 'rtl'
 * - document.documentElement.lang matches RTL language
 * - RTL class is present
 */
export function expectRTLLayout(): void {
  const state = getRTLState();

  expect(state.documentDir).toBe('rtl');
  expect(state.hasRTLClass).toBe(true);
  expect(['ar', 'he']).toContain(state.documentLang);
}

/**
 * Assert that the document is in LTR mode
 */
export function expectLTRLayout(): void {
  const state = getRTLState();

  expect(state.documentDir).toBe('ltr');
  expect(state.hasRTLClass).toBe(false);
}

/**
 * Assert document direction matches expected
 */
export function expectDirection(expected: TextDirection): void {
  const state = getRTLState();
  expect(state.documentDir).toBe(expected);

  if (expected === 'rtl') {
    expect(state.hasRTLClass).toBe(true);
  } else {
    expect(state.hasRTLClass).toBe(false);
  }
}

/**
 * Assert that an element respects the current direction
 */
export function expectElementDirection(element: HTMLElement, expected: TextDirection): void {
  const computedDir = getComputedStyle(element).direction;
  expect(computedDir).toBe(expected);
}

// ============================================================================
// CSS Logical Property Validation
// ============================================================================

/**
 * Physical properties that should be replaced with logical properties
 */
const PHYSICAL_TO_LOGICAL = {
  // Margin
  'margin-left': 'margin-inline-start',
  'margin-right': 'margin-inline-end',
  // Padding
  'padding-left': 'padding-inline-start',
  'padding-right': 'padding-inline-end',
  // Position
  left: 'inset-inline-start',
  right: 'inset-inline-end',
  // Text alignment
  'text-align: left': 'text-align: start',
  'text-align: right': 'text-align: end',
  // Float
  'float: left': 'float: inline-start',
  'float: right': 'float: inline-end',
};

/**
 * Tailwind class patterns that should use logical properties
 */
const TAILWIND_VIOLATIONS = {
  // Margin
  'ml-': 'ms-',
  'mr-': 'me-',
  // Padding
  'pl-': 'ps-',
  'pr-': 'pe-',
  // Position
  'left-': 'start-',
  'right-': 'end-',
  // Text alignment
  'text-left': 'text-start',
  'text-right': 'text-end',
};

/**
 * Check if an element uses CSS logical properties correctly
 *
 * @param element - Element to check
 * @returns Array of violations found
 */
export function checkLogicalProperties(element: HTMLElement): LogicalPropertyCheck[] {
  const violations: LogicalPropertyCheck[] = [];
  const style = element.style;

  // Check inline styles for physical properties
  for (const [physical, logical] of Object.entries(PHYSICAL_TO_LOGICAL)) {
    if (physical.includes(':')) {
      // Handle property:value patterns (like text-align: left)
      const [prop, value] = physical.split(': ');
      const currentValue = style.getPropertyValue(prop);
      if (currentValue === value) {
        violations.push({
          property: physical,
          hasViolation: true,
          violatingValue: `Use "${logical}" instead`,
        });
      }
    } else {
      // Handle property-only patterns
      const value = style.getPropertyValue(physical);
      if (value && value !== '0px' && value !== 'auto') {
        violations.push({
          property: physical,
          hasViolation: true,
          violatingValue: `${value} - Use "${logical}" instead`,
        });
      }
    }
  }

  return violations;
}

/**
 * Check if element classes use RTL-friendly Tailwind classes
 */
export function checkTailwindClasses(element: HTMLElement): LogicalPropertyCheck[] {
  const violations: LogicalPropertyCheck[] = [];
  const classList = element.className;

  for (const [violation, replacement] of Object.entries(TAILWIND_VIOLATIONS)) {
    // Use regex to match class patterns
    const pattern = new RegExp(`\\b${violation.replace('-', '-?')}\\d*\\b`, 'g');
    const matches = classList.match(pattern);

    if (matches) {
      violations.push({
        property: `Tailwind class: ${matches.join(', ')}`,
        hasViolation: true,
        violatingValue: `Use "${replacement}*" instead`,
      });
    }
  }

  return violations;
}

/**
 * Assert that an element uses logical properties correctly
 */
export function expectLogicalProperties(element: HTMLElement): void {
  const styleViolations = checkLogicalProperties(element);
  const classViolations = checkTailwindClasses(element);
  const allViolations = [...styleViolations, ...classViolations];

  if (allViolations.length > 0) {
    const violationDetails = allViolations
      .map((v) => `  - ${v.property}: ${v.violatingValue}`)
      .join('\n');

    throw new Error(
      `Element uses physical (non-RTL-friendly) CSS properties:\n${violationDetails}\n\n` +
        `For RTL support, use CSS logical properties or Tailwind logical classes.`
    );
  }
}

/**
 * Recursively check an element and its descendants for RTL violations
 */
export function checkTreeForRTLViolations(
  root: HTMLElement,
  options?: { maxDepth?: number; skipSelectors?: string[] }
): LogicalPropertyCheck[] {
  const { maxDepth = 10, skipSelectors = [] } = options || {};
  const violations: LogicalPropertyCheck[] = [];

  function traverse(element: HTMLElement, depth: number): void {
    if (depth > maxDepth) return;

    // Skip elements matching skip selectors
    if (skipSelectors.some((selector) => element.matches(selector))) {
      return;
    }

    // Check this element
    violations.push(...checkLogicalProperties(element));
    violations.push(...checkTailwindClasses(element));

    // Check children
    for (const child of element.children) {
      if (child instanceof HTMLElement) {
        traverse(child, depth + 1);
      }
    }
  }

  traverse(root, 0);
  return violations;
}

// ============================================================================
// RTL Meta Tags Verification
// ============================================================================

/**
 * Assert that RTL meta tags are correctly set
 */
export function expectRTLMetaTags(expectedDir: TextDirection, expectedLang: string): void {
  const dirMeta = document.querySelector('meta[name="direction"]');
  const langMeta = document.querySelector('meta[http-equiv="content-language"]');

  expect(dirMeta).not.toBeNull();
  expect(dirMeta?.getAttribute('content')).toBe(expectedDir);

  expect(langMeta).not.toBeNull();
  expect(langMeta?.getAttribute('content')).toBe(expectedLang);
}

// ============================================================================
// RTL Visual Testing Utilities
// ============================================================================

/**
 * Get the effective text direction of an element
 * (accounts for inheritance from parent elements)
 */
export function getEffectiveDirection(element: HTMLElement): TextDirection {
  const computed = getComputedStyle(element);
  return computed.direction as TextDirection;
}

/**
 * Check if element's text alignment is "start" (respects direction)
 */
export function isTextAlignStart(element: HTMLElement): boolean {
  const computed = getComputedStyle(element);
  const textAlign = computed.textAlign;
  const direction = computed.direction;

  // 'start' is explicit, or 'left' in LTR / 'right' in RTL
  if (textAlign === 'start') return true;
  if (direction === 'ltr' && textAlign === 'left') return true;
  if (direction === 'rtl' && textAlign === 'right') return true;

  return false;
}

/**
 * Create a mock viewport size for RTL testing
 */
export function mockViewport(width: number, height: number): () => void {
  const originalInnerWidth = window.innerWidth;
  const originalInnerHeight = window.innerHeight;

  Object.defineProperty(window, 'innerWidth', {
    writable: true,
    configurable: true,
    value: width,
  });

  Object.defineProperty(window, 'innerHeight', {
    writable: true,
    configurable: true,
    value: height,
  });

  // Return cleanup function
  return () => {
    Object.defineProperty(window, 'innerWidth', {
      value: originalInnerWidth,
    });
    Object.defineProperty(window, 'innerHeight', {
      value: originalInnerHeight,
    });
  };
}

// ============================================================================
// Bidirectional Text Testing
// ============================================================================

/**
 * Test cases for bidirectional text handling
 */
export const BIDI_TEST_CASES = {
  // Arabic with embedded English
  arabicWithEnglish: 'مرحبا Hello العالم',
  // Hebrew with numbers
  hebrewWithNumbers: 'שלום 12345 עולם',
  // Arabic with URL
  arabicWithURL: 'زيارة https://example.com للمزيد',
  // Mixed direction with punctuation
  mixedWithPunctuation: 'שלום (Hello!) עולם',
  // RTL with email
  rtlWithEmail: 'البريد: user@example.com',
  // RTL with path
  rtlWithPath: 'المسار: /home/user/file.txt',
};

/**
 * Assert that bidirectional text is rendered (browser handles bidi algorithm)
 */
export function expectBidiTextRendered(
  element: HTMLElement,
  testCase: keyof typeof BIDI_TEST_CASES
): void {
  const expectedText = BIDI_TEST_CASES[testCase];
  expect(element.textContent).toContain(expectedText);
}

// ============================================================================
// RTL Snapshot Testing Helpers
// ============================================================================

/**
 * Prepare an element for RTL snapshot comparison
 * Normalizes classes and attributes that may vary
 */
export function prepareForRTLSnapshot(element: HTMLElement): string {
  const clone = element.cloneNode(true) as HTMLElement;

  // Normalize dynamic attributes
  const walker = document.createTreeWalker(clone, NodeFilter.SHOW_ELEMENT);

  let node: Element | null = walker.currentNode as Element;
  while (node) {
    // Remove data-* attributes that may be dynamic
    Array.from(node.attributes)
      .filter((attr) => attr.name.startsWith('data-'))
      .forEach((attr) => node!.removeAttribute(attr.name));

    // Normalize id attributes
    if (node.id && node.id.includes('-')) {
      node.id = node.id.replace(/-\w+$/, '-[id]');
    }

    node = walker.nextNode() as Element | null;
  }

  return clone.outerHTML;
}

// ============================================================================
// Exports for Test Setup
// ============================================================================

export const rtlTestUtils = {
  getRTLState,
  resetRTLState,
  setRTLState,
  expectRTLLayout,
  expectLTRLayout,
  expectDirection,
  expectElementDirection,
  checkLogicalProperties,
  checkTailwindClasses,
  expectLogicalProperties,
  checkTreeForRTLViolations,
  expectRTLMetaTags,
  getEffectiveDirection,
  isTextAlignStart,
  mockViewport,
  BIDI_TEST_CASES,
  expectBidiTextRendered,
  prepareForRTLSnapshot,
};
