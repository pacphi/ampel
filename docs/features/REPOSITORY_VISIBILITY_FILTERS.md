# Repository Visibility Filters Feature

## Overview

Repository Visibility Filters allow users to control which repositories are displayed in the Ampel dashboard based on their visibility status (public/private) and archive state. This feature provides:

- Visual indicators showing repository status at a glance
- User-configurable filters to show/hide repositories by type
- Consistent icons across all dashboard views
- Persistence of filter preferences

## Architecture

### Visual Indicators

Icons are displayed on repository tiles across all views:

| Status   | Icon    | Color                          | Tooltip               |
| -------- | ------- | ------------------------------ | --------------------- |
| Public   | Globe   | Green (`text-green-600`)       | "Public repository"   |
| Private  | Lock    | Amber (`text-amber-600`)       | "Private repository"  |
| Archived | Archive | Gray (`text-muted-foreground`) | "Archived repository" |

### Data Model

The feature leverages existing fields in the repository model:

```typescript
// frontend/src/types/index.ts
export interface Repository {
  // ... other fields
  isPrivate: boolean;
  isArchived: boolean;
}
```

```rust
// crates/ampel-db/src/entities/repository.rs
pub struct Model {
    // ... other fields
    pub is_private: bool,
    pub is_archived: bool,
}
```

### Filter Settings Schema

```typescript
// frontend/src/hooks/useRepositoryFilters.ts
export interface RepositoryFilters {
  includePublic: boolean; // Default: true
  includePrivate: boolean; // Default: true
  includeArchived: boolean; // Default: true
}
```

## Provider Support Matrix

| Feature                 | GitHub           | GitLab               | Bitbucket Cloud    |
| ----------------------- | ---------------- | -------------------- | ------------------ |
| **Public/Private**      | `private: bool`  | `visibility: string` | `is_private: bool` |
| **Archived**            | `archived: bool` | `archived: bool`     | Not supported      |
| **Internal Visibility** | Enterprise only  | Yes                  | N/A                |

### Provider-Specific Notes

**GitHub**:

- Uses both `private` boolean and `visibility` string
- Internal visibility only available on Enterprise accounts
- Cannot unarchive via API (requires web UI)

**GitLab**:

- Three-tier visibility: public, internal, private
- Internal visibility disabled for new projects on GitLab.com since July 2019
- Archived projects excluded from search by default

**Bitbucket Cloud**:

- No archive support (hardcoded to `false` in Ampel)
- Uses `is_private` boolean only
- App passwords deprecated in 2025/2026

## API Endpoints

### Get Repository Filters

Retrieve user's repository filter preferences.

**Endpoint:** `GET /api/settings/repository-filters`

**Authentication:** Required

**Response:** `200 OK`

```json
{
  "success": true,
  "data": {
    "includePublic": true,
    "includePrivate": true,
    "includeArchived": true
  }
}
```

### Update Repository Filters

Update user's repository filter preferences.

**Endpoint:** `PUT /api/settings/repository-filters`

**Authentication:** Required

**Request Body:**

```json
{
  "includePublic": true,
  "includePrivate": false,
  "includeArchived": true
}
```

**Validation:**

- At least one of `includePublic` or `includePrivate` must be `true`

**Response:** `200 OK`

```json
{
  "success": true,
  "data": {
    "includePublic": true,
    "includePrivate": false,
    "includeArchived": true
  }
}
```

**Error Response:** `400 Bad Request`

```json
{
  "success": false,
  "error": "At least one of public or private must be enabled"
}
```

## Frontend Components

### RepositoryStatusIcons

Displays visibility status icons with tooltips.

**File:** `frontend/src/components/dashboard/RepositoryStatusIcons.tsx`

**Props:**

```typescript
interface RepositoryStatusIconsProps {
  isPrivate: boolean;
  isArchived: boolean;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}
```

**Usage:**

```tsx
import RepositoryStatusIcons from '@/components/dashboard/RepositoryStatusIcons';

<RepositoryStatusIcons isPrivate={repo.isPrivate} isArchived={repo.isArchived} size="md" />;
```

### RepositoryFilterSettings

Settings panel for configuring repository filters.

**File:** `frontend/src/components/settings/RepositoryFilterSettings.tsx`

**Features:**

- Three toggle switches for public/private/archived
- Descriptive labels and icons
- Note about Bitbucket archive limitation
- Real-time persistence to localStorage

### useRepositoryFilters Hook

Custom hook for managing filter state.

**File:** `frontend/src/hooks/useRepositoryFilters.ts`

**Returns:**

```typescript
{
  filters: RepositoryFilters;
  setFilters: (filters: RepositoryFilters) => void;
  filterRepositories: <T extends { isPrivate: boolean; isArchived: boolean }>(
    repos: T[]
  ) => T[];
}
```

**Usage:**

```tsx
const { filters, filterRepositories } = useRepositoryFilters();

// Filter repositories based on user preferences
const visibleRepos = filterRepositories(repositories);
```

## Implementation Details

### Icon Display Logic

```typescript
// Shows Globe for public, Lock for private (mutually exclusive)
// Shows Archive only when isArchived is true (in addition to public/private)

{!isPrivate && (
  <Tooltip content="Public repository">
    <Globe className="text-green-600" />
  </Tooltip>
)}

{isPrivate && (
  <Tooltip content="Private repository">
    <Lock className="text-amber-600" />
  </Tooltip>
)}

{isArchived && (
  <Tooltip content="Archived repository">
    <Archive className="text-muted-foreground" />
  </Tooltip>
)}
```

### Filter Application

```typescript
const filterRepositories = <T extends { isPrivate: boolean; isArchived: boolean }>(
  repos: T[]
): T[] => {
  return repos.filter((repo) => {
    // Check visibility filters
    if (!filters.includePublic && !repo.isPrivate) return false;
    if (!filters.includePrivate && repo.isPrivate) return false;

    // Check archived filter
    if (!filters.includeArchived && repo.isArchived) return false;

    return true;
  });
};
```

### Storage

**Client-side (Phase 2):**

- localStorage key: `ampel-repository-filters`
- JSON serialized `RepositoryFilters` object
- Automatic sync on change

**Server-side (Phase 3):**

- Database table: `repository_filters`
- User-scoped preferences
- API-based persistence

## Database Schema

### repository_filters Table

```sql
CREATE TABLE repository_filters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    include_public BOOLEAN NOT NULL DEFAULT TRUE,
    include_private BOOLEAN NOT NULL DEFAULT TRUE,
    include_archived BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);
```

**Migration File:** `crates/ampel-db/src/migrations/m20251223_000001_repository_filters.rs`

## Views Updated

The visibility icons appear in:

1. **Dashboard Grid View** (`RepoCard.tsx`)
   - Icons in card header, next to repository name

2. **Dashboard List View** (`ListView.tsx`)
   - Dedicated "Visibility" column in table

3. **Repositories Page** (`Repositories.tsx`)
   - Dedicated "Visibility" column in repository table

## Settings Integration

The filter settings are accessible at:

**URL:** `/settings/filters`

**Navigation:** Settings > Filters (in sidebar)

The settings page includes:

- Repository Visibility Filters card
- Toggle switches for each filter type
- Informational note about Bitbucket limitations

## Testing

### Unit Tests

**RepositoryStatusIcons Tests** (`RepositoryStatusIcons.test.tsx`):

- Renders correct icons for public/private/archived states
- Applies correct size variants
- Shows tooltips on hover

**RepositoryFilterSettings Tests** (`RepositoryFilterSettings.test.tsx`):

- Renders all toggle switches
- Handles toggle interactions
- Persists changes to localStorage
- Loads stored preferences on mount

**useRepositoryFilters Tests** (`useRepositoryFilters.test.ts`):

- Returns default filter values
- Filters repositories correctly
- Persists changes to localStorage

### Test Coverage

- Component rendering: 11 tests
- Settings interactions: 18 tests
- Hook behavior: 9 tests

## Usage Examples

### Dashboard Integration

```tsx
// Dashboard.tsx
import { useRepositoryFilters } from '@/hooks/useRepositoryFilters';

export default function Dashboard() {
  const { filterRepositories } = useRepositoryFilters();
  const { data: repositories } = useRepositories();

  const visibleRepos = repositories ? filterRepositories(repositories) : [];

  return (
    <div>
      {visibleRepos.map((repo) => (
        <RepoCard key={repo.id} repo={repo} />
      ))}
    </div>
  );
}
```

### Settings Page Integration

```tsx
// Settings.tsx
import { RepositoryFilterSettings } from '@/components/settings/RepositoryFilterSettings';

// Add to settings navigation
const settingsNav = [
  { name: 'Profile', href: '/settings' },
  { name: 'Filters', href: '/settings/filters' },
  // ...
];

// Render component at /settings/filters route
<Route path="filters" element={<RepositoryFilterSettings />} />;
```

## Related Files

### Frontend

- `frontend/src/components/dashboard/RepositoryStatusIcons.tsx`
- `frontend/src/components/settings/RepositoryFilterSettings.tsx`
- `frontend/src/hooks/useRepositoryFilters.ts`
- `frontend/src/components/ui/tooltip.tsx`
- `frontend/src/pages/Dashboard.tsx`
- `frontend/src/pages/Settings.tsx`
- `frontend/src/components/dashboard/RepoCard.tsx`
- `frontend/src/components/dashboard/ListView.tsx`
- `frontend/src/pages/Repositories.tsx`

### Backend

- `crates/ampel-db/src/migrations/m20251223_000001_repository_filters.rs`
- `crates/ampel-db/src/entities/repository_filter.rs`
- `crates/ampel-db/src/queries/repository_filter_queries.rs`
- `crates/ampel-api/src/handlers/repository_filters.rs`

### Tests

- `frontend/src/components/dashboard/RepositoryStatusIcons.test.tsx`
- `frontend/src/components/settings/RepositoryFilterSettings.test.tsx`
- `frontend/src/hooks/useRepositoryFilters.test.ts`

## See Also

- [Visibility Breakdown Tiles](./VISIBILITY-BREAKDOWN-TILES.md)
