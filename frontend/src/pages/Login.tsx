import { useState } from 'react';
import { Link, useNavigate, useLocation } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useTranslation } from 'react-i18next';
import { useAuth } from '@/hooks/useAuth';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useToast } from '@/components/ui/use-toast';
import { CircleDot } from 'lucide-react';
import { LanguageSwitcher } from '@/components/LanguageSwitcher';

export default function Login() {
  const { t } = useTranslation(['common', 'validation', 'errors']);
  const { login } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const { toast } = useToast();
  const [isLoading, setIsLoading] = useState(false);

  const loginSchema = z.object({
    email: z.string().email(t('validation:invalidEmail')),
    password: z.string().min(1, t('validation:messages.passwordRequired')),
  });

  type LoginForm = z.infer<typeof loginSchema>;

  const from = (location.state as { from?: { pathname: string } })?.from?.pathname || '/dashboard';

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginForm>({
    resolver: zodResolver(loginSchema),
  });

  const onSubmit = async (data: LoginForm) => {
    setIsLoading(true);
    try {
      await login(data.email, data.password);
      navigate(from, { replace: true });
    } catch (error: unknown) {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('errors:auth.loginFailed'),
        description: axiosError.response?.data?.error || t('errors:auth.invalidEmailOrPassword'),
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md relative">
        <div className="absolute top-4 right-4 rtl:right-auto rtl:left-4">
          <LanguageSwitcher
            variant="dropdown"
            size="sm"
            showSearch={false}
            showFavorites={false}
          />
        </div>
        <CardHeader className="text-center">
          <div className="flex justify-center mb-4">
            <CircleDot className="h-12 w-12 text-ampel-green" />
          </div>
          <CardTitle className="text-2xl">{t('common:auth.welcomeToAmpel')}</CardTitle>
          <CardDescription>{t('common:auth.signInToContinue')}</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="email">{t('common:auth.email')}</Label>
              <Input id="email" type="email" placeholder={t('common:auth.emailPlaceholder')} {...register('email')} />
              {errors.email && <p className="text-sm text-destructive">{errors.email.message}</p>}
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">{t('common:auth.password')}</Label>
              <Input id="password" type="password" {...register('password')} />
              {errors.password && (
                <p className="text-sm text-destructive">{errors.password.message}</p>
              )}
            </div>
            <Button type="submit" className="w-full" disabled={isLoading}>
              {isLoading ? t('common:auth.signingIn') : t('common:auth.signIn')}
            </Button>
          </form>
          <div className="mt-4 text-center text-sm">
            {t('common:auth.dontHaveAccount')}{' '}
            <Link to="/register" className="text-primary hover:underline">
              {t('common:auth.signUp')}
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
