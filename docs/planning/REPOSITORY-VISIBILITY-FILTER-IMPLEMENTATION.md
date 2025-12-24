# Technical Implementation Plan: Repository Visibility Indicators and Filtering

**Document Version**: 1.1
**Created**: 2025-12-23
**Updated**: 2025-12-23
**Status**: ✅ Completed
**Target Branch**: feature/repo-visibility-filters

---

## Executive Summary

This document outlines the implementation plan for adding repository visibility indicators (public/private/archived icons) to the Ampel dashboard and introducing filter settings that allow users to control which repositories are displayed based on these designations.

### Key Deliverables

1. **Visual Indicators**: Add icons to repository tiles showing public, private, and archived status
2. **Filter Settings**: New Settings section allowing users to toggle visibility of public/private/archived repos
3. **Consistent UI**: Apply indicators across all views (Dashboard Grid, Dashboard List, Repositories page)
4. **Backend Integration**: Leverage existing `isPrivate` and `isArchived` fields already in the data model

---

## 1. Research Findings Summary

### 1.1 Provider Support Matrix

| Feature                 | GitHub                                   | GitLab                                            | Bitbucket Cloud                   |
| ----------------------- | ---------------------------------------- | ------------------------------------------------- | --------------------------------- |
| **Public/Private**      | ✅ `private: bool`, `visibility: string` | ✅ `visibility: string` (public/internal/private) | ✅ `is_private: bool`             |
| **Archived**            | ✅ `archived: bool`                      | ✅ `archived: bool`                               | ❌ Not supported (always `false`) |
| **Internal Visibility** | Enterprise only                          | Yes (disabled on GitLab.com for new projects)     | N/A                               |

### 1.2 Key Idiosyncratic Behaviors

**GitHub**:

- `visibility` field overrides `private` field
- Cannot unarchive via API (must use web UI)
- Internal visibility only on Enterprise accounts

**GitLab**:

- Three-tier visibility (public/internal/private)
- Internal visibility disabled for NEW projects on GitLab.com since July 2019
- Archived projects excluded from search by default (recent change)
- Visibility must respect parent group hierarchy

**Bitbucket Cloud**:

- No archive support (hardcoded to `false` in Ampel provider)
- Uses `is_private` boolean (no intermediate visibility)
- Workspace → Project → Repository hierarchy affects permissions
- App passwords deprecated in 2025/2026 (plan for OAuth migration)

### 1.3 Current Ampel Implementation Status

The backend and data models **already support** these fields:

```typescript
// frontend/src/types/index.ts
export interface Repository {
  // ... other fields
  isPrivate: boolean;
  isArchived: boolean;
}
```

```rust
// crates/ampel-core/src/models/repository.rs
pub struct Repository {
    // ... other fields
    pub is_private: bool,
    pub is_archived: bool,
}
```

The provider implementations correctly map these fields:

- GitHub: Maps `private` and `archived` directly
- GitLab: Maps `visibility != "public"` to `is_private`, `archived` directly
- Bitbucket: Maps `is_private`, hardcodes `is_archived: false`

---

## 2. Frontend Implementation

### 2.1 Icon System Design

#### Recommended Icons (from lucide-react)

| Status   | Icon      | Color            | Tooltip               |
| -------- | --------- | ---------------- | --------------------- |
| Public   | `Globe`   | `text-green-600` | "Public repository"   |
| Private  | `Lock`    | `text-amber-600` | "Private repository"  |
| Archived | `Archive` | `text-gray-500`  | "Archived repository" |

#### Icon Sizing

- **RepoCard (Grid view)**: 14px (`h-3.5 w-3.5`)
- **ListView (Table)**: 16px (`h-4 w-4`)
- **Compact/Dense displays**: 12px (`h-3 w-3`)

### 2.2 New Component: RepositoryStatusIcons

**File**: `frontend/src/components/dashboard/RepositoryStatusIcons.tsx`

```typescript
import { Globe, Lock, Archive } from 'lucide-react';
import { cn } from '@/lib/utils';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';

interface RepositoryStatusIconsProps {
  isPrivate: boolean;
  isArchived: boolean;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const sizeClasses = {
  sm: 'h-3 w-3',
  md: 'h-3.5 w-3.5',
  lg: 'h-4 w-4',
};

export function RepositoryStatusIcons({
  isPrivate,
  isArchived,
  size = 'md',
  className,
}: RepositoryStatusIconsProps) {
  const iconClass = cn(sizeClasses[size], 'shrink-0');

  return (
    <TooltipProvider>
      <div className={cn('flex items-center gap-1', className)}>
        {/* Visibility icon - always show one */}
        <Tooltip>
          <TooltipTrigger asChild>
            {isPrivate ? (
              <Lock className={cn(iconClass, 'text-amber-600')} />
            ) : (
              <Globe className={cn(iconClass, 'text-green-600')} />
            )}
          </TooltipTrigger>
          <TooltipContent>
            <p>{isPrivate ? 'Private repository' : 'Public repository'}</p>
          </TooltipContent>
        </Tooltip>

        {/* Archived icon - only show if archived */}
        {isArchived && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Archive className={cn(iconClass, 'text-gray-500')} />
            </TooltipTrigger>
            <TooltipContent>
              <p>Archived repository</p>
            </TooltipContent>
          </Tooltip>
        )}
      </div>
    </TooltipProvider>
  );
}
```

### 2.3 Component Updates

#### 2.3.1 RepoCard.tsx Updates

```diff
 import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
 import StatusBadge from './StatusBadge';
+import { RepositoryStatusIcons } from './RepositoryStatusIcons';
 import { formatRelativeTime } from '@/lib/utils';
 import type { RepositoryWithStatus } from '@/types';
 import { ExternalLink, GitPullRequest } from 'lucide-react';
 import { GithubIcon, GitlabIcon, BitbucketIcon } from '@/components/icons/ProviderIcons';

 export default function RepoCard({ repository }: RepoCardProps) {
   const ProviderIcon = providerIcons[repository.provider] || GithubIcon;

   return (
     <Card className="hover:shadow-md transition-shadow">
       <CardHeader className="pb-2">
         <div className="flex items-start justify-between">
           <div className="flex items-center gap-2">
             <StatusBadge status={repository.status} size="lg" />
             <ProviderIcon className="h-4 w-4 text-muted-foreground" />
+            <RepositoryStatusIcons
+              isPrivate={repository.isPrivate}
+              isArchived={repository.isArchived}
+              size="md"
+            />
           </div>
           <a href={repository.url} ...>
```

#### 2.3.2 ListView.tsx Updates

Add "Visibility" column to table:

```diff
 <thead>
   <tr className="border-b bg-muted/50">
     <th className="px-4 py-3 text-left text-sm font-medium">Status</th>
     <th className="px-4 py-3 text-left text-sm font-medium">Repository</th>
+    <th className="px-4 py-3 text-left text-sm font-medium">Visibility</th>
     <th className="px-4 py-3 text-left text-sm font-medium">Provider</th>
     <th className="px-4 py-3 text-left text-sm font-medium">PRs</th>
     <th className="px-4 py-3 text-left text-sm font-medium">Last Updated</th>
     <th className="px-4 py-3 text-left text-sm font-medium"></th>
   </tr>
 </thead>
 <tbody>
   {repositories.map((repo) => (
     <tr key={repo.id} ...>
       <td className="px-4 py-3">
         <StatusBadge status={repo.status} showLabel />
       </td>
       <td className="px-4 py-3">
         <div>
           <p className="font-medium">{repo.name}</p>
           <p className="text-sm text-muted-foreground">{repo.owner}</p>
         </div>
       </td>
+      <td className="px-4 py-3">
+        <RepositoryStatusIcons
+          isPrivate={repo.isPrivate}
+          isArchived={repo.isArchived}
+          size="lg"
+        />
+      </td>
       <td className="px-4 py-3 capitalize">{repo.provider}</td>
```

#### 2.3.3 Repositories.tsx Updates

Similar updates to the repository list table.

---

## 3. Filter Settings Implementation

### 3.1 Settings Data Model

#### New Filter Type

**File**: `frontend/src/api/settings.ts` (additions)

```typescript
export interface RepositoryFilters {
  includePublic: boolean;
  includePrivate: boolean;
  includeArchived: boolean;
}

export interface UpdateRepositoryFiltersRequest {
  includePublic?: boolean;
  includePrivate?: boolean;
  includeArchived?: boolean;
}
```

#### Default Values

All filters should be **enabled by default** (include all repository types):

```typescript
const defaultRepositoryFilters: RepositoryFilters = {
  includePublic: true,
  includePrivate: true,
  includeArchived: true,
};
```

### 3.2 Settings API Additions

**File**: `frontend/src/api/settings.ts`

```typescript
export const settingsApi = {
  // ... existing methods

  // Repository filters
  async getRepositoryFilters(): Promise<RepositoryFilters> {
    const response = await apiClient.get<ApiResponse<RepositoryFilters>>(
      '/settings/repository-filters'
    );
    return response.data.data!;
  },

  async updateRepositoryFilters(data: UpdateRepositoryFiltersRequest): Promise<RepositoryFilters> {
    const response = await apiClient.put<ApiResponse<RepositoryFilters>>(
      '/settings/repository-filters',
      data
    );
    return response.data.data!;
  },
};
```

### 3.3 New Settings Component

**File**: `frontend/src/components/settings/RepositoryFilterSettings.tsx`

```typescript
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { settingsApi, type RepositoryFilters } from '@/api/settings';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { useToast } from '@/components/ui/use-toast';
import { Globe, Lock, Archive } from 'lucide-react';

export function RepositoryFilterSettings() {
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const { data: filters, isLoading } = useQuery({
    queryKey: ['repository-filters'],
    queryFn: () => settingsApi.getRepositoryFilters(),
  });

  const updateMutation = useMutation({
    mutationFn: (data: Partial<RepositoryFilters>) =>
      settingsApi.updateRepositoryFilters(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repository-filters'] });
      queryClient.invalidateQueries({ queryKey: ['dashboard'] });
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      toast({
        title: 'Filters updated',
        description: 'Repository visibility filters have been saved.',
      });
    },
    onError: () => {
      toast({
        variant: 'destructive',
        title: 'Failed to update filters',
        description: 'An error occurred while saving filters.',
      });
    },
  });

  const handleToggle = (key: keyof RepositoryFilters) => {
    if (!filters) return;
    updateMutation.mutate({ [key]: !filters[key] });
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-8">
          <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary" />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Repository Visibility Filters</CardTitle>
        <CardDescription>
          Control which repositories are displayed in the dashboard and lists.
          Unchecked categories will be hidden from all views.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center space-x-3">
          <Checkbox
            id="includePublic"
            checked={filters?.includePublic ?? true}
            onCheckedChange={() => handleToggle('includePublic')}
            disabled={updateMutation.isPending}
          />
          <div className="flex items-center gap-2">
            <Globe className="h-4 w-4 text-green-600" />
            <Label htmlFor="includePublic" className="cursor-pointer">
              Show public repositories
            </Label>
          </div>
        </div>

        <div className="flex items-center space-x-3">
          <Checkbox
            id="includePrivate"
            checked={filters?.includePrivate ?? true}
            onCheckedChange={() => handleToggle('includePrivate')}
            disabled={updateMutation.isPending}
          />
          <div className="flex items-center gap-2">
            <Lock className="h-4 w-4 text-amber-600" />
            <Label htmlFor="includePrivate" className="cursor-pointer">
              Show private repositories
            </Label>
          </div>
        </div>

        <div className="flex items-center space-x-3">
          <Checkbox
            id="includeArchived"
            checked={filters?.includeArchived ?? true}
            onCheckedChange={() => handleToggle('includeArchived')}
            disabled={updateMutation.isPending}
          />
          <div className="flex items-center gap-2">
            <Archive className="h-4 w-4 text-gray-500" />
            <Label htmlFor="includeArchived" className="cursor-pointer">
              Show archived repositories
            </Label>
          </div>
        </div>

        <p className="text-xs text-muted-foreground mt-4">
          Note: Bitbucket Cloud does not support repository archiving. All Bitbucket
          repositories are treated as active.
        </p>
      </CardContent>
    </Card>
  );
}
```

### 3.4 Settings Page Integration

Update `frontend/src/pages/Settings.tsx`:

```diff
 import { NotificationsSettings } from '@/components/settings/NotificationsSettings';
 import { BehaviorSettings } from '@/components/settings/BehaviorSettings';
+import { RepositoryFilterSettings } from '@/components/settings/RepositoryFilterSettings';
+import { Eye } from 'lucide-react';

 function SettingsNav() {
   const links = [
     { href: '/settings', label: 'Profile', icon: User },
     { href: '/settings/accounts', label: 'Accounts', icon: User },
     { href: '/settings/filters', label: 'PR Filters', icon: Filter },
+    { href: '/settings/visibility', label: 'Visibility', icon: Eye },
     { href: '/settings/notifications', label: 'Notifications', icon: Bell },
     { href: '/settings/behavior', label: 'Behavior', icon: Settings2 },
   ];
   // ...
 }

 // In Routes:
 <Route path="visibility" element={<RepositoryFilterSettings />} />
```

---

## 4. Backend Implementation

### 4.1 Database Schema Addition

Add new table or extend user_settings:

```sql
-- Option A: New table
CREATE TABLE repository_filters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    include_public BOOLEAN NOT NULL DEFAULT true,
    include_private BOOLEAN NOT NULL DEFAULT true,
    include_archived BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);

-- Option B: Add columns to existing user_settings table
ALTER TABLE user_settings
ADD COLUMN include_public_repos BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN include_private_repos BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN include_archived_repos BOOLEAN NOT NULL DEFAULT true;
```

### 4.2 API Endpoints

**New endpoints for repository filters**:

```rust
// GET /api/settings/repository-filters
// Returns: RepositoryFilters

// PUT /api/settings/repository-filters
// Body: UpdateRepositoryFiltersRequest
// Returns: RepositoryFilters
```

### 4.3 Filter Application

Filters should be applied at the API level to reduce data transfer:

```rust
// In dashboard/grid handler
pub async fn get_dashboard_grid(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<RepositoryWithStatus>>, ApiError> {
    // Get user's filter preferences
    let filters = state.db.get_repository_filters(user.id).await?;

    // Get repositories with filters applied
    let repos = state.db.get_user_repositories_filtered(
        user.id,
        filters.include_public,
        filters.include_private,
        filters.include_archived,
    ).await?;

    Ok(Json(repos))
}
```

---

## 5. Client-Side Filtering (Alternative)

For simplicity, filtering can be done entirely client-side:

### 5.1 Filter Hook

**File**: `frontend/src/hooks/useRepositoryFilters.ts`

```typescript
import { useQuery } from '@tanstack/react-query';
import { settingsApi, type RepositoryFilters } from '@/api/settings';
import type { RepositoryWithStatus } from '@/types';

const defaultFilters: RepositoryFilters = {
  includePublic: true,
  includePrivate: true,
  includeArchived: true,
};

export function useRepositoryFilters() {
  const { data: filters = defaultFilters } = useQuery({
    queryKey: ['repository-filters'],
    queryFn: () => settingsApi.getRepositoryFilters(),
    staleTime: 60000,
  });

  const filterRepositories = (repositories: RepositoryWithStatus[]): RepositoryWithStatus[] => {
    return repositories.filter((repo) => {
      // Filter by public/private
      if (!filters.includePublic && !repo.isPrivate) return false;
      if (!filters.includePrivate && repo.isPrivate) return false;

      // Filter by archived
      if (!filters.includeArchived && repo.isArchived) return false;

      return true;
    });
  };

  return { filters, filterRepositories };
}
```

### 5.2 Usage in Dashboard

```typescript
import { useRepositoryFilters } from '@/hooks/useRepositoryFilters';

export default function Dashboard() {
  const { filterRepositories } = useRepositoryFilters();

  const { data: repositories } = useQuery({
    queryKey: ['dashboard', 'grid'],
    queryFn: () => dashboardApi.getGrid(),
  });

  const filteredRepositories = filterRepositories(repositories || []);

  return (
    // ...
    <GridView repositories={filteredRepositories} />
  );
}
```

---

## 6. Testing Plan

### 6.1 Unit Tests

1. **RepositoryStatusIcons Component**
   - Renders public icon when `isPrivate: false`
   - Renders private icon when `isPrivate: true`
   - Renders archive icon only when `isArchived: true`
   - Tooltip content is correct for each state
   - Size prop affects icon dimensions

2. **useRepositoryFilters Hook**
   - Returns default filters when API call fails
   - Filters repositories correctly based on settings
   - All combinations of filter states work correctly

3. **RepositoryFilterSettings Component**
   - Renders all three checkboxes
   - Checkboxes reflect current filter state
   - Toggle updates trigger API mutation
   - Loading state is displayed

### 6.2 Integration Tests

1. **Dashboard with Filters**
   - Repositories are filtered based on user settings
   - Changing filters refreshes the view
   - Empty state when all repos filtered out

2. **Settings Persistence**
   - Filter changes persist across page reloads
   - Filter changes persist across sessions

### 6.3 E2E Tests

1. Toggle each filter and verify repository list updates
2. Verify icons display correctly for each repository type
3. Verify Bitbucket repos always show as non-archived

---

## 7. Implementation Order

### Phase 1: Visual Indicators (No Backend Changes)

1. Create `RepositoryStatusIcons` component
2. Update `RepoCard.tsx` to include icons
3. Update `ListView.tsx` to include icons
4. Update `Repositories.tsx` to include icons
5. Add unit tests for new component

### Phase 2: Client-Side Filtering

1. Implement `useRepositoryFilters` hook with localStorage persistence
2. Create `RepositoryFilterSettings` component
3. Integrate filter settings into Settings page
4. Apply filtering to Dashboard and Repositories views
5. Add integration tests

### Phase 3: Backend Persistence (Optional Enhancement)

1. Add database migration for filter settings
2. Implement API endpoints for filter CRUD
3. Migrate from localStorage to API-based persistence
4. Update frontend to use API

---

## 8. File Changes Summary

### New Files

- `frontend/src/components/dashboard/RepositoryStatusIcons.tsx`
- `frontend/src/components/dashboard/RepositoryStatusIcons.test.tsx`
- `frontend/src/components/settings/RepositoryFilterSettings.tsx`
- `frontend/src/components/settings/RepositoryFilterSettings.test.tsx`
- `frontend/src/hooks/useRepositoryFilters.ts`
- `frontend/src/hooks/useRepositoryFilters.test.ts`

### Modified Files

- `frontend/src/components/dashboard/RepoCard.tsx`
- `frontend/src/components/dashboard/ListView.tsx`
- `frontend/src/pages/Repositories.tsx`
- `frontend/src/pages/Settings.tsx`
- `frontend/src/api/settings.ts`

### Backend (Phase 3)

- `crates/ampel-db/migrations/*/repository_filters.sql`
- `crates/ampel-db/src/entities/repository_filter.rs`
- `crates/ampel-db/src/queries/filter_queries.rs`
- `crates/ampel-api/src/handlers/settings.rs`
- `crates/ampel-api/src/routes.rs`

---

## 9. Risks and Mitigations

| Risk                               | Impact                                                 | Mitigation                                         |
| ---------------------------------- | ------------------------------------------------------ | -------------------------------------------------- |
| Bitbucket archive field confusion  | Users may wonder why Bitbucket repos can't be archived | Add tooltip/note explaining Bitbucket limitation   |
| GitLab internal visibility         | May confuse users                                      | Treat internal as private in UI, document behavior |
| Filter defaults causing confusion  | Users may not see all repos initially                  | Default to all filters enabled                     |
| Performance with large repo counts | Slow filtering                                         | Use server-side filtering in Phase 3               |

---

## 10. Documentation Updates

After implementation, update:

- User guide: Explain visibility indicators
- User guide: Explain filter settings
- API documentation: New filter endpoints
- CLAUDE.md: Update with new components

---

## Appendix A: Icon Visual Reference

```
┌────────────────────────────────────────┐
│ Grid View Tile                         │
├────────────────────────────────────────┤
│ ⬤ Ready  🔗  🌐  🔒  📦               │
│ my-repository                          │
│ organization-name                      │
│                                        │
│ 🔀 3 PRs            2 hours ago        │
└────────────────────────────────────────┘

Legend:
⬤ = Ampel status (green/yellow/red)
🔗 = Provider icon (GitHub/GitLab/Bitbucket)
🌐 = Public repository
🔒 = Private repository
📦 = Archived repository (only shown if archived)
```

---

## Appendix B: Research Documentation References

Detailed API research documents are available at:

- `docs/research/github-api-visibility-archived.md`
- `docs/research/gitlab-visibility-archived-api.md`
- (Bitbucket research in agent output - consolidate if needed)

---

**Document Prepared By**: Hivemind Research Swarm
**Review Required By**: Development Team Lead
**Implementation Target**: Sprint TBD
