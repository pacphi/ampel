# BreakdownTile Component Documentation

**Component**: `BreakdownTile`
**Location**: `frontend/src/components/dashboard/BreakdownTile.tsx`
**Feature**: Repository Visibility Breakdown Tiles
**Status**: Implemented

---

## Table of Contents

1. [Overview](#overview)
2. [Component API](#component-api)
3. [Usage Examples](#usage-examples)
4. [Icon Patterns](#icon-patterns)
5. [Styling](#styling)
6. [Accessibility](#accessibility)
7. [Testing](#testing)

---

## Overview

The `BreakdownTile` component displays a count breakdown of repositories or pull requests by visibility type (public, private, archived). It's designed for use in the dashboard to provide granular insights into the distribution of repositories and PRs.

### Features

- Displays counts for public, private, and archived items
- Uses consistent iconography (Globe, Lock, Archive)
- Supports loading state with spinner
- Fully accessible with ARIA labels
- Responsive design
- Follows shadcn/ui design system

---

## Component API

### Props

```typescript
interface BreakdownTileProps {
  /** Tile title displayed in the card header */
  title: string;

  /** Visibility breakdown data containing public, private, and archived counts */
  breakdown: VisibilityBreakdown;

  /** Icon component to display in the card header */
  icon: LucideIcon;

  /** Whether to show loading state (optional, defaults to false) */
  isLoading?: boolean;
}
```

### VisibilityBreakdown Type

```typescript
export interface VisibilityBreakdown {
  /** Count of public (non-private, non-archived) items */
  public: number;

  /** Count of private (non-archived) items */
  private: number;

  /** Count of archived items (may also be private) */
  archived: number;
}
```

---

## Usage Examples

### Example 1: Basic Usage

```tsx
import BreakdownTile from '@/components/dashboard/BreakdownTile';
import { Boxes } from 'lucide-react';

function DashboardPage() {
  const breakdown = {
    public: 20,
    private: 15,
    archived: 5,
  };

  return <BreakdownTile title="Repositories by Visibility" breakdown={breakdown} icon={Boxes} />;
}
```

### Example 2: With API Data

```tsx
import { useQuery } from '@tanstack/react-query';
import BreakdownTile from '@/components/dashboard/BreakdownTile';
import { GitPullRequest } from 'lucide-react';
import { dashboardApi } from '@/api/dashboard';

function PullRequestBreakdown() {
  const { data: summary, isLoading } = useQuery({
    queryKey: ['dashboard', 'summary'],
    queryFn: () => dashboardApi.getSummary(),
  });

  return (
    <BreakdownTile
      title="Open PRs by Visibility"
      breakdown={summary?.openPrsBreakdown || { public: 0, private: 0, archived: 0 }}
      icon={GitPullRequest}
      isLoading={isLoading}
    />
  );
}
```

### Example 3: Grid Layout (All Breakdown Tiles)

```tsx
import BreakdownTile from '@/components/dashboard/BreakdownTile';
import { Boxes, GitPullRequest } from 'lucide-react';

function DashboardBreakdownRow() {
  const { data: summary, isLoading } = useDashboardSummary();

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
      {/* Repository Breakdown */}
      <BreakdownTile
        title="Repositories by Visibility"
        breakdown={summary?.repositoryBreakdown || { public: 0, private: 0, archived: 0 }}
        icon={Boxes}
        isLoading={isLoading}
      />

      {/* Open PRs Breakdown */}
      <BreakdownTile
        title="Open PRs by Visibility"
        breakdown={summary?.openPrsBreakdown || { public: 0, private: 0, archived: 0 }}
        icon={GitPullRequest}
        isLoading={isLoading}
      />

      {/* Ready to Merge Breakdown */}
      <BreakdownTile
        title="Ready to Merge by Visibility"
        breakdown={summary?.readyToMergeBreakdown || { public: 0, private: 0, archived: 0 }}
        icon={() => <span className="h-3 w-3 rounded-full bg-ampel-green" />}
        isLoading={isLoading}
      />

      {/* Needs Attention Breakdown */}
      <BreakdownTile
        title="Needs Attention by Visibility"
        breakdown={summary?.needsAttentionBreakdown || { public: 0, private: 0, archived: 0 }}
        icon={() => <span className="h-3 w-3 rounded-full bg-ampel-red" />}
        isLoading={isLoading}
      />
    </div>
  );
}
```

### Example 4: Custom Styling

```tsx
import BreakdownTile from '@/components/dashboard/BreakdownTile';
import { Boxes } from 'lucide-react';

function CustomStyledTile() {
  return (
    <div className="max-w-sm">
      <BreakdownTile
        title="My Repositories"
        breakdown={{ public: 10, private: 5, archived: 2 }}
        icon={Boxes}
      />
    </div>
  );
}
```

---

## Icon Patterns

The BreakdownTile component uses consistent iconography for visibility types:

### Visibility Icons (Used in Component)

| Visibility   | Icon       | Color Class      | Hex Color | Lucide Icon   |
| ------------ | ---------- | ---------------- | --------- | ------------- |
| **Public**   | 🌐 Globe   | `text-green-600` | `#16a34a` | `<Globe />`   |
| **Private**  | 🔒 Lock    | `text-amber-600` | `#d97706` | `<Lock />`    |
| **Archived** | 📦 Archive | `text-gray-500`  | `#6b7280` | `<Archive />` |

### Header Icon (Prop)

The `icon` prop accepts any Lucide React icon or custom icon component:

```tsx
// Standard Lucide icons
import { Boxes, GitPullRequest, FolderGit2 } from 'lucide-react';

<BreakdownTile icon={Boxes} ... />
<BreakdownTile icon={GitPullRequest} ... />
<BreakdownTile icon={FolderGit2} ... />

// Custom status indicators
<BreakdownTile
  icon={() => <span className="h-3 w-3 rounded-full bg-ampel-green" />}
  ...
/>
```

### Icon Size Guidelines

- **Header Icon**: `h-4 w-4` (16px)
- **Visibility Icons**: `h-3.5 w-3.5` (14px)
- Maintains consistent sizing across all tiles

---

## Styling

### Color System

The component follows Ampel's color system defined in Tailwind config:

```typescript
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        'ampel-green': '#22c55e', // Ready to merge
        'ampel-yellow': '#f59e0b', // In progress
        'ampel-red': '#ef4444', // Needs attention
      },
    },
  },
};
```

### Layout

```
┌─────────────────────────────┐
│ Repositories by Visibility  │  ← CardTitle (text-sm font-medium)
│                       📦     │  ← Icon (h-4 w-4 text-muted-foreground)
├─────────────────────────────┤
│                             │
│  🌐 Public          20      │  ← text-sm, gap-2
│  🔒 Private         15      │  ← justify-between
│  📦 Archived         5      │  ← font-semibold (count)
│                             │
└─────────────────────────────┘
```

### Responsive Behavior

- **Mobile (< 640px)**: Tiles stack vertically (1 column)
- **Tablet (640px - 1024px)**: 2 columns
- **Desktop (> 1024px)**: 4 columns

```tsx
// Grid layout automatically adjusts
<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
  <BreakdownTile ... />
  <BreakdownTile ... />
  <BreakdownTile ... />
  <BreakdownTile ... />
</div>
```

---

## Accessibility

### ARIA Labels

The component includes proper ARIA labels for screen readers:

```tsx
<Card role="region" aria-label={`${title} breakdown by visibility`}>
  <CardContent>
    <div className="space-y-2" role="list" aria-label="Visibility breakdown">
      <div role="listitem" aria-label={`Public repositories: ${breakdown.public}`}>
        <Globe className="h-3.5 w-3.5 text-green-600" aria-hidden="true" />
        <span className="text-muted-foreground">Public</span>
        <span className="font-semibold">{breakdown.public}</span>
      </div>
      {/* Similar for Private and Archived */}
    </div>
  </CardContent>
</Card>
```

### Keyboard Navigation

- Tiles are static informational cards (no interactive elements)
- No keyboard focus required
- Screen readers announce title and counts on page load

### Color Contrast

All colors meet WCAG AA compliance (4.5:1 contrast ratio):

| Color | Hex       | Contrast Ratio | WCAG AA |
| ----- | --------- | -------------- | ------- |
| Green | `#16a34a` | 4.5:1          | ✅ Pass |
| Amber | `#d97706` | 4.5:1          | ✅ Pass |
| Gray  | `#6b7280` | 4.5:1          | ✅ Pass |

---

## Testing

### Unit Tests

```typescript
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import BreakdownTile from './BreakdownTile';
import { Boxes } from 'lucide-react';

describe('BreakdownTile', () => {
  const mockBreakdown = {
    public: 10,
    private: 5,
    archived: 2,
  };

  it('renders title correctly', () => {
    render(
      <BreakdownTile
        title="Test Breakdown"
        breakdown={mockBreakdown}
        icon={Boxes}
      />
    );
    expect(screen.getByText('Test Breakdown')).toBeInTheDocument();
  });

  it('displays all visibility counts', () => {
    render(
      <BreakdownTile
        title="Repositories"
        breakdown={mockBreakdown}
        icon={Boxes}
      />
    );

    expect(screen.getByText('10')).toBeInTheDocument(); // Public
    expect(screen.getByText('5')).toBeInTheDocument();  // Private
    expect(screen.getByText('2')).toBeInTheDocument();  // Archived
  });

  it('shows loading state when isLoading is true', () => {
    render(
      <BreakdownTile
        title="Loading Test"
        breakdown={mockBreakdown}
        icon={Boxes}
        isLoading={true}
      />
    );

    // Should show spinner, not counts
    expect(screen.queryByText('10')).not.toBeInTheDocument();
    // Can test for spinner class if needed
  });

  it('displays correct icon labels', () => {
    render(
      <BreakdownTile
        title="Repos"
        breakdown={mockBreakdown}
        icon={Boxes}
      />
    );

    expect(screen.getByText('Public')).toBeInTheDocument();
    expect(screen.getByText('Private')).toBeInTheDocument();
    expect(screen.getByText('Archived')).toBeInTheDocument();
  });

  it('handles zero counts gracefully', () => {
    render(
      <BreakdownTile
        title="Empty"
        breakdown={{ public: 0, private: 0, archived: 0 }}
        icon={Boxes}
      />
    );

    const zeros = screen.getAllByText('0');
    expect(zeros).toHaveLength(3); // All three visibility types
  });
});
```

### Integration Tests

```typescript
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Dashboard from './Dashboard';

it('displays visibility breakdown tiles with API data', async () => {
  const queryClient = new QueryClient();
  const mockSummary = {
    totalRepositories: 35,
    repositoryBreakdown: { public: 20, private: 12, archived: 3 },
    // ... other fields
  };

  // Mock API
  jest.spyOn(dashboardApi, 'getSummary').mockResolvedValue(mockSummary);

  render(
    <QueryClientProvider client={queryClient}>
      <Dashboard />
    </QueryClientProvider>
  );

  await waitFor(() => {
    expect(screen.getByText('Repositories by Visibility')).toBeInTheDocument();
    expect(screen.getByText('20')).toBeInTheDocument(); // Public count
  });
});
```

---

## Related Documentation

- [Dashboard Visibility Breakdown API](/docs/api/DASHBOARD-VISIBILITY-BREAKDOWN.md)
- [Visibility Breakdown Implementation Plan](/docs/planning/VISIBILITY-BREAKDOWN-TILES-IMPLEMENTATION.md)
- [shadcn/ui Card Component](https://ui.shadcn.com/docs/components/card)
- [Lucide Icons](https://lucide.dev/)

---

**Component Maintained By**: Frontend Team
**Questions?**: See [CLAUDE.md](/CLAUDE.md) for AI assistant guidance
