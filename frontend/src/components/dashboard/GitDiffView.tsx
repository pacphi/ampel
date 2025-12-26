import { useQuery } from '@tanstack/react-query';
import { useState, useMemo } from 'react';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';

interface DiffFile {
  filename: string;
  status: 'added' | 'modified' | 'removed' | 'renamed';
  additions: number;
  deletions: number;
  changes: number;
  patch?: string;
  previous_filename?: string;
}

interface DiffResponse {
  files: DiffFile[];
  total_additions: number;
  total_deletions: number;
  total_files: number;
}

interface GitDiffViewProps {
  repoId: string;
  prId: string;
}

// Virtual scrolling configuration
const ITEM_HEIGHT = 400; // Approximate height per file diff
const VISIBLE_ITEMS = 5; // Number of items to render at once

export default function GitDiffView({ repoId, prId }: GitDiffViewProps) {
  const [scrollOffset, setScrollOffset] = useState(0);
  const [expandedFiles, setExpandedFiles] = useState<Set<string>>(new Set());

  // Fetch diff data with optimized cache settings
  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ['pr-diff', repoId, prId],
    queryFn: async () => {
      const response = await apiClient.get<ApiResponse<DiffResponse>>(
        `/repositories/${repoId}/pull-requests/${prId}/diff`
      );
      return response.data.data!;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes for open PRs
    gcTime: 60 * 60 * 1000, // 1 hour for closed PRs
    refetchOnWindowFocus: false,
  });

  // Virtual scrolling: only render visible files
  const visibleFiles = useMemo(() => {
    if (!data?.files) return [];

    const startIndex = Math.floor(scrollOffset / ITEM_HEIGHT);
    const endIndex = Math.min(startIndex + VISIBLE_ITEMS + 1, data.files.length);

    return data.files.slice(startIndex, endIndex).map((file, index) => ({
      file,
      virtualIndex: startIndex + index,
    }));
  }, [data, scrollOffset]);

  const toggleFileExpansion = (filename: string) => {
    const newExpanded = new Set(expandedFiles);
    if (newExpanded.has(filename)) {
      newExpanded.delete(filename);
    } else {
      newExpanded.add(filename);
    }
    setExpandedFiles(newExpanded);
  };

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    setScrollOffset(e.currentTarget.scrollTop);
  };

  const handleRefresh = async () => {
    await apiClient.post(`/repositories/${repoId}/pull-requests/${prId}/refresh`);
    refetch();
  };

  if (isLoading) {
    return (
      <div className="space-y-4 p-4">
        <Skeleton className="h-8 w-full" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (error) {
    return <div className="p-4 text-red-600">Error loading diff: {error.message}</div>;
  }

  if (!data) {
    return <div className="p-4">No diff data available</div>;
  }

  const totalHeight = (data.files?.length || 0) * ITEM_HEIGHT;

  return (
    <div className="space-y-4">
      {/* Summary header */}
      <div className="flex items-center justify-between border-b pb-4">
        <div className="flex gap-4 text-sm">
          <span className="text-green-600">+{data.total_additions}</span>
          <span className="text-red-600">-{data.total_deletions}</span>
          <span className="text-gray-600">{data.total_files} files</span>
        </div>
        <Button onClick={handleRefresh} variant="outline" size="sm">
          Refresh
        </Button>
      </div>

      {/* Virtual scrolling container */}
      <div className="relative overflow-y-auto" style={{ height: '600px' }} onScroll={handleScroll}>
        <div style={{ height: `${totalHeight}px`, position: 'relative' }}>
          {visibleFiles.map(({ file, virtualIndex }) => {
            const isExpanded = expandedFiles.has(file.filename);

            return (
              <div
                key={file.filename}
                className="absolute left-0 right-0 border-b p-4"
                style={{
                  top: `${virtualIndex * ITEM_HEIGHT}px`,
                  height: `${ITEM_HEIGHT}px`,
                }}
              >
                <div
                  className="cursor-pointer hover:bg-gray-50"
                  onClick={() => toggleFileExpansion(file.filename)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <FileStatusBadge status={file.status} />
                      <span className="font-mono text-sm">{file.filename}</span>
                      {file.previous_filename && (
                        <span className="text-xs text-gray-500">← {file.previous_filename}</span>
                      )}
                    </div>
                    <div className="flex gap-2 text-xs">
                      <span className="text-green-600">+{file.additions}</span>
                      <span className="text-red-600">-{file.deletions}</span>
                    </div>
                  </div>
                </div>

                {/* Lazy render patch only when expanded */}
                {isExpanded && file.patch && (
                  <pre className="mt-2 overflow-x-auto rounded bg-gray-100 p-2 text-xs">
                    {file.patch}
                  </pre>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function FileStatusBadge({ status }: { status: string }) {
  const colors = {
    added: 'bg-green-100 text-green-800',
    modified: 'bg-yellow-100 text-yellow-800',
    removed: 'bg-red-100 text-red-800',
    renamed: 'bg-blue-100 text-blue-800',
  };

  return (
    <span
      className={`rounded px-2 py-1 text-xs font-medium ${colors[status as keyof typeof colors] || 'bg-gray-100 text-gray-800'}`}
    >
      {status.charAt(0).toUpperCase()}
    </span>
  );
}
