import type { GitProvider } from '@/types';
import { ExternalLink } from 'lucide-react';

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
  const instructions: Record<GitProvider, InstructionInfo> = {
    github: {
      title: 'Create a GitHub Personal Access Token',
      steps: [
        'Go to GitHub Settings > Developer settings > Personal access tokens',
        'Click "Generate new token (classic)" or "Fine-grained tokens"',
        'For Classic: Select scopes: repo, read:user',
        'For Fine-grained: Grant repository access and select permissions: Contents (Read), Metadata (Read), Pull requests (Read/Write)',
        "Copy the generated token immediately (it won't be shown again)",
      ],
      link: 'https://github.com/settings/tokens',
      linkText: 'GitHub Token Settings',
    },
    gitlab: {
      title: 'Create a GitLab Personal Access Token',
      steps: [
        'Go to GitLab User Settings > Access Tokens',
        'Enter a name and optional expiration date',
        'Select scopes: api, read_user',
        'Click "Create personal access token"',
        "Copy the token immediately (it won't be shown again)",
      ],
      link: 'https://gitlab.com/-/user_settings/personal_access_tokens',
      linkText: 'GitLab Token Settings',
    },
    bitbucket: {
      title: 'Create a Bitbucket App Password',
      steps: [
        'Go to Bitbucket Settings > Personal Settings > App passwords',
        'Click "Create app password"',
        'Enter a label for the password',
        'Select permissions: Account (Read), Repositories (Read), Pull requests (Read/Write)',
        'Click "Create" and copy the generated password',
        "You'll also need to provide your Bitbucket username",
      ],
      link: 'https://bitbucket.org/account/settings/app-passwords/',
      linkText: 'Bitbucket App Passwords',
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
