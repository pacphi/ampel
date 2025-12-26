/**
 * DiffStatsBar Component
 *
 * Visual statistics bar for diff overview
 */

import React from 'react';
import { File, Plus, Minus, FileText } from 'lucide-react';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import type { DiffStats } from '../../types/diff';

export interface DiffStatsBarProps {
  stats: DiffStats;
}

/**
 * DiffStatsBar displays summary statistics for the entire diff
 */
export const DiffStatsBar: React.FC<DiffStatsBarProps> = ({ stats }) => {
  const totalChanges = stats.totalAdditions + stats.totalDeletions;
  const additionPercentage = totalChanges > 0 ? (stats.totalAdditions / totalChanges) * 100 : 0;

  return (
    <Card className="p-4">
      <div className="flex flex-col md:flex-row gap-4 items-start md:items-center justify-between">
        {/* File Counts */}
        <div className="flex flex-wrap gap-4 items-center">
          <div className="flex items-center gap-2">
            <File className="h-4 w-4 text-muted-foreground" />
            <span className="font-semibold">{stats.totalFiles}</span>
            <span className="text-sm text-muted-foreground">
              {stats.totalFiles === 1 ? 'file' : 'files'} changed
            </span>
          </div>

          {stats.addedFiles > 0 && (
            <Badge variant="default" className="bg-green-600">
              {stats.addedFiles} added
            </Badge>
          )}

          {stats.modifiedFiles > 0 && (
            <Badge variant="secondary">{stats.modifiedFiles} modified</Badge>
          )}

          {stats.deletedFiles > 0 && (
            <Badge variant="destructive">{stats.deletedFiles} deleted</Badge>
          )}

          {stats.renamedFiles > 0 && <Badge variant="outline">{stats.renamedFiles} renamed</Badge>}

          {stats.binaryFiles > 0 && (
            <div className="flex items-center gap-1 text-sm text-muted-foreground">
              <FileText className="h-4 w-4" />
              <span>{stats.binaryFiles} binary</span>
            </div>
          )}
        </div>

        {/* Change Counts */}
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1 text-green-600">
            <Plus className="h-4 w-4" />
            <span className="font-semibold">{stats.totalAdditions}</span>
          </div>

          <div className="flex items-center gap-1 text-red-600">
            <Minus className="h-4 w-4" />
            <span className="font-semibold">{stats.totalDeletions}</span>
          </div>
        </div>
      </div>

      {/* Visual Progress Bar */}
      {totalChanges > 0 && (
        <div className="mt-3 h-2 bg-red-100 rounded-full overflow-hidden">
          <div
            className="h-full bg-green-600 transition-all"
            style={{ width: `${additionPercentage}%` }}
          />
        </div>
      )}
    </Card>
  );
};

export default DiffStatsBar;
