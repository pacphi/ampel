import { useState, useMemo } from 'react';
import StatusBadge from './StatusBadge';
import RepositoryStatusIcons from './RepositoryStatusIcons';
import { formatRelativeTime } from '@/lib/utils';
import type { RepositoryWithStatus, AmpelStatus } from '@/types';
import { ExternalLink, GitPullRequest, ArrowUpDown, ArrowUp, ArrowDown } from 'lucide-react';

type SortColumn = 'status' | 'name' | 'visibility' | 'provider' | 'prs' | 'lastUpdated';
type SortDirection = 'asc' | 'desc';

interface ListViewProps {
  repositories: RepositoryWithStatus[];
}

// Status priority for sorting (green is best, none is worst)
const STATUS_PRIORITY: Record<AmpelStatus, number> = {
  green: 0,
  yellow: 1,
  red: 2,
  none: 3,
};

function SortIcon({
  column,
  sortColumn,
  sortDirection,
}: {
  column: SortColumn;
  sortColumn: SortColumn | null;
  sortDirection: SortDirection;
}) {
  if (sortColumn !== column) {
    return <ArrowUpDown className="h-4 w-4 ml-1 opacity-50" />;
  }
  return sortDirection === 'asc' ? (
    <ArrowUp className="h-4 w-4 ml-1" />
  ) : (
    <ArrowDown className="h-4 w-4 ml-1" />
  );
}

export default function ListView({ repositories }: ListViewProps) {
  const [sortColumn, setSortColumn] = useState<SortColumn | null>(null);
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');

  const handleSort = (column: SortColumn) => {
    if (sortColumn === column) {
      // Toggle direction if same column
      setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      // New column - set to ascending
      setSortColumn(column);
      setSortDirection('asc');
    }
  };

  const sortedRepositories = useMemo(() => {
    if (!sortColumn) return repositories;

    return [...repositories].sort((a, b) => {
      let comparison = 0;

      switch (sortColumn) {
        case 'status':
          comparison = STATUS_PRIORITY[a.status] - STATUS_PRIORITY[b.status];
          break;
        case 'name':
          comparison = a.name.toLowerCase().localeCompare(b.name.toLowerCase());
          break;
        case 'visibility': {
          // Sort: public first, then private, then archived
          const visA = a.isArchived ? 2 : a.isPrivate ? 1 : 0;
          const visB = b.isArchived ? 2 : b.isPrivate ? 1 : 0;
          comparison = visA - visB;
          break;
        }
        case 'provider':
          comparison = a.provider.localeCompare(b.provider);
          break;
        case 'prs':
          comparison = a.openPrCount - b.openPrCount;
          break;
        case 'lastUpdated': {
          const dateA = a.lastPolledAt ? new Date(a.lastPolledAt).getTime() : 0;
          const dateB = b.lastPolledAt ? new Date(b.lastPolledAt).getTime() : 0;
          comparison = dateA - dateB;
          break;
        }
      }

      return sortDirection === 'asc' ? comparison : -comparison;
    });
  }, [repositories, sortColumn, sortDirection]);

  if (repositories.length === 0) {
    return (
      <div className="text-center py-12">
        <p className="text-muted-foreground">No repositories found</p>
        <p className="text-sm text-muted-foreground mt-1">
          Add repositories from the Repositories page to get started
        </p>
      </div>
    );
  }

  const headerClass =
    'px-4 py-3 text-left text-sm font-medium cursor-pointer hover:bg-muted/80 select-none';

  return (
    <div className="rounded-lg border">
      <table className="w-full">
        <thead>
          <tr className="border-b bg-muted/50">
            <th className={headerClass} onClick={() => handleSort('status')}>
              <div className="flex items-center">
                Status
                <SortIcon column="status" sortColumn={sortColumn} sortDirection={sortDirection} />
              </div>
            </th>
            <th className={headerClass} onClick={() => handleSort('name')}>
              <div className="flex items-center">
                Repository
                <SortIcon column="name" sortColumn={sortColumn} sortDirection={sortDirection} />
              </div>
            </th>
            <th className={headerClass} onClick={() => handleSort('visibility')}>
              <div className="flex items-center">
                Visibility
                <SortIcon
                  column="visibility"
                  sortColumn={sortColumn}
                  sortDirection={sortDirection}
                />
              </div>
            </th>
            <th className={headerClass} onClick={() => handleSort('provider')}>
              <div className="flex items-center">
                Provider
                <SortIcon column="provider" sortColumn={sortColumn} sortDirection={sortDirection} />
              </div>
            </th>
            <th className={headerClass} onClick={() => handleSort('prs')}>
              <div className="flex items-center">
                PRs
                <SortIcon column="prs" sortColumn={sortColumn} sortDirection={sortDirection} />
              </div>
            </th>
            <th className={headerClass} onClick={() => handleSort('lastUpdated')}>
              <div className="flex items-center">
                Last Updated
                <SortIcon
                  column="lastUpdated"
                  sortColumn={sortColumn}
                  sortDirection={sortDirection}
                />
              </div>
            </th>
            <th className="px-4 py-3 text-left text-sm font-medium"></th>
          </tr>
        </thead>
        <tbody>
          {sortedRepositories.map((repo) => (
            <tr key={repo.id} className="border-b last:border-0 hover:bg-muted/50">
              <td className="px-4 py-3">
                <StatusBadge status={repo.status} showLabel />
              </td>
              <td className="px-4 py-3">
                <div>
                  <p className="font-medium">{repo.name}</p>
                  <p className="text-sm text-muted-foreground">{repo.owner}</p>
                </div>
              </td>
              <td className="px-4 py-3">
                <RepositoryStatusIcons
                  isPrivate={repo.isPrivate}
                  isArchived={repo.isArchived}
                  size="md"
                />
              </td>
              <td className="px-4 py-3 capitalize">{repo.provider}</td>
              <td className="px-4 py-3">
                <div className="flex items-center gap-1">
                  <GitPullRequest className="h-4 w-4 text-muted-foreground" />
                  <span>{repo.openPrCount}</span>
                </div>
              </td>
              <td className="px-4 py-3 text-sm text-muted-foreground">
                {repo.lastPolledAt ? formatRelativeTime(repo.lastPolledAt) : 'Never'}
              </td>
              <td className="px-4 py-3">
                <a
                  href={repo.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-muted-foreground hover:text-foreground"
                >
                  <ExternalLink className="h-4 w-4" />
                </a>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
