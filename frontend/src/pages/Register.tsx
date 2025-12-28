import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
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

export default function Register() {
  const { t } = useTranslation(['common', 'validation', 'errors']);
  const { register: registerUser } = useAuth();
  const navigate = useNavigate();
  const { toast } = useToast();
  const [isLoading, setIsLoading] = useState(false);

  const registerSchema = z
    .object({
      email: z.string().email(t('validation:invalidEmail')),
      displayName: z.string().min(1, t('validation:messages.displayNameRequired')).max(100),
      password: z.string().min(8, t('validation:messages.passwordMinLength')),
      confirmPassword: z.string(),
    })
    .refine((data) => data.password === data.confirmPassword, {
      message: t('validation:passwordsDontMatch'),
      path: ['confirmPassword'],
    });

  type RegisterForm = z.infer<typeof registerSchema>;

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<RegisterForm>({
    resolver: zodResolver(registerSchema),
  });

  const onSubmit = async (data: RegisterForm) => {
    setIsLoading(true);
    try {
      await registerUser(data.email, data.password, data.displayName);
      navigate('/dashboard');
    } catch (error: unknown) {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('errors:auth.registrationFailed'),
        description: axiosError.response?.data?.error || t('errors:auth.failedToCreateAccount'),
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="flex justify-center mb-4">
            <CircleDot className="h-12 w-12 text-ampel-green" />
          </div>
          <CardTitle className="text-2xl">{t('common:auth.createAccount')}</CardTitle>
          <CardDescription>{t('common:auth.getStarted')}</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="displayName">{t('common:auth.displayName')}</Label>
              <Input id="displayName" placeholder="John Doe" {...register('displayName')} />
              {errors.displayName && (
                <p className="text-sm text-destructive">{errors.displayName.message}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="email">{t('common:auth.email')}</Label>
              <Input id="email" type="email" placeholder="you@example.com" {...register('email')} />
              {errors.email && <p className="text-sm text-destructive">{errors.email.message}</p>}
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">{t('common:auth.password')}</Label>
              <Input id="password" type="password" {...register('password')} />
              {errors.password && (
                <p className="text-sm text-destructive">{errors.password.message}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="confirmPassword">{t('common:auth.confirmPassword')}</Label>
              <Input id="confirmPassword" type="password" {...register('confirmPassword')} />
              {errors.confirmPassword && (
                <p className="text-sm text-destructive">{errors.confirmPassword.message}</p>
              )}
            </div>
            <Button type="submit" className="w-full" disabled={isLoading}>
              {isLoading ? t('common:auth.creatingAccount') : t('common:auth.createAccount')}
            </Button>
          </form>
          <div className="mt-4 text-center text-sm">
            {t('common:auth.alreadyHaveAccount')}{' '}
            <Link to="/login" className="text-primary hover:underline">
              {t('common:auth.signIn')}
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
