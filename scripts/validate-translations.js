#!/usr/bin/env node
/**
 * Translation Quality Validation Script v2.0
 *
 * Comprehensive validation of i18n translation files including:
 * - Completeness checking (missing keys/files)
 * - Placeholder preservation validation
 * - Untranslated content detection using:
 *   - Character set analysis (Cyrillic, CJK, Arabic, Hebrew, etc.)
 *   - Levenshtein distance similarity scoring
 *   - Statistical confidence levels
 * - Special handling for en-GB (British English)
 * - CI-friendly exit codes
 *
 * Usage:
 *   node scripts/validate-translations.js           # Validate all languages
 *   node scripts/validate-translations.js de        # Validate specific language
 *   node scripts/validate-translations.js --help    # Show help
 */

const fs = require('fs');
const path = require('path');

const LOCALES_DIR = path.join(__dirname, '../frontend/public/locales');
const BASE_LOCALE = 'en';
const NAMESPACES = ['common', 'dashboard', 'settings', 'errors', 'validation'];

// Quality thresholds
const THRESHOLDS = {
  EXCELLENT: 95,
  GOOD: 80,
  ACCEPTABLE: 60,
  POOR: 40,
};

// ANSI color codes
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
  gray: '\x1b[90m',
  bold: '\x1b[1m',
};

// Track all issues
const issues = {
  missingKeys: [],
  placeholderMismatches: [],
  emptyTranslations: [],
  incompleteFiles: [],
  untranslatedContent: [],
  lowConfidence: [],
};

let stats = {
  totalLanguages: 0,
  totalFiles: 0,
  totalKeys: 0,
  validTranslations: 0,
  issues: 0,
};

// ============================================================================
// String Analysis Utilities
// ============================================================================

/**
 * Calculate Levenshtein distance between two strings
 * Used for similarity scoring to detect near-matches
 */
function levenshtein(a, b) {
  const matrix = [];

  for (let i = 0; i <= b.length; i++) {
    matrix[i] = [i];
  }

  for (let j = 0; j <= a.length; j++) {
    matrix[0][j] = j;
  }

  for (let i = 1; i <= b.length; i++) {
    for (let j = 1; j <= a.length; j++) {
      if (b.charAt(i - 1) === a.charAt(j - 1)) {
        matrix[i][j] = matrix[i - 1][j - 1];
      } else {
        matrix[i][j] = Math.min(
          matrix[i - 1][j - 1] + 1,
          matrix[i][j - 1] + 1,
          matrix[i - 1][j] + 1
        );
      }
    }
  }

  return matrix[b.length][a.length];
}

/**
 * Calculate similarity ratio (0-1) using Levenshtein distance
 * 1.0 = identical, 0.0 = completely different
 */
function similarityRatio(source, target) {
  const maxLen = Math.max(source.length, target.length);
  if (maxLen === 0) return 1.0;

  const distance = levenshtein(source.toLowerCase(), target.toLowerCase());
  return 1.0 - distance / maxLen;
}

/**
 * Detect the dominant character set of a string
 * Returns: 'latin', 'cyrillic', 'cjk', 'arabic', 'hebrew', 'devanagari', 'thai', 'unknown'
 */
function detectCharacterSet(text) {
  const str = String(text);

  let latin = 0;
  let cyrillic = 0;
  let cjk = 0;
  let arabic = 0;
  let hebrew = 0;
  let devanagari = 0;
  let thai = 0;
  let greek = 0;
  let korean = 0;
  let japanese = 0;

  for (const char of str) {
    const code = char.charCodeAt(0);

    // Latin (Basic + Extended)
    if (
      (code >= 0x0041 && code <= 0x005a) ||
      (code >= 0x0061 && code <= 0x007a) ||
      (code >= 0x00c0 && code <= 0x00ff) ||
      (code >= 0x0100 && code <= 0x017f)
    ) {
      latin++;
    }
    // Cyrillic
    else if (code >= 0x0400 && code <= 0x04ff) {
      cyrillic++;
    }
    // CJK Unified Ideographs
    else if ((code >= 0x4e00 && code <= 0x9fff) || (code >= 0x3400 && code <= 0x4dbf)) {
      cjk++;
    }
    // Arabic
    else if (code >= 0x0600 && code <= 0x06ff) {
      arabic++;
    }
    // Hebrew
    else if (code >= 0x0590 && code <= 0x05ff) {
      hebrew++;
    }
    // Devanagari (Hindi)
    else if (code >= 0x0900 && code <= 0x097f) {
      devanagari++;
    }
    // Thai
    else if (code >= 0x0e00 && code <= 0x0e7f) {
      thai++;
    }
    // Greek
    else if (code >= 0x0370 && code <= 0x03ff) {
      greek++;
    }
    // Korean Hangul
    else if ((code >= 0xac00 && code <= 0xd7af) || (code >= 0x1100 && code <= 0x11ff)) {
      korean++;
    }
    // Japanese Hiragana/Katakana
    else if ((code >= 0x3040 && code <= 0x309f) || (code >= 0x30a0 && code <= 0x30ff)) {
      japanese++;
    }
  }

  const counts = {
    latin,
    cyrillic,
    cjk,
    arabic,
    hebrew,
    devanagari,
    thai,
    greek,
    korean,
    japanese,
  };
  const total = Object.values(counts).reduce((a, b) => a + b, 0);

  if (total === 0) return 'unknown';

  // Find dominant character set
  const max = Math.max(...Object.values(counts));
  for (const [charset, count] of Object.entries(counts)) {
    if (count === max && count > 0) return charset;
  }

  return 'unknown';
}

/**
 * Extract placeholders from a string (e.g., {{name}}, {count})
 */
function extractPlaceholders(text) {
  if (typeof text !== 'string') return [];
  // Match {{...}} and {...} patterns
  const matches = text.match(/\{\{?[^}]+\}?\}/g);
  return matches ? matches.sort() : [];
}

/**
 * Check if a translation is untranslated with statistical confidence
 * Returns: { untranslated: boolean, confidence: number, reason: string }
 */
function isUntranslated(sourceText, targetText, targetLang) {
  const source = String(sourceText);
  const target = String(targetText);

  // 1. Exact match = definitely untranslated (unless it's a brand name, number, etc.)
  if (source === target) {
    // Allow exact matches for very short strings (likely technical terms, numbers)
    if (source.length <= 3) {
      return { untranslated: false, confidence: 0.5, reason: 'short_term' };
    }
    // Allow brand names and technical terms
    const brandNames = [
      'GitHub',
      'GitLab',
      'Bitbucket',
      'Ampel',
      'OAuth',
      'API',
      'URL',
      'HTTP',
      'HTTPS',
    ];
    if (brandNames.some((brand) => source.includes(brand))) {
      return { untranslated: false, confidence: 0.8, reason: 'brand_name' };
    }
    return { untranslated: true, confidence: 1.0, reason: 'exact_match' };
  }

  // 2. Character set analysis - most reliable for non-Latin target languages
  const sourceCharSet = detectCharacterSet(source);
  const targetCharSet = detectCharacterSet(target);

  // Special case: en-GB uses Latin like source English
  const isLatinTarget = [
    'en-GB',
    'de',
    'fr',
    'es-ES',
    'es-MX',
    'pt-BR',
    'it',
    'nl',
    'da',
    'sv',
    'no',
    'fi',
    'pl',
    'cs',
    'tr',
    'vi',
  ].includes(targetLang);

  // If source is Latin and target uses a different script, definitely translated
  if (sourceCharSet === 'latin' && targetCharSet !== 'latin' && targetCharSet !== 'unknown') {
    return { untranslated: false, confidence: 1.0, reason: 'different_script' };
  }

  // 3. Very high similarity (>95%) for Latin languages = likely untranslated
  const similarity = similarityRatio(source, target);
  if (similarity > 0.95) {
    return { untranslated: true, confidence: similarity, reason: 'high_similarity' };
  }

  // 4. For Latin-to-Latin translations, use word overlap analysis
  if (sourceCharSet === 'latin' && (targetCharSet === 'latin' || targetCharSet === 'unknown')) {
    const targetLower = target.toLowerCase();
    const sourceLower = source.toLowerCase();

    // Extract significant words (3+ chars)
    const targetWords = targetLower.match(/[a-z]{3,}/g) || [];
    const sourceWords = sourceLower.match(/[a-z]{3,}/g) || [];

    if (targetWords.length === 0 || sourceWords.length === 0) {
      return { untranslated: similarity > 0.8, confidence: 0.6, reason: 'no_significant_words' };
    }

    // Count word overlap
    let matchingWords = 0;
    for (const targetWord of targetWords) {
      if (sourceWords.includes(targetWord)) {
        matchingWords++;
      }
    }

    const matchRatio = matchingWords / Math.max(targetWords.length, 1);

    // High word overlap = likely untranslated
    if (matchRatio > 0.6) {
      return { untranslated: true, confidence: matchRatio, reason: 'word_overlap' };
    }

    // Moderate overlap = uncertain
    if (matchRatio > 0.3) {
      return { untranslated: false, confidence: 0.5, reason: 'moderate_overlap' };
    }

    // Low overlap = likely translated
    return { untranslated: false, confidence: 1.0 - matchRatio, reason: 'low_overlap' };
  }

  // 5. Default: consider translated if passed above checks
  return { untranslated: false, confidence: 0.7, reason: 'passed_checks' };
}

// ============================================================================
// Validation Functions
// ============================================================================

/**
 * Get all leaf keys from a nested object
 */
function getLeafKeys(obj, prefix = '') {
  let keys = [];
  for (const key in obj) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (typeof obj[key] === 'object' && obj[key] !== null) {
      keys = keys.concat(getLeafKeys(obj[key], path));
    } else {
      keys.push(path);
    }
  }
  return keys;
}

/**
 * Flatten a nested object to key-value pairs
 */
function flattenObject(obj, prefix = '') {
  const result = {};

  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === 'object' && !Array.isArray(value)) {
      Object.assign(result, flattenObject(value, fullKey));
    } else {
      result[fullKey] = value;
    }
  }

  return result;
}

/**
 * Load JSON file safely
 */
function loadJSON(filePath) {
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    return JSON.parse(content);
  } catch (error) {
    return null;
  }
}

/**
 * Validate translations for a single namespace
 */
function validateNamespace(baseData, transData, locale, namespace) {
  const localIssues = {
    missing: [],
    empty: [],
    placeholders: [],
    untranslated: [],
    lowConfidence: [],
  };

  const baseFlat = flattenObject(baseData);
  const transFlat = flattenObject(transData);

  for (const [key, baseValue] of Object.entries(baseFlat)) {
    const transValue = transFlat[key];

    // Check for missing key
    if (transValue === undefined) {
      localIssues.missing.push({
        type: 'missing_key',
        locale,
        file: namespace,
        path: key,
        message: `Missing translation for: ${key}`,
        baseValue,
      });
      continue;
    }

    // Check for empty translation
    if (transValue === '' || transValue === null) {
      localIssues.empty.push({
        type: 'empty_translation',
        locale,
        file: namespace,
        path: key,
        message: `Empty translation: ${key}`,
        baseValue,
      });
      continue;
    }

    // Check placeholder preservation
    const basePlaceholders = extractPlaceholders(baseValue);
    const transPlaceholders = extractPlaceholders(transValue);

    if (JSON.stringify(basePlaceholders) !== JSON.stringify(transPlaceholders)) {
      localIssues.placeholders.push({
        type: 'placeholder_mismatch',
        locale,
        file: namespace,
        path: key,
        message: `Placeholder mismatch: Expected [${basePlaceholders.join(', ')}], got [${transPlaceholders.join(', ')}]`,
        baseValue,
        transValue,
      });
    }

    // Check for untranslated content (skip if placeholder mismatch already found)
    if (typeof transValue === 'string' && locale !== BASE_LOCALE) {
      const result = isUntranslated(baseValue, transValue, locale);

      if (result.untranslated && result.confidence >= 0.8) {
        localIssues.untranslated.push({
          type: 'untranslated_content',
          locale,
          file: namespace,
          path: key,
          message: `Untranslated (${result.reason}): ${key}`,
          baseValue,
          transValue,
          confidence: result.confidence,
          reason: result.reason,
        });
      } else if (result.untranslated && result.confidence >= 0.5) {
        localIssues.lowConfidence.push({
          type: 'low_confidence',
          locale,
          file: namespace,
          path: key,
          message: `Possibly untranslated (${result.reason}): ${key}`,
          baseValue,
          transValue,
          confidence: result.confidence,
          reason: result.reason,
        });
      }
    }
  }

  return localIssues;
}

/**
 * Get all translation files for a locale
 */
function getTranslationFiles(locale) {
  const localeDir = path.join(LOCALES_DIR, locale);
  if (!fs.existsSync(localeDir)) return [];

  return fs
    .readdirSync(localeDir)
    .filter((file) => file.endsWith('.json'))
    .map((file) => ({
      name: file.replace('.json', ''),
      path: path.join(localeDir, file),
    }));
}

/**
 * Get all available locales
 */
function getAvailableLocales() {
  if (!fs.existsSync(LOCALES_DIR)) return [];

  return fs
    .readdirSync(LOCALES_DIR)
    .filter((item) => {
      const itemPath = path.join(LOCALES_DIR, item);
      return fs.statSync(itemPath).isDirectory() && item !== BASE_LOCALE;
    })
    .sort();
}

/**
 * Create a visual progress bar
 */
function progressBar(percentage, width = 20) {
  const filled = Math.floor((percentage / 100) * width);
  const empty = width - filled;
  return '█'.repeat(filled) + '░'.repeat(empty);
}

/**
 * Validate all translations for a single locale
 */
function validateLocale(locale) {
  console.log(`\n${colors.blue}${'━'.repeat(60)}${colors.reset}`);
  console.log(`${colors.blue}${colors.bold} Validating: ${colors.cyan}${locale}${colors.reset}`);
  console.log(`${colors.blue}${'━'.repeat(60)}${colors.reset}`);

  const localeStats = {
    totalKeys: 0,
    translatedKeys: 0,
    missingKeys: 0,
    emptyKeys: 0,
    placeholderIssues: 0,
    untranslatedKeys: 0,
    lowConfidenceKeys: 0,
    files: {},
  };

  for (const namespace of NAMESPACES) {
    const basePath = path.join(LOCALES_DIR, BASE_LOCALE, `${namespace}.json`);
    const transPath = path.join(LOCALES_DIR, locale, `${namespace}.json`);

    const baseData = loadJSON(basePath);
    const transData = loadJSON(transPath);

    stats.totalFiles++;

    if (!baseData) {
      console.log(`  ${colors.gray}⊘ ${namespace}: Base file not found${colors.reset}`);
      continue;
    }

    if (!transData) {
      console.log(`  ${colors.red}✗ ${namespace}: Translation file missing${colors.reset}`);
      issues.incompleteFiles.push({
        locale,
        file: namespace,
        completion: 0,
        missing: 'entire file',
      });
      continue;
    }

    // Validate namespace
    const namespaceIssues = validateNamespace(baseData, transData, locale, namespace);

    // Count keys
    const baseKeys = getLeafKeys(baseData);
    const transKeys = getLeafKeys(transData);
    const namespaceTotal = baseKeys.length;

    localeStats.totalKeys += namespaceTotal;
    stats.totalKeys += namespaceTotal;

    // Calculate effective translations (excluding issues)
    const effectiveIssues =
      namespaceIssues.missing.length +
      namespaceIssues.empty.length +
      namespaceIssues.untranslated.length;
    const effectiveTranslated = Math.max(0, transKeys.length - effectiveIssues);

    localeStats.translatedKeys += effectiveTranslated;
    localeStats.missingKeys += namespaceIssues.missing.length;
    localeStats.emptyKeys += namespaceIssues.empty.length;
    localeStats.placeholderIssues += namespaceIssues.placeholders.length;
    localeStats.untranslatedKeys += namespaceIssues.untranslated.length;
    localeStats.lowConfidenceKeys += namespaceIssues.lowConfidence.length;

    // Store issues
    issues.missingKeys.push(...namespaceIssues.missing);
    issues.emptyTranslations.push(...namespaceIssues.empty);
    issues.placeholderMismatches.push(...namespaceIssues.placeholders);
    issues.untranslatedContent.push(...namespaceIssues.untranslated);
    issues.lowConfidence.push(...namespaceIssues.lowConfidence);

    // Calculate coverage
    const coverage = namespaceTotal > 0 ? (effectiveTranslated / namespaceTotal) * 100 : 0;
    localeStats.files[namespace] = {
      coverage: coverage.toFixed(1),
      keys: effectiveTranslated,
      total: namespaceTotal,
    };

    // Display namespace result
    const icon = coverage >= 95 ? '✓' : coverage >= 70 ? '⚠' : '✗';
    const color = coverage >= 95 ? colors.green : coverage >= 70 ? colors.yellow : colors.red;

    console.log(
      `  ${color}${icon} ${namespace}:${colors.reset} ${coverage.toFixed(1)}% (${effectiveTranslated}/${namespaceTotal} keys)`
    );

    // Show critical issues (placeholder mismatches)
    if (namespaceIssues.placeholders.length > 0) {
      namespaceIssues.placeholders.slice(0, 3).forEach((issue) => {
        console.log(
          `    ${colors.red}⚠ PLACEHOLDER:${colors.reset} ${colors.gray}${issue.path}${colors.reset}`
        );
      });
      if (namespaceIssues.placeholders.length > 3) {
        console.log(
          `    ${colors.gray}... and ${namespaceIssues.placeholders.length - 3} more placeholder issues${colors.reset}`
        );
      }
    }

    // Show untranslated (high confidence)
    if (namespaceIssues.untranslated.length > 0 && namespaceIssues.untranslated.length <= 5) {
      namespaceIssues.untranslated.forEach((issue) => {
        console.log(
          `    ${colors.yellow}→${colors.reset} ${colors.gray}${issue.path}${colors.reset} [${(issue.confidence * 100).toFixed(0)}%]`
        );
      });
    } else if (namespaceIssues.untranslated.length > 5) {
      console.log(
        `    ${colors.gray}... ${namespaceIssues.untranslated.length} untranslated keys${colors.reset}`
      );
    }
  }

  // Locale summary
  const overallCoverage =
    localeStats.totalKeys > 0 ? (localeStats.translatedKeys / localeStats.totalKeys) * 100 : 0;

  console.log(`\n${colors.blue}${'─'.repeat(40)}${colors.reset}`);

  const icon =
    overallCoverage >= THRESHOLDS.EXCELLENT ? '✓' : overallCoverage >= THRESHOLDS.GOOD ? '⚠' : '✗';
  const color =
    overallCoverage >= THRESHOLDS.EXCELLENT
      ? colors.green
      : overallCoverage >= THRESHOLDS.GOOD
        ? colors.yellow
        : colors.red;

  console.log(
    `${color}${icon} Overall: ${overallCoverage.toFixed(1)}%${colors.reset} ${progressBar(overallCoverage)}`
  );
  console.log(`  ${colors.green}✓ Translated: ${localeStats.translatedKeys}${colors.reset}`);

  if (localeStats.missingKeys > 0) {
    console.log(`  ${colors.red}✗ Missing: ${localeStats.missingKeys}${colors.reset}`);
  }
  if (localeStats.untranslatedKeys > 0) {
    console.log(`  ${colors.yellow}⚠ Untranslated: ${localeStats.untranslatedKeys}${colors.reset}`);
  }
  if (localeStats.placeholderIssues > 0) {
    console.log(
      `  ${colors.red}⚠ Placeholder issues: ${localeStats.placeholderIssues}${colors.reset}`
    );
  }
  if (localeStats.lowConfidenceKeys > 0) {
    console.log(`  ${colors.gray}? Uncertain: ${localeStats.lowConfidenceKeys}${colors.reset}`);
  }

  stats.validTranslations += localeStats.translatedKeys;
  stats.issues +=
    localeStats.missingKeys +
    localeStats.emptyKeys +
    localeStats.placeholderIssues +
    localeStats.untranslatedKeys;

  return {
    locale,
    coverage: overallCoverage,
    stats: localeStats,
  };
}

/**
 * Print comprehensive summary
 */
function printSummary(results) {
  console.log(`\n\n${colors.blue}${'═'.repeat(70)}${colors.reset}`);
  console.log(`${colors.blue}${colors.bold} 📊 VALIDATION SUMMARY${colors.reset}`);
  console.log(`${colors.blue}${'═'.repeat(70)}${colors.reset}\n`);

  // Overall stats
  console.log(`${colors.cyan}Overall Statistics:${colors.reset}`);
  console.log(`  Languages validated: ${results.length}`);
  console.log(`  Total keys checked: ${stats.totalKeys}`);
  console.log(`  Valid translations: ${stats.validTranslations}`);
  console.log(`  Total issues: ${stats.issues}\n`);

  // Sort by coverage
  results.sort((a, b) => b.coverage - a.coverage);

  // Coverage breakdown
  console.log(`${colors.cyan}Coverage by Language:${colors.reset}\n`);

  for (const result of results) {
    const icon =
      result.coverage >= THRESHOLDS.EXCELLENT
        ? '✓'
        : result.coverage >= THRESHOLDS.GOOD
          ? '⚠'
          : '✗';
    const color =
      result.coverage >= THRESHOLDS.EXCELLENT
        ? colors.green
        : result.coverage >= THRESHOLDS.GOOD
          ? colors.yellow
          : colors.red;

    const bar = progressBar(result.coverage);
    let line = `  ${color}${icon} ${result.locale.padEnd(8)}${colors.reset} ${bar} ${color}${result.coverage.toFixed(1).padStart(5)}%${colors.reset}`;
    line += ` (${result.stats.translatedKeys}/${result.stats.totalKeys})`;

    if (result.stats.lowConfidenceKeys > 0) {
      line += ` ${colors.gray}[${result.stats.lowConfidenceKeys} uncertain]${colors.reset}`;
    }

    console.log(line);
  }

  // Issue breakdown
  console.log(`\n${colors.cyan}Issue Breakdown:${colors.reset}\n`);
  console.log(`  Missing keys:        ${issues.missingKeys.length}`);
  console.log(`  Empty translations:  ${issues.emptyTranslations.length}`);
  console.log(
    `  Placeholder issues:  ${colors.red}${issues.placeholderMismatches.length}${colors.reset} (critical)`
  );
  console.log(`  Untranslated:        ${issues.untranslatedContent.length}`);
  console.log(`  Low confidence:      ${issues.lowConfidence.length}`);

  // Critical issues detail
  if (issues.placeholderMismatches.length > 0) {
    console.log(
      `\n${colors.red}${colors.bold}🚨 CRITICAL: Placeholder Mismatches${colors.reset}\n`
    );
    console.log(`${colors.gray}These will cause runtime errors!${colors.reset}\n`);

    issues.placeholderMismatches.slice(0, 10).forEach((issue) => {
      console.log(`  ${colors.cyan}${issue.locale}/${issue.file}${colors.reset} → ${issue.path}`);
      console.log(`    Base:  "${issue.baseValue}"`);
      console.log(`    Trans: "${issue.transValue}"`);
      console.log(`    ${colors.red}${issue.message}${colors.reset}\n`);
    });

    if (issues.placeholderMismatches.length > 10) {
      console.log(
        `  ${colors.gray}... and ${issues.placeholderMismatches.length - 10} more${colors.reset}\n`
      );
    }
  }

  // Quality tiers
  const excellent = results.filter((r) => r.coverage >= THRESHOLDS.EXCELLENT).length;
  const good = results.filter(
    (r) => r.coverage >= THRESHOLDS.GOOD && r.coverage < THRESHOLDS.EXCELLENT
  ).length;
  const acceptable = results.filter(
    (r) => r.coverage >= THRESHOLDS.ACCEPTABLE && r.coverage < THRESHOLDS.GOOD
  ).length;
  const poor = results.filter((r) => r.coverage < THRESHOLDS.ACCEPTABLE).length;

  console.log(`\n${colors.cyan}Quality Tiers:${colors.reset}\n`);
  console.log(`  ${colors.green}⭐⭐⭐ Excellent (95%+): ${excellent} languages${colors.reset}`);
  console.log(`  ${colors.yellow}⭐⭐  Good (80-94%):     ${good} languages${colors.reset}`);
  console.log(
    `  ${colors.yellow}⭐    Acceptable (60-79%): ${acceptable} languages${colors.reset}`
  );
  console.log(`  ${colors.red}      Poor (<60%):      ${poor} languages${colors.reset}`);

  console.log(`\n${colors.blue}${'═'.repeat(70)}${colors.reset}\n`);
}

/**
 * Print help message
 */
function printHelp() {
  console.log(`
${colors.blue}${colors.bold}Translation Quality Validation Tool v2.0${colors.reset}

${colors.cyan}Usage:${colors.reset}
  node scripts/validate-translations.js              # Validate all languages
  node scripts/validate-translations.js <locale>     # Validate specific language
  node scripts/validate-translations.js --help       # Show this help

${colors.cyan}Features:${colors.reset}
  • Placeholder preservation validation (critical for runtime)
  • Character set detection (Cyrillic, CJK, Arabic, Hebrew, etc.)
  • Levenshtein distance similarity scoring
  • Statistical confidence for untranslated detection
  • Special handling for en-GB and Latin-script languages
  • Visual progress bars and quality tiers

${colors.cyan}Examples:${colors.reset}
  node scripts/validate-translations.js de           # Validate German
  node scripts/validate-translations.js zh-CN        # Validate Chinese (Simplified)
  node scripts/validate-translations.js              # Validate all 26 languages

${colors.cyan}Exit Codes:${colors.reset}
  0  Success (no critical issues)
  1  Critical issues found (placeholder mismatches)
`);
}

// ============================================================================
// Main Execution
// ============================================================================

function main() {
  const args = process.argv.slice(2);

  if (args.includes('--help') || args.includes('-h')) {
    printHelp();
    process.exit(0);
  }

  console.log(`${colors.blue}${colors.bold}🔍 Translation Quality Validation${colors.reset}`);
  console.log(`${colors.gray}Base locale: ${BASE_LOCALE}${colors.reset}`);
  console.log(`${colors.gray}Locales directory: ${LOCALES_DIR}${colors.reset}`);

  const results = [];

  if (args.length > 0 && !args[0].startsWith('-')) {
    // Validate specific locale
    const locale = args[0];
    const result = validateLocale(locale);
    results.push(result);
  } else {
    // Validate all locales
    const locales = getAvailableLocales();
    stats.totalLanguages = locales.length;

    console.log(`${colors.gray}Languages to validate: ${locales.length}${colors.reset}`);

    for (const locale of locales) {
      const result = validateLocale(locale);
      results.push(result);
    }
  }

  printSummary(results);

  // Exit with appropriate code
  if (issues.placeholderMismatches.length > 0) {
    console.log(
      `${colors.red}❌ Validation FAILED: Critical placeholder mismatches found${colors.reset}`
    );
    process.exit(1);
  } else if (stats.issues > 0) {
    console.log(`${colors.yellow}⚠️  Validation completed with warnings${colors.reset}`);
    process.exit(0);
  } else {
    console.log(`${colors.green}✅ Validation PASSED: No issues found${colors.reset}`);
    process.exit(0);
  }
}

main();
