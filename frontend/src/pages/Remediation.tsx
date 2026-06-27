import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Bot, ClipboardList, ListChecks, Plus, Server, SlidersHorizontal } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import { FleetOverview } from '@/components/remediation/FleetOverview';
import { PolicyEditor } from '@/components/remediation/PolicyEditor';
import { RunsPanel } from '@/components/remediation/RunsPanel';
import { AuditLog } from '@/components/remediation/AuditLog';
import { KillSwitch } from '@/components/remediation/KillSwitch';
import {
  useDeleteRemediationPolicy,
  useRemediationPolicies,
  useToggleRemediationPolicy,
} from '@/hooks/useRemediationPolicies';
import type { RemediationPolicy } from '@/types/remediation';

type Tab = 'fleet' | 'runs' | 'policies' | 'audit';

const TAB_ICON = {
  fleet: Server,
  runs: ListChecks,
  policies: SlidersHorizontal,
  audit: ClipboardList,
} as const;

export default function Remediation() {
  const { t } = useTranslation(['remediation', 'common']);
  const [tab, setTab] = useState<Tab>('fleet');
  const [editing, setEditing] = useState<RemediationPolicy | null>(null);
  const [creating, setCreating] = useState(false);

  const { data: policies, isLoading } = useRemediationPolicies();
  const toggleMutation = useToggleRemediationPolicy();
  const deleteMutation = useDeleteRemediationPolicy();

  const showEditor = creating || !!editing;

  const closeEditor = () => {
    setCreating(false);
    setEditing(null);
  };

  const tabs: { id: Tab; label: string }[] = [
    { id: 'fleet', label: t('remediation:tabs.fleet') },
    { id: 'runs', label: t('remediation:tabs.runs') },
    { id: 'policies', label: t('remediation:tabs.policies') },
    { id: 'audit', label: t('remediation:tabs.audit') },
  ];

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Bot className="h-6 w-6" />
            {t('remediation:title')}
          </h1>
          <p className="text-muted-foreground">{t('remediation:subtitle')}</p>
        </div>
        <KillSwitch />
      </div>

      {/* Tabs */}
      <div role="tablist" aria-label={t('remediation:title')} className="flex gap-1 border-b">
        {tabs.map((item) => {
          const Icon = TAB_ICON[item.id];
          return (
            <button
              key={item.id}
              role="tab"
              id={`remediation-tab-${item.id}`}
              aria-selected={tab === item.id}
              aria-controls={`remediation-panel-${item.id}`}
              onClick={() => setTab(item.id)}
              className={cn(
                'flex items-center gap-2 border-b-2 px-4 py-2 text-sm font-medium transition-colors',
                tab === item.id
                  ? 'border-primary text-foreground'
                  : 'border-transparent text-muted-foreground hover:text-foreground'
              )}
            >
              <Icon className="h-4 w-4" aria-hidden="true" />
              {item.label}
            </button>
          );
        })}
      </div>

      {tab === 'fleet' && (
        <div role="tabpanel" id="remediation-panel-fleet" aria-labelledby="remediation-tab-fleet">
          <Card>
            <CardHeader>
              <CardTitle>{t('remediation:fleet.title')}</CardTitle>
              <CardDescription>{t('remediation:fleet.description')}</CardDescription>
            </CardHeader>
            <CardContent>
              <FleetOverview />
            </CardContent>
          </Card>
        </div>
      )}

      {tab === 'runs' && (
        <div role="tabpanel" id="remediation-panel-runs" aria-labelledby="remediation-tab-runs">
          <Card>
            <CardHeader>
              <CardTitle>{t('remediation:runs.title')}</CardTitle>
              <CardDescription>{t('remediation:runs.description')}</CardDescription>
            </CardHeader>
            <CardContent>
              <RunsPanel />
            </CardContent>
          </Card>
        </div>
      )}

      {tab === 'audit' && (
        <div role="tabpanel" id="remediation-panel-audit" aria-labelledby="remediation-tab-audit">
          <Card>
            <CardHeader>
              <CardTitle>{t('remediation:audit.title')}</CardTitle>
              <CardDescription>{t('remediation:audit.description')}</CardDescription>
            </CardHeader>
            <CardContent>
              <AuditLog />
            </CardContent>
          </Card>
        </div>
      )}

      {tab === 'policies' && (
        <div
          role="tabpanel"
          id="remediation-panel-policies"
          aria-labelledby="remediation-tab-policies"
          className="space-y-4"
        >
          {showEditor ? (
            <Card>
              <CardHeader>
                <CardTitle>
                  {editing
                    ? t('remediation:editor.editTitle')
                    : t('remediation:editor.createTitle')}
                </CardTitle>
                <CardDescription>{t('remediation:editor.description')}</CardDescription>
              </CardHeader>
              <CardContent>
                <PolicyEditor
                  policy={editing ?? undefined}
                  onSaved={closeEditor}
                  onCancel={closeEditor}
                />
              </CardContent>
            </Card>
          ) : (
            <>
              <div className="flex justify-end">
                <Button onClick={() => setCreating(true)}>
                  <Plus className="h-4 w-4 mr-1.5" aria-hidden="true" />
                  {t('remediation:policies.create')}
                </Button>
              </div>

              <Card>
                <CardHeader>
                  <CardTitle>{t('remediation:policies.title')}</CardTitle>
                  <CardDescription>{t('remediation:policies.description')}</CardDescription>
                </CardHeader>
                <CardContent>
                  {isLoading ? (
                    <div
                      className="flex items-center justify-center h-32"
                      role="status"
                      aria-live="polite"
                    >
                      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                    </div>
                  ) : !policies || policies.length === 0 ? (
                    <div className="text-center py-8 text-muted-foreground">
                      {t('remediation:policies.empty')}
                    </div>
                  ) : (
                    <ul className="space-y-2">
                      {policies.map((policy) => (
                        <li
                          key={policy.id}
                          className="flex items-center gap-4 rounded-lg border p-3"
                        >
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2">
                              <span className="font-medium">
                                {t(`remediation:scopeType.${policy.scopeType}`)}
                              </span>
                              <Badge variant={policy.enabled ? 'success' : 'secondary'}>
                                {policy.enabled
                                  ? t('remediation:policies.statusEnabled')
                                  : t('remediation:policies.statusDisabled')}
                              </Badge>
                              <Badge variant="outline">
                                {t(`remediation:autonomyLevel.${policy.autonomyLevel}`)}
                              </Badge>
                            </div>
                            <p className="text-xs text-muted-foreground truncate">
                              {policy.scopeId}
                            </p>
                          </div>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => toggleMutation.mutate(policy.id)}
                            disabled={toggleMutation.isPending}
                          >
                            {policy.enabled
                              ? t('remediation:policies.disable')
                              : t('remediation:policies.enable')}
                          </Button>
                          <Button variant="outline" size="sm" onClick={() => setEditing(policy)}>
                            {t('common:actions.edit')}
                          </Button>
                          <Button
                            variant="destructive"
                            size="sm"
                            onClick={() => deleteMutation.mutate(policy.id)}
                            disabled={deleteMutation.isPending}
                          >
                            {t('common:actions.delete')}
                          </Button>
                        </li>
                      ))}
                    </ul>
                  )}
                </CardContent>
              </Card>
            </>
          )}
        </div>
      )}
    </div>
  );
}
