import RepoCard from './RepoCard';
import type { RepositoryWithStatus } from '@/types';

interface GridViewProps {
  repositories: RepositoryWithStatus[];
}

export default function GridView({ repositories }: GridViewProps) {
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
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {repositories.map((repo) => (
        <RepoCard key={repo.id} repository={repo} />
      ))}
    </div>
  );
}
