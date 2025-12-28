import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useAuth } from '@/hooks/useAuth';
import { useTheme } from '@/hooks/useTheme';
import { Button } from '@/components/ui/button';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { LogOut, Moon, Sun } from 'lucide-react';

export default function Header() {
  const { t } = useTranslation(['dashboard', 'common']);
  const { user, logout } = useAuth();
  const { setTheme, resolvedTheme } = useTheme();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  const toggleTheme = () => {
    setTheme(resolvedTheme === 'dark' ? 'light' : 'dark');
  };

  const initials = user?.displayName
    ? user.displayName
        .split(' ')
        .map((n) => n[0])
        .join('')
        .toUpperCase()
        .slice(0, 2)
    : 'U';

  return (
    <header className="flex h-16 items-center justify-between border-b bg-card px-6">
      <div className="flex items-center gap-4">
        <h1 className="text-lg font-semibold">{t('dashboard:prDashboard')}</h1>
      </div>
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" onClick={toggleTheme} title={t('common:theme.toggle')}>
          {resolvedTheme === 'dark' ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
        </Button>
        <div className="flex items-center gap-3">
          <Avatar className="h-8 w-8">
            <AvatarImage src={user?.avatarUrl} />
            <AvatarFallback>{initials}</AvatarFallback>
          </Avatar>
          {user?.displayName && (
            <div className="hidden sm:block">
              <p className="text-sm font-medium">{user.displayName}</p>
            </div>
          )}
        </div>
        <Button variant="ghost" size="icon" onClick={handleLogout} title={t('common:auth.logout')}>
          <LogOut className="h-5 w-5" />
        </Button>
      </div>
    </header>
  );
}
