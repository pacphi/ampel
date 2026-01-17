import type { GitProvider } from '@/types';
import { ExternalLink } from 'lucide-react';
import { useTranslation } from 'react-i18next';

interface TokenInstructionsProps {
  provider: GitProvider;
}

interface InstructionInfo {
  title: string;
  steps: string[];
  link: string;
  linkText: string;
}

export function TokenInstructions({ provider }: TokenInstructionsProps) {
  const { t } = useTranslation(['providers']);

  const instructions: Record<GitProvider, InstructionInfo> = {
    github: {
      title: t('providers:github.title'),
      steps: [
        t('providers:github.steps.step1'),
        t('providers:github.steps.step2'),
        t('providers:github.steps.step3'),
        t('providers:github.steps.step4'),
        t('providers:github.steps.step5'),
      ],
      link: 'https://github.com/settings/tokens',
      linkText: t('providers:github.linkText'),
    },
    gitlab: {
      title: t('providers:gitlab.title'),
      steps: [
        t('providers:gitlab.steps.step1'),
        t('providers:gitlab.steps.step2'),
        t('providers:gitlab.steps.step3'),
        t('providers:gitlab.steps.step4'),
        t('providers:gitlab.steps.step5'),
      ],
      link: 'https://gitlab.com/-/user_settings/personal_access_tokens',
      linkText: t('providers:gitlab.linkText'),
    },
    bitbucket: {
      title: t('providers:bitbucket.title'),
      steps: [
        t('providers:bitbucket.steps.step1'),
        t('providers:bitbucket.steps.step2'),
        t('providers:bitbucket.steps.step3'),
        t('providers:bitbucket.steps.step4'),
        t('providers:bitbucket.steps.step5'),
        t('providers:bitbucket.steps.step6'),
      ],
      link: 'https://bitbucket.org/account/settings/app-passwords/',
      linkText: t('providers:bitbucket.linkText'),
    },
  };

  const info = instructions[provider];

  return (
    <div className="bg-muted p-4 rounded-lg space-y-3">
      <h4 className="font-medium text-sm">{info.title}</h4>
      <ol className="list-decimal list-inside space-y-1.5">
        {info.steps.map((step, i) => (
          <li key={i} className="text-sm text-muted-foreground">
            {step}
          </li>
        ))}
      </ol>
      <a
        href={info.link}
        target="_blank"
        rel="noopener noreferrer"
        className="inline-flex items-center gap-1.5 text-sm text-primary hover:underline"
      >
        {info.linkText}
        <ExternalLink className="h-3 w-3" />
      </a>
    </div>
  );
}
