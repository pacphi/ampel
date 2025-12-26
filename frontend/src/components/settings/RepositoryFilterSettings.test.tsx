import { describe, expect, it, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { RepositoryFilterSettings } from './RepositoryFilterSettings';

describe('RepositoryFilterSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe('Component Rendering', () => {
    it('renders repository visibility filters card', () => {
      render(<RepositoryFilterSettings />);

      expect(screen.getByText('Repository Visibility Filters')).toBeInTheDocument();
      expect(screen.getByText(/control which repositories are displayed/i)).toBeInTheDocument();
    });

    it('renders all three switches', () => {
      render(<RepositoryFilterSettings />);

      expect(screen.getByText('Show public repositories')).toBeInTheDocument();
      expect(screen.getByText('Show private repositories')).toBeInTheDocument();
      expect(screen.getByText('Show archived repositories')).toBeInTheDocument();
    });

    it('displays icons for each filter option', () => {
      const { container } = render(<RepositoryFilterSettings />);

      // Check for Globe, Lock, and Archive icons
      const globeIcon = container.querySelector('svg.lucide-globe');
      const lockIcon = container.querySelector('svg.lucide-lock');
      const archiveIcon = container.querySelector('svg.lucide-archive');

      expect(globeIcon).toBeInTheDocument();
      expect(lockIcon).toBeInTheDocument();
      expect(archiveIcon).toBeInTheDocument();
    });

    it('shows Bitbucket note about archive feature', () => {
      render(<RepositoryFilterSettings />);

      expect(
        screen.getByText(/bitbucket does not support the archive feature/i)
      ).toBeInTheDocument();
    });

    it('shows descriptive text for each filter', () => {
      render(<RepositoryFilterSettings />);

      expect(
        screen.getByText(/display repositories that are publicly accessible/i)
      ).toBeInTheDocument();
      expect(screen.getByText(/display repositories with restricted access/i)).toBeInTheDocument();
      expect(screen.getByText(/display repositories that have been archived/i)).toBeInTheDocument();
    });
  });

  describe('Initial Switch State', () => {
    it('all switches are checked by default', () => {
      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');

      expect(switches).toHaveLength(3);
      switches.forEach((switchElement) => {
        expect(switchElement).toHaveAttribute('data-state', 'checked');
      });
    });

    it('reflects stored filter state from localStorage', () => {
      const storedFilters = {
        includePublic: false,
        includePrivate: true,
        includeArchived: false,
      };
      localStorage.setItem('ampel-repository-filters', JSON.stringify(storedFilters));

      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');

      expect(switches[0]).toHaveAttribute('data-state', 'unchecked'); // Public
      expect(switches[1]).toHaveAttribute('data-state', 'checked'); // Private
      expect(switches[2]).toHaveAttribute('data-state', 'unchecked'); // Archived
    });
  });

  describe('Switch Interactions', () => {
    it('toggles public repositories filter when clicked', async () => {
      const user = userEvent.setup();
      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');
      const publicSwitch = switches[0];

      expect(publicSwitch).toHaveAttribute('data-state', 'checked');

      await user.click(publicSwitch);

      expect(publicSwitch).toHaveAttribute('data-state', 'unchecked');

      // Verify localStorage was updated
      const stored = JSON.parse(localStorage.getItem('ampel-repository-filters') || '{}');
      expect(stored.includePublic).toBe(false);
    });

    it('toggles private repositories filter when clicked', async () => {
      const user = userEvent.setup();
      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');
      const privateSwitch = switches[1];

      expect(privateSwitch).toHaveAttribute('data-state', 'checked');

      await user.click(privateSwitch);

      expect(privateSwitch).toHaveAttribute('data-state', 'unchecked');

      const stored = JSON.parse(localStorage.getItem('ampel-repository-filters') || '{}');
      expect(stored.includePrivate).toBe(false);
    });

    it('toggles archived repositories filter when clicked', async () => {
      const user = userEvent.setup();
      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');
      const archivedSwitch = switches[2];

      expect(archivedSwitch).toHaveAttribute('data-state', 'checked');

      await user.click(archivedSwitch);

      expect(archivedSwitch).toHaveAttribute('data-state', 'unchecked');

      const stored = JSON.parse(localStorage.getItem('ampel-repository-filters') || '{}');
      expect(stored.includeArchived).toBe(false);
    });

    it('allows toggling switch back on', async () => {
      const user = userEvent.setup();
      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');
      const publicSwitch = switches[0];

      // Toggle off
      await user.click(publicSwitch);
      expect(publicSwitch).toHaveAttribute('data-state', 'unchecked');

      // Toggle back on
      await user.click(publicSwitch);
      expect(publicSwitch).toHaveAttribute('data-state', 'checked');

      const stored = JSON.parse(localStorage.getItem('ampel-repository-filters') || '{}');
      expect(stored.includePublic).toBe(true);
    });

    it('updates only the clicked filter, preserving others', async () => {
      const user = userEvent.setup();
      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');

      // Toggle public off
      await user.click(switches[0]);

      expect(switches[0]).toHaveAttribute('data-state', 'unchecked');
      expect(switches[1]).toHaveAttribute('data-state', 'checked');
      expect(switches[2]).toHaveAttribute('data-state', 'checked');

      const stored = JSON.parse(localStorage.getItem('ampel-repository-filters') || '{}');
      expect(stored).toEqual({
        includePublic: false,
        includePrivate: true,
        includeArchived: true,
        onlyWithPrs: false,
      });
    });

    it('allows all filters to be disabled', async () => {
      const user = userEvent.setup();
      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');

      await user.click(switches[0]);
      await user.click(switches[1]);
      await user.click(switches[2]);

      switches.forEach((switchElement) => {
        expect(switchElement).toHaveAttribute('data-state', 'unchecked');
      });

      const stored = JSON.parse(localStorage.getItem('ampel-repository-filters') || '{}');
      expect(stored).toEqual({
        includePublic: false,
        includePrivate: false,
        includeArchived: false,
        onlyWithPrs: false,
      });
    });
  });

  describe('Bitbucket Note', () => {
    it('displays note in muted background', () => {
      const { container } = render(<RepositoryFilterSettings />);

      const note = container.querySelector('.bg-muted');
      expect(note).toBeInTheDocument();
      expect(note?.textContent).toContain('Bitbucket does not support the archive feature');
    });

    it('emphasizes "Note:" text', () => {
      render(<RepositoryFilterSettings />);

      const noteLabel = screen.getByText('Note:');
      expect(noteLabel.tagName).toBe('STRONG');
    });
  });

  describe('Accessibility', () => {
    it('uses switch role for toggle controls', () => {
      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');
      expect(switches).toHaveLength(3);
    });

    it('switches are keyboard accessible', () => {
      render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');
      const publicSwitch = switches[0];

      // Focus the switch
      publicSwitch.focus();
      expect(publicSwitch).toHaveFocus();

      // Verify switch can be toggled (switches are clickable and focusable)
      expect(publicSwitch).toHaveAttribute('data-state', 'checked');
    });
  });

  describe('Persistence', () => {
    it('persists filter changes across component remounts', async () => {
      const user = userEvent.setup();
      const { unmount } = render(<RepositoryFilterSettings />);

      const switches = screen.getAllByRole('switch');
      await user.click(switches[0]);

      unmount();

      // Remount component
      render(<RepositoryFilterSettings />);

      const newSwitches = screen.getAllByRole('switch');
      expect(newSwitches[0]).toHaveAttribute('data-state', 'unchecked');
    });
  });
});
