import StatusBadge from './StatusBadge';
import RepositoryStatusIcons from './RepositoryStatusIcons';
import { formatRelativeTime } from '@/lib/utils';
import type { RepositoryWithStatus } from '@/types';
import { ExternalLink, GitPullRequest } from 'lucide-react';

interface ListViewProps {
  repositories: RepositoryWithStatus[];
}

export default function ListView({ repositories }: ListViewProps) {
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

  return (
    <div className="rounded-lg border">
      <table className="w-full">
        <thead>
          <tr className="border-b bg-muted/50">
            <th className="px-4 py-3 text-left text-sm font-medium">Status</th>
            <th className="px-4 py-3 text-left text-sm font-medium">Repository</th>
            <th className="px-4 py-3 text-left text-sm font-medium">Visibility</th>
            <th className="px-4 py-3 text-left text-sm font-medium">Provider</th>
            <th className="px-4 py-3 text-left text-sm font-medium">PRs</th>
            <th className="px-4 py-3 text-left text-sm font-medium">Last Updated</th>
            <th className="px-4 py-3 text-left text-sm font-medium"></th>
          </tr>
        </thead>
        <tbody>
          {repositories.map((repo) => (
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
