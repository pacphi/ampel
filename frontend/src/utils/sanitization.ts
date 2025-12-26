/**
 * Sanitization Utilities for User-Generated Content
 *
 * Uses DOMPurify to sanitize HTML content and prevent XSS attacks.
 * Applied to file paths and metadata (NOT code content - React handles that).
 */

import DOMPurify from 'dompurify';

/**
 * Sanitize file path for display
 *
 * Removes any HTML tags and dangerous attributes from file paths.
 * File paths should be plain text but users may control repository names.
 */
export function sanitizeFilePath(filePath: string | null | undefined): string {
  if (!filePath) return '';

  // DOMPurify configuration for strict text-only output
  const clean = DOMPurify.sanitize(filePath, {
    ALLOWED_TAGS: [], // No HTML tags allowed in file paths
    ALLOWED_ATTR: [],
    KEEP_CONTENT: true, // Keep text content
  });

  return clean;
}

/**
 * Sanitize metadata fields (language, mode, etc.)
 *
 * Similar to file paths - should be text only.
 */
export function sanitizeMetadata(value: string | null | undefined): string {
  if (!value) return '';

  const clean = DOMPurify.sanitize(value, {
    ALLOWED_TAGS: [],
    ALLOWED_ATTR: [],
    KEEP_CONTENT: true,
  });

  return clean;
}

/**
 * Sanitize previous filename (for renamed files)
 */
export function sanitizePreviousFilename(filename: string | null | undefined): string {
  if (!filename) return '';

  return sanitizeFilePath(filename);
}

/**
 * Validate that a string does not contain dangerous URL schemes
 */
export function isSafeUrl(url: string | null | undefined): boolean {
  if (!url) return true;

  const dangerous = ['javascript:', 'data:', 'vbscript:', 'file:', 'about:'];
  const lower = url.toLowerCase().trim();

  return !dangerous.some((scheme) => lower.startsWith(scheme));
}

/**
 * Remove dangerous URL schemes
 */
export function sanitizeUrl(url: string | null | undefined): string {
  if (!url) return '';

  if (!isSafeUrl(url)) {
    return '#'; // Safe fallback
  }

  return url;
}
