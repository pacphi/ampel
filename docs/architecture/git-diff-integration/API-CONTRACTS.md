# API Contracts: Git Diff Integration

**Document Version:** 1.0
**Date:** 2025-12-25
**Status:** Architecture Design

## Overview

This document defines the complete API contracts for git diff integration, including REST endpoints, request/response schemas, error codes, and versioning strategy.

## API Endpoints

### 1. Get Pull Request Diff

**Endpoint:**

```
GET /api/v1/pull-requests/{pull_request_id}/diff
```

**Description:** Fetch complete diff for a pull request, including all changed files and their patches.

**Authentication:** Required (JWT Bearer token)

**Path Parameters:**

| Parameter         | Type | Required | Description                     |
| ----------------- | ---- | -------- | ------------------------------- |
| `pull_request_id` | UUID | Yes      | Pull request UUID from database |

**Query Parameters:**

| Parameter           | Type    | Required | Default | Description                          |
| ------------------- | ------- | -------- | ------- | ------------------------------------ |
| `view_type`         | String  | No       | unified | Diff view type: `unified` or `split` |
| `context_lines`     | Integer | No       | 3       | Number of context lines (0-10)       |
| `ignore_whitespace` | Boolean | No       | false   | Ignore whitespace changes            |

**Request Headers:**

```http
GET /api/v1/pull-requests/123e4567-e89b-12d3-a456-426614174000/diff HTTP/1.1
Host: api.ampel.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
```

**Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "pullRequestId": "123e4567-e89b-12d3-a456-426614174000",
    "provider": "github",
    "repositoryId": "456e7890-e89b-12d3-a456-426614174000",
    "files": [
      {
        "id": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
        "oldPath": null,
        "newPath": "src/components/Button.tsx",
        "status": "modified",
        "additions": 50,
        "deletions": 10,
        "changes": 60,
        "patch": "@@ -1,4 +1,5 @@\n import React from 'react';\n+import { cn } from '@/lib/utils';\n\n export function Button({ children, className, ...props }: ButtonProps) {\n   return (\n-    <button className={className} {...props}>\n+    <button className={cn('btn', className)} {...props}>\n       {children}\n     </button>\n   );\n }",
        "language": "typescript",
        "isBinary": false,
        "isTruncated": false
      },
      {
        "id": "abc123def456",
        "oldPath": "old/logo.png",
        "newPath": "new/logo.png",
        "status": "renamed",
        "additions": 0,
        "deletions": 0,
        "changes": 0,
        "patch": "",
        "language": null,
        "isBinary": true,
        "isTruncated": false
      }
    ],
    "totalAdditions": 104,
    "totalDeletions": 23,
    "totalFiles": 5,
    "baseCommit": "abc123def456",
    "headCommit": "def456ghi789",
    "fetchedAt": "2025-12-25T10:00:00Z",
    "cachedAt": "2025-12-25T09:55:00Z"
  }
}
```

**Response Fields:**

| Field            | Type     | Description                                      |
| ---------------- | -------- | ------------------------------------------------ |
| `pullRequestId`  | UUID     | Pull request identifier                          |
| `provider`       | String   | Git provider: `github`, `gitlab`, or `bitbucket` |
| `repositoryId`   | UUID     | Repository identifier                            |
| `files`          | Array    | List of changed files (see File schema below)    |
| `totalAdditions` | Integer  | Total lines added across all files               |
| `totalDeletions` | Integer  | Total lines deleted across all files             |
| `totalFiles`     | Integer  | Number of files changed                          |
| `baseCommit`     | String   | Base commit SHA (merge target)                   |
| `headCommit`     | String   | Head commit SHA (PR branch)                      |
| `fetchedAt`      | ISO 8601 | When diff was fetched from provider              |
| `cachedAt`       | ISO 8601 | When diff was cached (if served from cache)      |

**File Schema:**

| Field         | Type    | Description                                             |
| ------------- | ------- | ------------------------------------------------------- |
| `id`          | String  | Git object SHA for this file                            |
| `oldPath`     | String? | Original file path (for renames/deletes), null if added |
| `newPath`     | String  | New file path                                           |
| `status`      | Enum    | `added`, `deleted`, `modified`, `renamed`, `copied`     |
| `additions`   | Integer | Lines added in this file                                |
| `deletions`   | Integer | Lines deleted in this file                              |
| `changes`     | Integer | Total changes (additions + deletions)                   |
| `patch`       | String  | Unified diff patch (git diff format)                    |
| `language`    | String? | Detected language for syntax highlighting               |
| `isBinary`    | Boolean | True if file is binary (image, executable, etc.)        |
| `isTruncated` | Boolean | True if patch was truncated (file too large)            |

**Error Responses:**

**401 Unauthorized** - Invalid or expired JWT token:

```json
{
  "success": false,
  "error": "Invalid authentication token"
}
```

**403 Forbidden** - User lacks access to this PR:

```json
{
  "success": false,
  "error": "You do not have permission to view this pull request"
}
```

**404 Not Found** - Pull request not found:

```json
{
  "success": false,
  "error": "Pull request not found"
}
```

**429 Too Many Requests** - Rate limit exceeded:

```json
{
  "success": false,
  "error": "Rate limit exceeded. Retry after 60 seconds.",
  "retryAfter": 60
}
```

**500 Internal Server Error** - Provider API failure:

```json
{
  "success": false,
  "error": "Failed to fetch diff from GitHub. Please try again."
}
```

**503 Service Unavailable** - Provider temporarily down:

```json
{
  "success": false,
  "error": "GitHub provider is temporarily unavailable. Please try again in a few minutes.",
  "retryAfter": 120
}
```

---

### 2. Refresh Pull Request Diff

**Endpoint:**

```
POST /api/v1/pull-requests/{pull_request_id}/diff/refresh
```

**Description:** Force re-fetch diff from provider, bypassing cache.

**Authentication:** Required

**Request:**

```http
POST /api/v1/pull-requests/123e4567-e89b-12d3-a456-426614174000/diff/refresh HTTP/1.1
Host: api.ampel.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
```

**Response (200 OK):**

Same as GET /diff endpoint, but always returns fresh data from provider.

**Response (202 Accepted)** - For large diffs (100+ files):

```json
{
  "success": true,
  "data": {
    "jobId": "job_abc123",
    "status": "processing",
    "message": "Diff refresh queued. Check status at /api/v1/jobs/job_abc123"
  }
}
```

---

## TypeScript Client SDK

**Auto-generated from OpenAPI spec:**

```typescript
// frontend/src/api/generated/diff.ts

import { apiClient } from '../client';

export interface PullRequestDiff {
  pullRequestId: string;
  provider: 'github' | 'gitlab' | 'bitbucket';
  repositoryId: string;
  files: DiffFile[];
  totalAdditions: number;
  totalDeletions: number;
  totalFiles: number;
  baseCommit: string;
  headCommit: string;
  fetchedAt: Date;
  cachedAt?: Date;
}

export interface DiffFile {
  id: string;
  oldPath: string | null;
  newPath: string;
  status: 'added' | 'deleted' | 'modified' | 'renamed' | 'copied';
  additions: number;
  deletions: number;
  changes: number;
  patch: string;
  language: string | null;
  isBinary: boolean;
  isTruncated: boolean;
}

export interface GetDiffOptions {
  viewType?: 'unified' | 'split';
  contextLines?: number;
  ignoreWhitespace?: boolean;
}

export const diffApi = {
  /**
   * Get pull request diff
   */
  async getDiff(pullRequestId: string, options?: GetDiffOptions): Promise<PullRequestDiff> {
    const params = new URLSearchParams();
    if (options?.viewType) params.set('view_type', options.viewType);
    if (options?.contextLines !== undefined)
      params.set('context_lines', options.contextLines.toString());
    if (options?.ignoreWhitespace) params.set('ignore_whitespace', 'true');

    const response = await apiClient.get<ApiResponse<PullRequestDiff>>(
      `/pull-requests/${pullRequestId}/diff?${params}`
    );

    return {
      ...response.data.data!,
      fetchedAt: new Date(response.data.data!.fetchedAt),
      cachedAt: response.data.data!.cachedAt ? new Date(response.data.data!.cachedAt) : undefined,
    };
  },

  /**
   * Refresh pull request diff (bypass cache)
   */
  async refreshDiff(pullRequestId: string): Promise<PullRequestDiff> {
    const response = await apiClient.post<ApiResponse<PullRequestDiff>>(
      `/pull-requests/${pullRequestId}/diff/refresh`
    );

    return {
      ...response.data.data!,
      fetchedAt: new Date(response.data.data!.fetchedAt),
    };
  },
};
```

## OpenAPI Specification

```yaml
# docs/api/openapi.yaml

openapi: 3.0.3
info:
  title: Ampel API - Git Diff Integration
  version: 1.0.0
  description: REST API for git diff viewing

paths:
  /api/v1/pull-requests/{pull_request_id}/diff:
    get:
      summary: Get pull request diff
      tags:
        - Diffs
      security:
        - BearerAuth: []
      parameters:
        - name: pull_request_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
        - name: view_type
          in: query
          schema:
            type: string
            enum: [unified, split]
            default: unified
        - name: context_lines
          in: query
          schema:
            type: integer
            minimum: 0
            maximum: 10
            default: 3
        - name: ignore_whitespace
          in: query
          schema:
            type: boolean
            default: false

      responses:
        '200':
          description: Diff fetched successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  success:
                    type: boolean
                    example: true
                  data:
                    $ref: '#/components/schemas/PullRequestDiff'

        '401':
          $ref: '#/components/responses/Unauthorized'
        '403':
          $ref: '#/components/responses/Forbidden'
        '404':
          $ref: '#/components/responses/NotFound'
        '429':
          $ref: '#/components/responses/RateLimitExceeded'
        '500':
          $ref: '#/components/responses/InternalServerError'
        '503':
          $ref: '#/components/responses/ServiceUnavailable'

components:
  schemas:
    PullRequestDiff:
      type: object
      required:
        - pullRequestId
        - provider
        - repositoryId
        - files
        - totalAdditions
        - totalDeletions
        - totalFiles
        - baseCommit
        - headCommit
        - fetchedAt
      properties:
        pullRequestId:
          type: string
          format: uuid
        provider:
          type: string
          enum: [github, gitlab, bitbucket]
        repositoryId:
          type: string
          format: uuid
        files:
          type: array
          items:
            $ref: '#/components/schemas/DiffFile'
        totalAdditions:
          type: integer
          minimum: 0
        totalDeletions:
          type: integer
          minimum: 0
        totalFiles:
          type: integer
          minimum: 0
        baseCommit:
          type: string
          pattern: '^[a-f0-9]{40}$'
        headCommit:
          type: string
          pattern: '^[a-f0-9]{40}$'
        fetchedAt:
          type: string
          format: date-time
        cachedAt:
          type: string
          format: date-time

    DiffFile:
      type: object
      required:
        - id
        - newPath
        - status
        - additions
        - deletions
        - changes
        - patch
        - isBinary
        - isTruncated
      properties:
        id:
          type: string
          pattern: '^[a-f0-9]{40}$'
        oldPath:
          type: string
          nullable: true
        newPath:
          type: string
        status:
          type: string
          enum: [added, deleted, modified, renamed, copied]
        additions:
          type: integer
          minimum: 0
        deletions:
          type: integer
          minimum: 0
        changes:
          type: integer
          minimum: 0
        patch:
          type: string
        language:
          type: string
          nullable: true
        isBinary:
          type: boolean
        isTruncated:
          type: boolean

  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT

  responses:
    Unauthorized:
      description: Unauthorized - Invalid or expired token
      content:
        application/json:
          schema:
            type: object
            properties:
              success:
                type: boolean
                example: false
              error:
                type: string
                example: Invalid authentication token

    Forbidden:
      description: Forbidden - Insufficient permissions
      content:
        application/json:
          schema:
            type: object
            properties:
              success:
                type: boolean
                example: false
              error:
                type: string
                example: You do not have permission to view this pull request

    NotFound:
      description: Not Found - Resource does not exist
      content:
        application/json:
          schema:
            type: object
            properties:
              success:
                type: boolean
                example: false
              error:
                type: string
                example: Pull request not found

    RateLimitExceeded:
      description: Rate limit exceeded
      content:
        application/json:
          schema:
            type: object
            properties:
              success:
                type: boolean
                example: false
              error:
                type: string
                example: Rate limit exceeded. Retry after 60 seconds.
              retryAfter:
                type: integer
                example: 60

    InternalServerError:
      description: Internal server error
      content:
        application/json:
          schema:
            type: object
            properties:
              success:
                type: boolean
                example: false
              error:
                type: string
                example: Failed to fetch diff from provider

    ServiceUnavailable:
      description: Service temporarily unavailable
      content:
        application/json:
          schema:
            type: object
            properties:
              success:
                type: boolean
                example: false
              error:
                type: string
                example: Provider is temporarily unavailable
              retryAfter:
                type: integer
                example: 120
```

## Versioning Strategy

### API Versioning

- **URL Versioning**: `/api/v1/...`, `/api/v2/...`
- **Backward Compatibility**: v1 supported for minimum 12 months after v2 release
- **Deprecation Warnings**: `Deprecation: true` header on deprecated endpoints
- **Migration Guide**: Provided 90 days before deprecation

### Schema Evolution

**Additive Changes (Non-Breaking):**

- Adding new optional fields ✓
- Adding new enum values ✓
- Adding new endpoints ✓

**Breaking Changes (Require New Version):**

- Removing fields ✗
- Changing field types ✗
- Renaming fields ✗
- Making optional fields required ✗

**Example Migration:**

```json
// v1: oldPath is string | null
{
  "oldPath": null
}

// v2: oldPath + previousPath for clarity
{
  "oldPath": null,
  "previousPath": null  // New field, backward compatible
}
```

## Related Documents

- [ADR-002: Provider Diff Abstraction](/docs/architecture/git-diff-integration/ADR-002-provider-diff-abstraction.md)
- [Data Transformation Flow](/docs/architecture/git-diff-integration/DATA-TRANSFORMATION-FLOW.md)
