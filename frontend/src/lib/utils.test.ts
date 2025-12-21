import { describe, expect, it, beforeEach, afterEach, vi } from 'vitest';
import {
  cn,
  formatDate,
  formatRelativeTime,
  getProviderIcon,
  getStatusColor,
  getStatusBgColor,
} from './utils';

describe('utils', () => {
  describe('cn', () => {
    it('merges class names', () => {
      expect(cn('foo', 'bar')).toBe('foo bar');
    });

    it('handles conditional classes', () => {
      const showBar = false;
      const showBarTrue = true;
      expect(cn('foo', showBar && 'bar', 'baz')).toBe('foo baz');
      expect(cn('foo', showBarTrue && 'bar', 'baz')).toBe('foo bar baz');
    });

    it('merges tailwind classes correctly', () => {
      expect(cn('px-2 py-1', 'px-4')).toBe('py-1 px-4');
    });

    it('handles undefined and null values', () => {
      expect(cn('foo', undefined, null, 'bar')).toBe('foo bar');
    });

    it('handles empty input', () => {
      expect(cn()).toBe('');
    });

    it('handles arrays of classes', () => {
      expect(cn(['foo', 'bar'])).toBe('foo bar');
    });

    it('handles object syntax', () => {
      expect(cn({ foo: true, bar: false, baz: true })).toBe('foo baz');
    });
  });

  describe('formatDate', () => {
    it('formats a Date object', () => {
      const date = new Date('2024-01-15T12:00:00Z');
      const result = formatDate(date);
      expect(result).toContain('Jan');
      expect(result).toContain('2024');
      // Date may show as 14 or 15 depending on timezone, so we check for either
      expect(result).toMatch(/14|15/);
    });

    it('formats a date string', () => {
      const result = formatDate('2024-06-20T12:00:00Z');
      expect(result).toContain('Jun');
      expect(result).toContain('2024');
      // Date may show as 19 or 20 depending on timezone
      expect(result).toMatch(/19|20/);
    });

    it('formats ISO date strings', () => {
      const result = formatDate('2024-12-25T10:30:00Z');
      expect(result).toContain('Dec');
      expect(result).toContain('2024');
      // Date may show as 24 or 25 depending on timezone
      expect(result).toMatch(/24|25/);
    });
  });

  describe('formatRelativeTime', () => {
    beforeEach(() => {
      // Mock Date to have consistent test results
      const mockNow = new Date('2024-06-15T12:00:00Z');
      vi.useFakeTimers();
      vi.setSystemTime(mockNow);
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('returns empty string for null input', () => {
      expect(formatRelativeTime(null)).toBe('');
    });

    it('returns empty string for undefined input', () => {
      expect(formatRelativeTime(undefined)).toBe('');
    });

    it('returns empty string for invalid date', () => {
      expect(formatRelativeTime('invalid-date')).toBe('');
    });

    it('returns "just now" for times less than 60 seconds ago', () => {
      const date = new Date('2024-06-15T11:59:30Z');
      expect(formatRelativeTime(date)).toBe('just now');
    });

    it('returns minutes ago for times less than 60 minutes ago', () => {
      const date = new Date('2024-06-15T11:30:00Z');
      expect(formatRelativeTime(date)).toBe('30m ago');
    });

    it('returns hours ago for times less than 24 hours ago', () => {
      const date = new Date('2024-06-15T06:00:00Z');
      expect(formatRelativeTime(date)).toBe('6h ago');
    });

    it('returns days ago for times less than 7 days ago', () => {
      const date = new Date('2024-06-12T12:00:00Z');
      expect(formatRelativeTime(date)).toBe('3d ago');
    });

    it('returns formatted date for times 7 or more days ago', () => {
      const date = new Date('2024-06-01T12:00:00Z');
      const result = formatRelativeTime(date);
      expect(result).toContain('Jun');
      expect(result).toContain('1');
      expect(result).toContain('2024');
    });

    it('handles string dates', () => {
      const result = formatRelativeTime('2024-06-15T11:55:00Z');
      expect(result).toBe('5m ago');
    });
  });

  describe('getProviderIcon', () => {
    it('returns Github for github provider', () => {
      expect(getProviderIcon('github')).toBe('Github');
      expect(getProviderIcon('GitHub')).toBe('Github');
      expect(getProviderIcon('GITHUB')).toBe('Github');
    });

    it('returns Gitlab for gitlab provider', () => {
      expect(getProviderIcon('gitlab')).toBe('Gitlab');
      expect(getProviderIcon('GitLab')).toBe('Gitlab');
      expect(getProviderIcon('GITLAB')).toBe('Gitlab');
    });

    it('returns Boxes for bitbucket provider', () => {
      expect(getProviderIcon('bitbucket')).toBe('Boxes');
      expect(getProviderIcon('Bitbucket')).toBe('Boxes');
      expect(getProviderIcon('BITBUCKET')).toBe('Boxes');
    });

    it('returns GitBranch for unknown provider', () => {
      expect(getProviderIcon('unknown')).toBe('GitBranch');
      expect(getProviderIcon('')).toBe('GitBranch');
      expect(getProviderIcon('azure')).toBe('GitBranch');
    });
  });

  describe('getStatusColor', () => {
    it('returns correct color for green status', () => {
      expect(getStatusColor('green')).toBe('text-ampel-green');
      expect(getStatusColor('Green')).toBe('text-ampel-green');
      expect(getStatusColor('GREEN')).toBe('text-ampel-green');
    });

    it('returns correct color for yellow status', () => {
      expect(getStatusColor('yellow')).toBe('text-ampel-yellow');
      expect(getStatusColor('Yellow')).toBe('text-ampel-yellow');
      expect(getStatusColor('YELLOW')).toBe('text-ampel-yellow');
    });

    it('returns correct color for red status', () => {
      expect(getStatusColor('red')).toBe('text-ampel-red');
      expect(getStatusColor('Red')).toBe('text-ampel-red');
      expect(getStatusColor('RED')).toBe('text-ampel-red');
    });

    it('returns muted color for unknown status', () => {
      expect(getStatusColor('unknown')).toBe('text-muted-foreground');
      expect(getStatusColor('')).toBe('text-muted-foreground');
      expect(getStatusColor('none')).toBe('text-muted-foreground');
    });
  });

  describe('getStatusBgColor', () => {
    it('returns correct background color for green status', () => {
      expect(getStatusBgColor('green')).toBe('bg-ampel-green');
      expect(getStatusBgColor('Green')).toBe('bg-ampel-green');
      expect(getStatusBgColor('GREEN')).toBe('bg-ampel-green');
    });

    it('returns correct background color for yellow status', () => {
      expect(getStatusBgColor('yellow')).toBe('bg-ampel-yellow');
      expect(getStatusBgColor('Yellow')).toBe('bg-ampel-yellow');
      expect(getStatusBgColor('YELLOW')).toBe('bg-ampel-yellow');
    });

    it('returns correct background color for red status', () => {
      expect(getStatusBgColor('red')).toBe('bg-ampel-red');
      expect(getStatusBgColor('Red')).toBe('bg-ampel-red');
      expect(getStatusBgColor('RED')).toBe('bg-ampel-red');
    });

    it('returns muted background for unknown status', () => {
      expect(getStatusBgColor('unknown')).toBe('bg-muted');
      expect(getStatusBgColor('')).toBe('bg-muted');
      expect(getStatusBgColor('none')).toBe('bg-muted');
    });
  });
});
