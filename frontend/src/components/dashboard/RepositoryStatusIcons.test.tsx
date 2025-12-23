import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import RepositoryStatusIcons from './RepositoryStatusIcons';

describe('RepositoryStatusIcons', () => {
  describe('Visibility Icons', () => {
    it('renders Globe icon for public repository', () => {
      const { container } = render(<RepositoryStatusIcons isPrivate={false} isArchived={false} />);

      // Globe icon should be present
      const globeIcon = container.querySelector('svg.lucide-globe');
      expect(globeIcon).toBeInTheDocument();
      expect(globeIcon?.getAttribute('class')).toContain('text-green-600');
    });

    it('renders Lock icon for private repository', () => {
      const { container } = render(<RepositoryStatusIcons isPrivate={true} isArchived={false} />);

      // Lock icon should be present
      const lockIcon = container.querySelector('svg.lucide-lock');
      expect(lockIcon).toBeInTheDocument();
      expect(lockIcon?.getAttribute('class')).toContain('text-amber-600');
    });

    it('does not render Globe icon when repository is private', () => {
      const { container } = render(<RepositoryStatusIcons isPrivate={true} isArchived={false} />);

      const globeIcon = container.querySelector('svg.lucide-globe');
      expect(globeIcon).not.toBeInTheDocument();
    });

    it('does not render Lock icon when repository is public', () => {
      const { container } = render(<RepositoryStatusIcons isPrivate={false} isArchived={false} />);

      const lockIcon = container.querySelector('svg.lucide-lock');
      expect(lockIcon).not.toBeInTheDocument();
    });
  });

  describe('Archive Icon', () => {
    it('does not render Archive icon when repository is not archived', () => {
      const { container } = render(<RepositoryStatusIcons isPrivate={false} isArchived={false} />);

      const archiveIcon = container.querySelector('svg.lucide-archive');
      expect(archiveIcon).not.toBeInTheDocument();
    });

    it('renders Archive icon when repository is archived', () => {
      const { container } = render(<RepositoryStatusIcons isPrivate={false} isArchived={true} />);

      const archiveIcon = container.querySelector('svg.lucide-archive');
      expect(archiveIcon).toBeInTheDocument();
      expect(archiveIcon?.getAttribute('class')).toContain('text-gray-500');
    });

    it('renders both Lock and Archive icons for private archived repository', () => {
      const { container } = render(<RepositoryStatusIcons isPrivate={true} isArchived={true} />);

      const lockIcon = container.querySelector('svg.lucide-lock');
      const archiveIcon = container.querySelector('svg.lucide-archive');

      expect(lockIcon).toBeInTheDocument();
      expect(archiveIcon).toBeInTheDocument();
    });
  });

  describe('Size Variants', () => {
    it('respects sm size prop', () => {
      const { container } = render(
        <RepositoryStatusIcons isPrivate={false} isArchived={false} size="sm" />
      );

      const icon = container.querySelector('svg');
      const iconClass = icon?.getAttribute('class') || '';
      expect(iconClass).toContain('h-3');
      expect(iconClass).toContain('w-3');
    });

    it('respects md size prop (default)', () => {
      const { container } = render(
        <RepositoryStatusIcons isPrivate={false} isArchived={false} size="md" />
      );

      const icon = container.querySelector('svg');
      const iconClass = icon?.getAttribute('class') || '';
      expect(iconClass).toContain('h-3.5');
      expect(iconClass).toContain('w-3.5');
    });

    it('respects lg size prop', () => {
      const { container } = render(
        <RepositoryStatusIcons isPrivate={false} isArchived={false} size="lg" />
      );

      const icon = container.querySelector('svg');
      const iconClass = icon?.getAttribute('class') || '';
      expect(iconClass).toContain('h-4');
      expect(iconClass).toContain('w-4');
    });

    it('uses md size by default when size prop not provided', () => {
      const { container } = render(<RepositoryStatusIcons isPrivate={false} isArchived={false} />);

      const icon = container.querySelector('svg');
      const iconClass = icon?.getAttribute('class') || '';
      expect(iconClass).toContain('h-3.5');
      expect(iconClass).toContain('w-3.5');
    });
  });

  describe('Tooltips', () => {
    it('shows "Public repository" tooltip for public repository', async () => {
      const user = userEvent.setup();
      const { container } = render(<RepositoryStatusIcons isPrivate={false} isArchived={false} />);

      const icon = container.querySelector('svg.lucide-globe');
      expect(icon).toBeInTheDocument();

      // Hover over the icon
      if (icon) {
        await user.hover(icon);
        // Tooltip content appears asynchronously
        const tooltips = await screen.findAllByText('Public repository');
        expect(tooltips.length).toBeGreaterThan(0);
      }
    });

    it('shows "Private repository" tooltip for private repository', async () => {
      const user = userEvent.setup();
      const { container } = render(<RepositoryStatusIcons isPrivate={true} isArchived={false} />);

      const icon = container.querySelector('svg.lucide-lock');
      expect(icon).toBeInTheDocument();

      if (icon) {
        await user.hover(icon);
        const tooltips = await screen.findAllByText('Private repository');
        expect(tooltips.length).toBeGreaterThan(0);
      }
    });

    it('shows "Archived repository" tooltip for archived repository', async () => {
      const user = userEvent.setup();
      const { container } = render(<RepositoryStatusIcons isPrivate={false} isArchived={true} />);

      const icon = container.querySelector('svg.lucide-archive');
      expect(icon).toBeInTheDocument();

      if (icon) {
        await user.hover(icon);
        const tooltips = await screen.findAllByText('Archived repository');
        expect(tooltips.length).toBeGreaterThan(0);
      }
    });
  });

  describe('Custom Styling', () => {
    it('applies custom className', () => {
      const { container } = render(
        <RepositoryStatusIcons isPrivate={false} isArchived={false} className="custom-class" />
      );

      const wrapper = container.querySelector('.custom-class');
      expect(wrapper).toBeInTheDocument();
    });

    it('maintains flex layout with gap', () => {
      const { container } = render(<RepositoryStatusIcons isPrivate={false} isArchived={false} />);

      const wrapper = container.querySelector('div');
      const wrapperClass = wrapper?.getAttribute('class') || '';
      expect(wrapperClass).toContain('flex');
      expect(wrapperClass).toContain('items-center');
      expect(wrapperClass).toContain('gap-1');
    });
  });
});
