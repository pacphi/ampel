import { useState } from 'react';
import { Routes, Route, Link, useLocation } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { authApi } from '@/api/auth';
import { prFiltersApi, type PrFilter } from '@/api/prFilters';
import { useAuth } from '@/hooks/useAuth';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { useToast } from '@/components/ui/use-toast';
import { cn } from '@/lib/utils';
import {
  User,
  Eye,
  EyeOff,
  Pencil,
  X,
  Check,
  Filter,
  Plus,
  RotateCcw,
  Bell,
  Settings2,
  Boxes,
} from 'lucide-react';
import { NotificationsSettings } from '@/components/settings/NotificationsSettings';
import { BehaviorSettings } from '@/components/settings/BehaviorSettings';
import { RepositoryFilterSettings } from '@/components/settings/RepositoryFilterSettings';
import { AccountsListPage } from './settings/AccountsListPage';
import { AddAccountPage } from './settings/AddAccountPage';
import { EditAccountPage } from './settings/EditAccountPage';

function SettingsNav() {
  const { t } = useTranslation(['settings']);
  const location = useLocation();

  const links = [
    { href: '/settings', label: t('settings:tabs.profile'), icon: User },
    { href: '/settings/accounts', label: t('settings:tabs.accounts'), icon: User },
    { href: '/settings/filters', label: t('settings:tabs.prFilters'), icon: Filter },
    {
      href: '/settings/repository-filters',
      label: t('settings:tabs.repositoryFilters'),
      icon: Boxes,
    },
    { href: '/settings/notifications', label: t('settings:tabs.notifications'), icon: Bell },
    { href: '/settings/behavior', label: t('settings:tabs.behavior'), icon: Settings2 },
  ];

  return (
    <nav className="space-y-1">
      {links.map((link) => {
        const isActive =
          location.pathname === link.href ||
          (link.href !== '/settings' && location.pathname.startsWith(link.href + '/'));
        return (
          <Link
            key={link.href}
            to={link.href}
            className={cn(
              'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
              isActive
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
            )}
          >
            <link.icon className="h-4 w-4" />
            {link.label}
          </Link>
        );
      })}
    </nav>
  );
}

function maskEmail(email: string): string {
  const [local, domain] = email.split('@');
  if (!domain) return email;
  const maskedLocal =
    local.length <= 2
      ? '*'.repeat(local.length)
      : local[0] + '*'.repeat(local.length - 2) + local[local.length - 1];
  return `${maskedLocal}@${domain}`;
}

function ProfileSettings() {
  const { t } = useTranslation(['settings', 'common', 'errors']);
  const { user, refreshUser } = useAuth();
  const { toast } = useToast();
  const [isEditing, setIsEditing] = useState(false);
  const [showEmail, setShowEmail] = useState(false);
  const [email, setEmail] = useState(user?.email || '');
  const [displayName, setDisplayName] = useState(user?.displayName || '');

  const updateMutation = useMutation({
    mutationFn: (data: { email?: string; displayName?: string }) => authApi.updateProfile(data),
    onSuccess: async () => {
      await refreshUser();
      setIsEditing(false);
      toast({
        title: t('settings:profileUpdated'),
        description: t('settings:profileUpdateSuccess'),
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('settings:profileUpdateFailed'),
        description: axiosError.response?.data?.error || t('common:app.error'),
      });
    },
  });

  const handleEdit = () => {
    setEmail(user?.email || '');
    setDisplayName(user?.displayName || '');
    setIsEditing(true);
  };

  const handleCancel = () => {
    setIsEditing(false);
    setEmail(user?.email || '');
    setDisplayName(user?.displayName || '');
  };

  const handleSave = () => {
    const updates: { email?: string; displayName?: string } = {};

    if (email !== user?.email) {
      updates.email = email;
    }

    if (displayName !== (user?.displayName || '')) {
      updates.displayName = displayName || undefined;
    }

    if (Object.keys(updates).length === 0) {
      setIsEditing(false);
      return;
    }

    updateMutation.mutate(updates);
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <div>
          <CardTitle>{t('settings:account.profile')}</CardTitle>
          <CardDescription>{t('settings:account.yourAccountInfo')}</CardDescription>
        </div>
        {!isEditing ? (
          <Button variant="outline" size="sm" onClick={handleEdit}>
            <Pencil className="h-4 w-4 mr-2" />
            {t('common:app.edit')}
          </Button>
        ) : (
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleCancel}
              disabled={updateMutation.isPending}
            >
              <X className="h-4 w-4 mr-2" />
              {t('common:app.cancel')}
            </Button>
            <Button size="sm" onClick={handleSave} disabled={updateMutation.isPending}>
              <Check className="h-4 w-4 mr-2" />
              {updateMutation.isPending ? t('common:actions.saving') : t('common:app.save')}
            </Button>
          </div>
        )}
      </CardHeader>
      <CardContent className="space-y-4">
        <div>
          <label className="text-sm font-medium">{t('settings:account.email')}</label>
          {isEditing ? (
            <Input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder={t('settings:account.emailPlaceholder')}
              className="mt-1"
            />
          ) : (
            <div className="flex items-center gap-2 mt-1">
              <p className="text-muted-foreground">
                {showEmail ? user?.email : maskEmail(user?.email || '')}
              </p>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={() => setShowEmail(!showEmail)}
                title={showEmail ? t('common:actions.hideEmail') : t('common:actions.showEmail')}
              >
                {showEmail ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              </Button>
            </div>
          )}
        </div>
        <div>
          <label className="text-sm font-medium">{t('settings:account.displayName')}</label>
          {isEditing ? (
            <Input
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder={t('settings:account.displayNamePlaceholder')}
              className="mt-1"
            />
          ) : (
            <p className="text-muted-foreground mt-1">
              {user?.displayName || t('settings:account.notSet')}
            </p>
          )}
        </div>
        <div>
          <label className="text-sm font-medium">{t('settings:account.memberSince')}</label>
          <p className="text-muted-foreground mt-1">
            {user?.createdAt
              ? new Date(user.createdAt).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric',
                  timeZone: 'UTC',
                })
              : t('settings:account.unknown')}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

function FiltersSettings() {
  const { t } = useTranslation(['settings', 'common']);
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [newActor, setNewActor] = useState('');
  const [newLabel, setNewLabel] = useState('');

  const { data: filters, isLoading } = useQuery({
    queryKey: ['pr-filters'],
    queryFn: () => prFiltersApi.get(),
  });

  const updateMutation = useMutation({
    mutationFn: (data: Partial<PrFilter>) => prFiltersApi.update(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pr-filters'] });
      toast({
        title: t('settings:filtersUpdated'),
        description: t('settings:filtersUpdateSuccess'),
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('settings:filtersUpdateFailed'),
        description: axiosError.response?.data?.error || t('common:app.error'),
      });
    },
  });

  const resetMutation = useMutation({
    mutationFn: () => prFiltersApi.reset(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pr-filters'] });
      toast({
        title: t('settings:filtersReset'),
        description: t('settings:filtersResetSuccess'),
      });
    },
  });

  const addActor = () => {
    if (!newActor.trim() || !filters) return;
    const updated = [...filters.allowedActors, newActor.trim()];
    updateMutation.mutate({ allowedActors: updated });
    setNewActor('');
  };

  const removeActor = (actor: string) => {
    if (!filters) return;
    const updated = filters.allowedActors.filter((a) => a !== actor);
    updateMutation.mutate({ allowedActors: updated });
  };

  const addLabel = () => {
    if (!newLabel.trim() || !filters) return;
    const updated = [...filters.skipLabels, newLabel.trim()];
    updateMutation.mutate({ skipLabels: updated });
    setNewLabel('');
  };

  const removeLabel = (label: string) => {
    if (!filters) return;
    const updated = filters.skipLabels.filter((l) => l !== label);
    updateMutation.mutate({ skipLabels: updated });
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-8">
          <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
          <div>
            <CardTitle>{t('settings:prFilters.title')}</CardTitle>
            <CardDescription>{t('settings:prFilters.description')}</CardDescription>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => resetMutation.mutate()}
            disabled={resetMutation.isPending}
          >
            <RotateCcw className="h-4 w-4 mr-2" />
            {t('common:actions.reset')}
          </Button>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Allowed Actors */}
          <div>
            <label className="text-sm font-medium">{t('settings:prFilters.allowedActors')}</label>
            <p className="text-sm text-muted-foreground mb-3">
              {t('settings:prFilters.allowedActorsDescription')}
            </p>
            <div className="flex flex-wrap gap-2 mb-3">
              {filters?.allowedActors.map((actor) => (
                <span
                  key={actor}
                  className="inline-flex items-center gap-1 px-3 py-1 rounded-full bg-primary/10 text-sm"
                >
                  {actor}
                  <button
                    onClick={() => removeActor(actor)}
                    className="hover:text-destructive ml-1"
                    disabled={updateMutation.isPending}
                  >
                    <X className="h-3 w-3" />
                  </button>
                </span>
              ))}
            </div>
            <div className="flex gap-2">
              <Input
                placeholder={t('settings:prFilters.allowedActorsPlaceholder')}
                value={newActor}
                onChange={(e) => setNewActor(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && addActor()}
                className="max-w-xs"
              />
              <Button
                variant="outline"
                size="sm"
                onClick={addActor}
                disabled={!newActor.trim() || updateMutation.isPending}
              >
                <Plus className="h-4 w-4 mr-1" />
                {t('common:app.add')}
              </Button>
            </div>
          </div>

          {/* Skip Labels */}
          <div>
            <label className="text-sm font-medium">{t('settings:prFilters.skipLabels')}</label>
            <p className="text-sm text-muted-foreground mb-3">
              {t('settings:prFilters.skipLabelsDescription')}
            </p>
            <div className="flex flex-wrap gap-2 mb-3">
              {filters?.skipLabels.map((label) => (
                <span
                  key={label}
                  className="inline-flex items-center gap-1 px-3 py-1 rounded-full bg-destructive/10 text-sm"
                >
                  {label}
                  <button
                    onClick={() => removeLabel(label)}
                    className="hover:text-destructive ml-1"
                    disabled={updateMutation.isPending}
                  >
                    <X className="h-3 w-3" />
                  </button>
                </span>
              ))}
            </div>
            <div className="flex gap-2">
              <Input
                placeholder={t('settings:prFilters.skipLabelsPlaceholder')}
                value={newLabel}
                onChange={(e) => setNewLabel(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && addLabel()}
                className="max-w-xs"
              />
              <Button
                variant="outline"
                size="sm"
                onClick={addLabel}
                disabled={!newLabel.trim() || updateMutation.isPending}
              >
                <Plus className="h-4 w-4 mr-1" />
                {t('common:app.add')}
              </Button>
            </div>
          </div>

          {/* Max Age */}
          <div>
            <label className="text-sm font-medium">{t('settings:prFilters.maxAgeDays')}</label>
            <p className="text-sm text-muted-foreground mb-3">
              {t('settings:prFilters.maxAgeDaysDescription')}
            </p>
            <Input
              type="number"
              placeholder={t('settings:prFilters.maxAgeDaysPlaceholder')}
              value={filters?.maxAgeDays ?? ''}
              onChange={(e) => {
                const value = e.target.value ? parseInt(e.target.value, 10) : null;
                updateMutation.mutate({ maxAgeDays: value });
              }}
              className="max-w-[120px]"
              min={1}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default function Settings() {
  const { t } = useTranslation(['settings']);

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">{t('settings:title')}</h1>
      <div className="grid gap-6 md:grid-cols-[200px_1fr]">
        <SettingsNav />
        <div>
          <Routes>
            <Route index element={<ProfileSettings />} />
            <Route path="accounts" element={<AccountsListPage />} />
            <Route path="accounts/add" element={<AddAccountPage />} />
            <Route path="accounts/:id/edit" element={<EditAccountPage />} />
            <Route path="filters" element={<FiltersSettings />} />
            <Route path="repository-filters" element={<RepositoryFilterSettings />} />
            <Route path="notifications" element={<NotificationsSettings />} />
            <Route path="behavior" element={<BehaviorSettings />} />
          </Routes>
        </div>
      </div>
    </div>
  );
}
