/**
 * Git Diff Types
 *
 * Types for displaying and interacting with Git diffs in pull requests
 */

import type { File as ParsedDiffFile } from 'parse-diff';

/**
 * Diff view mode - split shows side-by-side, unified shows single column
 */
export type DiffViewMode = 'split' | 'unified';

/**
 * Diff syntax highlighting options
 */
export interface DiffSyntaxHighlight {
  enabled: boolean;
  language?: string;
}

/**
 * Extended file information with UI state
 */
export interface DiffFile extends ParsedDiffFile {
  id: string;
  isExpanded: boolean;
  isBinary: boolean;
  language?: string;
  additions: number;
  deletions: number;
  renamed?: boolean;
  binary?: boolean;
}

/**
 * Diff statistics summary
 */
export interface DiffStats {
  totalFiles: number;
  totalAdditions: number;
  totalDeletions: number;
  binaryFiles: number;
  modifiedFiles: number;
  addedFiles: number;
  deletedFiles: number;
  renamedFiles: number;
}

/**
 * Diff search result
 */
export interface DiffSearchResult {
  fileId: string;
  fileName: string;
  lineNumber: number;
  lineContent: string;
  matchStart: number;
  matchEnd: number;
}

/**
 * Diff navigation item for jump-to-file
 */
export interface DiffNavigationItem {
  id: string;
  fileName: string;
  status: 'added' | 'deleted' | 'modified' | 'renamed';
  additions: number;
  deletions: number;
}

/**
 * Diff viewer preferences stored in localStorage
 */
export interface DiffViewerPreferences {
  viewMode: DiffViewMode;
  syntaxHighlighting: boolean;
  wrapLines: boolean;
  showLineNumbers: boolean;
  expandAllByDefault: boolean;
}

/**
 * Pull request diff response from API
 */
export interface PullRequestDiffResponse {
  pullRequestId: number;
  diff: string;
  files: DiffFile[];
  stats: DiffStats;
}

/**
 * Diff hunk for granular viewing
 */
export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  content: string;
  changes: DiffChange[];
}

/**
 * Individual line change in a diff
 */
export interface DiffChange {
  type: 'add' | 'delete' | 'normal' | 'context';
  lineNumber?: number;
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}

/**
 * File change type enum
 */
export enum FileChangeType {
  ADDED = 'added',
  DELETED = 'deleted',
  MODIFIED = 'modified',
  RENAMED = 'renamed',
  COPIED = 'copied',
}

/**
 * Diff render options
 */
export interface DiffRenderOptions {
  viewMode: DiffViewMode;
  syntaxHighlighting: boolean;
  wrapLines: boolean;
  showLineNumbers: boolean;
  highlightChanges: boolean;
  contextLines?: number;
}

/**
 * Diff file filter options
 */
export interface DiffFileFilter {
  fileType?: string;
  changeType?: FileChangeType;
  searchQuery?: string;
  minAdditions?: number;
  minDeletions?: number;
}

/**
 * Language detection result
 */
export interface LanguageDetection {
  language: string;
  confidence: number;
  fileExtension: string;
}
