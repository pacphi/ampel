import { describe, expect, it, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useRepositoryFilters } from './useRepositoryFilters';
import type { RepositoryWithStatus } from '@/types';

describe('useRepositoryFilters', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe('Initial State', () => {
    it('returns default filters initially', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      expect(result.current.filters).toEqual({
        includePublic: true,
        includePrivate: true,
        includeArchived: true,
      });
    });

    it('loads filters from localStorage on initialization', () => {
      const storedFilters = {
        includePublic: false,
        includePrivate: true,
        includeArchived: false,
      };
      localStorage.setItem('ampel-repository-filters', JSON.stringify(storedFilters));

      const { result } = renderHook(() => useRepositoryFilters());

      expect(result.current.filters).toEqual(storedFilters);
    });

    it('uses default filters when localStorage is empty', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      expect(result.current.filters.includePublic).toBe(true);
      expect(result.current.filters.includePrivate).toBe(true);
      expect(result.current.filters.includeArchived).toBe(true);
    });

    it('handles corrupted localStorage data gracefully', () => {
      localStorage.setItem('ampel-repository-filters', 'invalid-json');

      const { result } = renderHook(() => useRepositoryFilters());

      expect(result.current.filters).toEqual({
        includePublic: true,
        includePrivate: true,
        includeArchived: true,
      });
    });

    it('merges partial stored filters with defaults', () => {
      const partialFilters = { includePublic: false };
      localStorage.setItem('ampel-repository-filters', JSON.stringify(partialFilters));

      const { result } = renderHook(() => useRepositoryFilters());

      expect(result.current.filters).toEqual({
        includePublic: false,
        includePrivate: true,
        includeArchived: true,
      });
    });
  });

  describe('Filter Updates', () => {
    it('updates filters via setFilters', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      act(() => {
        result.current.setFilters({
          includePublic: false,
          includePrivate: true,
          includeArchived: false,
        });
      });

      expect(result.current.filters).toEqual({
        includePublic: false,
        includePrivate: true,
        includeArchived: false,
      });
    });

    it('persists filters to localStorage when updated', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      const newFilters = {
        includePublic: false,
        includePrivate: true,
        includeArchived: false,
      };

      act(() => {
        result.current.setFilters(newFilters);
      });

      const stored = localStorage.getItem('ampel-repository-filters');
      expect(stored).toBe(JSON.stringify(newFilters));
    });

    it('allows multiple filter updates', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      act(() => {
        result.current.setFilters({
          includePublic: false,
          includePrivate: true,
          includeArchived: true,
        });
      });

      act(() => {
        result.current.setFilters({
          includePublic: false,
          includePrivate: false,
          includeArchived: true,
        });
      });

      expect(result.current.filters).toEqual({
        includePublic: false,
        includePrivate: false,
        includeArchived: true,
      });
    });
  });

  describe('filterRepositories', () => {
    const createTestRepo = (
      id: string,
      isPrivate: boolean,
      isArchived: boolean
    ): RepositoryWithStatus => ({
      id,
      userId: 'user1',
      provider: 'github',
      providerId: `provider-${id}`,
      owner: 'owner',
      name: `repo-${id}`,
      fullName: `owner/repo-${id}`,
      description: 'Test repository',
      url: `https://github.com/owner/repo-${id}`,
      defaultBranch: 'main',
      isPrivate,
      isArchived,
      pollIntervalSeconds: 300,
      createdAt: '2025-01-01T00:00:00Z',
      updatedAt: '2025-01-01T00:00:00Z',
      status: 'green',
      openPrCount: 0,
    });

    it('returns all repositories when all filters are enabled', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      const repositories: RepositoryWithStatus[] = [
        createTestRepo('1', false, false), // public, not archived
        createTestRepo('2', true, false), // private, not archived
        createTestRepo('3', false, true), // public, archived
        createTestRepo('4', true, true), // private, archived
      ];

      const filtered = result.current.filterRepositories(repositories);

      expect(filtered).toHaveLength(4);
      expect(filtered).toEqual(repositories);
    });

    it('excludes public repositories when includePublic is false', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      act(() => {
        result.current.setFilters({
          includePublic: false,
          includePrivate: true,
          includeArchived: true,
        });
      });

      const repositories: RepositoryWithStatus[] = [
        createTestRepo('1', false, false), // public, not archived
        createTestRepo('2', true, false), // private, not archived
        createTestRepo('3', false, true), // public, archived
      ];

      const filtered = result.current.filterRepositories(repositories);

      expect(filtered).toHaveLength(1);
      expect(filtered[0].id).toBe('2');
      expect(filtered[0].isPrivate).toBe(true);
    });

    it('excludes private repositories when includePrivate is false', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      act(() => {
        result.current.setFilters({
          includePublic: true,
          includePrivate: false,
          includeArchived: true,
        });
      });

      const repositories: RepositoryWithStatus[] = [
        createTestRepo('1', false, false), // public, not archived
        createTestRepo('2', true, false), // private, not archived
        createTestRepo('3', false, true), // public, archived
      ];

      const filtered = result.current.filterRepositories(repositories);

      expect(filtered).toHaveLength(2);
      expect(filtered.map((r) => r.id)).toEqual(['1', '3']);
      expect(filtered.every((r) => !r.isPrivate)).toBe(true);
    });

    it('excludes archived repositories when includeArchived is false', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      act(() => {
        result.current.setFilters({
          includePublic: true,
          includePrivate: true,
          includeArchived: false,
        });
      });

      const repositories: RepositoryWithStatus[] = [
        createTestRepo('1', false, false), // public, not archived
        createTestRepo('2', true, false), // private, not archived
        createTestRepo('3', false, true), // public, archived
        createTestRepo('4', true, true), // private, archived
      ];

      const filtered = result.current.filterRepositories(repositories);

      expect(filtered).toHaveLength(2);
      expect(filtered.map((r) => r.id)).toEqual(['1', '2']);
      expect(filtered.every((r) => !r.isArchived)).toBe(true);
    });

    it('applies multiple filters correctly', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      act(() => {
        result.current.setFilters({
          includePublic: false,
          includePrivate: true,
          includeArchived: false,
        });
      });

      const repositories: RepositoryWithStatus[] = [
        createTestRepo('1', false, false), // public, not archived - excluded (public)
        createTestRepo('2', true, false), // private, not archived - included
        createTestRepo('3', false, true), // public, archived - excluded (public + archived)
        createTestRepo('4', true, true), // private, archived - excluded (archived)
      ];

      const filtered = result.current.filterRepositories(repositories);

      expect(filtered).toHaveLength(1);
      expect(filtered[0].id).toBe('2');
      expect(filtered[0].isPrivate).toBe(true);
      expect(filtered[0].isArchived).toBe(false);
    });

    it('returns empty array when no repositories match filters', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      act(() => {
        result.current.setFilters({
          includePublic: false,
          includePrivate: false,
          includeArchived: true,
        });
      });

      const repositories: RepositoryWithStatus[] = [
        createTestRepo('1', false, false),
        createTestRepo('2', true, false),
      ];

      const filtered = result.current.filterRepositories(repositories);

      expect(filtered).toHaveLength(0);
    });

    it('handles empty repository array', () => {
      const { result } = renderHook(() => useRepositoryFilters());

      const filtered = result.current.filterRepositories([]);

      expect(filtered).toHaveLength(0);
      expect(filtered).toEqual([]);
    });
  });
});
