# Dashboard Tile Components Documentation

**Components**: `SummaryBreakdownTile`, `BreakdownTile`
**Location**: `frontend/src/components/dashboard/`
**Feature**: Repository Visibility Breakdown Tiles
**Status**: Implemented

---

## Table of Contents

1. [Overview](#overview)
2. [SummaryBreakdownTile Component](#summarybreakdowntile-component)
3. [BreakdownTile Component (Legacy)](#breakdowntile-component-legacy)
4. [Migration Guide](#migration-guide)
5. [Icon Patterns](#icon-patterns)
6. [Styling](#styling)
7. [Accessibility](#accessibility)
8. [Testing](#testing)

---

## Overview

The Dashboard uses combined tiles that display both summary counts and visibility breakdowns in a single card. This provides a compact, information-dense view while maintaining readability.

### Current Architecture (v2.0)

The Dashboard now uses **`SummaryBreakdownTile`** which combines:

- Main count (e.g., "135 Total Repositories")
- Visibility breakdown (Public/Private/Archived)

```
┌─────────────────────────────┐
│ Total Repositories     📦   │  ← Title + Icon
├─────────────────────────────┤
│                             │
│     135                     │  ← Main Count
│                             │
│─────────────────────────────│
│  🌐 Public          104     │  ← Breakdown
│  🔒 Private          17     │
│  📦 Archived         14     │
└─────────────────────────────┘
```

### Features

- Displays main count prominently at top
- Shows visibility breakdown (public, private, archived) below
- Optional count color (e.g., green for "Ready to Merge")
- Uses consistent iconography (Globe, Lock, Archive)
- Supports loading state with spinner
- Fully accessible with ARIA labels
- Responsive design
- Follows shadcn/ui design system

---

## SummaryBreakdownTile Component

**File**: `frontend/src/components/dashboard/SummaryBreakdownTile.tsx`

### Props

```typescript
interface SummaryBreakdownTileProps {
  /** Tile title displayed in the card header */
  title: string;

  /** Main count to display prominently */
  count: number;

  /** Visibility breakdown data containing public, private, and archived counts */
  breakdown: VisibilityBreakdown;

  /** Icon component to display in the card header */
  icon: ComponentType<{ className?: string }>;

  /** Whether to show loading state (optional, defaults to false) */
  isLoading?: boolean;

  /** Optional Tailwind color class for the count (e.g., "text-ampel-green") */
  countColor?: string;
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

### Usage Examples

#### Example 1: Basic Usage

```tsx
import SummaryBreakdownTile from '@/components/dashboard/SummaryBreakdownTile';
import { Boxes } from 'lucide-react';

function DashboardPage() {
  const breakdown = {
    public: 104,
    private: 17,
    archived: 14,
  };

  return (
    <SummaryBreakdownTile
      title="Total Repositories"
      count={135}
      breakdown={breakdown}
      icon={Boxes}
    />
  );
}
```

#### Example 2: With Color and API Data

```tsx
import { useQuery } from '@tanstack/react-query';
import SummaryBreakdownTile from '@/components/dashboard/SummaryBreakdownTile';
import { dashboardApi } from '@/api/dashboard';

// Green status icon component
const GreenStatusIcon = () => <span className="h-3 w-3 rounded-full bg-ampel-green" />;

function ReadyToMergeTile() {
  const { data: summary, isLoading } = useQuery({
    queryKey: ['dashboard', 'summary'],
    queryFn: () => dashboardApi.getSummary(),
  });

  return (
    <SummaryBreakdownTile
      title="Ready to Merge"
      count={summary?.statusCounts.green || 0}
      breakdown={summary?.readyToMergeBreakdown || { public: 0, private: 0, archived: 0 }}
      icon={GreenStatusIcon}
      isLoading={isLoading}
      countColor="text-ampel-green"
    />
  );
}
```

#### Example 3: Complete Dashboard Grid (All 4 Tiles)

```tsx
import SummaryBreakdownTile from '@/components/dashboard/SummaryBreakdownTile';
import { Boxes, GitPullRequest } from 'lucide-react';

function DashboardSummaryRow() {
  const { data: summary, isLoading } = useDashboardSummary();

  const GreenStatusIcon = () => <span className="h-3 w-3 rounded-full bg-ampel-green" />;

  const RedStatusIcon = () => <span className="h-3 w-3 rounded-full bg-ampel-red" />;

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
      {/* Total Repositories */}
      <SummaryBreakdownTile
        title="Total Repositories"
        count={summary?.totalRepositories || 0}
        breakdown={summary?.repositoryBreakdown || { public: 0, private: 0, archived: 0 }}
        icon={Boxes}
        isLoading={isLoading}
      />

      {/* Open PRs */}
      <SummaryBreakdownTile
        title="Open PRs"
        count={summary?.totalOpenPrs || 0}
        breakdown={summary?.openPrsBreakdown || { public: 0, private: 0, archived: 0 }}
        icon={GitPullRequest}
        isLoading={isLoading}
      />

      {/* Ready to Merge */}
      <SummaryBreakdownTile
        title="Ready to Merge"
        count={readyToMergeCount}
        breakdown={readyToMergeBreakdown}
        icon={GreenStatusIcon}
        isLoading={isLoading}
        countColor="text-ampel-green"
      />

      {/* Needs Attention */}
      <SummaryBreakdownTile
        title="Needs Attention"
        count={summary?.statusCounts.red || 0}
        breakdown={needsAttentionBreakdown}
        icon={RedStatusIcon}
        isLoading={isLoading}
        countColor="text-ampel-red"
      />
    </div>
  );
}
```

---

## BreakdownTile Component (Legacy)

**File**: `frontend/src/components/dashboard/BreakdownTile.tsx`

> **Note**: This component is still available but is no longer used on the main Dashboard.
> The Dashboard now uses `SummaryBreakdownTile` which combines summary + breakdown in one tile.
> `BreakdownTile` may still be useful for other pages that need only breakdown display.

### Props

```typescript
interface BreakdownTileProps {
  title: string;
  breakdown: VisibilityBreakdown;
  icon: LucideIcon;
  isLoading?: boolean;
}
```

### Basic Usage

```tsx
import BreakdownTile from '@/components/dashboard/BreakdownTile';
import { Boxes } from 'lucide-react';

<BreakdownTile
  title="Repositories by Visibility"
  breakdown={{ public: 20, private: 15, archived: 5 }}
  icon={Boxes}
/>;
```

---

## Migration Guide

### From Separate Tiles to Combined Tiles

**Before (v1.0)** - Two rows, 8 tiles total:

```tsx
{/* Row 1: Summary Cards */}
<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
  <Card>Total Repositories: 135</Card>
  <Card>Open PRs: 107</Card>
  <Card>Ready to Merge: 39</Card>
  <Card>Needs Attention: 69</Card>
</div>

{/* Row 2: Breakdown Tiles */}
<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
  <BreakdownTile title="Repositories by Visibility" ... />
  <BreakdownTile title="Open PRs by Visibility" ... />
  <BreakdownTile title="Ready to Merge by Visibility" ... />
  <BreakdownTile title="Needs Attention by Visibility" ... />
</div>
```

**After (v2.0)** - Single row, 4 combined tiles:

```tsx
{/* Combined Summary + Breakdown Tiles */}
<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
  <SummaryBreakdownTile title="Total Repositories" count={135} breakdown={...} ... />
  <SummaryBreakdownTile title="Open PRs" count={107} breakdown={...} ... />
  <SummaryBreakdownTile title="Ready to Merge" count={39} breakdown={...} countColor="text-ampel-green" />
  <SummaryBreakdownTile title="Needs Attention" count={69} breakdown={...} countColor="text-ampel-red" />
</div>
```

### Key Changes

| Aspect        | Before (v1.0)                | After (v2.0)            |
| ------------- | ---------------------------- | ----------------------- |
| Layout        | 2 rows, 8 tiles              | 1 row, 4 tiles          |
| Component     | Card + BreakdownTile         | SummaryBreakdownTile    |
| Title         | "Repositories by Visibility" | "Total Repositories"    |
| Count Display | Separate card                | Integrated in tile      |
| Color Support | No                           | Yes (`countColor` prop) |

---

## Icon Patterns

### Visibility Icons (Used in Components)

| Visibility   | Icon    | Color Class      | Hex Color | Lucide Icon   |
| ------------ | ------- | ---------------- | --------- | ------------- |
| **Public**   | Globe   | `text-green-600` | `#16a34a` | `<Globe />`   |
| **Private**  | Lock    | `text-amber-600` | `#d97706` | `<Lock />`    |
| **Archived** | Archive | `text-gray-500`  | `#6b7280` | `<Archive />` |

### Header Icons

```tsx
// Standard Lucide icons
import { Boxes, GitPullRequest } from 'lucide-react';

<SummaryBreakdownTile icon={Boxes} ... />
<SummaryBreakdownTile icon={GitPullRequest} ... />

// Custom status indicators
const GreenStatusIcon = () => (
  <span className="h-3 w-3 rounded-full bg-ampel-green" />
);
<SummaryBreakdownTile icon={GreenStatusIcon} ... />
```

### Icon Size Guidelines

- **Header Icon**: `h-4 w-4` (16px)
- **Visibility Icons**: `h-3.5 w-3.5` (14px)

---

## Styling

### Color System

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
│ Total Repositories     📦   │  ← CardHeader (text-sm font-medium)
├─────────────────────────────┤
│                             │
│       135                   │  ← Main count (text-2xl font-bold)
│                             │
│─────────────────────────────│  ← Border separator
│  🌐 Public          104     │  ← Breakdown items (text-sm)
│  🔒 Private          17     │
│  📦 Archived         14     │
└─────────────────────────────┘
```

### Responsive Behavior

- **Mobile (< 640px)**: Tiles stack vertically (1 column)
- **Tablet (640px - 1024px)**: 2 columns
- **Desktop (> 1024px)**: 4 columns

---

## Accessibility

### ARIA Labels

```tsx
<Card role="region" aria-label={`${title} summary with visibility breakdown`}>
  <CardContent>
    <div
      className={`text-2xl font-bold ${countColor || ''}`}
      role="status"
      aria-label={`${title}: ${count}`}
    >
      {count}
    </div>
    <div role="list" aria-label="Visibility breakdown">
      <div role="listitem" aria-label={`Public: ${breakdown.public}`}>
        ...
      </div>
      <div role="listitem" aria-label={`Private: ${breakdown.private}`}>
        ...
      </div>
      <div role="listitem" aria-label={`Archived: ${breakdown.archived}`}>
        ...
      </div>
    </div>
  </CardContent>
</Card>
```

### Color Contrast

All colors meet WCAG AA compliance (4.5:1 contrast ratio):

| Color | Hex       | Contrast Ratio | WCAG AA |
| ----- | --------- | -------------- | ------- |
| Green | `#16a34a` | 4.5:1          | Pass    |
| Amber | `#d97706` | 4.5:1          | Pass    |
| Gray  | `#6b7280` | 4.5:1          | Pass    |

---

## Testing

### Test File

```
frontend/src/components/dashboard/SummaryBreakdownTile.test.tsx
```

### Test Categories

1. **Component Rendering** - Title, count, icon display
2. **Visibility Counts** - All breakdown values displayed
3. **Loading State** - Spinner shown, values hidden
4. **Count Color** - Custom color classes applied
5. **Accessibility** - ARIA labels, roles, screen reader support
6. **Edge Cases** - Zero values, large numbers

### Running Tests

```bash
# Run all frontend tests
pnpm test

# Run specific component tests
pnpm test -- SummaryBreakdownTile
```

---

## Related Documentation

- [Dashboard Visibility Breakdown API](/docs/api/DASHBOARD-VISIBILITY-BREAKDOWN.md)
- [Visibility Breakdown Feature](/docs/features/VISIBILITY-BREAKDOWN-TILES.md)
- [Visibility Breakdown Implementation Plan](/docs/planning/VISIBILITY-BREAKDOWN-TILES-IMPLEMENTATION.md)
- [shadcn/ui Card Component](https://ui.shadcn.com/docs/components/card)
- [Lucide Icons](https://lucide.dev/)

---

**Component Maintained By**: Frontend Team
**Last Updated**: 2025-12-24
