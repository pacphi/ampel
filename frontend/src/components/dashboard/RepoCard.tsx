import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import StatusBadge from './StatusBadge';
import { formatRelativeTime } from '@/lib/utils';
import type { RepositoryWithStatus } from '@/types';
import { ExternalLink, GitPullRequest } from 'lucide-react';
import { GithubIcon, GitlabIcon, BitbucketIcon } from '@/components/icons/ProviderIcons';

interface RepoCardProps {
  repository: RepositoryWithStatus;
}

const providerIcons = {
  github: GithubIcon,
  gitlab: GitlabIcon,
  bitbucket: BitbucketIcon,
};

export default function RepoCard({ repository }: RepoCardProps) {
  const ProviderIcon = providerIcons[repository.provider] || GithubIcon;

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
