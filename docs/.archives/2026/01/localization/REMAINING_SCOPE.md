# Remaining i18n Scope: Complete Internationalization Plan

**Version:** 1.0
**Date:** 2026-01-17
**Status:** Implementation Ready
**Estimated Effort:** 8-12 hours
**Priority:** High

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Assessment](#current-state-assessment)
3. [Scope Definition](#scope-definition)
4. [Implementation Plan](#implementation-plan)
5. [Detailed Task Breakdown](#detailed-task-breakdown)
6. [File Inventory](#file-inventory)
7. [Translation Key Specifications](#translation-key-specifications)
8. [Backend i18n Integration](#backend-i18n-integration)
9. [Testing Strategy](#testing-strategy)
10. [Validation Checklist](#validation-checklist)
11. [Rollout Plan](#rollout-plan)

---

## Executive Summary

This document outlines the complete implementation plan for internationalizing the remaining user interfaces in the Ampel application. The scope covers 4 pages (Analytics, Merge, Register, Repositories), their associated components, modals, and backend API responses.

### Key Deliverables

- **4 Frontend Pages**: Full i18n integration with useTranslation hooks
- **2 New Translation Namespaces**: `analytics.json`, `merge.json`, `repositories.json`
- **3 New Backend Locale Files**: `analytics.yml`, `merge.yml`, `repositories.yml`
- **27 Language Translations**: All keys translated via ampel-i18n-builder
- **Updated TypeScript Types**: Regenerated `frontend/src/i18n/types.ts`
- **Test Coverage**: Updated test files with i18n mocking

### Recent Completions (Reference)

| Commit | Description |
|--------|-------------|
| `24bc31b` | LanguageSwitcher added to Login Page |
| `c11be77` | `emailPlaceholder` added to CommonTranslations.auth |

---

## Current State Assessment

### Pages with Full i18n ✅

| Page | Namespaces Used | Status |
|------|-----------------|--------|
| Login.tsx | common, validation, errors | Complete |
| Dashboard.tsx | common, dashboard, errors | Complete |
| Settings.tsx | common, settings, validation, errors | Complete |

### Pages Requiring i18n ⚠️

| Page | Current State | i18n Status |
|------|---------------|-------------|
| Analytics.tsx | Hardcoded English | **None** |
| Merge.tsx | Hardcoded English | **None** |
| Register.tsx | Partial useTranslation | **Partial** (placeholder still hardcoded) |
| Repositories.tsx | Hardcoded English | **None** |

### Components Requiring i18n ⚠️

| Component | Location | Status |
|-----------|----------|--------|
| MergeResultsDialog.tsx | `components/merge/` | **None** |

---

## Scope Definition

### In Scope

1. **Analytics Page** (`frontend/src/pages/Analytics.tsx`)
   - Page title and subtitle
   - Summary card labels (PRs Merged, Avg Merge Time, etc.)
   - Health card labels (Merge Time, Throughput, Review Time, Stale PRs)
   - Empty state messages
   - Loading states

2. **Merge Page** (`frontend/src/pages/Merge.tsx`)
   - Page title and description
   - Merge options (Strategy labels, Delete branch toggle)
   - Selection controls (Select All, Deselect All)
   - PR list section headers (Ready to Merge, Not Ready)
   - Blocker labels (Draft, Conflicts, CI failed, etc.)
   - Toast messages (success, failure)
   - Button labels (Merge X PRs, Merging...)

3. **Merge Results Dialog** (`frontend/src/components/merge/MergeResultsDialog.tsx`)
   - Dialog title and description
   - Summary labels (Total, Merged, Failed, Skipped)
   - Status labels (success, failed, skipped)
   - Close button

4. **Register Page** (`frontend/src/pages/Register.tsx`)
   - Add missing placeholder for displayName field
   - Ensure all validation messages use i18n

5. **Repositories Page** (`frontend/src/pages/Repositories.tsx`)
   - Page title
   - Provider connection section
   - Add from provider section
   - Search placeholder
   - Table headers (Status, Repository, Provider, PRs, Actions)
   - Empty states
   - Toast messages
   - Button labels

6. **Backend Locale Files**
   - New keys for API responses in merge, analytics, repositories handlers
   - Error messages currently hardcoded in Rust handlers

### Out of Scope

- Layout components (already using navigation keys)
- Settings pages (already complete)
- Login page (already complete)
- Dashboard page (already complete)

---

## Implementation Plan

### Phase 1: Frontend Key Extraction (2-3 hours)

#### Step 1.1: Create New Namespace Files

Create new English source files:

```bash
frontend/public/locales/en/analytics.json
frontend/public/locales/en/merge.json
frontend/public/locales/en/repositories.json
```

#### Step 1.2: Extract Analytics Keys

```json
// frontend/public/locales/en/analytics.json
{
  "title": "Analytics",
  "subtitle": "PR metrics and repository health insights",
  "summary": {
    "prsMerged": "PRs Merged ({{days}}d)",
    "avgMergeTime": "Avg Merge Time",
    "avgReviewTime": "Avg Review Time",
    "botPrs": "Bot PRs"
  },
  "health": {
    "title": "Repository Health Scores",
    "mergeTime": "Merge Time",
    "throughput": "Throughput",
    "perWeek": "{{count}}/week",
    "reviewTime": "Review Time",
    "stalePrs": "Stale PRs"
  },
  "empty": {
    "title": "No health data available yet",
    "description": "Health scores are calculated hourly based on PR metrics"
  },
  "trends": {
    "up": "Trending up",
    "down": "Trending down",
    "stable": "Stable"
  }
}
```

#### Step 1.3: Extract Merge Keys

```json
// frontend/public/locales/en/merge.json
{
  "title": "Bulk Merge",
  "subtitle": "Select and merge multiple pull requests at once",
  "options": {
    "title": "Merge Options",
    "description": "Configure how selected PRs will be merged",
    "strategy": "Merge Strategy",
    "strategies": {
      "squash": "Squash and merge",
      "merge": "Create a merge commit",
      "rebase": "Rebase and merge"
    },
    "deleteBranch": "Delete branches after merge"
  },
  "selection": {
    "count": "{{selected}} of {{total}} PRs selected",
    "selectAll": "Select All",
    "deselectAll": "Deselect All"
  },
  "actions": {
    "merge": "Merge {{count}} PR",
    "merge_other": "Merge {{count}} PRs",
    "merging": "Merging..."
  },
  "sections": {
    "readyTitle": "Ready to Merge",
    "readyDescription": "PRs that have passed all checks and are ready to merge",
    "notReadyTitle": "Not Ready",
    "notReadyDescription": "PRs that need attention before they can be merged"
  },
  "empty": {
    "title": "No PRs are ready to merge. PRs must have:",
    "requirements": {
      "ci": "All CI checks passing",
      "approvals": "Required approvals",
      "noConflicts": "No merge conflicts"
    }
  },
  "blockers": {
    "draft": "Draft",
    "conflicts": "Conflicts",
    "ciFailed": "CI failed",
    "ciPending": "CI pending",
    "changesRequested": "Changes requested",
    "awaitingReview": "Awaiting review",
    "needsReview": "Needs review"
  },
  "pr": {
    "count_one": "{{count}} PR",
    "count_other": "{{count}} PRs"
  },
  "toast": {
    "successTitle": "Merge successful",
    "successDescription": "Successfully merged {{count}} PR(s)",
    "partialTitle": "Some merges failed",
    "partialDescription": "{{success}} merged, {{failed}} failed",
    "failedTitle": "Merge failed"
  },
  "results": {
    "title": "Merge Results",
    "allSuccess": "All PRs merged successfully!",
    "summary": "Completed with {{success}} merged, {{failed}} failed, {{skipped}} skipped",
    "labels": {
      "total": "Total",
      "merged": "Merged",
      "failed": "Failed",
      "skipped": "Skipped"
    },
    "statuses": {
      "success": "success",
      "failed": "failed",
      "skipped": "skipped"
    },
    "close": "Close"
  }
}
```

#### Step 1.4: Extract Repositories Keys

```json
// frontend/public/locales/en/repositories.json
{
  "title": "Repositories",
  "providers": {
    "title": "Connect Providers",
    "connected": "{{provider}} (Connected)",
    "github": "GitHub",
    "gitlab": "GitLab",
    "bitbucket": "Bitbucket"
  },
  "addFrom": {
    "title": "Add from {{provider}}",
    "addAll": "Add all ({{count}})",
    "close": "Close",
    "empty": "No additional repositories found"
  },
  "search": {
    "placeholder": "Search repositories..."
  },
  "table": {
    "status": "Status",
    "repository": "Repository",
    "provider": "Provider",
    "prs": "PRs",
    "actions": "Actions"
  },
  "empty": {
    "title": "No repositories found",
    "description": "Connect a provider and add repositories to get started"
  },
  "toast": {
    "addedTitle": "Repository added",
    "addedDescription": "{{name}} has been added to your watchlist",
    "addFailedTitle": "Failed to add repository",
    "removedTitle": "Repository removed",
    "removedDescription": "{{name}} has been removed from your watchlist",
    "removeFailedTitle": "Failed to remove repository",
    "bulkAddedTitle": "Repositories added",
    "bulkAddedDescription": "{{success}} {{repo}} added to your watchlist",
    "bulkAddedPartial": "{{success}} {{repo}} added to your watchlist, {{failed}} failed",
    "bulkFailedTitle": "Failed to add repositories",
    "bulkFailedDescription": "Could not add any repositories"
  },
  "plurals": {
    "repository_one": "repository",
    "repository_other": "repositories"
  }
}
```

#### Step 1.5: Update Common Namespace

Add missing keys to `frontend/public/locales/en/common.json`:

```json
{
  "auth": {
    "displayNamePlaceholder": "John Doe"
  },
  "providers": {
    "github": "GitHub",
    "gitlab": "GitLab",
    "bitbucket": "Bitbucket"
  }
}
```

---

### Phase 2: Frontend Component Updates (2-3 hours)

#### Step 2.1: Update Analytics.tsx

```tsx
import { useTranslation } from 'react-i18next';

export default function Analytics() {
  const { t } = useTranslation(['analytics', 'common']);

  // Replace all hardcoded strings with t() calls
  // Example: "Analytics" → t('analytics:title')
  // Example: "PRs Merged (30d)" → t('analytics:summary.prsMerged', { days: 30 })
}
```

**Key Mappings:**

| Hardcoded String | Translation Key |
|------------------|-----------------|
| `"Analytics"` | `analytics:title` |
| `"PRs Merged (30d)"` | `analytics:summary.prsMerged` |
| `"Avg Merge Time"` | `analytics:summary.avgMergeTime` |
| `"Avg Review Time"` | `analytics:summary.avgReviewTime` |
| `"Bot PRs"` | `analytics:summary.botPrs` |
| `"Repository Health Scores"` | `analytics:health.title` |
| `"Merge Time"` | `analytics:health.mergeTime` |
| `"Throughput"` | `analytics:health.throughput` |
| `"Review Time"` | `analytics:health.reviewTime` |
| `"Stale PRs"` | `analytics:health.stalePrs` |
| `"/week"` | `analytics:health.perWeek` |
| `"No health data available yet"` | `analytics:empty.title` |
| `"Health scores are calculated..."` | `analytics:empty.description` |

#### Step 2.2: Update Merge.tsx

```tsx
import { useTranslation } from 'react-i18next';

export default function Merge() {
  const { t } = useTranslation(['merge', 'common', 'errors']);

  // Replace all hardcoded strings with t() calls
}
```

**Key Mappings:**

| Hardcoded String | Translation Key |
|------------------|-----------------|
| `"Bulk Merge"` | `merge:title` |
| `"Select and merge multiple..."` | `merge:subtitle` |
| `"Merge Options"` | `merge:options.title` |
| `"Configure how selected..."` | `merge:options.description` |
| `"Merge Strategy"` | `merge:options.strategy` |
| `"Squash and merge"` | `merge:options.strategies.squash` |
| `"Create a merge commit"` | `merge:options.strategies.merge` |
| `"Rebase and merge"` | `merge:options.strategies.rebase` |
| `"Delete branches after merge"` | `merge:options.deleteBranch` |
| `"Select All"` / `"Deselect All"` | `merge:selection.selectAll` / `merge:selection.deselectAll` |
| `"Ready to Merge"` | `merge:sections.readyTitle` |
| `"Not Ready"` | `merge:sections.notReadyTitle` |
| `"Draft"`, `"Conflicts"`, etc. | `merge:blockers.*` |
| Toast messages | `merge:toast.*` |

#### Step 2.3: Update MergeResultsDialog.tsx

```tsx
import { useTranslation } from 'react-i18next';

export function MergeResultsDialog({ ... }) {
  const { t } = useTranslation(['merge']);

  // Replace all hardcoded strings with t() calls
}
```

#### Step 2.4: Update Register.tsx

Add missing placeholder:

```tsx
<Input
  id="displayName"
  placeholder={t('common:auth.displayNamePlaceholder')}
  {...register('displayName')}
/>
```

#### Step 2.5: Update Repositories.tsx

```tsx
import { useTranslation } from 'react-i18next';

export default function Repositories() {
  const { t } = useTranslation(['repositories', 'common', 'errors']);

  // Replace all hardcoded strings with t() calls
}
```

---

### Phase 3: Translation Generation (1-2 hours)

#### Step 3.1: Run Missing Keys Check

```bash
cd crates/ampel-i18n-builder
cargo run --bin cargo-i18n -- missing --translation-dir ../../frontend/public/locales
```

#### Step 3.2: Translate All Languages

```bash
# Navigate to i18n-builder directory (important for .env loading)
cd crates/ampel-i18n-builder

# Translate new namespaces for all 26 target languages
for lang in ar cs da de en-GB es-ES es-MX fi fr he hi it ja ko nl no pl pt-BR ru sr sv th tr vi zh-CN zh-TW; do
  echo "Translating analytics for $lang..."
  cargo run --bin cargo-i18n -- translate \
    --lang "$lang" \
    --namespace analytics \
    --translation-dir ../../frontend/public/locales

  echo "Translating merge for $lang..."
  cargo run --bin cargo-i18n -- translate \
    --lang "$lang" \
    --namespace merge \
    --translation-dir ../../frontend/public/locales

  echo "Translating repositories for $lang..."
  cargo run --bin cargo-i18n -- translate \
    --lang "$lang" \
    --namespace repositories \
    --translation-dir ../../frontend/public/locales
done
```

#### Step 3.3: Regenerate TypeScript Types

```bash
cargo run --bin cargo-i18n -- generate-types --translation-dir ../../frontend/public/locales
```

This updates `frontend/src/i18n/types.ts` with:

```typescript
export type TranslationNamespace =
  | 'common'
  | 'dashboard'
  | 'errors'
  | 'settings'
  | 'validation'
  | 'analytics'  // NEW
  | 'merge'      // NEW
  | 'repositories'; // NEW

export interface AnalyticsTranslations { ... }  // NEW
export interface MergeTranslations { ... }      // NEW
export interface RepositoriesTranslations { ... } // NEW
```

---

### Phase 4: Backend i18n Integration (1-2 hours)

#### Step 4.1: Create Backend Locale Files

Create English source files for new namespaces:

```yaml
# crates/ampel-api/locales/en/analytics.yml
analytics:
  errors:
    repository_not_found: "Repository not found"
    no_data_available: "No analytics data available"
    calculation_failed: "Failed to calculate health score"
```

```yaml
# crates/ampel-api/locales/en/merge.yml
merge:
  errors:
    no_prs_specified: "No pull requests specified"
    too_many_prs: "Cannot merge more than %{max} PRs at once"
    pr_not_found: "Pull request %{id} not found"
    repository_not_found: "Repository not found"
    provider_account_not_found: "Provider account not found"
    repository_not_linked: "Repository not linked to account"
    token_error: "Token error: %{error}"
    verify_state_failed: "Failed to verify PR state: %{error}"
    merge_not_completed: "Merge not completed"
    operation_not_found: "Merge operation not found"

  status:
    success: "Success"
    failed: "Failed"
    skipped: "Skipped"
    unknown: "Unknown"
```

```yaml
# crates/ampel-api/locales/en/repositories.yml
repositories:
  errors:
    not_found: "Repository not found"
    already_added: "Repository already added"
    provider_not_connected: "Provider not connected"
    invalid_provider: "Invalid provider type"

  success:
    added: "Repository added successfully"
    removed: "Repository removed successfully"
```

#### Step 4.2: Update Handler Files

Update handlers to use `t!()` macro instead of hardcoded strings:

```rust
// crates/ampel-api/src/handlers/bulk_merge.rs
use rust_i18n::t;

// Replace:
return Err(ApiError::bad_request("No pull requests specified"));
// With:
return Err(ApiError::bad_request(t!("merge.errors.no_prs_specified")));
```

#### Step 4.3: Translate Backend Locales

```bash
cd crates/ampel-i18n-builder

# Create backend locale translation script
for lang in ar cs da de en-GB es-ES es-MX fi fr he hi it ja ko nl no pl pt-BR ru sr sv th tr vi zh-CN zh-TW; do
  # Copy English files to target language if they don't exist
  mkdir -p ../ampel-api/locales/$lang

  # Use CLI to translate (backend uses YAML format)
  cargo run --bin cargo-i18n -- translate \
    --lang "$lang" \
    --namespace analytics \
    --translation-dir ../ampel-api/locales

  cargo run --bin cargo-i18n -- translate \
    --lang "$lang" \
    --namespace merge \
    --translation-dir ../ampel-api/locales

  cargo run --bin cargo-i18n -- translate \
    --lang "$lang" \
    --namespace repositories \
    --translation-dir ../ampel-api/locales
done
```

---

### Phase 5: Testing Updates (1 hour)

#### Step 5.1: Update Test Files

Each test file needs i18n mocking:

```typescript
// Analytics.test.tsx
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: Record<string, unknown>) => {
      // Return key or interpolated string for testing
      if (options) {
        return Object.entries(options).reduce(
          (acc, [k, v]) => acc.replace(`{{${k}}}`, String(v)),
          key.split(':').pop() || key
        );
      }
      return key.split(':').pop() || key;
    },
    i18n: { language: 'en' },
  }),
}));
```

#### Step 5.2: Run Tests

```bash
make test-frontend
```

---

### Phase 6: Validation (30 min)

#### Step 6.1: Coverage Report

```bash
node scripts/i18n-coverage-report.js --format text
```

Expected output:
```
✓ All 27 languages at 100% coverage
✓ No missing keys detected
✓ No placeholder mismatches
```

#### Step 6.2: Quality Validation

```bash
node scripts/validate-translations.js
```

#### Step 6.3: Build Verification

```bash
make build-release
make test
```

---

## File Inventory

### Files to Create

| Path | Type | Description |
|------|------|-------------|
| `frontend/public/locales/en/analytics.json` | JSON | Analytics namespace (English source) |
| `frontend/public/locales/en/merge.json` | JSON | Merge namespace (English source) |
| `frontend/public/locales/en/repositories.json` | JSON | Repositories namespace (English source) |
| `frontend/public/locales/{26 langs}/analytics.json` | JSON | Translated analytics (auto-generated) |
| `frontend/public/locales/{26 langs}/merge.json` | JSON | Translated merge (auto-generated) |
| `frontend/public/locales/{26 langs}/repositories.json` | JSON | Translated repositories (auto-generated) |
| `crates/ampel-api/locales/en/analytics.yml` | YAML | Backend analytics locale |
| `crates/ampel-api/locales/en/merge.yml` | YAML | Backend merge locale |
| `crates/ampel-api/locales/en/repositories.yml` | YAML | Backend repositories locale |

### Files to Modify

| Path | Changes |
|------|---------|
| `frontend/src/pages/Analytics.tsx` | Add useTranslation, replace hardcoded strings |
| `frontend/src/pages/Merge.tsx` | Add useTranslation, replace hardcoded strings |
| `frontend/src/pages/Register.tsx` | Add displayName placeholder |
| `frontend/src/pages/Repositories.tsx` | Add useTranslation, replace hardcoded strings |
| `frontend/src/components/merge/MergeResultsDialog.tsx` | Add useTranslation |
| `frontend/public/locales/en/common.json` | Add displayNamePlaceholder, provider names |
| `frontend/src/i18n/types.ts` | Auto-regenerated with new interfaces |
| `crates/ampel-api/src/handlers/bulk_merge.rs` | Replace hardcoded strings with t!() |
| `crates/ampel-api/src/handlers/analytics.rs` | Replace hardcoded strings with t!() |
| `crates/ampel-api/src/handlers/repositories.rs` | Replace hardcoded strings with t!() |

---

## Translation Key Specifications

### Pluralization Rules

Use i18next pluralization suffix pattern:

```json
{
  "pr_one": "{{count}} PR",
  "pr_other": "{{count}} PRs"
}
```

### Placeholder Format

- Frontend (i18next): `{{variable}}`
- Backend (rust-i18n): `%{variable}`

### Naming Conventions

- **camelCase** for key names
- **Nested objects** for related keys
- **Descriptive names** indicating purpose

---

## Backend i18n Integration

### Accept-Language Header Processing

The backend already supports locale detection via `LocaleLayer` middleware:

```rust
// middleware/locale_layer.rs
// Extracts locale from Accept-Language header
// Falls back to "en" if not detected
```

### Using t!() Macro

```rust
use rust_i18n::t;

// Simple translation
t!("merge.errors.no_prs_specified")

// With interpolation
t!("merge.errors.too_many_prs", max = 50)
```

---

## Testing Strategy

### Unit Tests

- Mock `useTranslation` hook
- Verify translation keys are passed correctly
- Test placeholder interpolation

### Integration Tests

- Test full page render with translations
- Verify no missing translation warnings
- Test RTL layout for Arabic/Hebrew

### Visual Regression

- Screenshot comparison for each language
- Verify text overflow handling
- Check RTL alignment

---

## Validation Checklist

- [ ] All hardcoded strings extracted to JSON files
- [ ] useTranslation hook added to all target components
- [ ] All 27 languages have complete translations
- [ ] TypeScript types regenerated and valid
- [ ] Placeholder format consistent ({{name}})
- [ ] No console warnings for missing translations
- [ ] Tests passing with i18n mocking
- [ ] Build succeeds
- [ ] Coverage report shows 100%
- [ ] Quality validation passes
- [ ] RTL languages display correctly
- [ ] Backend handlers use t!() macro
- [ ] Backend locales translated

---

## Rollout Plan

### Stage 1: Development (This PR)

1. Create English source files
2. Update React components
3. Generate translations
4. Update tests
5. PR review

### Stage 2: QA Validation

1. Manual testing of each language
2. RTL testing (Arabic, Hebrew)
3. Screenshot validation
4. Performance testing

### Stage 3: Production Deploy

1. Merge to main
2. Deploy to staging
3. Final validation
4. Production release

---

## Appendix: Command Reference

```bash
# Check missing translations
cd crates/ampel-i18n-builder
cargo run --bin cargo-i18n -- missing --translation-dir ../../frontend/public/locales

# Translate specific namespace/language
cargo run --bin cargo-i18n -- translate --lang de --namespace merge --translation-dir ../../frontend/public/locales

# Force retranslate all keys
cargo run --bin cargo-i18n -- translate --lang de --force --translation-dir ../../frontend/public/locales

# Generate TypeScript types
cargo run --bin cargo-i18n -- generate-types --translation-dir ../../frontend/public/locales

# Check coverage
cargo run --bin cargo-i18n -- coverage --min-coverage 95 --translation-dir ../../frontend/public/locales

# Validate translations
node scripts/validate-translations.js

# Generate coverage report
node scripts/i18n-coverage-report.js --format text
```

---

**Last Updated:** 2026-01-17
**Maintained By:** Ampel Development Team
