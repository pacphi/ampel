import { SVGProps } from 'react';

interface IconProps extends SVGProps<SVGSVGElement> {
  className?: string;
}

// GitLab logo - orange fox head icon
export function GitlabIcon({ className, ...props }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" className={className} {...props}>
      <path d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 0 1-.3-.94l1.22-3.78 2.44-7.51A.42.42 0 0 1 4.82 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.49h8.1l2.44-7.51A.42.42 0 0 1 18.6 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.51L23 13.45a.84.84 0 0 1-.35.94z" />
    </svg>
  );
}

// Bitbucket logo - blue gradient bucket shape
export function BitbucketIcon({ className, ...props }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" className={className} {...props}>
      <path d="M2.65 3A.65.65 0 0 0 2 3.75l2.74 16.63a.89.89 0 0 0 .87.74h12.91a.65.65 0 0 0 .65-.55L22 3.75A.65.65 0 0 0 21.35 3H2.65zM14.1 15H9.9l-1.14-5.93h6.48L14.1 15z" />
    </svg>
  );
}

// GitHub icon (re-export from lucide for consistency)
export { Github as GithubIcon } from 'lucide-react';
