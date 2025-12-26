/**
 * usePullRequestDiff Hook
 *
 * TanStack Query hook for fetching and managing pull request diffs
 */

import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import parseDiff from 'parse-diff';
import type { DiffFile, DiffStats, PullRequestDiffResponse } from '../types/diff';
import apiClient from '../api/client';

/**
 * Fetch pull request diff from API
 */
async function fetchPullRequestDiff(pullRequestId: number): Promise<PullRequestDiffResponse> {
  const response = await apiClient.get<{ diff: string }>(`/pull-requests/${pullRequestId}/diff`);

  const diffText = response.data.diff;
  const parsedFiles = parseDiff(diffText);

  // Transform parsed files to DiffFile with UI state
  const files: DiffFile[] = parsedFiles.map((file, index) => {
    const fileWithExtra = file as typeof file & { binary?: boolean; renamed?: boolean };
    return {
      ...file,
      id: `file-${index}`,
      isExpanded: false,
      isBinary: fileWithExtra.binary || false,
      language: detectLanguage(file.to || file.from || ''),
      additions: file.additions || 0,
      deletions: file.deletions || 0,
      renamed: fileWithExtra.renamed || false,
      binary: fileWithExtra.binary || false,
    };
  });

  // Calculate statistics
  const stats: DiffStats = {
    totalFiles: files.length,
    totalAdditions: files.reduce((sum, f) => sum + f.additions, 0),
    totalDeletions: files.reduce((sum, f) => sum + f.deletions, 0),
    binaryFiles: files.filter((f) => f.isBinary).length,
    modifiedFiles: files.filter((f) => !f.new && !f.deleted && !f.renamed).length,
    addedFiles: files.filter((f) => f.new).length,
    deletedFiles: files.filter((f) => f.deleted).length,
    renamedFiles: files.filter((f) => {
      const fileWithExtra = f as typeof f & { renamed?: boolean };
      return fileWithExtra.renamed;
    }).length,
  };

  return {
    pullRequestId,
    diff: diffText,
    files,
    stats,
  };
}

/**
 * Detect programming language from file path
 */
function detectLanguage(filePath: string): string {
  const extension = filePath.split('.').pop()?.toLowerCase();

  const languageMap: Record<string, string> = {
    ts: 'typescript',
    tsx: 'typescript',
    js: 'javascript',
    jsx: 'javascript',
    rs: 'rust',
    py: 'python',
    go: 'go',
    java: 'java',
    rb: 'ruby',
    php: 'php',
    cpp: 'cpp',
    c: 'c',
    cs: 'csharp',
    swift: 'swift',
    kt: 'kotlin',
    scala: 'scala',
    sql: 'sql',
    md: 'markdown',
    json: 'json',
    yaml: 'yaml',
    yml: 'yaml',
    xml: 'xml',
    html: 'html',
    css: 'css',
    scss: 'scss',
    sass: 'sass',
    sh: 'bash',
    bash: 'bash',
    zsh: 'bash',
    dockerfile: 'dockerfile',
  };

  return languageMap[extension || ''] || 'plaintext';
}

/**
 * Hook for fetching pull request diff with caching
 */
export function usePullRequestDiff(
  pullRequestId: number | undefined
): UseQueryResult<PullRequestDiffResponse, Error> {
  return useQuery({
    queryKey: ['pullRequestDiff', pullRequestId],
    queryFn: () => fetchPullRequestDiff(pullRequestId!),
    enabled: !!pullRequestId,
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 30 * 60 * 1000, // 30 minutes (formerly cacheTime)
    retry: 2,
  });
}

/**
 * Hook for managing diff viewer preferences
 */
export function useDiffViewerPreferences() {
  const STORAGE_KEY = 'ampel-diff-viewer-preferences';

  const getPreferences = () => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        return JSON.parse(stored);
      } catch {
        return getDefaultPreferences();
      }
    }
    return getDefaultPreferences();
  };

  const getDefaultPreferences = () => ({
    viewMode: 'unified' as const,
    syntaxHighlighting: true,
    wrapLines: false,
    showLineNumbers: true,
    expandAllByDefault: false,
  });

  const setPreferences = (preferences: Partial<ReturnType<typeof getPreferences>>) => {
    const current = getPreferences();
    const updated = { ...current, ...preferences };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  };

  return {
    preferences: getPreferences(),
    setPreferences,
  };
}
