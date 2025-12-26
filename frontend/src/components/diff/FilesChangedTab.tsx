/**
 * FilesChangedTab Component
 *
 * Main tab component for displaying files changed in a pull request
 */

import React, { useState, useMemo } from 'react';
import { Search, ChevronsDown, ChevronsUp, Columns2, FileText, Filter, X } from 'lucide-react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Card } from '../ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { DiffFileItem } from './DiffFileItem';
import { DiffStatsBar } from './DiffStatsBar';
import { FileNavigation } from './FileNavigation';
import { usePullRequestDiff, useDiffViewerPreferences } from '../../hooks/usePullRequestDiff';
import type { DiffViewMode, DiffFile } from '../../types/diff';

export interface FilesChangedTabProps {
  pullRequestId?: number;
  prId?: string; // For testing compatibility
  files?: Array<{
    filePath: string;
    status: string;
    additions: number;
    deletions: number;
    changes: number;
    language?: string;
  }>;
}

/**
 * FilesChangedTab displays all changed files with controls for viewing and navigation
 */
export const FilesChangedTab: React.FC<FilesChangedTabProps> = ({
  pullRequestId,
  prId,
  files: filesProp,
}) => {
  const effectivePullRequestId = pullRequestId || (prId ? parseInt(prId, 10) : undefined);
  const { data, isLoading, error } = usePullRequestDiff(effectivePullRequestId);
  const { preferences, setPreferences } = useDiffViewerPreferences();

  // Use provided files prop for testing, otherwise use fetched data
  const filesData = useMemo(() => {
    return filesProp
      ? {
          files: filesProp.map(
            (f, idx) =>
              ({
                id: `file-${idx}`,
                to: f.filePath,
                from: f.filePath,
                new: f.status === 'added',
                deleted: f.status === 'deleted',
                renamed: f.status === 'renamed',
                isBinary: false,
                additions: f.additions,
                deletions: f.deletions,
                chunks: [],
                language: f.language,
                isExpanded: false,
                binary: false,
              }) as DiffFile
          ),
          stats: {
            totalFiles: filesProp.length,
            totalAdditions: filesProp.reduce((sum, f) => sum + f.additions, 0),
            totalDeletions: filesProp.reduce((sum, f) => sum + f.deletions, 0),
            binaryFiles: 0,
            modifiedFiles: filesProp.filter((f) => f.status === 'modified').length,
            addedFiles: filesProp.filter((f) => f.status === 'added').length,
            deletedFiles: filesProp.filter((f) => f.status === 'deleted').length,
            renamedFiles: filesProp.filter((f) => f.status === 'renamed').length,
          },
        }
      : data;
  }, [filesProp, data]);

  const [searchQuery, setSearchQuery] = useState('');
  const [fileFilter, setFileFilter] = useState<string>('all');
  const [expandedFiles, setExpandedFiles] = useState<Set<string>>(new Set());
  const [viewMode, setViewMode] = useState<DiffViewMode>(preferences.viewMode);

  // Filter files based on search and filter criteria
  const filteredFiles = useMemo(() => {
    if (!filesData?.files) return [];

    return filesData.files.filter((file) => {
      // Search filter
      const fileName = (file.to || file.from || '').toLowerCase();
      if (searchQuery && !fileName.includes(searchQuery.toLowerCase())) {
        return false;
      }

      // File type filter
      if (fileFilter === 'added' && !file.new) return false;
      if (fileFilter === 'deleted' && !file.deleted) return false;
      if (fileFilter === 'modified' && (file.new || file.deleted)) return false;
      if (fileFilter === 'binary' && !file.isBinary) return false;

      return true;
    });
  }, [filesData, searchQuery, fileFilter]);

  // Handle expand/collapse all
  const handleExpandAll = () => {
    const allFileIds = new Set(filteredFiles.map((f) => f.id));
    setExpandedFiles(allFileIds);
  };

  const handleCollapseAll = () => {
    setExpandedFiles(new Set());
  };

  // Handle individual file toggle
  const handleToggleFile = (fileId: string, expanded: boolean) => {
    setExpandedFiles((prev) => {
      const next = new Set(prev);
      if (expanded) {
        next.add(fileId);
      } else {
        next.delete(fileId);
      }
      return next;
    });
  };

  // Handle view mode change
  const handleViewModeChange = (mode: DiffViewMode) => {
    setViewMode(mode);
    setPreferences({ viewMode: mode });
  };

  // Clear search
  const handleClearSearch = () => {
    setSearchQuery('');
  };

  if (isLoading && !filesProp) {
    return (
      <div className="flex items-center justify-center p-12">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">Loading diff...</p>
        </div>
      </div>
    );
  }

  if (error && !filesProp) {
    return (
      <Card className="p-8">
        <div className="text-center text-destructive">
          <p className="font-semibold mb-2">Error loading diff</p>
          <p className="text-sm">{error.message}</p>
        </div>
      </Card>
    );
  }

  if (!filesData || filesData.files.length === 0) {
    return (
      <Card className="p-8">
        <div className="text-center text-muted-foreground">
          <FileText className="h-12 w-12 mx-auto mb-4 opacity-50" />
          <p>No files changed in this pull request</p>
        </div>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      {/* Stats Bar */}
      <DiffStatsBar stats={filesData.stats} />

      {/* Controls Bar */}
      <div className="flex flex-col md:flex-row gap-4 items-start md:items-center justify-between">
        <div className="flex flex-col sm:flex-row gap-2 flex-1 w-full md:w-auto">
          {/* Search */}
          <div className="relative flex-1 min-w-0">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              type="text"
              placeholder="Search files..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9 pr-9"
            />
            {searchQuery && (
              <Button
                variant="ghost"
                size="sm"
                className="absolute right-1 top-1/2 transform -translate-y-1/2 h-7 w-7 p-0"
                onClick={handleClearSearch}
                aria-label="Clear search"
              >
                <X className="h-4 w-4" />
              </Button>
            )}
          </div>

          {/* File Type Filter */}
          <Select value={fileFilter} onValueChange={setFileFilter}>
            <SelectTrigger className="w-full sm:w-[160px]" aria-label="Filter files by type">
              <Filter className="h-4 w-4 mr-2" aria-hidden="true" />
              <SelectValue placeholder="All files" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All files</SelectItem>
              <SelectItem value="added">Added</SelectItem>
              <SelectItem value="deleted">Deleted</SelectItem>
              <SelectItem value="modified">Modified</SelectItem>
              <SelectItem value="binary">Binary</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="flex gap-2 w-full md:w-auto">
          {/* View Mode Toggle */}
          <div
            className="flex gap-1 border rounded-md p-1"
            role="group"
            aria-label="Diff view mode"
          >
            <Button
              variant={viewMode === 'unified' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => handleViewModeChange('unified')}
              className="h-8"
              aria-label="Unified view mode"
              aria-pressed={viewMode === 'unified'}
            >
              <FileText className="h-4 w-4 mr-1" />
              Unified
            </Button>
            <Button
              variant={viewMode === 'split' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => handleViewModeChange('split')}
              className="h-8"
              aria-label="Split view mode"
              aria-pressed={viewMode === 'split'}
            >
              <Columns2 className="h-4 w-4 mr-1" />
              Split
            </Button>
          </div>

          {/* Expand/Collapse All */}
          <Button
            variant="outline"
            size="sm"
            onClick={handleExpandAll}
            disabled={expandedFiles.size === filteredFiles.length}
            className="h-8"
          >
            <ChevronsDown className="h-4 w-4 mr-1" />
            Expand All
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleCollapseAll}
            disabled={expandedFiles.size === 0}
            className="h-8"
          >
            <ChevronsUp className="h-4 w-4 mr-1" />
            Collapse All
          </Button>
        </div>
      </div>

      {/* File Navigation Sidebar */}
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-4">
        <div className="lg:col-span-1">
          <FileNavigation files={filteredFiles} expandedFiles={expandedFiles} />
        </div>

        {/* File List */}
        <div className="lg:col-span-3">
          {filteredFiles.length === 0 ? (
            <Card className="p-8">
              <div className="text-center text-muted-foreground">
                <FileText className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>No files match your filters</p>
              </div>
            </Card>
          ) : (
            <div className="space-y-0">
              {filteredFiles.map((file) => (
                <DiffFileItem
                  key={file.id}
                  file={file}
                  viewMode={viewMode}
                  syntaxHighlighting={preferences.syntaxHighlighting}
                  isExpanded={expandedFiles.has(file.id)}
                  onToggleExpand={handleToggleFile}
                />
              ))}
            </div>
          )}

          {/* Results Count */}
          {filteredFiles.length > 0 && (
            <div className="mt-4 text-center text-sm text-muted-foreground">
              Showing {filteredFiles.length} of {filesData.files.length} files
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default FilesChangedTab;
