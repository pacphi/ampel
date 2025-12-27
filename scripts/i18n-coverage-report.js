#!/usr/bin/env node
/**
 * I18n Coverage Report Generator
 * Generates coverage reports for frontend translations
 *
 * Usage:
 *   node scripts/i18n-coverage-report.js [options]
 *
 * Options:
 *   --format <json|markdown|text>  Output format (default: text)
 *   --check                        Check mode: exit 1 if coverage below threshold
 *   --min-coverage <number>        Minimum coverage percentage (default: 95)
 *   --check-missing                Check for missing translation keys
 *   --output <file>                Output file (default: stdout)
 */

const fs = require('fs');
const path = require('path');

// Configuration
const LOCALES_DIR = path.join(__dirname, '../frontend/public/locales');
const BASE_LOCALE = 'en';
const COVERAGE_THRESHOLD = 95;

// Supported languages (20 total)
const SUPPORTED_LANGUAGES = [
  'en', 'es', 'fr', 'de', 'it', 'pt', 'nl', 'pl', 'ru', 'ja',
  'ko', 'zh', 'ar', 'he', 'hi', 'th', 'tr', 'cs', 'fi', 'sv'
];

// Parse command line arguments
const args = process.argv.slice(2);
const options = {
  format: 'text',
  check: false,
  checkMissing: false,
  minCoverage: COVERAGE_THRESHOLD,
  output: null
};

for (let i = 0; i < args.length; i++) {
  const arg = args[i];
  switch (arg) {
    case '--format':
      options.format = args[++i];
      break;
    case '--check':
      options.check = true;
      break;
    case '--check-missing':
      options.checkMissing = true;
      break;
    case '--min-coverage':
      options.minCoverage = parseFloat(args[++i]);
      break;
    case '--output':
      options.output = args[++i];
      break;
    case '--help':
      console.log(`
I18n Coverage Report Generator

Usage: node scripts/i18n-coverage-report.js [options]

Options:
  --format <json|markdown|text>  Output format (default: text)
  --check                        Check mode: exit 1 if coverage below threshold
  --min-coverage <number>        Minimum coverage percentage (default: 95)
  --check-missing                Check for missing translation keys
  --output <file>                Output file (default: stdout)
  --help                         Show this help message
`);
      process.exit(0);
  }
}

/**
 * Load translation file for a locale
 */
function loadTranslations(locale) {
  const filePath = path.join(LOCALES_DIR, locale, 'common.json');

  if (!fs.existsSync(filePath)) {
    return null;
  }

  try {
    const content = fs.readFileSync(filePath, 'utf8');
    return JSON.parse(content);
  } catch (error) {
    console.error(`Error loading ${filePath}: ${error.message}`);
    return null;
  }
}

/**
 * Get all translation keys from an object (nested)
 */
function getAllKeys(obj, prefix = '') {
  const keys = [];

  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;

    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      keys.push(...getAllKeys(value, fullKey));
    } else {
      keys.push(fullKey);
    }
  }

  return keys;
}

/**
 * Get value from nested object by key path
 */
function getValueByPath(obj, path) {
  return path.split('.').reduce((current, key) => current?.[key], obj);
}

/**
 * Check if a translation value is empty or placeholder
 */
function isEmpty(value) {
  if (value === null || value === undefined) return true;
  if (typeof value === 'string') {
    return value.trim() === '' || value === 'TODO' || value.startsWith('[TODO');
  }
  return false;
}

/**
 * Calculate coverage for all locales
 */
function calculateCoverage() {
  const baseTranslations = loadTranslations(BASE_LOCALE);

  if (!baseTranslations) {
    throw new Error(`Base locale (${BASE_LOCALE}) not found`);
  }

  const baseKeys = getAllKeys(baseTranslations);
  const totalKeys = baseKeys.length;

  const results = {
    baseLocale: BASE_LOCALE,
    totalKeys,
    locales: {},
    overallCoverage: 0,
    generatedAt: new Date().toISOString()
  };

  let totalCoverage = 0;
  let validLocalesCount = 0;

  for (const locale of SUPPORTED_LANGUAGES) {
    if (locale === BASE_LOCALE) {
      results.locales[locale] = {
        coverage: 100,
        translatedKeys: totalKeys,
        missingKeys: [],
        emptyKeys: []
      };
      continue;
    }

    const translations = loadTranslations(locale);

    if (!translations) {
      results.locales[locale] = {
        coverage: 0,
        translatedKeys: 0,
        missingKeys: baseKeys,
        emptyKeys: [],
        error: 'File not found'
      };
      continue;
    }

    const localeKeys = getAllKeys(translations);
    const missingKeys = [];
    const emptyKeys = [];

    for (const key of baseKeys) {
      const value = getValueByPath(translations, key);

      if (value === undefined) {
        missingKeys.push(key);
      } else if (isEmpty(value)) {
        emptyKeys.push(key);
      }
    }

    const translatedKeys = totalKeys - missingKeys.length - emptyKeys.length;
    const coverage = (translatedKeys / totalKeys) * 100;

    results.locales[locale] = {
      coverage: parseFloat(coverage.toFixed(2)),
      translatedKeys,
      missingKeys,
      emptyKeys
    };

    totalCoverage += coverage;
    validLocalesCount++;
  }

  results.overallCoverage = parseFloat((totalCoverage / validLocalesCount).toFixed(2));

  return results;
}

/**
 * Format results as JSON
 */
function formatJSON(results) {
  return JSON.stringify(results, null, 2);
}

/**
 * Format results as Markdown
 */
function formatMarkdown(results) {
  const lines = [];

  lines.push('# Translation Coverage Report');
  lines.push('');
  lines.push(`**Generated:** ${results.generatedAt}`);
  lines.push(`**Base Locale:** ${results.baseLocale}`);
  lines.push(`**Total Keys:** ${results.totalKeys}`);
  lines.push(`**Overall Coverage:** ${results.overallCoverage}%`);
  lines.push('');

  lines.push('## Coverage by Language');
  lines.push('');
  lines.push('| Language | Coverage | Translated | Missing | Empty | Status |');
  lines.push('|----------|----------|------------|---------|-------|--------|');

  for (const [locale, data] of Object.entries(results.locales)) {
    const status = data.coverage >= options.minCoverage ? '✅' : '❌';
    const missing = data.missingKeys?.length || 0;
    const empty = data.emptyKeys?.length || 0;

    lines.push(`| ${locale} | ${data.coverage}% | ${data.translatedKeys} | ${missing} | ${empty} | ${status} |`);
  }

  lines.push('');
  lines.push('## Missing Translations');
  lines.push('');

  const localesWithMissing = Object.entries(results.locales)
    .filter(([_, data]) => data.missingKeys && data.missingKeys.length > 0)
    .sort((a, b) => b[1].missingKeys.length - a[1].missingKeys.length);

  if (localesWithMissing.length === 0) {
    lines.push('✅ No missing translations');
  } else {
    for (const [locale, data] of localesWithMissing) {
      lines.push(`### ${locale} (${data.missingKeys.length} missing)`);
      lines.push('');
      for (const key of data.missingKeys.slice(0, 10)) {
        lines.push(`- \`${key}\``);
      }
      if (data.missingKeys.length > 10) {
        lines.push(`- ... and ${data.missingKeys.length - 10} more`);
      }
      lines.push('');
    }
  }

  return lines.join('\n');
}

/**
 * Format results as plain text
 */
function formatText(results) {
  const lines = [];

  lines.push('='.repeat(60));
  lines.push('Translation Coverage Report');
  lines.push('='.repeat(60));
  lines.push(`Generated: ${results.generatedAt}`);
  lines.push(`Base Locale: ${results.baseLocale}`);
  lines.push(`Total Keys: ${results.totalKeys}`);
  lines.push(`Overall Coverage: ${results.overallCoverage}%`);
  lines.push('');

  lines.push('Coverage by Language:');
  lines.push('-'.repeat(60));

  for (const [locale, data] of Object.entries(results.locales)) {
    const status = data.coverage >= options.minCoverage ? '✅' : '❌';
    lines.push(`${status} ${locale.padEnd(5)} ${data.coverage.toFixed(1).padStart(6)}% (${data.translatedKeys}/${results.totalKeys})`);

    if (data.missingKeys && data.missingKeys.length > 0) {
      lines.push(`   Missing: ${data.missingKeys.length} keys`);
    }
    if (data.emptyKeys && data.emptyKeys.length > 0) {
      lines.push(`   Empty: ${data.emptyKeys.length} keys`);
    }
  }

  lines.push('='.repeat(60));

  return lines.join('\n');
}

/**
 * Main execution
 */
function main() {
  try {
    // Calculate coverage
    const results = calculateCoverage();

    // Format output
    let output;
    switch (options.format) {
      case 'json':
        output = formatJSON(results);
        break;
      case 'markdown':
        output = formatMarkdown(results);
        break;
      case 'text':
      default:
        output = formatText(results);
        break;
    }

    // Write output
    if (options.output) {
      fs.writeFileSync(options.output, output, 'utf8');
      console.log(`Report written to: ${options.output}`);
    } else {
      console.log(output);
    }

    // Check mode: exit with error if coverage below threshold
    if (options.check || options.checkMissing) {
      const failedLocales = Object.entries(results.locales)
        .filter(([locale, data]) => locale !== BASE_LOCALE && data.coverage < options.minCoverage);

      const hasMissing = Object.values(results.locales)
        .some(data => data.missingKeys && data.missingKeys.length > 0);

      if (failedLocales.length > 0) {
        console.error(`\n❌ ${failedLocales.length} locale(s) below ${options.minCoverage}% coverage threshold`);
        process.exit(1);
      }

      if (options.checkMissing && hasMissing) {
        console.error('\n❌ Missing keys found');
        process.exit(1);
      }

      console.log(`\n✅ All locales meet ${options.minCoverage}% coverage threshold`);
    }

    process.exit(0);
  } catch (error) {
    console.error(`Error: ${error.message}`);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main();
}

module.exports = {
  calculateCoverage,
  formatJSON,
  formatMarkdown,
  formatText
};
