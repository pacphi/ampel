import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useMutation } from '@tanstack/react-query';
import { accountsApi } from '@/api/accounts';
import type { GitProvider } from '@/types';
import type { AddAccountRequest } from '@/types/account';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useToast } from '@/components/ui/use-toast';
import { TokenInstructions } from '@/components/settings/TokenInstructions';
import { ArrowLeft, Eye, EyeOff } from 'lucide-react';

export function AddAccountPage() {
  const { t } = useTranslation(['accounts', 'errors', 'common']);
  const navigate = useNavigate();
  const { toast } = useToast();
  const [showToken, setShowToken] = useState(false);

  const [formData, setFormData] = useState<AddAccountRequest>({
    provider: 'github',
    accountLabel: '',
    accessToken: '',
  });

  const [errors, setErrors] = useState<Partial<Record<keyof AddAccountRequest, string>>>({});

  const addAccountMutation = useMutation({
    mutationFn: (data: AddAccountRequest) => accountsApi.addAccount(data),
    onSuccess: () => {
      toast({
        title: t('accounts:toast.connected'),
        description: t('accounts:toast.connectedDescription', { provider: formData.provider }),
      });
      navigate('/settings/accounts');
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('accounts:toast.connectionFailed'),
        description: axiosError.response?.data?.error || t('errors:generic'),
      });
    },
  });

  const validateForm = (): boolean => {
    const newErrors: Partial<Record<keyof AddAccountRequest, string>> = {};

    if (!formData.accountLabel.trim()) {
      newErrors.accountLabel = t('accounts:add.form.accountLabelRequired');
    }

    if (!formData.accessToken.trim()) {
      newErrors.accessToken = t('accounts:add.form.tokenRequired');
    }

    if (formData.provider === 'bitbucket' && !formData.username?.trim()) {
      newErrors.username = t('accounts:add.form.usernameRequired');
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validateForm()) {
      addAccountMutation.mutate(formData);
    }
  };

  const handleProviderChange = (provider: GitProvider) => {
    setFormData({
      ...formData,
      provider,
      instanceUrl: undefined,
      username: undefined,
    });
    setErrors({});
  };

  return (
    <div className="space-y-4">
      <Button variant="ghost" onClick={() => navigate('/settings/accounts')} className="mb-2">
        <ArrowLeft className="h-4 w-4 mr-2" />
        {t('accounts:add.backToAccounts')}
      </Button>

      <Card>
        <CardHeader>
          <CardTitle>{t('accounts:add.title')}</CardTitle>
          <CardDescription>{t('accounts:add.subtitle')}</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Provider Selection */}
            <div className="space-y-2">
              <Label htmlFor="provider">{t('accounts:add.form.provider')}</Label>
              <Select
                value={formData.provider}
                onValueChange={(value) => handleProviderChange(value as GitProvider)}
              >
                <SelectTrigger id="provider">
                  <SelectValue placeholder={t('accounts:add.form.selectProvider')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="github">GitHub</SelectItem>
                  <SelectItem value="gitlab">GitLab</SelectItem>
                  <SelectItem value="bitbucket">Bitbucket</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* Instance URL for self-hosted */}
            {(formData.provider === 'gitlab' || formData.provider === 'bitbucket') && (
              <div className="space-y-2">
                <Label htmlFor="instanceUrl">{t('accounts:add.form.instanceUrl')}</Label>
                <Input
                  id="instanceUrl"
                  type="url"
                  placeholder={t('accounts:add.form.instanceUrlPlaceholder')}
                  value={formData.instanceUrl || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, instanceUrl: e.target.value || undefined })
                  }
                />
                <p className="text-xs text-muted-foreground">
                  {t('accounts:add.form.instanceUrlDescription', { provider: formData.provider })}
                </p>
              </div>
            )}

            {/* Account Label */}
            <div className="space-y-2">
              <Label htmlFor="accountLabel">
                {t('accounts:add.form.accountLabel')} <span className="text-destructive">*</span>
              </Label>
              <Input
                id="accountLabel"
                placeholder={t('accounts:add.form.accountLabelPlaceholder')}
                value={formData.accountLabel}
                onChange={(e) => setFormData({ ...formData, accountLabel: e.target.value })}
                className={errors.accountLabel ? 'border-destructive' : ''}
              />
              {errors.accountLabel && (
                <p className="text-xs text-destructive">{errors.accountLabel}</p>
              )}
              <p className="text-xs text-muted-foreground">
                {t('accounts:add.form.accountLabelDescription')}
              </p>
            </div>

            {/* Token Instructions */}
            <TokenInstructions provider={formData.provider} />

            {/* Access Token */}
            <div className="space-y-2">
              <Label htmlFor="accessToken">
                {formData.provider === 'bitbucket'
                  ? t('accounts:add.form.appPassword')
                  : t('accounts:add.form.token')}{' '}
                <span className="text-destructive">*</span>
              </Label>
              <div className="relative">
                <Input
                  id="accessToken"
                  type={showToken ? 'text' : 'password'}
                  placeholder={t(`accounts:add.form.tokenPlaceholder_${formData.provider}`)}
                  value={formData.accessToken}
                  onChange={(e) => setFormData({ ...formData, accessToken: e.target.value })}
                  className={errors.accessToken ? 'border-destructive pr-10' : 'pr-10'}
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  className="absolute right-0 top-0 h-full"
                  onClick={() => setShowToken(!showToken)}
                  title={
                    showToken ? t('accounts:add.form.hideToken') : t('accounts:add.form.showToken')
                  }
                >
                  {showToken ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </Button>
              </div>
              {errors.accessToken && (
                <p className="text-xs text-destructive">{errors.accessToken}</p>
              )}
            </div>

            {/* Username for Bitbucket */}
            {formData.provider === 'bitbucket' && (
              <div className="space-y-2">
                <Label htmlFor="username">
                  {t('accounts:add.form.bitbucketUsername')}{' '}
                  <span className="text-destructive">*</span>
                </Label>
                <Input
                  id="username"
                  placeholder={t('accounts:add.form.bitbucketUsernamePlaceholder')}
                  value={formData.username || ''}
                  onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                  className={errors.username ? 'border-destructive' : ''}
                />
                {errors.username && <p className="text-xs text-destructive">{errors.username}</p>}
                <p className="text-xs text-muted-foreground">
                  {t('accounts:add.form.bitbucketUsernameDescription')}
                </p>
              </div>
            )}

            {/* Submit Buttons */}
            <div className="flex gap-3">
              <Button
                type="button"
                variant="outline"
                onClick={() => navigate('/settings/accounts')}
                disabled={addAccountMutation.isPending}
              >
                {t('common:cancel')}
              </Button>
              <Button type="submit" disabled={addAccountMutation.isPending}>
                {addAccountMutation.isPending
                  ? t('accounts:add.submitting')
                  : t('accounts:add.submit')}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
