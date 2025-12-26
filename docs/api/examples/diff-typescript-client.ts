/**
 * TypeScript Client for Ampel Git Diff API
 *
 * Usage:
 *   const client = new AmpelDiffClient('http://localhost:8080', 'your-jwt-token');
 *   const diff = await client.getPullRequestDiff('pr-uuid-here');
 */

// Type definitions
export interface DiffFile {
  sha: string;
  old_path: string | null;
  new_path: string;
  status: 'added' | 'deleted' | 'modified' | 'renamed' | 'copied';
  additions: number;
  deletions: number;
  changes: number;
  patch: string;
}

export interface DiffResponse {
  files: DiffFile[];
  total_additions: number;
  total_deletions: number;
  total_files: number;
  base_commit: string;
  head_commit: string;
}

export interface DiffMetadata {
  provider: 'github' | 'gitlab' | 'bitbucket';
  cached: boolean;
  cache_age_seconds: number;
  timestamp: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T;
  metadata: DiffMetadata;
}

export interface ApiError {
  code: string;
  message: string;
  details: Record<string, any> | null;
}

export interface ApiErrorResponse {
  success: false;
  error: ApiError;
}

export interface DiffRequestOptions {
  format?: 'unified' | 'split';
  context?: number;
  bypassCache?: boolean;
}

// Client implementation
export class AmpelDiffClient {
  private baseUrl: string;
  private token: string;
  private defaultOptions: RequestInit;

  constructor(baseUrl: string, token: string) {
    this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.token = token;
    this.defaultOptions = {
      headers: {
        Authorization: `Bearer ${this.token}`,
        'Content-Type': 'application/json',
        Accept: 'application/json',
      },
    };
  }

  /**
   * Fetch pull request diff from Ampel API
   */
  async getPullRequestDiff(prId: string, options: DiffRequestOptions = {}): Promise<DiffResponse> {
    const params = new URLSearchParams();

    if (options.format) {
      params.append('format', options.format);
    }

    if (options.context !== undefined) {
      params.append('context', options.context.toString());
    }

    const headers = { ...this.defaultOptions.headers } as HeadersInit;

    if (options.bypassCache) {
      headers['Cache-Control'] = 'no-cache';
    }

    const queryString = params.toString();
    const url = `${this.baseUrl}/api/v1/pull-requests/${prId}/diff${
      queryString ? `?${queryString}` : ''
    }`;

    const response = await fetch(url, {
      ...this.defaultOptions,
      headers,
    });

    if (!response.ok) {
      const errorData: ApiErrorResponse = await response.json();
      throw new DiffApiError(
        errorData.error.message,
        response.status,
        errorData.error.code,
        errorData.error.details
      );
    }

    const data: ApiResponse<DiffResponse> = await response.json();
    return data.data;
  }

  /**
   * Get diff with metadata (includes cache info)
   */
  async getPullRequestDiffWithMetadata(
    prId: string,
    options: DiffRequestOptions = {}
  ): Promise<ApiResponse<DiffResponse>> {
    const params = new URLSearchParams();

    if (options.format) {
      params.append('format', options.format);
    }

    if (options.context !== undefined) {
      params.append('context', options.context.toString());
    }

    const headers = { ...this.defaultOptions.headers } as HeadersInit;

    if (options.bypassCache) {
      headers['Cache-Control'] = 'no-cache';
    }

    const queryString = params.toString();
    const url = `${this.baseUrl}/api/v1/pull-requests/${prId}/diff${
      queryString ? `?${queryString}` : ''
    }`;

    const response = await fetch(url, {
      ...this.defaultOptions,
      headers,
    });

    if (!response.ok) {
      const errorData: ApiErrorResponse = await response.json();
      throw new DiffApiError(
        errorData.error.message,
        response.status,
        errorData.error.code,
        errorData.error.details
      );
    }

    return response.json();
  }

  /**
   * Check if diff is cached (lightweight HEAD request)
   */
  async isDiffCached(prId: string): Promise<boolean> {
    const url = `${this.baseUrl}/api/v1/pull-requests/${prId}/diff`;

    const response = await fetch(url, {
      method: 'HEAD',
      headers: this.defaultOptions.headers as HeadersInit,
    });

    const cacheStatus = response.headers.get('X-Cache-Status');
    return cacheStatus === 'HIT';
  }

  /**
   * Update authentication token
   */
  setToken(token: string): void {
    this.token = token;
    this.defaultOptions.headers = {
      ...this.defaultOptions.headers,
      Authorization: `Bearer ${token}`,
    };
  }
}

// Custom error class
export class DiffApiError extends Error {
  constructor(
    message: string,
    public statusCode: number,
    public errorCode: string,
    public details: Record<string, any> | null
  ) {
    super(message);
    this.name = 'DiffApiError';
  }

  isUnauthorized(): boolean {
    return this.statusCode === 401;
  }

  isNotFound(): boolean {
    return this.statusCode === 404;
  }

  isRateLimited(): boolean {
    return this.statusCode === 429;
  }

  isServerError(): boolean {
    return this.statusCode >= 500;
  }
}

// Example usage
async function example() {
  const client = new AmpelDiffClient('http://localhost:8080', 'eyJhbGciOiJIUzI1NiIs...');

  try {
    // Basic usage
    const diff = await client.getPullRequestDiff('550e8400-e29b-41d4-a716-446655440000');
    console.log(`Total files changed: ${diff.total_files}`);
    console.log(`Lines added: ${diff.total_additions}`);
    console.log(`Lines deleted: ${diff.total_deletions}`);

    // With options
    const splitDiff = await client.getPullRequestDiff('550e8400-e29b-41d4-a716-446655440000', {
      format: 'split',
      context: 5,
      bypassCache: true,
    });

    // With metadata
    const { data, metadata } = await client.getPullRequestDiffWithMetadata(
      '550e8400-e29b-41d4-a716-446655440000'
    );
    console.log(`Provider: ${metadata.provider}`);
    console.log(`Cached: ${metadata.cached}`);
    console.log(`Cache age: ${metadata.cache_age_seconds}s`);

    // Check cache status
    const isCached = await client.isDiffCached('550e8400-e29b-41d4-a716-446655440000');
    console.log(`Diff is cached: ${isCached}`);
  } catch (error) {
    if (error instanceof DiffApiError) {
      console.error(`API Error [${error.errorCode}]: ${error.message}`);

      if (error.isUnauthorized()) {
        console.log('Please log in again');
      } else if (error.isNotFound()) {
        console.log('Pull request not found');
      } else if (error.isRateLimited()) {
        console.log('Rate limit exceeded, please wait');
      }

      if (error.details) {
        console.log('Error details:', error.details);
      }
    } else {
      console.error('Unexpected error:', error);
    }
  }
}
