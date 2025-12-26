/**
 * DiffViewer Component
 *
 * Main wrapper component for displaying Git diffs with @git-diff-view/react
 */

import React, { useState } from 'react';
import { DiffView } from '@git-diff-view/react';
import '@git-diff-view/react/styles/diff-view.css';
import type { DiffFile, DiffViewMode } from '../../types/diff';
import { detectLanguage, supportsHighlighting } from '../../utils/languageDetection';
import {
  sanitizeFilePath,
  sanitizeMetadata,
  sanitizePreviousFilename,
} from '../../utils/sanitization';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import { Input } from '../ui/input';
import { Button } from '../ui/button';
import { ChevronDown, FileText, ExternalLink, AlertTriangle } from 'lucide-react';

export interface DiffViewerProps {
  // For single file viewing
  file?: DiffFile;
  // For multi-file viewing (test compatibility)
  diff?: {
    files: Array<{
      filePath: string;
      status: string;
      additions: number;
      deletions: number;
      changes: number;
      language?: string | null;
      isBinary?: boolean;
      patch?: string | null;
      previousFilename?: string;
    }>;
    summary: {
      totalFiles: number;
      totalAdditions: number;
      totalDeletions: number;
      totalChanges: number;
    };
  };
  viewMode?: DiffViewMode;
  syntaxHighlighting?: boolean;
  wrapLines?: boolean;
  showLineNumbers?: boolean;
  className?: string;
  // Optional: Provide external link for "View on Provider" fallback
  externalDiffUrl?: string;
}

// Large diff protection: 5,000 lines per file limit
const MAX_LINES_PER_FILE = 5000;

/**
 * Count total lines in a file's chunks
 */
function countLinesInFile(file: DiffFile): number {
  if (!file.chunks || file.chunks.length === 0) return 0;

  return file.chunks.reduce((total, chunk) => {
    return total + (chunk.changes?.length || 0);
  }, 0);
}

/**
 * DiffViewer component wraps @git-diff-view/react with Ampel-specific configuration
 */
export const DiffViewer: React.FC<DiffViewerProps> = ({
  file,
  diff,
  viewMode = 'unified',
  syntaxHighlighting = true,
  wrapLines = false,
  className = '',
  externalDiffUrl,
}) => {
  const [expandedFiles, setExpandedFiles] = useState<Set<string>>(new Set());
  const [filterQuery, setFilterQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string | null>(null);

  // Multi-file view for tests
  if (diff) {
    const filteredFiles = diff.files.filter((f) => {
      if (filterQuery && !f.filePath.toLowerCase().includes(filterQuery.toLowerCase())) {
        return false;
      }
      if (statusFilter) {
        if (statusFilter === 'modified' && f.status !== 'modified') return false;
        if (statusFilter === 'added' && f.status !== 'added') return false;
        if (statusFilter === 'deleted' && f.status !== 'deleted') return false;
      }
      return true;
    });

    const toggleFile = (filePath: string) => {
      setExpandedFiles((prev) => {
        const next = new Set(prev);
        if (next.has(filePath)) {
          next.delete(filePath);
        } else {
          next.add(filePath);
        }
        return next;
      });
    };

    return (
      <div className={className}>
        {/* Summary */}
        <Card className="p-4 mb-4">
          <div className="flex items-center gap-4 text-sm">
            <span className="font-medium">{diff.summary.totalFiles} files changed</span>
            <span className="text-green-600">{diff.summary.totalAdditions} additions</span>
            <span className="text-red-600">{diff.summary.totalDeletions} deletions</span>
          </div>
        </Card>

        {/* Filters */}
        <div className="mb-4 flex gap-2">
          <Input
            placeholder="Filter files..."
            value={filterQuery}
            onChange={(e) => setFilterQuery(e.target.value)}
            className="max-w-xs"
          />
          <Button
            variant={statusFilter === 'modified' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setStatusFilter(statusFilter === 'modified' ? null : 'modified')}
          >
            Modified
          </Button>
          <Button
            variant={statusFilter === 'added' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setStatusFilter(statusFilter === 'added' ? null : 'added')}
          >
            Added
          </Button>
          <Button
            variant={statusFilter === 'deleted' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setStatusFilter(statusFilter === 'deleted' ? null : 'deleted')}
          >
            Deleted
          </Button>
        </div>

        {/* File List */}
        {filteredFiles.length === 0 ? (
          <Card className="p-8 text-center text-muted-foreground">
            <FileText className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>No files changed</p>
          </Card>
        ) : (
          <div className="space-y-2">
            {filteredFiles.map((fileItem) => {
              // Sanitize user-controlled content
              const sanitizedPath = sanitizeFilePath(fileItem.filePath);
              const sanitizedLanguage = sanitizeMetadata(fileItem.language);
              const sanitizedPrevFilename = sanitizePreviousFilename(fileItem.previousFilename);

              return (
                <Card key={sanitizedPath} className="overflow-hidden">
                  <div
                    className="p-4 cursor-pointer hover:bg-accent flex items-center justify-between"
                    onClick={() => toggleFile(fileItem.filePath)}
                  >
                    <div className="flex items-center gap-3 flex-1">
                      <ChevronDown
                        className={`h-4 w-4 transition-transform ${
                          expandedFiles.has(fileItem.filePath) ? '' : '-rotate-90'
                        }`}
                      />
                      <span className="font-medium">{sanitizedPath}</span>
                      <Badge variant="outline">{fileItem.status}</Badge>
                      {sanitizedLanguage && <Badge variant="secondary">{sanitizedLanguage}</Badge>}
                      {fileItem.isBinary && <Badge variant="secondary">Binary</Badge>}
                      {sanitizedPrevFilename && (
                        <span className="text-sm text-muted-foreground">
                          from {sanitizedPrevFilename}
                        </span>
                      )}
                    </div>
                    <div className="flex gap-4">
                      {fileItem.additions > 0 && (
                        <span className="text-green-600 text-sm">+{fileItem.additions}</span>
                      )}
                      {fileItem.deletions > 0 && (
                        <span className="text-red-600 text-sm">-{fileItem.deletions}</span>
                      )}
                    </div>
                  </div>
                  {expandedFiles.has(fileItem.filePath) && fileItem.patch && !fileItem.isBinary && (
                    <div className="border-t p-4">
                      <pre className="text-sm">{fileItem.patch}</pre>
                    </div>
                  )}
                </Card>
              );
            })}
          </div>
        )}
      </div>
    );
  }

  // Single file view
  if (!file) {
    return (
      <div className="flex items-center justify-center p-8 text-muted-foreground">
        <p>No file provided</p>
      </div>
    );
  }

  // Large diff protection: Check line count
  const totalLines = countLinesInFile(file);
  const isOversized = totalLines > MAX_LINES_PER_FILE;

  if (isOversized) {
    return (
      <Card className="p-8">
        <div className="flex flex-col items-center gap-4 text-center">
          <AlertTriangle className="h-16 w-16 text-yellow-600" />
          <div>
            <h3 className="text-lg font-semibold mb-2">Diff Too Large to Display</h3>
            <p className="text-muted-foreground mb-4">
              This file has {totalLines.toLocaleString()} lines of changes, which exceeds the
              display limit of {MAX_LINES_PER_FILE.toLocaleString()} lines.
            </p>
            <p className="text-sm text-muted-foreground mb-6">
              For performance and security reasons, very large diffs cannot be displayed in the
              browser.
            </p>
          </div>
          {externalDiffUrl && (
            <a
              href={externalDiffUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
            >
              View on Provider
              <ExternalLink className="h-4 w-4" />
            </a>
          )}
        </div>
      </Card>
    );
  }

  // Detect language for syntax highlighting
  const languageDetection = detectLanguage(file.to || file.from || '');
  const enableHighlight = syntaxHighlighting && supportsHighlighting(languageDetection.language);

  // Build diff text from file chunks
  const buildDiffText = (): string => {
    if (!file.chunks || file.chunks.length === 0) {
      return '';
    }

    const header = `--- ${file.from || '/dev/null'}\n+++ ${file.to || '/dev/null'}`;
    const chunks = file.chunks.map((chunk) => chunk.content).join('\n');
    return `${header}\n${chunks}`;
  };

  const diffText = buildDiffText();

  if (!diffText) {
    return (
      <div className="flex items-center justify-center p-8 text-muted-foreground">
        <p>No diff available for this file</p>
      </div>
    );
  }

  // Configure DiffView options
  const diffViewProps: Record<string, unknown> = {
    diffText,
    diffViewFontSize: 13,
    diffViewHighlight: enableHighlight,
    diffViewMode: viewMode === 'split' ? 1 : 0, // 0 = unified, 1 = split
    diffViewWrap: wrapLines,
    diffViewAddWidget: true,
  };

  return (
    <div className={`diff-viewer-container ${className}`}>
      <DiffView {...diffViewProps} />
    </div>
  );
};

export default DiffViewer;
