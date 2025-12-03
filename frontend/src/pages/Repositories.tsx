import { useState } from 'react';
import { Link } from 'react-router-dom';
import {
  useRepositories,
  useAddRepository,
  useRemoveRepository,
  useDiscoverRepositories,
} from '@/hooks/useRepositories';
import { useQuery } from '@tanstack/react-query';
import { connectionsApi } from '@/api/connections';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import StatusBadge from '@/components/dashboard/StatusBadge';
import { useToast } from '@/components/ui/use-toast';
import type { GitProvider, DiscoveredRepository, ProviderConnection } from '@/types';
import { Plus, Trash2, ExternalLink, Search, Github, RefreshCw, Settings } from 'lucide-react';

export default function Repositories() {
  const { data: repositories, isLoading } = useRepositories();
  const { data: connections } = useQuery({
    queryKey: ['connections'],
    queryFn: () => connectionsApi.list(),
  });
  const addRepository = useAddRepository();
  const removeRepository = useRemoveRepository();
  const { toast } = useToast();

  const [selectedProvider, setSelectedProvider] = useState<GitProvider | null>(null);
  const [searchQuery, setSearchQuery] = useState('');

  const { data: discoveredRepos, isLoading: discovering } =
    useDiscoverRepositories(selectedProvider);

  const handleAddRepo = async (repo: DiscoveredRepository) => {
    try {
      await addRepository.mutateAsync({
        provider: repo.provider,
        owner: repo.owner,
        name: repo.name,
      });
      toast({
        title: 'Repository added',
        description: `${repo.fullName} has been added to your watchlist`,
      });
    } catch (error: unknown) {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to add repository',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    }
  };

  const handleRemoveRepo = async (id: string, name: string) => {
    try {
      await removeRepository.mutateAsync(id);
      toast({
        title: 'Repository removed',
        description: `${name} has been removed from your watchlist`,
      });
    } catch (error: unknown) {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to remove repository',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    }
  };

  const filteredRepos = repositories?.filter(
    (repo) =>
      repo.fullName.toLowerCase().includes(searchQuery.toLowerCase()) ||
      repo.owner.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const isProviderConnected = (provider: GitProvider) =>
    connections?.some((c: ProviderConnection) => c.provider === provider);

  const filteredDiscovered = discoveredRepos?.filter(
    (repo) =>
      !repositories?.some((r) => r.provider === repo.provider && r.providerId === repo.providerId)
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Repositories</h1>
      </div>

      {/* Provider Selection */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Select Provider</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4 flex-wrap">
            {(['github', 'gitlab', 'bitbucket'] as GitProvider[]).map((provider) => {
              const connected = isProviderConnected(provider);
              return (
                <Button
                  key={provider}
                  variant={selectedProvider === provider ? 'default' : 'outline'}
                  onClick={() => connected && setSelectedProvider(provider)}
                  disabled={!connected}
                  className="capitalize"
                >
                  <Github className="h-4 w-4 mr-2" />
                  {provider}
                  {connected && ' (Connected)'}
                </Button>
              );
            })}
            <Link to="/settings/connections">
              <Button variant="ghost" size="sm">
                <Settings className="h-4 w-4 mr-2" />
                Manage Connections
              </Button>
            </Link>
          </div>
          {!connections?.length && (
            <p className="text-sm text-muted-foreground mt-4">
              No connections found.{' '}
              <Link to="/settings/connections" className="text-primary hover:underline">
                Add a connection
              </Link>{' '}
              to start discovering repositories.
            </p>
          )}
        </CardContent>
      </Card>

      {/* Add from Provider */}
      {selectedProvider && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-lg capitalize">Add from {selectedProvider}</CardTitle>
            <Button variant="ghost" size="sm" onClick={() => setSelectedProvider(null)}>
              Close
            </Button>
          </CardHeader>
          <CardContent>
            {discovering ? (
              <div className="flex items-center justify-center py-8">
                <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : filteredDiscovered && filteredDiscovered.length > 0 ? (
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {filteredDiscovered.map((repo) => (
                  <div
                    key={repo.providerId}
                    className="flex items-center justify-between p-2 rounded-lg hover:bg-muted"
                  >
                    <div>
                      <p className="font-medium">{repo.fullName}</p>
                      {repo.description && (
                        <p className="text-sm text-muted-foreground truncate max-w-md">
                          {repo.description}
                        </p>
                      )}
                    </div>
                    <Button
                      size="sm"
                      onClick={() => handleAddRepo(repo)}
                      disabled={addRepository.isPending}
                    >
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-center py-8 text-muted-foreground">
                No additional repositories found
              </p>
            )}
          </CardContent>
        </Card>
      )}

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          placeholder="Search repositories..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="pl-10"
        />
      </div>

      {/* Repository List */}
      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        </div>
      ) : filteredRepos && filteredRepos.length > 0 ? (
        <div className="rounded-lg border">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="px-4 py-3 text-left text-sm font-medium">Status</th>
                <th className="px-4 py-3 text-left text-sm font-medium">Repository</th>
                <th className="px-4 py-3 text-left text-sm font-medium">Provider</th>
                <th className="px-4 py-3 text-left text-sm font-medium">PRs</th>
                <th className="px-4 py-3 text-left text-sm font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredRepos.map((repo) => (
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
                  <td className="px-4 py-3 capitalize">{repo.provider}</td>
                  <td className="px-4 py-3">{repo.openPrCount}</td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      <a
                        href={repo.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-muted-foreground hover:text-foreground"
                      >
                        <ExternalLink className="h-4 w-4" />
                      </a>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleRemoveRepo(repo.id, repo.fullName)}
                        disabled={removeRepository.isPending}
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="text-center py-12">
          <p className="text-muted-foreground">No repositories found</p>
          <p className="text-sm text-muted-foreground mt-1">
            Connect a provider and add repositories to get started
          </p>
        </div>
      )}
    </div>
  );
}
