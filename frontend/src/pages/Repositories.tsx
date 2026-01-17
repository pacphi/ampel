import { useState } from 'react';
import {
  useRepositories,
  useAddRepository,
  useRemoveRepository,
  useDiscoverRepositories,
} from '@/hooks/useRepositories';
import { useQuery } from '@tanstack/react-query';
import { accountsApi } from '@/api/accounts';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import StatusBadge from '@/components/dashboard/StatusBadge';
import { useToast } from '@/components/ui/use-toast';
import type { GitProvider, DiscoveredRepository } from '@/types';
import { Plus, PlusCircle, Trash2, ExternalLink, Search, RefreshCw } from 'lucide-react';
import { GithubIcon, GitlabIcon, BitbucketIcon } from '@/components/icons/ProviderIcons';
import { useTranslation } from 'react-i18next';

export default function Repositories() {
  const { t } = useTranslation(['repositories', 'common', 'errors']);
  const { data: repositories, isLoading } = useRepositories();
  const { data: accounts } = useQuery({
    queryKey: ['accounts'],
    queryFn: () => accountsApi.listAccounts(),
  });
  const addRepository = useAddRepository();
  const removeRepository = useRemoveRepository();
  const { toast } = useToast();

  const [selectedProvider, setSelectedProvider] = useState<GitProvider | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [isAddingAll, setIsAddingAll] = useState(false);

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
        title: t('repositories:toast.added'),
        description: t('repositories:toast.addedDescription', { name: repo.fullName }),
      });
    } catch (error: unknown) {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('repositories:toast.addFailed'),
        description: axiosError.response?.data?.error || t('errors:general.unknownError'),
      });
    }
  };

  const handleRemoveRepo = async (id: string, name: string) => {
    try {
      await removeRepository.mutateAsync(id);
      toast({
        title: t('repositories:toast.removed'),
        description: t('repositories:toast.removedDescription', { name }),
      });
    } catch (error: unknown) {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('repositories:toast.removeFailed'),
        description: axiosError.response?.data?.error || t('errors:general.unknownError'),
      });
    }
  };

  const handleAddAll = async (repos: DiscoveredRepository[]) => {
    if (repos.length === 0) return;

    setIsAddingAll(true);
    let successCount = 0;
    let failCount = 0;

    for (const repo of repos) {
      try {
        await addRepository.mutateAsync({
          provider: repo.provider,
          owner: repo.owner,
          name: repo.name,
        });
        successCount++;
      } catch {
        failCount++;
      }
    }

    setIsAddingAll(false);

    if (successCount > 0) {
      toast({
        title: t('repositories:toast.bulkAdded'),
        description: t('repositories:toast.bulkAddedDescription', {
          count: successCount,
          failed:
            failCount > 0 ? t('repositories:toast.bulkAddedFailed', { count: failCount }) : '',
        }),
      });
    } else {
      toast({
        variant: 'destructive',
        title: t('repositories:toast.bulkAddFailed'),
        description: t('repositories:toast.bulkAddFailedDescription'),
      });
    }
  };

  const handleConnectProvider = () => {
    // Navigate to accounts page to add a new account
    window.location.href = '/settings/accounts/add';
  };

  const filteredRepos = repositories?.filter(
    (repo) =>
      repo.fullName.toLowerCase().includes(searchQuery.toLowerCase()) ||
      repo.owner.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const isProviderConnected = (provider: GitProvider) =>
    accounts?.some((a) => a.provider === provider && a.isActive);

  const filteredDiscovered = discoveredRepos?.filter(
    (repo) =>
      !repositories?.some(
        (r) => r.provider === repo.provider && r.providerId === repo.providerId
      ) &&
      (searchQuery === '' ||
        repo.fullName.toLowerCase().includes(searchQuery.toLowerCase()) ||
        repo.owner.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">{t('repositories:title')}</h1>
      </div>

      {/* Provider Connection */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">{t('repositories:providers.title')}</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            {(['github', 'gitlab', 'bitbucket'] as GitProvider[]).map((provider) => {
              const ProviderIcon =
                provider === 'github'
                  ? GithubIcon
                  : provider === 'gitlab'
                    ? GitlabIcon
                    : BitbucketIcon;
              return (
                <Button
                  key={provider}
                  variant={isProviderConnected(provider) ? 'default' : 'outline'}
                  onClick={() =>
                    isProviderConnected(provider)
                      ? setSelectedProvider(provider)
                      : handleConnectProvider()
                  }
                  className="capitalize"
                >
                  <ProviderIcon className="h-4 w-4 mr-2" />
                  {provider}
                  {isProviderConnected(provider) && ` ${t('repositories:providers.connected')}`}
                </Button>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Add from Provider */}
      {selectedProvider && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-lg capitalize">
              {t('repositories:addFrom.title', { provider: selectedProvider })}
            </CardTitle>
            <div className="flex items-center gap-2">
              {filteredDiscovered && filteredDiscovered.length > 0 && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleAddAll(filteredDiscovered)}
                  disabled={isAddingAll || addRepository.isPending}
                >
                  {isAddingAll ? (
                    <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                  ) : (
                    <PlusCircle className="h-4 w-4 mr-2" />
                  )}
                  {t('repositories:addFrom.addAll', { count: filteredDiscovered.length })}
                </Button>
              )}
              <Button variant="ghost" size="sm" onClick={() => setSelectedProvider(null)}>
                {t('repositories:addFrom.close')}
              </Button>
            </div>
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
                {t('repositories:addFrom.empty')}
              </p>
            )}
          </CardContent>
        </Card>
      )}

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          placeholder={t('repositories:search.placeholder')}
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
                <th className="px-4 py-3 text-left text-sm font-medium">
                  {t('repositories:table.status')}
                </th>
                <th className="px-4 py-3 text-left text-sm font-medium">
                  {t('repositories:table.repository')}
                </th>
                <th className="px-4 py-3 text-left text-sm font-medium">
                  {t('repositories:table.provider')}
                </th>
                <th className="px-4 py-3 text-left text-sm font-medium">
                  {t('repositories:table.prs')}
                </th>
                <th className="px-4 py-3 text-left text-sm font-medium">
                  {t('repositories:table.actions')}
                </th>
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
          <p className="text-muted-foreground">{t('repositories:empty.title')}</p>
          <p className="text-sm text-muted-foreground mt-1">
            {t('repositories:empty.description')}
          </p>
        </div>
      )}
    </div>
  );
}
