import { describe, expect, it, vi, beforeEach } from 'vitest';
import { authApi } from './auth';
import apiClient from './client';

// Mock the API client
vi.mock('./client', () => ({
  default: {
    post: vi.fn(),
    get: vi.fn(),
    put: vi.fn(),
  },
}));

const mockedApiClient = vi.mocked(apiClient, true);

describe('authApi', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('register', () => {
    it('should register a new user with all fields', async () => {
      const mockTokens = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        tokenType: 'Bearer',
        expiresIn: 3600,
      };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: mockTokens },
      });

      const result = await authApi.register('test@example.com', 'password123', 'Test User');

      expect(mockedApiClient.post).toHaveBeenCalledWith('/auth/register', {
        email: 'test@example.com',
        password: 'password123',
        displayName: 'Test User',
      });
      expect(result).toEqual(mockTokens);
    });

    it('should register a new user without displayName', async () => {
      const mockTokens = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        tokenType: 'Bearer',
        expiresIn: 3600,
      };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: mockTokens },
      });

      const result = await authApi.register('test@example.com', 'password123');

      expect(mockedApiClient.post).toHaveBeenCalledWith('/auth/register', {
        email: 'test@example.com',
        password: 'password123',
        displayName: undefined,
      });
      expect(result).toEqual(mockTokens);
    });
  });

  describe('login', () => {
    it('should login user and return tokens', async () => {
      const mockTokens = {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        tokenType: 'Bearer',
        expiresIn: 3600,
      };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: mockTokens },
      });

      const result = await authApi.login('test@example.com', 'password123');

      expect(mockedApiClient.post).toHaveBeenCalledWith('/auth/login', {
        email: 'test@example.com',
        password: 'password123',
      });
      expect(result).toEqual(mockTokens);
    });
  });

  describe('refresh', () => {
    it('should refresh tokens', async () => {
      const mockTokens = {
        accessToken: 'new-access-token',
        refreshToken: 'new-refresh-token',
        tokenType: 'Bearer',
        expiresIn: 3600,
      };

      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true, data: mockTokens },
      });

      const result = await authApi.refresh('old-refresh-token');

      expect(mockedApiClient.post).toHaveBeenCalledWith('/auth/refresh', {
        refreshToken: 'old-refresh-token',
      });
      expect(result).toEqual(mockTokens);
    });
  });

  describe('me', () => {
    it('should get current user profile', async () => {
      const mockUser = {
        id: '1',
        email: 'test@example.com',
        displayName: 'Test User',
        createdAt: '2024-01-01T00:00:00Z',
      };

      mockedApiClient.get.mockResolvedValueOnce({
        data: { success: true, data: mockUser },
      });

      const result = await authApi.me();

      expect(mockedApiClient.get).toHaveBeenCalledWith('/auth/me');
      expect(result).toEqual(mockUser);
    });
  });

  describe('logout', () => {
    it('should call logout endpoint', async () => {
      mockedApiClient.post.mockResolvedValueOnce({
        data: { success: true },
      });

      await authApi.logout();

      expect(mockedApiClient.post).toHaveBeenCalledWith('/auth/logout');
    });
  });

  describe('updateProfile', () => {
    it('should update user profile with email', async () => {
      const mockUser = {
        id: '1',
        email: 'new@example.com',
        displayName: 'Test User',
        createdAt: '2024-01-01T00:00:00Z',
      };

      mockedApiClient.put.mockResolvedValueOnce({
        data: { success: true, data: mockUser },
      });

      const result = await authApi.updateProfile({ email: 'new@example.com' });

      expect(mockedApiClient.put).toHaveBeenCalledWith('/auth/me', {
        email: 'new@example.com',
      });
      expect(result).toEqual(mockUser);
    });

    it('should update user profile with displayName', async () => {
      const mockUser = {
        id: '1',
        email: 'test@example.com',
        displayName: 'New Name',
        createdAt: '2024-01-01T00:00:00Z',
      };

      mockedApiClient.put.mockResolvedValueOnce({
        data: { success: true, data: mockUser },
      });

      const result = await authApi.updateProfile({ displayName: 'New Name' });

      expect(mockedApiClient.put).toHaveBeenCalledWith('/auth/me', {
        displayName: 'New Name',
      });
      expect(result).toEqual(mockUser);
    });

    it('should update user profile with both email and displayName', async () => {
      const mockUser = {
        id: '1',
        email: 'new@example.com',
        displayName: 'New Name',
        createdAt: '2024-01-01T00:00:00Z',
      };

      mockedApiClient.put.mockResolvedValueOnce({
        data: { success: true, data: mockUser },
      });

      const result = await authApi.updateProfile({
        email: 'new@example.com',
        displayName: 'New Name',
      });

      expect(mockedApiClient.put).toHaveBeenCalledWith('/auth/me', {
        email: 'new@example.com',
        displayName: 'New Name',
      });
      expect(result).toEqual(mockUser);
    });
  });
});
