import { Routes, Route, Link, useLocation } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { oauthApi } from '@/api/oauth';
import { useAuth } from '@/hooks/useAuth';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useToast } from '@/components/ui/use-toast';
import { cn } from '@/lib/utils';
import type { GitProvider } from '@/types';
import { User, Link2, Github, Unlink } from 'lucide-react';

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

function ConnectionsSettings() {
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const { data: connections, isLoading } = useQuery({
    queryKey: ['oauth', 'connections'],
    queryFn: () => oauthApi.listConnections(),
  });

  const disconnectMutation = useMutation({
    mutationFn: (provider: GitProvider) => oauthApi.disconnect(provider),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['oauth', 'connections'] });
      toast({
        title: 'Disconnected',
        description: 'Provider has been disconnected',
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to disconnect',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const handleConnect = async (provider: GitProvider) => {
    try {
      const url = await oauthApi.getOAuthUrl(provider);
      window.location.href = url;
    } catch (error: unknown) {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to connect',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    }
  };

  const providers: { id: GitProvider; name: string }[] = [
    { id: 'github', name: 'GitHub' },
    { id: 'gitlab', name: 'GitLab' },
    { id: 'bitbucket', name: 'Bitbucket' },
  ];

  const isConnected = (provider: GitProvider) => connections?.some((c) => c.provider === provider);

  const getConnection = (provider: GitProvider) =>
    connections?.find((c) => c.provider === provider);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Provider Connections</CardTitle>
        <CardDescription>Connect your Git providers to access repositories</CardDescription>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
          </div>
        ) : (
          <div className="space-y-4">
            {providers.map((provider) => {
              const connected = isConnected(provider.id);
              const connection = getConnection(provider.id);

              return (
                <div
                  key={provider.id}
                  className="flex items-center justify-between p-4 rounded-lg border"
                >
                  <div className="flex items-center gap-4">
                    <Github className="h-8 w-8" />
                    <div>
                      <p className="font-medium">{provider.name}</p>
                      {connected && connection && (
                        <p className="text-sm text-muted-foreground">
                          Connected as @{connection.providerUsername}
                        </p>
                      )}
                    </div>
                  </div>
                  {connected ? (
                    <Button
                      variant="outline"
                      onClick={() => disconnectMutation.mutate(provider.id)}
                      disabled={disconnectMutation.isPending}
                    >
                      <Unlink className="h-4 w-4 mr-2" />
                      Disconnect
                    </Button>
                  ) : (
                    <Button onClick={() => handleConnect(provider.id)}>
                      <Link2 className="h-4 w-4 mr-2" />
                      Connect
                    </Button>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </CardContent>
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
