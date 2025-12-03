import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAuth } from '@/hooks/useAuth';
import { useToast } from '@/components/ui/use-toast';
import { Loader2 } from 'lucide-react';

export default function AuthCallback() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { handleOAuthCallback } = useAuth();
  const { toast } = useToast();
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const processCallback = async () => {
      const accessToken = searchParams.get('access_token');
      const refreshToken = searchParams.get('refresh_token');
      const errorParam = searchParams.get('error');

      if (errorParam) {
        setError(errorParam);
        toast({
          variant: 'destructive',
          title: 'Authentication failed',
          description: errorParam,
        });
        setTimeout(() => navigate('/login'), 3000);
        return;
      }

      if (accessToken && refreshToken) {
        try {
          await handleOAuthCallback(accessToken, refreshToken);
          navigate('/dashboard', { replace: true });
        } catch {
          setError('Failed to complete authentication');
          toast({
            variant: 'destructive',
            title: 'Authentication failed',
            description: 'Failed to complete authentication. Please try again.',
          });
          setTimeout(() => navigate('/login'), 3000);
        }
      } else {
        setError('Missing authentication tokens');
        setTimeout(() => navigate('/login'), 3000);
      }
    };

    processCallback();
  }, [searchParams, handleOAuthCallback, navigate, toast]);

  return (
    <div className="flex min-h-screen items-center justify-center bg-background">
      <div className="text-center">
        {error ? (
          <div className="space-y-4">
            <p className="text-destructive">{error}</p>
            <p className="text-muted-foreground">Redirecting to login...</p>
          </div>
        ) : (
          <div className="space-y-4">
            <Loader2 className="h-8 w-8 animate-spin mx-auto text-primary" />
            <p className="text-muted-foreground">Completing sign in...</p>
          </div>
        )}
      </div>
    </div>
  );
}
