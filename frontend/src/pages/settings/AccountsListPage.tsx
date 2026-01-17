import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { accountsApi } from '@/api/accounts';
import type { ProviderAccount } from '@/types/account';
import type { GitProvider } from '@/types';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useToast } from '@/components/ui/use-toast';
import { AccountCard } from '@/components/settings/AccountCard';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Plus } from 'lucide-react';

export function AccountsListPage() {
  const { t } = useTranslation(['accounts', 'common']);
  const navigate = useNavigate();
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [deleteConfirm, setDeleteConfirm] = useState<ProviderAccount | null>(null);

  const { data: accounts, isLoading } = useQuery({
    queryKey: ['accounts'],
    queryFn: () => accountsApi.listAccounts(),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => accountsApi.deleteAccount(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] });
      toast({
        title: t('accounts:toast.deleted'),
        description: t('accounts:toast.deletedDescription'),
      });
      setDeleteConfirm(null);
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('accounts:toast.deleteFailed'),
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const validateMutation = useMutation({
    mutationFn: (id: string) => accountsApi.validateAccount(id),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] });
      if (data.isValid) {
        toast({
          title: t('accounts:toast.tokenValid'),
          description: t('accounts:toast.tokenValidDescription'),
        });
      } else {
        toast({
          variant: 'destructive',
          title: t('accounts:toast.tokenInvalid'),
          description: data.errorMessage || t('accounts:toast.tokenInvalidDescription'),
        });
      }
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('accounts:toast.validationFailed'),
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const setDefaultMutation = useMutation({
    mutationFn: (id: string) => accountsApi.setDefaultAccount(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] });
      toast({
        title: t('accounts:toast.defaultUpdated'),
        description: t('accounts:toast.defaultUpdatedDescription'),
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('accounts:toast.defaultUpdateFailed'),
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const handleEdit = (account: ProviderAccount) => {
    navigate(`/settings/accounts/${account.id}/edit`);
  };

  const handleDelete = (account: ProviderAccount) => {
    setDeleteConfirm(account);
  };

  const handleValidate = (account: ProviderAccount) => {
    validateMutation.mutate(account.id);
  };

  const handleSetDefault = (account: ProviderAccount) => {
    setDefaultMutation.mutate(account.id);
  };

  const confirmDelete = () => {
    if (deleteConfirm) {
      deleteMutation.mutate(deleteConfirm.id);
    }
  };

  const groupedAccounts = accounts?.reduce(
    (acc, account) => {
      if (!acc[account.provider]) {
        acc[account.provider] = [];
      }
      acc[account.provider].push(account);
      return acc;
    },
    {} as Record<GitProvider, ProviderAccount[]>
  );

  const providers: Array<{ id: GitProvider; name: string }> = [
    { id: 'github', name: 'GitHub' },
    { id: 'gitlab', name: 'GitLab' },
    { id: 'bitbucket', name: 'Bitbucket' },
  ];

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
          <div>
            <CardTitle>{t('accounts:title')}</CardTitle>
            <CardDescription>{t('accounts:description')}</CardDescription>
          </div>
          <Button onClick={() => navigate('/settings/accounts/add')}>
            <Plus className="h-4 w-4 mr-2" />
            {t('accounts:addAccount')}
          </Button>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
            </div>
          ) : !accounts || accounts.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-muted-foreground mb-4">{t('accounts:noAccountsYet')}</p>
              <Button onClick={() => navigate('/settings/accounts/add')}>
                <Plus className="h-4 w-4 mr-2" />
                {t('accounts:addFirstAccount')}
              </Button>
            </div>
          ) : (
            <div className="space-y-6">
              {providers.map((provider) => {
                const providerAccounts = groupedAccounts?.[provider.id] || [];
                if (providerAccounts.length === 0) return null;

                return (
                  <div key={provider.id} className="space-y-3">
                    <h3 className="font-medium text-sm text-muted-foreground uppercase tracking-wide">
                      {provider.name} ({providerAccounts.length})
                    </h3>
                    <div className="space-y-3">
                      {providerAccounts.map((account) => (
                        <AccountCard
                          key={account.id}
                          account={account}
                          onEdit={handleEdit}
                          onDelete={handleDelete}
                          onValidate={handleValidate}
                          onSetDefault={handleSetDefault}
                          isLoading={
                            deleteMutation.isPending ||
                            validateMutation.isPending ||
                            setDefaultMutation.isPending
                          }
                        />
                      ))}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={deleteConfirm !== null} onOpenChange={() => setDeleteConfirm(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('accounts:delete.title')}</DialogTitle>
            <DialogDescription>
              {t('accounts:delete.description', { label: deleteConfirm?.accountLabel })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteConfirm(null)}>
              {t('common:actions.cancel')}
            </Button>
            <Button
              variant="destructive"
              onClick={confirmDelete}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending
                ? t('accounts:delete.deleting')
                : t('accounts:delete.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
