/**
 * DiffSearch Component
 *
 * Search functionality for finding text within diffs
 */

import React, { useState, useCallback, useEffect } from 'react';
import { Search, X, ChevronUp, ChevronDown } from 'lucide-react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Badge } from '../ui/badge';
import type { DiffFile, DiffSearchResult } from '../../types/diff';
import type { Change } from 'parse-diff';

export interface DiffSearchProps {
  files: DiffFile[];
  onResultSelect?: (result: DiffSearchResult) => void;
}

/**
 * DiffSearch provides search functionality across all diff files
 */
export const DiffSearch: React.FC<DiffSearchProps> = ({ files, onResultSelect }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [results, setResults] = useState<DiffSearchResult[]>([]);
  const [currentResultIndex, setCurrentResultIndex] = useState(0);
  const [isSearching, setIsSearching] = useState(false);

  // Search through all files
  const performSearch = useCallback(
    (query: string) => {
      if (!query.trim()) {
        setResults([]);
        setCurrentResultIndex(0);
        return;
      }

      setIsSearching(true);
      const searchResults: DiffSearchResult[] = [];
      const lowerQuery = query.toLowerCase();

      files.forEach((file) => {
        if (!file.chunks) return;

        file.chunks.forEach((chunk) => {
          chunk.changes?.forEach((change: Change, lineIndex) => {
            const content = change.content || '';
            const lowerContent = content.toLowerCase();
            let matchStart = 0;

            // Find all occurrences in this line
            while ((matchStart = lowerContent.indexOf(lowerQuery, matchStart)) !== -1) {
              const lineNumber =
                change.type === 'add' ? change.ln : change.type === 'del' ? change.ln : change.ln1;

              searchResults.push({
                fileId: file.id,
                fileName: file.to || file.from || 'unknown',
                lineNumber: lineNumber || lineIndex,
                lineContent: content,
                matchStart,
                matchEnd: matchStart + query.length,
              });
              matchStart += query.length;
            }
          });
        });
      });

      setResults(searchResults);
      setCurrentResultIndex(0);
      setIsSearching(false);

      // Auto-select first result
      if (searchResults.length > 0 && onResultSelect) {
        onResultSelect(searchResults[0]);
      }
    },
    [files, onResultSelect]
  );

  // Debounced search
  useEffect(() => {
    const timer = setTimeout(() => {
      performSearch(searchQuery);
    }, 300);

    return () => clearTimeout(timer);
  }, [searchQuery, performSearch]);

  // Navigate to previous result
  const handlePrevious = useCallback(() => {
    if (results.length === 0) return;

    const newIndex = currentResultIndex > 0 ? currentResultIndex - 1 : results.length - 1;
    setCurrentResultIndex(newIndex);

    if (onResultSelect) {
      onResultSelect(results[newIndex]);
    }
  }, [results, currentResultIndex, onResultSelect]);

  // Navigate to next result
  const handleNext = useCallback(() => {
    if (results.length === 0) return;

    const newIndex = currentResultIndex < results.length - 1 ? currentResultIndex + 1 : 0;
    setCurrentResultIndex(newIndex);

    if (onResultSelect) {
      onResultSelect(results[newIndex]);
    }
  }, [results, currentResultIndex, onResultSelect]);

  // Clear search
  const handleClear = () => {
    setSearchQuery('');
    setResults([]);
    setCurrentResultIndex(0);
  };

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl + F to focus search
      if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
        e.preventDefault();
        document.getElementById('diff-search-input')?.focus();
      }

      // Enter to go to next result
      if (e.key === 'Enter' && document.activeElement?.id === 'diff-search-input') {
        e.preventDefault();
        if (e.shiftKey) {
          handlePrevious();
        } else {
          handleNext();
        }
      }

      // Escape to clear search
      if (e.key === 'Escape') {
        handleClear();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [results, currentResultIndex, handleNext, handlePrevious]);

  return (
    <div className="flex items-center gap-2 p-2 border rounded-md bg-background">
      <div className="relative flex-1">
        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          id="diff-search-input"
          type="text"
          placeholder="Search in diff... (Cmd+F)"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="pl-9 pr-9 h-9"
        />
        {searchQuery && (
          <Button
            variant="ghost"
            size="sm"
            className="absolute right-1 top-1/2 transform -translate-y-1/2 h-6 w-6 p-0"
            onClick={handleClear}
          >
            <X className="h-4 w-4" />
          </Button>
        )}
      </div>

      {results.length > 0 && (
        <>
          <Badge variant="secondary" className="text-xs whitespace-nowrap">
            {currentResultIndex + 1} / {results.length}
          </Badge>

          <div className="flex gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={handlePrevious}
              disabled={results.length === 0}
              className="h-9 w-9 p-0"
              title="Previous result (Shift+Enter)"
            >
              <ChevronUp className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleNext}
              disabled={results.length === 0}
              className="h-9 w-9 p-0"
              title="Next result (Enter)"
            >
              <ChevronDown className="h-4 w-4" />
            </Button>
          </div>
        </>
      )}

      {isSearching && <div className="text-xs text-muted-foreground">Searching...</div>}
    </div>
  );
};

export default DiffSearch;
