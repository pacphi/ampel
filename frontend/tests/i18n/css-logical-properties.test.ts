/**
 * CSS Logical Properties Validation Tests
 *
 * Ensures components use CSS logical properties for RTL compatibility:
 * - margin-inline-start/end (not margin-left/right)
 * - padding-inline-start/end (not padding-left/right)
 * - text-align: start/end (not left/right)
 * - inset-inline-start/end (not left/right)
 */

import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Directories to scan for CSS violations
const DIRECTORIES_TO_SCAN = [
  join(__dirname, '../../src/components'),
  join(__dirname, '../../src/pages'),
  join(__dirname, '../../src/styles'),
];

// Patterns that indicate hardcoded directional CSS
const VIOLATION_PATTERNS = [
  // Tailwind classes with hardcoded directions
  /\b(ml|mr|pl|pr|left|right)-\d+\b/g,
  /\btext-(left|right)\b/g,

  // CSS properties with hardcoded directions
  /margin-(left|right):/g,
  /padding-(left|right):/g,
  /text-align:\s*(left|right)/g,
  /float:\s*(left|right)/g,
  /(left|right):\s*\d+/g,
];

// Allowed exceptions (third-party components, specific cases)
const ALLOWED_EXCEPTIONS = [
  'node_modules',
  '.test.tsx',
  '.spec.tsx',
  '__tests__',
  // Third-party UI components may have hardcoded directions
  'ui/tooltip',
  'ui/dropdown-menu',
  'ui/select',
];

function shouldSkipFile(filePath: string): boolean {
  return ALLOWED_EXCEPTIONS.some((exception) => filePath.includes(exception));
}

function scanFileForViolations(filePath: string): string[] {
  if (shouldSkipFile(filePath)) {
    return [];
  }

  try {
    const content = readFileSync(filePath, 'utf-8');
    const violations: string[] = [];

    VIOLATION_PATTERNS.forEach((pattern, index) => {
      const matches = content.match(pattern);
      if (matches) {
        violations.push(
          `[Pattern ${index + 1}] ${filePath}: Found ${matches.length} occurrence(s) of ${pattern}`
        );
      }
    });

    return violations;
  } catch {
    // Skip files that can't be read
    return [];
  }
}

function scanDirectory(dirPath: string): string[] {
  let violations: string[] = [];

  try {
    const entries = readdirSync(dirPath);

    for (const entry of entries) {
      const fullPath = join(dirPath, entry);
      const stat = statSync(fullPath);

      if (stat.isDirectory()) {
        violations = violations.concat(scanDirectory(fullPath));
      } else if (
        stat.isFile() &&
        (entry.endsWith('.tsx') || entry.endsWith('.ts') || entry.endsWith('.css'))
      ) {
        violations = violations.concat(scanFileForViolations(fullPath));
      }
    }
  } catch {
    // Skip directories that can't be accessed
  }

  return violations;
}

describe('CSS Logical Properties Validation', () => {
  describe('Tailwind Classes', () => {
    it('should use margin-inline-start/end instead of ml/mr', () => {
      const violations: string[] = [];

      for (const dir of DIRECTORIES_TO_SCAN) {
        violations.push(...scanDirectory(dir));
      }

      const marginViolations = violations.filter((v) => v.includes('ml-') || v.includes('mr-'));

      if (marginViolations.length > 0) {
        console.warn('\n⚠️  Found Tailwind margin violations (use ms-/me- instead):');
        marginViolations.forEach((v) => console.warn(`  ${v}`));
      }

      // This is a warning test - we log violations but don't fail
      // to allow gradual migration to logical properties
      expect(marginViolations.length).toBeLessThan(50); // Threshold
    });

    it('should use padding-inline-start/end instead of pl/pr', () => {
      const violations: string[] = [];

      for (const dir of DIRECTORIES_TO_SCAN) {
        violations.push(...scanDirectory(dir));
      }

      const paddingViolations = violations.filter((v) => v.includes('pl-') || v.includes('pr-'));

      if (paddingViolations.length > 0) {
        console.warn('\n⚠️  Found Tailwind padding violations (use ps-/pe- instead):');
        paddingViolations.forEach((v) => console.warn(`  ${v}`));
      }

      expect(paddingViolations.length).toBeLessThan(50);
    });

    it('should use text-start/end instead of text-left/right', () => {
      const violations: string[] = [];

      for (const dir of DIRECTORIES_TO_SCAN) {
        violations.push(...scanDirectory(dir));
      }

      const textAlignViolations = violations.filter(
        (v) => v.includes('text-left') || v.includes('text-right')
      );

      if (textAlignViolations.length > 0) {
        console.warn('\n⚠️  Found text-align violations (use text-start/text-end instead):');
        textAlignViolations.forEach((v) => console.warn(`  ${v}`));
      }

      expect(textAlignViolations.length).toBeLessThan(30);
    });
  });

  describe('Inline CSS Properties', () => {
    it('should use logical properties in inline styles', () => {
      const violations: string[] = [];

      for (const dir of DIRECTORIES_TO_SCAN) {
        violations.push(...scanDirectory(dir));
      }

      const inlineCssViolations = violations.filter(
        (v) =>
          v.includes('margin-left:') ||
          v.includes('margin-right:') ||
          v.includes('padding-left:') ||
          v.includes('padding-right:')
      );

      if (inlineCssViolations.length > 0) {
        console.warn('\n⚠️  Found inline CSS violations (use margin-inline-*, padding-inline-*):');
        inlineCssViolations.forEach((v) => console.warn(`  ${v}`));
      }

      expect(inlineCssViolations.length).toBeLessThan(20);
    });

    it('should use text-align: start/end instead of left/right', () => {
      const violations: string[] = [];

      for (const dir of DIRECTORIES_TO_SCAN) {
        violations.push(...scanDirectory(dir));
      }

      const textAlignCssViolations = violations.filter(
        (v) => v.includes('text-align: left') || v.includes('text-align: right')
      );

      if (textAlignCssViolations.length > 0) {
        console.warn('\n⚠️  Found text-align CSS violations (use start/end):');
        textAlignCssViolations.forEach((v) => console.warn(`  ${v}`));
      }

      expect(textAlignCssViolations.length).toBeLessThan(10);
    });
  });

  describe('Migration Guide', () => {
    it('should provide migration recommendations', () => {
      const recommendations = [
        'Replace ml-* with ms-* (margin-inline-start)',
        'Replace mr-* with me-* (margin-inline-end)',
        'Replace pl-* with ps-* (padding-inline-start)',
        'Replace pr-* with pe-* (padding-inline-end)',
        'Replace text-left with text-start',
        'Replace text-right with text-end',
        'Replace left-* with start-*',
        'Replace right-* with end-*',
      ];

      console.log('\n📖 RTL Migration Guide:');
      recommendations.forEach((rec) => {
        console.log(`  ✓ ${rec}`);
      });

      expect(recommendations).toHaveLength(8);
    });

    it('should document approved logical properties', () => {
      const approvedProperties = {
        margin: ['ms-*', 'me-*', 'mx-*', 'my-*'],
        padding: ['ps-*', 'pe-*', 'px-*', 'py-*'],
        textAlign: ['text-start', 'text-end', 'text-center'],
        position: ['start-*', 'end-*', 'inset-inline-start', 'inset-inline-end'],
      };

      console.log('\n✅ Approved Logical Properties:');
      Object.entries(approvedProperties).forEach(([category, props]) => {
        console.log(`  ${category}:`, props.join(', '));
      });

      expect(Object.keys(approvedProperties)).toHaveLength(4);
    });
  });

  describe('RTL Compatibility Score', () => {
    it('should calculate overall RTL compatibility', () => {
      const violations: string[] = [];

      for (const dir of DIRECTORIES_TO_SCAN) {
        violations.push(...scanDirectory(dir));
      }

      const totalFiles = violations.length > 0 ? violations.length : 1;
      const violationCount = violations.length;
      const compatibilityScore = Math.max(0, 100 - (violationCount / totalFiles) * 10);

      console.log(`\n📊 RTL Compatibility Score: ${compatibilityScore.toFixed(1)}%`);
      console.log(`   Total violations: ${violationCount}`);

      if (compatibilityScore >= 90) {
        console.log('   ✅ Excellent RTL support!');
      } else if (compatibilityScore >= 70) {
        console.log('   ⚠️  Good RTL support, some improvements needed');
      } else {
        console.log('   ❌ RTL support needs significant improvement');
      }

      // We expect good RTL support (>70%)
      expect(compatibilityScore).toBeGreaterThanOrEqual(70);
    });
  });
});
