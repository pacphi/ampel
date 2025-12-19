import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
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
        title: 'Account connected',
        description: 'Your account has been added successfully.',
      });
      navigate('/settings/accounts');
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to add account',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const validateForm = (): boolean => {
    const newErrors: Partial<Record<keyof AddAccountRequest, string>> = {};

    if (!formData.accountLabel.trim()) {
      newErrors.accountLabel = 'Account label is required';
    }

    if (!formData.accessToken.trim()) {
      newErrors.accessToken = 'Access token is required';
    }

    if (formData.provider === 'bitbucket' && !formData.username?.trim()) {
      newErrors.username = 'Username is required for Bitbucket';
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
        Back to Accounts
      </Button>

      <Card>
        <CardHeader>
          <CardTitle>Add Provider Account</CardTitle>
          <CardDescription>
            Connect a new Git provider account using a Personal Access Token
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Provider Selection */}
            <div className="space-y-2">
              <Label htmlFor="provider">Provider</Label>
              <Select
                value={formData.provider}
                onValueChange={(value) => handleProviderChange(value as GitProvider)}
              >
                <SelectTrigger id="provider">
                  <SelectValue />
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
                <Label htmlFor="instanceUrl">Instance URL (optional, for self-hosted)</Label>
                <Input
                  id="instanceUrl"
                  type="url"
                  placeholder="https://gitlab.company.com"
                  value={formData.instanceUrl || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, instanceUrl: e.target.value || undefined })
                  }
                />
                <p className="text-xs text-muted-foreground">
                  Leave empty to use the cloud version ({formData.provider}.com)
                </p>
              </div>
            )}

            {/* Account Label */}
            <div className="space-y-2">
              <Label htmlFor="accountLabel">
                Account Label <span className="text-destructive">*</span>
              </Label>
              <Input
                id="accountLabel"
                placeholder="e.g., Work GitHub, Personal GitLab"
                value={formData.accountLabel}
                onChange={(e) => setFormData({ ...formData, accountLabel: e.target.value })}
                className={errors.accountLabel ? 'border-destructive' : ''}
              />
              {errors.accountLabel && (
                <p className="text-xs text-destructive">{errors.accountLabel}</p>
              )}
              <p className="text-xs text-muted-foreground">
                A friendly name to identify this account
              </p>
            </div>

            {/* Token Instructions */}
            <TokenInstructions provider={formData.provider} />

            {/* Access Token */}
            <div className="space-y-2">
              <Label htmlFor="accessToken">
                {formData.provider === 'bitbucket' ? 'App Password' : 'Personal Access Token'}{' '}
                <span className="text-destructive">*</span>
              </Label>
              <div className="relative">
                <Input
                  id="accessToken"
                  type={showToken ? 'text' : 'password'}
                  placeholder={
                    formData.provider === 'github'
                      ? 'ghp_...'
                      : formData.provider === 'gitlab'
                        ? 'glpat-...'
                        : 'App password'
                  }
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
                  title={showToken ? 'Hide token' : 'Show token'}
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
                  Bitbucket Username <span className="text-destructive">*</span>
                </Label>
                <Input
                  id="username"
                  placeholder="your-username"
                  value={formData.username || ''}
                  onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                  className={errors.username ? 'border-destructive' : ''}
                />
                {errors.username && <p className="text-xs text-destructive">{errors.username}</p>}
                <p className="text-xs text-muted-foreground">
                  Required for Basic Authentication with Bitbucket
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
                Cancel
              </Button>
              <Button type="submit" disabled={addAccountMutation.isPending}>
                {addAccountMutation.isPending ? 'Connecting...' : 'Connect Account'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
