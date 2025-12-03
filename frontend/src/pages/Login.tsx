import { useState } from 'react';
import { useAuth } from '@/hooks/useAuth';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useToast } from '@/components/ui/use-toast';
import { CircleDot, Github, Loader2 } from 'lucide-react';
import type { OAuthProvider } from '@/api/auth';

export default function Login() {
  const { loginWithOAuth } = useAuth();
  const { toast } = useToast();
  const [loadingProvider, setLoadingProvider] = useState<OAuthProvider | null>(null);

  const handleSocialLogin = async (provider: OAuthProvider) => {
    setLoadingProvider(provider);
    try {
      await loginWithOAuth(provider);
    } catch (error: unknown) {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Login failed',
        description: axiosError.response?.data?.error || `Failed to start ${provider} login`,
      });
      setLoadingProvider(null);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="flex justify-center mb-4">
            <CircleDot className="h-12 w-12 text-ampel-green" />
          </div>
          <CardTitle className="text-2xl">Welcome to Ampel</CardTitle>
          <CardDescription>Sign in with your account to continue</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button
            variant="outline"
            className="w-full"
            onClick={() => handleSocialLogin('github')}
            disabled={loadingProvider !== null}
          >
            {loadingProvider === 'github' ? (
              <Loader2 className="mr-2 h-5 w-5 animate-spin" />
            ) : (
              <Github className="mr-2 h-5 w-5" />
            )}
            Continue with GitHub
          </Button>

          <Button
            variant="outline"
            className="w-full"
            onClick={() => handleSocialLogin('google')}
            disabled={loadingProvider !== null}
          >
            {loadingProvider === 'google' ? (
              <Loader2 className="mr-2 h-5 w-5 animate-spin" />
            ) : (
              <svg className="mr-2 h-5 w-5" viewBox="0 0 24 24">
                <path
                  fill="currentColor"
                  d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
                />
                <path
                  fill="currentColor"
                  d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
                />
                <path
                  fill="currentColor"
                  d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
                />
                <path
                  fill="currentColor"
                  d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
                />
              </svg>
            )}
            Continue with Google
          </Button>

          <div className="relative my-6">
            <div className="absolute inset-0 flex items-center">
              <span className="w-full border-t" />
            </div>
            <div className="relative flex justify-center text-xs uppercase">
              <span className="bg-background px-2 text-muted-foreground">
                Unified PR Management
              </span>
            </div>
          </div>

          <p className="text-center text-sm text-muted-foreground">
            Consolidate pull requests from GitHub, GitLab, and Bitbucket into a single dashboard
            with traffic light status indicators.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
