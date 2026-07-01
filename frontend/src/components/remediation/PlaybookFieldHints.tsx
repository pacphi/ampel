import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { HelpCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';

/**
 * Ordered field-guide entries. `field` is the literal YAML key (a code token,
 * never translated); `hint` is the i18n key under `remediation:playbooks.hints`.
 * `values` carries code-token lists as interpolation values so they stay OUT of
 * the translatable string (and can't be corrupted by machine translation).
 */
const FIELD_HINTS: {
  field: string;
  hint: string;
  values?: Record<string, string>;
}[] = [
  { field: 'role', hint: 'role' },
  {
    field: 'tasks',
    hint: 'tasks',
    values: { vars: 'repo_full_name, base_branch, failure_class' },
  },
  { field: 'loop', hint: 'loop' },
  { field: 'tools_policy', hint: 'toolsPolicy' },
  { field: 'context_spec', hint: 'contextSpec' },
  {
    field: 'output_contract',
    hint: 'outputContract',
    values: { values: 'tool_use, unified_diff, classify_only' },
  },
  {
    field: 'provider_overlays',
    hint: 'providerOverlays',
    values: { kinds: 'claude, gemini, ollama, onnx' },
  },
];

interface PlaybookFieldHintsProps {
  /** Top-level YAML field to highlight (from a field-path validation error). */
  errorField?: string;
}

/**
 * Collapsible schema reference shown next to the YAML editor. Documents each
 * playbook field — including the ADR-006 tool-ceiling narrow-only rule and the
 * trusted-vars / untrusted-context split — and highlights the offending field
 * when a field-path validation error is present (auto-expanding to show it).
 */
export function PlaybookFieldHints({ errorField }: PlaybookFieldHintsProps) {
  const { t } = useTranslation(['remediation']);
  const [open, setOpen] = useState(false);
  const expanded = open || !!errorField;

  return (
    <div className="rounded-md border bg-muted/20 p-3">
      <div className="flex items-center justify-between gap-2">
        <h4 className="flex items-center gap-1.5 text-xs font-medium">
          <HelpCircle className="h-3.5 w-3.5" aria-hidden="true" />
          {t('remediation:playbooks.hints.heading')}
        </h4>
        <Button type="button" variant="ghost" size="sm" onClick={() => setOpen((o) => !o)}>
          {expanded ? t('remediation:playbooks.hints.hide') : t('remediation:playbooks.hints.show')}
        </Button>
      </div>

      {expanded && (
        <dl className="mt-2 space-y-1.5">
          {FIELD_HINTS.map(({ field, hint, values }) => {
            const isError = errorField === field;
            return (
              <div
                key={field}
                className={`rounded px-2 py-1 ${
                  isError ? 'bg-destructive/10 ring-1 ring-destructive' : ''
                }`}
              >
                <dt className="inline">
                  <code className="text-[11px] font-semibold">{field}</code>
                </dt>{' '}
                <dd className="inline text-xs text-muted-foreground">
                  {t(`remediation:playbooks.hints.${hint}`, values)}
                </dd>
              </div>
            );
          })}
        </dl>
      )}
    </div>
  );
}

export default PlaybookFieldHints;
