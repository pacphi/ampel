import { describe, expect, it, vi, beforeEach } from 'vitest';
import { repositoriesApi } from './repositories';
import apiClient from './client';
import type { RepositoryWithStatus, DiscoveredRepository, Repository } from '@/types';

// Mock the API client
vi.mock('./client', () => ({
  default: {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
  },
}));

const mockedApiClient = vi.mocked(apiClient);

describe('repositoriesApi', () => {
  const mockRepository: RepositoryWithStatus = {
    id: 'repo-1',
    userId: 'user-1',
    provider: 'github',
    providerId: '12345',
    owner: 'testuser',
    name: 'test-repo',
    fullName: 'testuser/test-repo',
    description: 'A test repository',
    url: 'https://github.com/testuser/test-repo',
    defaultBranch: 'main',
    isPrivate: false,
    isArchived: false,
    pollIntervalSeconds: 300,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-02T00:00:00Z',
    status: 'green',
    openPrCount: 3,
  };

  const mockDiscoveredRepo: DiscoveredRepository = {
    provider: 'github',
    providerId: '12345',
    owner: 'testuser',
    name: 'new-repo',
    fullName: 'testuser/new-repo',
    description: 'A newly discovered repository',
    url: 'https://github.com/testuser/new-repo',
    defaultBranch: 'main',
    isPrivate: false,
    isArchived: false,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('list', () => {
    it('should list all repositories', async () => {
      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: [mockRepository] },
      });

      const result = await repositoriesApi.list();

      expect(mockedApiClient.get).toHaveBeenCalledWith('/repositories');
      expect(result).toEqual([mockRepository]);
    });

    it('should return empty array when no repositories', async () => {
      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: [] },
      });

      const result = await repositoriesApi.list();

      expect(result).toEqual([]);
    });
  });

  describe('get', () => {
    it('should get a specific repository', async () => {
      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: mockRepository },
      });

      const result = await repositoriesApi.get('repo-1');

      expect(mockedApiClient.get).toHaveBeenCalledWith('/repositories/repo-1');
      expect(result).toEqual(mockRepository);
    });
  });

  describe('discover', () => {
    it('should discover repositories from GitHub', async () => {
      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: [mockDiscoveredRepo] },
      });

      const result = await repositoriesApi.discover('github');

      expect(mockedApiClient.get).toHaveBeenCalledWith('/repositories/discover', {
        params: { provider: 'github' },
      });
      expect(result).toEqual([mockDiscoveredRepo]);
    });

    it('should discover repositories from GitLab', async () => {
      const gitlabRepo = { ...mockDiscoveredRepo, provider: 'gitlab' as const };
      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: [gitlabRepo] },
      });

      const result = await repositoriesApi.discover('gitlab');

      expect(mockedApiClient.get).toHaveBeenCalledWith('/repositories/discover', {
        params: { provider: 'gitlab' },
      });
      expect(result).toEqual([gitlabRepo]);
    });

    it('should discover repositories from Bitbucket', async () => {
      const bitbucketRepo = { ...mockDiscoveredRepo, provider: 'bitbucket' as const };
      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: [bitbucketRepo] },
      });

      const result = await repositoriesApi.discover('bitbucket');

      expect(mockedApiClient.get).toHaveBeenCalledWith('/repositories/discover', {
        params: { provider: 'bitbucket' },
      });
      expect(result).toEqual([bitbucketRepo]);
    });
  });

  describe('add', () => {
    it('should add a repository with default poll interval', async () => {
      const newRepo: Repository = {
        id: 'repo-new',
        userId: 'user-1',
        provider: 'github',
        providerId: '12345',
        owner: 'testuser',
        name: 'new-repo',
        fullName: 'testuser/new-repo',
        url: 'https://github.com/testuser/new-repo',
        defaultBranch: 'main',
        isPrivate: false,
        isArchived: false,
        pollIntervalSeconds: 300,
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: newRepo },
      });

      const result = await repositoriesApi.add('github', 'testuser', 'new-repo');

      expect(mockedApiClient.post).toHaveBeenCalledWith('/repositories', {
        provider: 'github',
        owner: 'testuser',
        name: 'new-repo',
        pollIntervalSeconds: undefined,
      });
      expect(result).toEqual(newRepo);
    });

    it('should add a repository with custom poll interval', async () => {
      const newRepo: Repository = {
        id: 'repo-new',
        userId: 'user-1',
        provider: 'github',
        providerId: '12345',
        owner: 'testuser',
        name: 'new-repo',
        fullName: 'testuser/new-repo',
        url: 'https://github.com/testuser/new-repo',
        defaultBranch: 'main',
        isPrivate: false,
        isArchived: false,
        pollIntervalSeconds: 600,
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: newRepo },
      });

      const result = await repositoriesApi.add('github', 'testuser', 'new-repo', 600);

      expect(mockedApiClient.post).toHaveBeenCalledWith('/repositories', {
        provider: 'github',
        owner: 'testuser',
        name: 'new-repo',
        pollIntervalSeconds: 600,
      });
      expect(result).toEqual(newRepo);
    });
  });

  describe('update', () => {
    it('should update repository poll interval', async () => {
      const updatedRepo: Repository = {
        ...mockRepository,
        pollIntervalSeconds: 600,
      };

      mockedApiClient.put.mockResolvedValueOnce({
        data: { success: true, data: updatedRepo },
      });

      const result = await repositoriesApi.update('repo-1', { pollIntervalSeconds: 600 });

      expect(mockedApiClient.put).toHaveBeenCalledWith('/repositories/repo-1', {
        pollIntervalSeconds: 600,
      });
      expect(result).toEqual(updatedRepo);
    });
  });

  describe('remove', () => {
    it('should remove a repository', async () => {
      mockedApiClient.delete.mockResolvedValueOnce({
        data: { success: true },
      });

      await repositoriesApi.remove('repo-1');

      expect(mockedApiClient.delete).toHaveBeenCalledWith('/repositories/repo-1');
    });
  });
});
