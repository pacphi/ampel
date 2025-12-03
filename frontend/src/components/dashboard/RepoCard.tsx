import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import StatusBadge from './StatusBadge';
import { formatRelativeTime } from '@/lib/utils';
import type { RepositoryWithStatus } from '@/types';
import { Github, ExternalLink, GitPullRequest } from 'lucide-react';

interface RepoCardProps {
  repository: RepositoryWithStatus;
}

const providerIcons = {
  github: Github,
  gitlab: () => (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
      <path d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 0 1-.3-.94l1.22-3.78 2.44-7.51A.42.42 0 0 1 4.82 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.49h8.1l2.44-7.51A.42.42 0 0 1 18.6 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.51L23 13.45a.84.84 0 0 1-.35.94z" />
    </svg>
  ),
  bitbucket: () => (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
      <path d="M.778 1.211a.768.768 0 0 0-.768.892l3.263 19.81c.084.5.515.868 1.022.879H19.95a.772.772 0 0 0 .77-.646l3.27-20.03a.768.768 0 0 0-.768-.892zM14.52 15.53H9.522L8.17 8.466h7.561z" />
    </svg>
  ),
};

export default function RepoCard({ repository }: RepoCardProps) {
  const ProviderIcon = providerIcons[repository.provider] || Github;

  return (
    <Card className="hover:shadow-md transition-shadow">
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <StatusBadge status={repository.status} size="lg" />
            <ProviderIcon className="h-4 w-4 text-muted-foreground" />
          </div>
          <a
            href={repository.url}
            target="_blank"
            rel="noopener noreferrer"
            className="text-muted-foreground hover:text-foreground"
          >
            <ExternalLink className="h-4 w-4" />
          </a>
        </div>
        <CardTitle className="text-base truncate" title={repository.fullName}>
          {repository.name}
        </CardTitle>
        <p className="text-sm text-muted-foreground truncate">{repository.owner}</p>
      </CardHeader>
      <CardContent>
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-1 text-muted-foreground">
            <GitPullRequest className="h-4 w-4" />
            <span>{repository.openPrCount} PRs</span>
          </div>
          {repository.lastPolledAt && (
            <span className="text-muted-foreground text-xs">
              {formatRelativeTime(repository.lastPolledAt)}
            </span>
          )}
        </div>
        {repository.description && (
          <p className="mt-2 text-xs text-muted-foreground line-clamp-2">
            {repository.description}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
