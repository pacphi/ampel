import { useState } from 'react';
import { Routes, Route, Link, useLocation } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
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
  const location = useLocation();

  const links = [
    { href: '/settings', label: 'Profile', icon: User },
    { href: '/settings/accounts', label: 'Accounts', icon: User },
    { href: '/settings/filters', label: 'PR Filters', icon: Filter },
    { href: '/settings/repository-filters', label: 'Repository Filters', icon: Boxes },
    { href: '/settings/notifications', label: 'Notifications', icon: Bell },
    { href: '/settings/behavior', label: 'Behavior', icon: Settings2 },
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
        title: 'Profile updated',
        description: 'Your profile has been saved successfully.',
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to update profile',
        description: axiosError.response?.data?.error || 'An error occurred',
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
          <CardTitle>Profile</CardTitle>
          <CardDescription>Your account information</CardDescription>
        </div>
        {!isEditing ? (
          <Button variant="outline" size="sm" onClick={handleEdit}>
            <Pencil className="h-4 w-4 mr-2" />
            Edit
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
              Cancel
            </Button>
            <Button size="sm" onClick={handleSave} disabled={updateMutation.isPending}>
              <Check className="h-4 w-4 mr-2" />
              {updateMutation.isPending ? 'Saving...' : 'Save'}
            </Button>
          </div>
        )}
      </CardHeader>
      <CardContent className="space-y-4">
        <div>
          <label className="text-sm font-medium">Email</label>
          {isEditing ? (
            <Input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="Enter your email"
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
                title={showEmail ? 'Hide email' : 'Show email'}
              >
                {showEmail ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              </Button>
            </div>
          )}
        </div>
        <div>
          <label className="text-sm font-medium">Display Name</label>
          {isEditing ? (
            <Input
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder="Enter your display name"
              className="mt-1"
            />
          ) : (
            <p className="text-muted-foreground mt-1">{user?.displayName || 'Not set'}</p>
          )}
        </div>
        <div>
          <label className="text-sm font-medium">Member Since</label>
          <p className="text-muted-foreground mt-1">
            {user?.createdAt
              ? new Date(user.createdAt).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric',
                })
              : 'Unknown'}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

function FiltersSettings() {
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
        title: 'Filters updated',
        description: 'Your PR filter settings have been saved.',
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to update filters',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const resetMutation = useMutation({
    mutationFn: () => prFiltersApi.reset(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pr-filters'] });
      toast({
        title: 'Filters reset',
        description: 'Your PR filter settings have been reset to defaults.',
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
            <CardTitle>PR Filters</CardTitle>
            <CardDescription>
              Configure global filters for pull request processing. These apply to auto-merge rules
              across all repositories.
            </CardDescription>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => resetMutation.mutate()}
            disabled={resetMutation.isPending}
          >
            <RotateCcw className="h-4 w-4 mr-2" />
            Reset to Defaults
          </Button>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Allowed Actors */}
          <div>
            <label className="text-sm font-medium">Allowed Actors (Bots/Users)</label>
            <p className="text-sm text-muted-foreground mb-3">
              Only process PRs from these trusted actors. Typically bots like dependabot, renovate,
              etc.
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
                placeholder="e.g., dependabot[bot]"
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
                Add
              </Button>
            </div>
          </div>

          {/* Skip Labels */}
          <div>
            <label className="text-sm font-medium">Skip Labels</label>
            <p className="text-sm text-muted-foreground mb-3">
              PRs with these labels will be skipped during auto-merge processing.
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
                placeholder="e.g., do-not-merge"
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
                Add
              </Button>
            </div>
          </div>

          {/* Max Age */}
          <div>
            <label className="text-sm font-medium">Max PR Age (Days)</label>
            <p className="text-sm text-muted-foreground mb-3">
              Skip PRs older than this many days. Leave empty for no limit.
            </p>
            <Input
              type="number"
              placeholder="e.g., 30"
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
  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Settings</h1>
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
