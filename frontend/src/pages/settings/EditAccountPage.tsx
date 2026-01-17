import { useState, useEffect, useRef } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { accountsApi } from '@/api/accounts';
import type { UpdateAccountRequest } from '@/types/account';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { useToast } from '@/components/ui/use-toast';
import { ArrowLeft, Eye, EyeOff } from 'lucide-react';

export function EditAccountPage() {
  const { t } = useTranslation(['accounts', 'errors']);
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [showToken, setShowToken] = useState(false);

  const { data: account, isLoading } = useQuery({
    queryKey: ['accounts', id],
    queryFn: () => accountsApi.getAccount(id!),
    enabled: !!id,
  });

  // Initialize form data from account data (no useEffect needed)
  const [formData, setFormData] = useState<UpdateAccountRequest & { newToken?: string }>({
    accountLabel: account?.accountLabel || '',
    isActive: account?.isActive ?? true,
  });

  // Update form when account loads (only once to avoid cascading renders)
  const accountLoaded = useRef(false);
  useEffect(() => {
    if (account && !accountLoaded.current) {
      accountLoaded.current = true;
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setFormData({
        accountLabel: account.accountLabel,
        isActive: account.isActive,
      });
    }
  }, [account]);

  const updateAccountMutation = useMutation({
    mutationFn: (data: UpdateAccountRequest) => accountsApi.updateAccount(id!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] });
      queryClient.invalidateQueries({ queryKey: ['accounts', id] });
      toast({
        title: t('accounts:toast.updated'),
        description: t('accounts:toast.updatedDescription'),
      });
      navigate('/settings/accounts');
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('accounts:toast.updateFailed'),
        description: axiosError.response?.data?.error || t('errors:generic'),
      });
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const updateData: UpdateAccountRequest = {
      accountLabel: formData.accountLabel,
      isActive: formData.isActive,
    };

    if (formData.newToken && formData.newToken.trim()) {
      updateData.accessToken = formData.newToken;
    }

    updateAccountMutation.mutate(updateData);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (!account) {
    return (
      <Card>
        <CardContent className="py-8 text-center text-muted-foreground">
          {t('accounts:edit.notFound')}
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <Button variant="ghost" onClick={() => navigate('/settings/accounts')} className="mb-2">
        <ArrowLeft className="h-4 w-4 mr-2" />
        {t('accounts:edit.backToAccounts')}
      </Button>

      <Card>
        <CardHeader>
          <CardTitle>{t('accounts:edit.title')}</CardTitle>
          <CardDescription>{t('accounts:edit.subtitle')}</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Account Label */}
            <div className="space-y-2">
              <Label htmlFor="accountLabel">
                {t('accounts:edit.form.accountLabel')} <span className="text-destructive">*</span>
              </Label>
              <Input
                id="accountLabel"
                placeholder={t('accounts:edit.form.accountLabelPlaceholder')}
                value={formData.accountLabel}
                onChange={(e) => setFormData({ ...formData, accountLabel: e.target.value })}
                required
              />
              <p className="text-xs text-muted-foreground">
                {t('accounts:edit.form.accountLabelDescription')}
              </p>
            </div>

            {/* Update Token */}
            <div className="space-y-2">
              <Label htmlFor="newToken">{t('accounts:edit.form.token')}</Label>
              <div className="relative">
                <Input
                  id="newToken"
                  type={showToken ? 'text' : 'password'}
                  placeholder={t('accounts:edit.form.tokenPlaceholder')}
                  value={formData.newToken || ''}
                  onChange={(e) => setFormData({ ...formData, newToken: e.target.value })}
                  className="pr-10"
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  className="absolute right-0 top-0 h-full"
                  onClick={() => setShowToken(!showToken)}
                  title={
                    showToken
                      ? t('accounts:edit.form.hideToken')
                      : t('accounts:edit.form.showToken')
                  }
                >
                  {showToken ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                {t('accounts:edit.form.tokenDescription')}
              </p>
            </div>

            {/* Active Status */}
            <div className="flex items-center justify-between rounded-lg border p-4">
              <div className="space-y-0.5">
                <Label htmlFor="isActive" className="text-base">
                  {t('accounts:edit.form.active')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t('accounts:edit.form.activeDescription')}
                </p>
              </div>
              <Switch
                id="isActive"
                checked={formData.isActive}
                onCheckedChange={(checked) => setFormData({ ...formData, isActive: checked })}
              />
            </div>

            {/* Account Info */}
            <div className="rounded-lg bg-muted p-4 space-y-2">
              <h4 className="font-medium text-sm">{t('accounts:edit.accountInfo')}</h4>
              <div className="text-sm space-y-1">
                <p>
                  <span className="text-muted-foreground">{t('accounts:edit.info.username')}:</span>{' '}
                  @{account.providerUsername}
                </p>
                {account.providerEmail && (
                  <p>
                    <span className="text-muted-foreground">{t('accounts:edit.info.email')}:</span>{' '}
                    {account.providerEmail}
                  </p>
                )}
                {account.instanceUrl && (
                  <p>
                    <span className="text-muted-foreground">
                      {t('accounts:edit.info.instance')}:
                    </span>{' '}
                    {account.instanceUrl}
                  </p>
                )}
                <p>
                  <span className="text-muted-foreground">
                    {t('accounts:edit.info.repositories')}:
                  </span>{' '}
                  {account.repositoryCount}
                </p>
                <p>
                  <span className="text-muted-foreground">{t('accounts:edit.info.status')}:</span>{' '}
                  {account.validationStatus}
                </p>
              </div>
            </div>

            {/* Submit Buttons */}
            <div className="flex gap-3">
              <Button
                type="button"
                variant="outline"
                onClick={() => navigate('/settings/accounts')}
                disabled={updateAccountMutation.isPending}
              >
                {t('accounts:edit.cancel')}
              </Button>
              <Button type="submit" disabled={updateAccountMutation.isPending}>
                {updateAccountMutation.isPending
                  ? t('accounts:edit.submitting')
                  : t('accounts:edit.submit')}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
