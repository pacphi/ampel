# Dashboard Visibility Breakdown API Documentation

**Last Updated**: 2025-12-24
**Feature**: Repository Visibility Breakdown Tiles
**Status**: Implemented
**API Version**: v1

---

## Table of Contents

1. [Overview](#overview)
2. [Data Models](#data-models)
3. [API Endpoint](#api-endpoint)
4. [Usage Examples](#usage-examples)
5. [Backward Compatibility](#backward-compatibility)
6. [Provider-Specific Behavior](#provider-specific-behavior)

---

## Overview

The Dashboard Visibility Breakdown feature extends the `/api/dashboard/summary` endpoint to provide granular insights into repository and pull request distribution across visibility types (public, private, archived).

### Key Features

- **Repository Distribution**: See how many repositories are public, private, or archived
- **PR Distribution**: Understand which visibility types have the most open PRs
- **Status Breakdown**: Track ready-to-merge and needs-attention PRs by visibility
- **Provider Awareness**: Understand that Bitbucket doesn't support archived repositories

---

## Data Models

### VisibilityBreakdown

Represents a count breakdown by repository visibility type.

```rust
#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityBreakdown {
    /// Count of public (non-private, non-archived) items
    pub public: i32,

    /// Count of private (non-archived) items
    pub private: i32,

    /// Count of archived items (which may also be private)
    /// Note: Bitbucket does not support archived repositories
    pub archived: i32,
}
```

**Visibility Classification Rules:**

| Type         | Condition                                    |
| ------------ | -------------------------------------------- |
| **Public**   | `is_private = false AND is_archived = false` |
| **Private**  | `is_private = true AND is_archived = false`  |
| **Archived** | `is_archived = true` (may also be private)   |

### DashboardSummary (Extended)

The enhanced dashboard summary includes four new visibility breakdown fields:

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    /// Total number of repositories tracked by the user
    pub total_repositories: i32,

    /// Total number of open pull requests across all repositories
    pub total_open_prs: i32,

    /// Breakdown of pull requests by Ampel status (green/yellow/red)
    pub status_counts: StatusCounts,

    /// Breakdown of repositories by provider (GitHub/GitLab/Bitbucket)
    pub provider_counts: ProviderCounts,

    // NEW: Visibility breakdown fields
    /// Breakdown of repositories by visibility (public/private/archived)
    pub repository_breakdown: VisibilityBreakdown,

    /// Breakdown of open PRs by repository visibility
    pub open_prs_breakdown: VisibilityBreakdown,

    /// Breakdown of ready-to-merge PRs (green status) by repository visibility
    pub ready_to_merge_breakdown: VisibilityBreakdown,

    /// Breakdown of PRs needing attention (red status) by repository visibility
    pub needs_attention_breakdown: VisibilityBreakdown,
}
```

### StatusCounts

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCounts {
    /// Pull requests with green status (ready to merge)
    pub green: i32,

    /// Pull requests with yellow status (in progress)
    pub yellow: i32,

    /// Pull requests with red status (blocked)
    pub red: i32,
}
```

### ProviderCounts

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCounts {
    /// Number of GitHub repositories
    pub github: i32,

    /// Number of GitLab repositories
    pub gitlab: i32,

    /// Number of Bitbucket repositories
    pub bitbucket: i32,
}
```

---

## API Endpoint

### GET /api/dashboard/summary

Retrieves comprehensive dashboard statistics including repository counts, PR counts, status breakdowns, provider breakdowns, and visibility breakdowns.

**Authentication**: Required (JWT token)

**Request Headers**:

```
Authorization: Bearer <access_token>
```

**Response**: `200 OK`

```json
{
  "success": true,
  "data": {
    "totalRepositories": 42,
    "totalOpenPrs": 18,
    "statusCounts": {
      "green": 8,
      "yellow": 7,
      "red": 3
    },
    "providerCounts": {
      "github": 30,
      "gitlab": 10,
      "bitbucket": 2
    },
    "repositoryBreakdown": {
      "public": 20,
      "private": 18,
      "archived": 4
    },
    "openPrsBreakdown": {
      "public": 10,
      "private": 6,
      "archived": 2
    },
    "readyToMergeBreakdown": {
      "public": 5,
      "private": 2,
      "archived": 1
    },
    "needsAttentionBreakdown": {
      "public": 2,
      "private": 1,
      "archived": 0
    }
  }
}
```

**Error Responses**:

- `401 Unauthorized`: Missing or invalid authentication token
- `500 Internal Server Error`: Database or server error

---

## Usage Examples

### Example 1: Fetching Dashboard Summary

**cURL**:

```bash
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/dashboard/summary
```

**TypeScript (Axios)**:

```typescript
import axios from 'axios';
import type { DashboardSummary } from '@/types';

async function getDashboardSummary(): Promise<DashboardSummary> {
  const response = await axios.get<{ data: DashboardSummary }>('/api/dashboard/summary', {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  return response.data.data;
}
```

**TypeScript (TanStack Query)**:

```typescript
import { useQuery } from '@tanstack/react-query';
import { api } from '@/api/client';

export function useDashboardSummary() {
  return useQuery({
    queryKey: ['dashboard', 'summary'],
    queryFn: async () => {
      const response = await api.get<DashboardSummary>('/dashboard/summary');
      return response.data;
    },
    staleTime: 60000, // Cache for 1 minute
  });
}
```

### Example 2: Validating Breakdown Totals

The breakdown totals should always match the top-level counts:

```typescript
function validateBreakdowns(summary: DashboardSummary): boolean {
  const repoSum =
    summary.repositoryBreakdown.public +
    summary.repositoryBreakdown.private +
    summary.repositoryBreakdown.archived;

  const prsSum =
    summary.openPrsBreakdown.public +
    summary.openPrsBreakdown.private +
    summary.openPrsBreakdown.archived;

  const readySum =
    summary.readyToMergeBreakdown.public +
    summary.readyToMergeBreakdown.private +
    summary.readyToMergeBreakdown.archived;

  const needsAttentionSum =
    summary.needsAttentionBreakdown.public +
    summary.needsAttentionBreakdown.private +
    summary.needsAttentionBreakdown.archived;

  return (
    repoSum === summary.totalRepositories &&
    prsSum === summary.totalOpenPrs &&
    readySum === summary.statusCounts.green &&
    needsAttentionSum === summary.statusCounts.red
  );
}
```

### Example 3: Frontend Component Integration

```typescript
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Globe, Lock, Archive } from 'lucide-react';
import type { VisibilityBreakdown } from '@/types';

interface BreakdownTileProps {
  title: string;
  breakdown: VisibilityBreakdown;
  icon: React.ComponentType<{ className?: string }>;
}

export function BreakdownTile({ title, breakdown, icon: Icon }: BreakdownTileProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {/* Public Count */}
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-2">
              <Globe className="h-3.5 w-3.5 text-green-600" />
              <span className="text-muted-foreground">Public</span>
            </div>
            <span className="font-semibold">{breakdown.public}</span>
          </div>

          {/* Private Count */}
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-2">
              <Lock className="h-3.5 w-3.5 text-amber-600" />
              <span className="text-muted-foreground">Private</span>
            </div>
            <span className="font-semibold">{breakdown.private}</span>
          </div>

          {/* Archived Count */}
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-2">
              <Archive className="h-3.5 w-3.5 text-gray-500" />
              <span className="text-muted-foreground">Archived</span>
            </div>
            <span className="font-semibold">{breakdown.archived}</span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
```

---

## Backward Compatibility

The visibility breakdown fields are **additive changes** to the API response. This ensures backward compatibility:

### Old Clients (Before Visibility Breakdown)

- Will receive the new fields but ignore them
- Continue to work without errors
- Top-level counts (`totalRepositories`, `totalOpenPrs`, etc.) remain unchanged

### New Clients (After Visibility Breakdown)

- Consume and display the new breakdown fields
- Can validate that breakdowns sum to top-level counts
- Enhanced user experience with granular visibility

### API Versioning

- No API version bump required (additive change)
- New fields are always present (never null or missing)
- Breakdown defaults to all zeros if no repositories exist

---

## Provider-Specific Behavior

### GitHub

- ✅ Fully supports all visibility types (public, private, archived)
- Repositories can be explicitly archived via GitHub settings
- Archived repositories are read-only and clearly marked

### GitLab

- ✅ Fully supports all visibility types (public, private, archived)
- Uses "archived" project setting
- Archived projects are read-only

### Bitbucket

- ✅ Supports public and private repositories
- ❌ **Does NOT support archived repositories**
- `archived` count will always be `0` for Bitbucket-only users
- `is_archived` field is always `false` in Ampel database for Bitbucket repos

### Mixed Provider Scenarios

If a user has repositories from multiple providers:

```json
{
  "providerCounts": {
    "github": 20,
    "gitlab": 10,
    "bitbucket": 5
  },
  "repositoryBreakdown": {
    "public": 15,
    "private": 16,
    "archived": 4
  }
}
```

The 4 archived repositories are from GitHub and/or GitLab only (not Bitbucket).

---

## Performance Considerations

### Response Time

- **Target**: < 500ms for 100 repositories
- **Current**: ~500ms (single-pass iteration)
- **Future Optimization**: Database-level aggregation via SQL

### Caching Strategy

- Client-side caching: 60 seconds (TanStack Query `staleTime`)
- Server-side caching: Future Redis cache (60 seconds)
- Cache invalidation: On PR status changes or repository updates

### Calculation Complexity

- **Time Complexity**: O(n \* m) where n = repos, m = PRs per repo
- **Future Optimization**: Parallel CI/review queries, SQL aggregation

---

## OpenAPI/Swagger Schema

```yaml
components:
  schemas:
    VisibilityBreakdown:
      type: object
      required:
        - public
        - private
        - archived
      properties:
        public:
          type: integer
          description: Count of public (non-private, non-archived) items
          example: 20
        private:
          type: integer
          description: Count of private (non-archived) items
          example: 15
        archived:
          type: integer
          description: Count of archived items (may also be private)
          example: 5

    DashboardSummary:
      type: object
      required:
        - totalRepositories
        - totalOpenPrs
        - statusCounts
        - providerCounts
        - repositoryBreakdown
        - openPrsBreakdown
        - readyToMergeBreakdown
        - needsAttentionBreakdown
      properties:
        totalRepositories:
          type: integer
          description: Total number of repositories tracked
          example: 42
        totalOpenPrs:
          type: integer
          description: Total number of open pull requests
          example: 18
        statusCounts:
          $ref: '#/components/schemas/StatusCounts'
        providerCounts:
          $ref: '#/components/schemas/ProviderCounts'
        repositoryBreakdown:
          $ref: '#/components/schemas/VisibilityBreakdown'
        openPrsBreakdown:
          $ref: '#/components/schemas/VisibilityBreakdown'
        readyToMergeBreakdown:
          $ref: '#/components/schemas/VisibilityBreakdown'
        needsAttentionBreakdown:
          $ref: '#/components/schemas/VisibilityBreakdown'
```

---

## Related Documentation

- [Repository Visibility Filters](../features/REPOSITORY_VISIBILITY_FILTERS.md)
- [Visibility Breakdown Tiles](../features/VISIBILITY-BREAKDOWN-TILES.md)
- [Architecture Documentation](../ARCHITECTURE.md)
- [Testing Guide](../TESTING.md)

---

**Document Maintained By**: Engineering Team
**Questions?**: See [CLAUDE.md](/CLAUDE.md) for AI assistant guidance
