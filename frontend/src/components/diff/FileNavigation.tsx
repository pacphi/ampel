/**
 * FileNavigation Component
 *
 * Sidebar navigation for jumping to specific files
 */

import React from 'react';
import { File, Plus, Minus, FileText, Check } from 'lucide-react';
import { Card } from '../ui/card';
import { ScrollArea } from '../ui/scroll-area';
import type { DiffFile } from '../../types/diff';

export interface FileNavigationProps {
  files: DiffFile[];
  expandedFiles: Set<string>;
}

/**
 * FileNavigation provides a sidebar for jumping to specific files
 */
export const FileNavigation: React.FC<FileNavigationProps> = ({ files, expandedFiles }) => {
  const scrollToFile = (fileId: string) => {
    const element = document.getElementById(`file-${fileId}`);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  };

  const getFileIcon = (file: DiffFile) => {
    if (file.isBinary) {
      return <FileText className="h-4 w-4 text-muted-foreground" />;
    }
    return <File className="h-4 w-4 text-muted-foreground" />;
  };

  const getStatusColor = (file: DiffFile): string => {
    if (file.new) return 'text-green-600';
    if (file.deleted) return 'text-red-600';
    if (file.renamed) return 'text-blue-600';
    return 'text-muted-foreground';
  };

  return (
    <Card className="p-4">
      <div className="mb-3">
        <h3 className="font-semibold text-sm">Files Changed</h3>
        <p className="text-xs text-muted-foreground mt-1">
          {files.length} {files.length === 1 ? 'file' : 'files'}
        </p>
      </div>

      <ScrollArea className="h-[600px]">
        <div className="space-y-1">
          {files.map((file) => {
            const fileName = (file.to || file.from || '').split('/').pop() || 'unknown';
            const isExpanded = expandedFiles.has(file.id);

            return (
              <button
                key={file.id}
                onClick={() => scrollToFile(file.id)}
                className="w-full text-left p-2 rounded-md hover:bg-accent transition-colors group"
                aria-label={`Jump to ${fileName}`}
                type="button"
              >
                <div className="flex items-start gap-2">
                  <div className="mt-0.5">{getFileIcon(file)}</div>

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1 mb-1">
                      <span className={`text-sm truncate ${getStatusColor(file)}`}>{fileName}</span>
                      {isExpanded && (
                        <Check className="h-3 w-3 text-primary flex-shrink-0" aria-hidden="true" />
                      )}
                    </div>

                    {!file.isBinary && (
                      <div className="flex items-center gap-2 text-xs" aria-hidden="true">
                        {file.additions > 0 && (
                          <span className="flex items-center gap-0.5 text-green-600">
                            <Plus className="h-3 w-3" />
                            {file.additions}
                          </span>
                        )}
                        {file.deletions > 0 && (
                          <span className="flex items-center gap-0.5 text-red-600">
                            <Minus className="h-3 w-3" />
                            {file.deletions}
                          </span>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      </ScrollArea>
    </Card>
  );
};

export default FileNavigation;
