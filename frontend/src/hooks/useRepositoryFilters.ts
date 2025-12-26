import { useState, useCallback } from 'react';
import type { RepositoryWithStatus } from '@/types';

export interface RepositoryFilters {
  includePublic: boolean;
  includePrivate: boolean;
  includeArchived: boolean;
  onlyWithPrs: boolean;
}

const STORAGE_KEY = 'ampel-repository-filters';

const defaultFilters: RepositoryFilters = {
  includePublic: true,
  includePrivate: true,
  includeArchived: true,
  onlyWithPrs: false,
};

function loadFilters(): RepositoryFilters {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        includePublic: parsed.includePublic ?? defaultFilters.includePublic,
        includePrivate: parsed.includePrivate ?? defaultFilters.includePrivate,
        includeArchived: parsed.includeArchived ?? defaultFilters.includeArchived,
        onlyWithPrs: parsed.onlyWithPrs ?? defaultFilters.onlyWithPrs,
      };
    }
  } catch {
    // localStorage unavailable or invalid JSON - use defaults
  }
  return defaultFilters;
}

function saveFilters(filters: RepositoryFilters): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(filters));
  } catch {
    // localStorage unavailable - filters won't persist across sessions
  }
}

export function useRepositoryFilters() {
  const [filters, setFiltersState] = useState<RepositoryFilters>(loadFilters);

  const setFilters = useCallback((newFilters: RepositoryFilters) => {
    setFiltersState(newFilters);
    saveFilters(newFilters);
  }, []);

  const filterRepositories = useCallback(
    (repositories: RepositoryWithStatus[]): RepositoryWithStatus[] => {
      return repositories.filter((repo) => {
        // Filter by privacy
        if (repo.isPrivate && !filters.includePrivate) {
          return false;
        }
        if (!repo.isPrivate && !filters.includePublic) {
          return false;
        }

        // Filter by archived status
        if (repo.isArchived && !filters.includeArchived) {
          return false;
        }

        // Filter to only show repos with open PRs
        if (filters.onlyWithPrs && repo.openPrCount === 0) {
          return false;
        }

        return true;
      });
    },
    [filters]
  );

  return {
    filters,
    setFilters,
    filterRepositories,
  };
}
