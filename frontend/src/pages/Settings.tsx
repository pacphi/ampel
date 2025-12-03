import { useState } from 'react';
import { Routes, Route, Link, useLocation } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { connectionsApi } from '@/api/connections';
import { useAuth } from '@/hooks/useAuth';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useToast } from '@/components/ui/use-toast';
import { cn } from '@/lib/utils';
import type { GitProvider, ProviderConnection } from '@/types';
import {
  User,
  Link2,
  Github,
  Trash2,
  Plus,
  CheckCircle,
  XCircle,
  RefreshCw,
  Loader2,
} from 'lucide-react';

function SettingsNav() {
  const location = useLocation();

  const links = [
    { href: '/settings', label: 'Profile', icon: User },
    { href: '/settings/connections', label: 'Connections', icon: Link2 },
  ];

  return (
    <nav className="space-y-1">
      {links.map((link) => {
        const isActive = location.pathname === link.href;
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

function ProfileSettings() {
  const { user } = useAuth();

  return (
    <Card>
      <CardHeader>
        <CardTitle>Profile</CardTitle>
        <CardDescription>Your account information</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div>
          <label className="text-sm font-medium">Email</label>
          <p className="text-muted-foreground">{user?.email}</p>
        </div>
        <div>
          <label className="text-sm font-medium">Display Name</label>
          <p className="text-muted-foreground">{user?.displayName || 'Not set'}</p>
        </div>
        <div>
          <label className="text-sm font-medium">Member Since</label>
          <p className="text-muted-foreground">
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

function AddConnectionDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [provider, setProvider] = useState<GitProvider>('github');
  const [connectionName, setConnectionName] = useState('');
  const [accessToken, setAccessToken] = useState('');
  const [baseUrl, setBaseUrl] = useState('');

  const addMutation = useMutation({
    mutationFn: () =>
      connectionsApi.add({
        provider,
        connectionName,
        accessToken,
        baseUrl: baseUrl || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] });
      toast({
        title: 'Connection added',
        description: 'Your connection has been added successfully',
      });
      onOpenChange(false);
      setConnectionName('');
      setAccessToken('');
      setBaseUrl('');
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to add connection',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const getTokenUrl = (p: GitProvider) => {
    switch (p) {
      case 'github':
        return 'https://github.com/settings/tokens';
      case 'gitlab':
        return 'https://gitlab.com/-/profile/personal_access_tokens';
      case 'bitbucket':
        return 'https://bitbucket.org/account/settings/app-passwords/';
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add Connection</DialogTitle>
          <DialogDescription>
            Connect a Git provider using a Personal Access Token
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>Provider</Label>
            <Select value={provider} onValueChange={(v) => setProvider(v as GitProvider)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="github">GitHub</SelectItem>
                <SelectItem value="gitlab">GitLab</SelectItem>
                <SelectItem value="bitbucket">Bitbucket</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>Connection Name</Label>
            <Input
              placeholder="e.g., work-github, personal-gitlab"
              value={connectionName}
              onChange={(e) => setConnectionName(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label>Personal Access Token</Label>
            <Input
              type="password"
              placeholder="Enter your PAT"
              value={accessToken}
              onChange={(e) => setAccessToken(e.target.value)}
            />
            <p className="text-xs text-muted-foreground">
              <a
                href={getTokenUrl(provider)}
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary hover:underline"
              >
                Create a token
              </a>{' '}
              with repo access
            </p>
          </div>
          <div className="space-y-2">
            <Label>Base URL (optional)</Label>
            <Input
              placeholder="https://github.mycompany.com"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
            />
            <p className="text-xs text-muted-foreground">
              For self-hosted instances (GitHub Enterprise, GitLab CE/EE, Bitbucket Server)
            </p>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={() => addMutation.mutate()}
            disabled={!connectionName || !accessToken || addMutation.isPending}
          >
            {addMutation.isPending ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Plus className="mr-2 h-4 w-4" />
            )}
            Add Connection
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function ConnectionCard({ connection }: { connection: ProviderConnection }) {
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const deleteMutation = useMutation({
    mutationFn: () => connectionsApi.delete(connection.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] });
      toast({
        title: 'Connection removed',
        description: 'The connection has been removed',
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to remove',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const validateMutation = useMutation({
    mutationFn: () => connectionsApi.validate(connection.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['connections'] });
      toast({
        title: 'Validation complete',
        description: 'Connection has been validated',
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Validation failed',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const getProviderIcon = () => {
    return <Github className="h-6 w-6" />;
  };

  return (
    <div className="flex items-center justify-between p-4 rounded-lg border">
      <div className="flex items-center gap-4">
        {getProviderIcon()}
        <div>
          <div className="flex items-center gap-2">
            <p className="font-medium">{connection.connectionName}</p>
            {connection.isValidated ? (
              <CheckCircle className="h-4 w-4 text-green-500" />
            ) : (
              <XCircle className="h-4 w-4 text-red-500" />
            )}
          </div>
          <p className="text-sm text-muted-foreground">
            {connection.provider} - @{connection.providerUsername}
          </p>
          {connection.baseUrl && (
            <p className="text-xs text-muted-foreground">{connection.baseUrl}</p>
          )}
          {connection.validationError && (
            <p className="text-xs text-destructive">{connection.validationError}</p>
          )}
        </div>
      </div>
      <div className="flex gap-2">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => validateMutation.mutate()}
          disabled={validateMutation.isPending}
        >
          {validateMutation.isPending ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="h-4 w-4" />
          )}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onClick={() => deleteMutation.mutate()}
          disabled={deleteMutation.isPending}
        >
          {deleteMutation.isPending ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Trash2 className="h-4 w-4 text-destructive" />
          )}
        </Button>
      </div>
    </div>
  );
}

function ConnectionsSettings() {
  const [addDialogOpen, setAddDialogOpen] = useState(false);

  const { data: connections, isLoading } = useQuery({
    queryKey: ['connections'],
    queryFn: () => connectionsApi.list(),
  });

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Provider Connections</CardTitle>
            <CardDescription>
              Connect your Git providers using Personal Access Tokens
            </CardDescription>
          </div>
          <Button onClick={() => setAddDialogOpen(true)}>
            <Plus className="mr-2 h-4 w-4" />
            Add Connection
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : connections && connections.length > 0 ? (
          <div className="space-y-4">
            {connections.map((connection) => (
              <ConnectionCard key={connection.id} connection={connection} />
            ))}
          </div>
        ) : (
          <div className="text-center py-8 text-muted-foreground">
            <Link2 className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>No connections yet</p>
            <p className="text-sm">Add a connection to start tracking repositories</p>
          </div>
        )}
      </CardContent>
      <AddConnectionDialog open={addDialogOpen} onOpenChange={setAddDialogOpen} />
    </Card>
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
            <Route path="connections" element={<ConnectionsSettings />} />
          </Routes>
        </div>
      </div>
    </div>
  );
}
