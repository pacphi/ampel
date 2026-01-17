/**
 * DiffFileItem Component
 *
 * Individual file item with expand/collapse and diff display
 */

import React, { useState } from 'react';
import { ChevronDown, ChevronRight, File, Plus, Minus, FileText } from 'lucide-react';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import { DiffViewer } from './DiffViewer';
import type { DiffFile, DiffViewMode } from '../../types/diff';
import { getLanguageDisplayName } from '../../utils/languageDetection';

export interface DiffFileItemProps {
  file: DiffFile;
  viewMode: DiffViewMode;
  syntaxHighlighting: boolean;
  isExpanded?: boolean;
  onToggleExpand?: (fileId: string, expanded: boolean) => void;
}

/**
 * DiffFileItem displays a single file's diff with expand/collapse capability
 */
export const DiffFileItem: React.FC<DiffFileItemProps> = ({
  file,
  viewMode,
  syntaxHighlighting,
  isExpanded: controlledExpanded,
  onToggleExpand,
}) => {
  const [internalExpanded, setInternalExpanded] = useState(false);
  const isExpanded = controlledExpanded !== undefined ? controlledExpanded : internalExpanded;

  const handleToggle = () => {
    const newExpanded = !isExpanded;
    setInternalExpanded(newExpanded);
    onToggleExpand?.(file.id, newExpanded);
  };

  const getFileStatus = (): string => {
    if (file.new) return 'added';
    if (file.deleted) return 'deleted';
    if (file.renamed) return 'renamed';
    return 'modified';
  };

  const getStatusBadge = () => {
    const status = getFileStatus();
    const variants: Record<string, 'default' | 'destructive' | 'secondary' | 'outline'> = {
      added: 'default',
      deleted: 'destructive',
      modified: 'secondary',
      renamed: 'outline',
    };

    return (
      <Badge variant={variants[status]} className="ml-2">
        {status}
      </Badge>
    );
  };

  const fileName = file.to || file.from || 'unknown';
  const fileNameParts = fileName.split('/');
  const shortName = fileNameParts[fileNameParts.length - 1];
  const filePath = fileNameParts.slice(0, -1).join('/');

  return (
    <Card className="mb-4 overflow-hidden">
      <div
        className="flex items-center justify-between p-4 cursor-pointer hover:bg-accent transition-colors"
        onClick={handleToggle}
      >
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0"
            aria-label={isExpanded ? 'Collapse file diff' : 'Expand file diff'}
            aria-expanded={isExpanded}
          >
            {isExpanded ? (
              <ChevronDown className="h-4 w-4" />
            ) : (
              <ChevronRight className="h-4 w-4" />
            )}
          </Button>

          <div className="flex items-center gap-2 min-w-0 flex-1">
            {file.isBinary ? (
              <FileText className="h-4 w-4 text-muted-foreground flex-shrink-0" />
            ) : (
              <File className="h-4 w-4 text-muted-foreground flex-shrink-0" />
            )}

            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <span className="font-medium truncate">{shortName}</span>
                {getStatusBadge()}
                {file.language && file.language !== 'plaintext' && (
                  <Badge variant="outline" className="text-xs">
                    {getLanguageDisplayName(file.language)}
                  </Badge>
                )}
              </div>
              {filePath && <p className="text-sm text-muted-foreground truncate">{filePath}</p>}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-4 ml-4 flex-shrink-0">
          {!file.isBinary && (
            <>
              {file.additions > 0 && (
                <div className="flex items-center gap-1 text-green-600">
                  <Plus className="h-4 w-4" />
                  <span className="text-sm font-medium">{file.additions}</span>
                </div>
              )}
              {file.deletions > 0 && (
                <div className="flex items-center gap-1 text-red-600">
                  <Minus className="h-4 w-4" />
                  <span className="text-sm font-medium">{file.deletions}</span>
                </div>
              )}
            </>
          )}
          {file.isBinary && (
            <Badge variant="secondary" className="text-xs">
              Binary
            </Badge>
          )}
        </div>
      </div>

      {isExpanded && (
        <div className="border-t">
          {file.isBinary ? (
            <div className="p-8 text-center text-muted-foreground">
              <FileText className="h-12 w-12 mx-auto mb-2 opacity-50" />
              <p>Binary file - no diff available</p>
            </div>
          ) : (
            <DiffViewer
              file={file}
              viewMode={viewMode}
              syntaxHighlighting={syntaxHighlighting}
              className="diff-file-content"
            />
          )}
        </div>
      )}
    </Card>
  );
};

export default DiffFileItem;
