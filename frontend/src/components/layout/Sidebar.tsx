import { Link, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { cn } from '@/lib/utils';
import { LayoutDashboard, GitBranch, GitMerge, Settings, CircleDot, BarChart3 } from 'lucide-react';

export default function Sidebar() {
  const { t } = useTranslation(['common']);
  const location = useLocation();

  const navigation = [
    { name: t('common:navigation.dashboard'), href: '/dashboard', icon: LayoutDashboard },
    { name: t('common:navigation.repositories'), href: '/repositories', icon: GitBranch },
    { name: t('common:navigation.merge'), href: '/merge', icon: GitMerge },
    { name: t('common:navigation.analytics'), href: '/analytics', icon: BarChart3 },
    { name: t('common:navigation.settings'), href: '/settings', icon: Settings },
  ];

  return (
    <div className="flex w-64 flex-col border-r bg-card">
      <div className="flex h-16 items-center gap-2 border-b px-6">
        <CircleDot className="h-8 w-8 text-ampel-green" />
        <span className="text-xl font-bold">{t('common:app.name')}</span>
      </div>
      <nav className="flex-1 space-y-1 p-4">
        {navigation.map((item) => {
          const isActive =
            item.href === '/' ? location.pathname === '/' : location.pathname.startsWith(item.href);

          return (
            <Link
              key={item.name}
              to={item.href}
              className={cn(
                'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
                isActive
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
              )}
            >
              <item.icon className="h-5 w-5" />
              {item.name}
            </Link>
          );
        })}
      </nav>
      <div className="border-t p-4">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <div className="flex gap-1">
            <span className="h-2 w-2 rounded-full bg-ampel-green" />
            <span className="h-2 w-2 rounded-full bg-ampel-yellow" />
            <span className="h-2 w-2 rounded-full bg-ampel-red" />
          </div>
          <span>{t('common:status.trafficLight')}</span>
        </div>
      </div>
    </div>
  );
}
