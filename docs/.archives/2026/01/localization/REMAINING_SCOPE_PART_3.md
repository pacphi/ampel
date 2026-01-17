# Table Column Headers & PR Filter Localization - Technical Implementation Plan

**Version:** 1.1
**Date:** 2026-01-17
**Status:** Planning
**Target Components:**

- `ListView.tsx` (Repository List View - Table Headers)
- `PRListView.tsx` (Pull Request List View - Filter Dropdown)
  **Languages Affected:** All 27 supported languages

---

## 📋 Table of Contents

1. [Problem Statement](#problem-statement)
2. [Current State Analysis](#current-state-analysis)
3. [Solution Design](#solution-design)
4. [Implementation Steps](#implementation-steps)
5. [Translation Key Structure](#translation-key-structure)
6. [Code Changes](#code-changes)
7. [Testing Strategy](#testing-strategy)
8. [Rollout Plan](#rollout-plan)
9. [Maintenance](#maintenance)

---

## 🎯 Problem Statement

### Issue #1: Table Column Headers

The `ListView` component (`frontend/src/components/dashboard/ListView.tsx`) currently displays hardcoded English table column headers:

- **Status** (line 116)
- **Repository** (line 122)
- **Visibility** (line 128)
- **Provider** (line 138)
- **PRs** (line 144)
- **Last Updated** (line 150)

### Issue #2: PR Filter Dropdown

The `PRListView` component (`frontend/src/components/dashboard/PRListView.tsx`) displays hardcoded English filter options:

- **All PRs** (line 136)
- **Ready** (line 137)
- **Pending** (line 139)
- **Blocked** (line 141)

### Impact

- When users switch to non-English languages (e.g., Polish), the entire UI is localized **except** for these elements
- Creates inconsistent user experience
- Affects **100% of repository list views** and **100% of PR list views** across all 27 supported languages

### Discovery Context

User reported seeing Polish translations throughout the interface, but table headers and PR filter remained in English:

```
Interface Language: Polish ✅
Dashboard: Localized ✅
Navigation: Localized ✅
Buttons: Localized ✅
Table Headers: English ❌ <- ISSUE #1
PR Filter Dropdown: English ❌ <- ISSUE #2
```

---

## 🔍 Current State Analysis

### Component Locations

#### Component #1: ListView (Repository Table)

- **File:** `frontend/src/components/dashboard/ListView.tsx`
- **Lines:** 114-157 (table header definition)
- **Component Type:** React functional component with sorting capability
- **Hardcoded Strings:** 6 column headers

#### Component #2: PRListView (PR Filter Dropdown)

- **File:** `frontend/src/components/dashboard/PRListView.tsx`
- **Lines:** 136-141 (filter dropdown options)
- **Component Type:** React functional component with filtering
- **Hardcoded Strings:** 4 filter options

### Current Implementation

```tsx
<thead>
  <tr className="border-b bg-muted/50">
    <th className={headerClass} onClick={() => handleSort('status')}>
      <div className="flex items-center">
        Status {/* ❌ Hardcoded */}
        <SortIcon column="status" sortColumn={sortColumn} sortDirection={sortDirection} />
      </div>
    </th>
    {/* ... 5 more hardcoded headers ... */}
  </tr>
</thead>
```

### i18n Setup Already in Place

✅ **React-i18next** configured
✅ **27 language JSON files** exist in `frontend/public/locales/{lang}/`
✅ **Dashboard namespace** already used elsewhere: `dashboard.json`
✅ **Translation workflow** documented and operational
✅ **CI/CD validation** checks for missing keys

### Namespace Structure

```
frontend/public/locales/{lang}/
├── analytics.json
├── behavior.json
├── common.json
├── dashboard.json      ← TARGET NAMESPACE
├── errors.json
├── notifications.json
├── providers.json
├── repositories.json
├── settings.json
└── validation.json
```

---

## 🏗️ Solution Design

### Design Principles

1. **Namespace Reuse:** Add keys to existing `dashboard.json` (component already in dashboard context)
2. **Hierarchical Structure:** Group under `table.columns.*` for clarity
3. **Consistency:** Match existing translation patterns in project
4. **Type Safety:** Leverage react-i18next TypeScript support
5. **Zero Breaking Changes:** Backward compatible implementation

### Translation Key Hierarchy

```json
{
  "dashboard": {
    "table": {
      "columns": {
        "status": "Status",
        "repository": "Repository",
        "visibility": "Visibility",
        "provider": "Provider",
        "prs": "PRs",
        "lastUpdated": "Last Updated"
      }
    },
    "filters": {
      "prStatus": {
        "all": "All PRs",
        "ready": "Ready",
        "pending": "Pending",
        "blocked": "Blocked"
      }
    }
  }
}
```

### Why This Structure?

- **`dashboard`** - Existing namespace, both components live in dashboard feature
- **`table.columns`** - Groups table column headers
- **`filters.prStatus`** - Groups PR status filter options (separate from existing `filters` which has different semantics)
- **Flat keys within groups** - Simple lookup, no over-nesting
- **Reuses existing namespace** - Consistent with current i18n patterns

---

## 📝 Implementation Steps

### Phase 1: Update English Source Files (10 minutes)

**File:** `frontend/public/locales/en/dashboard.json`

**Action:** Add new `table` and `filters.prStatus` sections to existing JSON structure

```json
{
  "title": "Dashboard",
  "prDashboard": "PR Dashboard",
  "subtitle": "Manage your pull requests across all providers",

  // ... existing keys (empty, filters, status, actions, stats, views, etc.) ...

  "table": {
    "columns": {
      "status": "Status",
      "repository": "Repository",
      "visibility": "Visibility",
      "provider": "Provider",
      "prs": "PRs",
      "lastUpdated": "Last Updated"
    }
  },
  "filters": {
    "all": "All",
    "open": "Open",
    "closed": "Closed",
    // ... existing filter keys ...
    "prStatus": {
      "all": "All PRs",
      "ready": "Ready",
      "pending": "Pending",
      "blocked": "Blocked"
    }
  }
}
```

**Notes:**

- Preserve all existing keys (no deletions)
- Add `table` as new top-level object
- Add `prStatus` as nested object under existing `filters` object
- Alphabetical ordering recommended

---

### Phase 2: Update Component Code (20 minutes)

#### 2A: Update ListView.tsx

**File:** `frontend/src/components/dashboard/ListView.tsx`

**Changes Required:**

#### 2.1: Import useTranslation Hook

```tsx
// Add to existing imports (line 1-6)
import { useTranslation } from 'react-i18next';
```

#### 2.2: Initialize Translation Hook

```tsx
export default function ListView({ repositories }: ListViewProps) {
  // Add this line after function declaration (line 42)
  const { t } = useTranslation('dashboard');

  const [sortColumn, setSortColumn] = useState<SortColumn | null>(null);
  // ... rest of component
}
```

#### 2.3: Replace Hardcoded Strings

**Before:**

```tsx
<th className={headerClass} onClick={() => handleSort('status')}>
  <div className="flex items-center">
    Status
    <SortIcon column="status" sortColumn={sortColumn} sortDirection={sortDirection} />
  </div>
</th>
```

**After:**

```tsx
<th className={headerClass} onClick={() => handleSort('status')}>
  <div className="flex items-center">
    {t('table.columns.status')}
    <SortIcon column="status" sortColumn={sortColumn} sortDirection={sortDirection} />
  </div>
</th>
```

**Apply to All 6 Headers:**

| Line | Column Key                     | Translation Key                  |
| ---- | ------------------------------ | -------------------------------- |
| 116  | `status`                       | `t('table.columns.status')`      |
| 122  | `name` (displays "Repository") | `t('table.columns.repository')`  |
| 128  | `visibility`                   | `t('table.columns.visibility')`  |
| 138  | `provider`                     | `t('table.columns.provider')`    |
| 144  | `prs`                          | `t('table.columns.prs')`         |
| 150  | `lastUpdated`                  | `t('table.columns.lastUpdated')` |

---

#### 2B: Update PRListView.tsx

**File:** `frontend/src/components/dashboard/PRListView.tsx`

**Changes Required:**

#### 2B.1: Import useTranslation Hook

```tsx
// Add to existing imports (line 1-12)
import { useTranslation } from 'react-i18next';
```

#### 2B.2: Initialize Translation Hook

```tsx
export default function PRListView({ filterStatus }: PRListViewProps) {
  // Add this line after function declaration (line 18-24)
  const { t } = useTranslation('dashboard');

  const { toast } = useToast();
  // ... rest of component
}
```

#### 2B.3: Replace Hardcoded Filter Options

**Before:**

```tsx
<select
  value={statusFilter}
  onChange={(e) => setStatusFilter(e.target.value as AmpelStatus | 'all')}
  className="text-sm border rounded-md px-2 py-1 bg-background"
>
  <option value="all">All PRs ({totalPrs})</option>
  <option value="green">Ready ({prs.filter((p) => p.status === 'green').length})</option>
  <option value="yellow">Pending ({prs.filter((p) => p.status === 'yellow').length})</option>
  <option value="red">Blocked ({prs.filter((p) => p.status === 'red').length})</option>
</select>
```

**After:**

```tsx
<select
  value={statusFilter}
  onChange={(e) => setStatusFilter(e.target.value as AmpelStatus | 'all')}
  className="text-sm border rounded-md px-2 py-1 bg-background"
>
  <option value="all">
    {t('filters.prStatus.all')} ({totalPrs})
  </option>
  <option value="green">
    {t('filters.prStatus.ready')} ({prs.filter((p) => p.status === 'green').length})
  </option>
  <option value="yellow">
    {t('filters.prStatus.pending')} ({prs.filter((p) => p.status === 'yellow').length})
  </option>
  <option value="red">
    {t('filters.prStatus.blocked')} ({prs.filter((p) => p.status === 'red').length})
  </option>
</select>
```

**Summary of Changes:**

| Line | Original Text | Translation Key                 |
| ---- | ------------- | ------------------------------- |
| 136  | `"All PRs"`   | `t('filters.prStatus.all')`     |
| 137  | `"Ready"`     | `t('filters.prStatus.ready')`   |
| 139  | `"Pending"`   | `t('filters.prStatus.pending')` |
| 141  | `"Blocked"`   | `t('filters.prStatus.blocked')` |

---

### Phase 3: Generate Translations for 27 Languages (45 minutes)

**Strategy:** Use existing 4-tier translation provider architecture

#### Translation Provider Routing

**DeepL API** (Primary - 25 languages):

- Arabic (ar), Czech (cs), Danish (da), German (de), Spanish-Spain (es-ES),
- Spanish-Mexico (es-MX), Finnish (fi), French (fr), Italian (it), Japanese (ja),
- Korean (ko), Dutch (nl), Norwegian (no), Polish (pl), Portuguese-Brazil (pt-BR),
- Russian (ru), Swedish (sv), Chinese-Simplified (zh-CN), Chinese-Traditional (zh-TW),
- English-GB (en-GB), Hebrew (he), Hindi (hi), Serbian (sr), Turkish (tr), Vietnamese (vi)

**Google Cloud Translation API** (Fallback - 2 languages):

- Thai (th)
- Additional fallback coverage

#### Manual Process (Recommended for 10 keys)

Given the small number of keys (10 translations × 26 non-English languages = 260 translations), manual updating is efficient:

**Keys to translate:**

- 6 table column headers: `table.columns.*`
- 4 PR filter options: `filters.prStatus.*`

**Option A: Use Existing Translation Files as Reference**

Many languages already have `dashboard.json` with similar terms. Example pattern:

```bash
# Check existing translations for "Status" and "Provider"
grep -r "Status\|Provider" frontend/public/locales/*/dashboard.json
```

**Option B: DeepL API Batch Translation (Automated)**

```bash
# Pseudo-command (assuming ampel-i18n-builder is available)
ampel-i18n translate \
  --namespace dashboard \
  --key-paths table.columns,filters.prStatus \
  --source en \
  --targets ar,cs,da,de,es-ES,es-MX,fi,fr,he,hi,it,ja,ko,nl,no,pl,pt-BR,ru,sr,sv,th,tr,vi,zh-CN,zh-TW,en-GB

# Cost estimate: ~350 characters × 26 languages = 9,100 characters
# DeepL cost: ~€0.02 (negligible)
```

**Option C: Manual Translation Table (Recommended)**

For transparency and quality control, see **Appendix A** below for complete translation matrix (includes both table columns and PR filters).

---

### Phase 4: Add Translations to 26 Language Files (60 minutes)

**Process for Each Language:**

1. Open `frontend/public/locales/{lang}/dashboard.json`
2. Add the `table.columns` structure
3. Add the `filters.prStatus` structure under existing `filters` object
4. Translate the 10 values into target language
5. Validate JSON syntax

**Example: Polish (pl)**

```json
{
  "title": "Panel",
  "prDashboard": "Panel PR",
  "subtitle": "Zarządzaj swoimi żądaniami ściągnięcia u wszystkich dostawców",

  // ... existing keys ...

  "filters": {
    "all": "Wszystko",
    "open": "Otwarte",
    // ... existing filter keys ...
    "prStatus": {
      "all": "Wszystkie PR-y",
      "ready": "Gotowe",
      "pending": "Oczekujące",
      "blocked": "Zablokowane"
    }
  },

  // ... other existing keys ...

  "table": {
    "columns": {
      "status": "Status",
      "repository": "Repozytorium",
      "visibility": "Widoczność",
      "provider": "Dostawca",
      "prs": "PR-y",
      "lastUpdated": "Ostatnia aktualizacja"
    }
  }
}
```

**Batch Edit Script (Optional):**

```bash
#!/bin/bash
# scripts/add-dashboard-i18n-keys.sh

LANGUAGES=(ar cs da de en-GB es-ES es-MX fi fr he hi it ja ko nl no pl pt-BR ru sr sv th tr vi zh-CN zh-TW)

for LANG in "${LANGUAGES[@]}"; do
  FILE="frontend/public/locales/$LANG/dashboard.json"

  # Backup
  cp "$FILE" "$FILE.backup"

  # Add table.columns section using jq
  jq '.table = {
    "columns": {
      "status": "STATUS_TRANSLATION",
      "repository": "REPOSITORY_TRANSLATION",
      "visibility": "VISIBILITY_TRANSLATION",
      "provider": "PROVIDER_TRANSLATION",
      "prs": "PRS_TRANSLATION",
      "lastUpdated": "LASTUPDATED_TRANSLATION"
    }
  }' "$FILE" > "$FILE.tmp" && mv "$FILE.tmp" "$FILE"

  # Add filters.prStatus section using jq
  jq '.filters.prStatus = {
    "all": "ALLPRS_TRANSLATION",
    "ready": "READY_TRANSLATION",
    "pending": "PENDING_TRANSLATION",
    "blocked": "BLOCKED_TRANSLATION"
  }' "$FILE" > "$FILE.tmp" && mv "$FILE.tmp" "$FILE"

  echo "Updated $LANG - MANUAL TRANSLATION REQUIRED FOR 10 KEYS"
done
```

**Then manually replace placeholder values with actual translations from Appendix A.**

---

### Phase 5: Update Tests (25 minutes)

#### 5A: Update ListView Tests

**File:** `frontend/src/components/dashboard/ListView.test.tsx`

**Mock react-i18next:**

```tsx
// Add to test file
import { vi } from 'vitest';

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'table.columns.status': 'Status',
        'table.columns.repository': 'Repository',
        'table.columns.visibility': 'Visibility',
        'table.columns.provider': 'Provider',
        'table.columns.prs': 'PRs',
        'table.columns.lastUpdated': 'Last Updated',
      };
      return translations[key] || key;
    },
    i18n: { language: 'en' },
  }),
}));
```

**Verify Tests:**

```bash
npm test -- ListView.test.tsx
```

---

#### 5B: Update PRListView Tests

**File:** `frontend/src/components/dashboard/PRListView.test.tsx`

**Add to existing react-i18next mock:**

```tsx
// Extend existing mock or create new one
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        // ... existing translations ...
        'filters.prStatus.all': 'All PRs',
        'filters.prStatus.ready': 'Ready',
        'filters.prStatus.pending': 'Pending',
        'filters.prStatus.blocked': 'Blocked',
      };
      return translations[key] || key;
    },
    i18n: { language: 'en' },
  }),
}));
```

**Verify Tests:**

```bash
npm test -- PRListView.test.tsx
```

---

### Phase 6: Validation (15 minutes)

#### 6.1 Visual Testing

```bash
# Start dev server
make dev-frontend

# Test in browser:
# 1. Navigate to dashboard repository list view
# 2. Switch language to Polish (PL)
# 3. Verify all 6 column headers are translated
# 4. Test sorting still works
# 5. Navigate to dashboard PR list view
# 6. Verify all 4 filter options are translated
# 7. Test filtering still works
# 8. Repeat for 2-3 other languages (Arabic for RTL, Japanese for CJK)
```

#### 6.2 Missing Key Detection

```bash
# Run i18n validation (if tooling exists)
ampel-i18n validate --namespace dashboard --all-languages

# Expected output: 100% coverage for table.columns and filters.prStatus keys
```

#### 6.3 Type Safety Check

```bash
# Ensure TypeScript compilation passes
npm run type-check
```

---

## 🧪 Testing Strategy

### Unit Tests

**File:** `ListView.test.tsx`

**Test Cases:**

```tsx
describe('ListView i18n', () => {
  it('should render translated column headers', () => {
    const { getByText } = render(<ListView repositories={mockRepos} />);

    expect(getByText('Status')).toBeInTheDocument();
    expect(getByText('Repository')).toBeInTheDocument();
    expect(getByText('Visibility')).toBeInTheDocument();
    expect(getByText('Provider')).toBeInTheDocument();
    expect(getByText('PRs')).toBeInTheDocument();
    expect(getByText('Last Updated')).toBeInTheDocument();
  });

  it('should use dashboard namespace', () => {
    const { useTranslation } = require('react-i18next');
    render(<ListView repositories={mockRepos} />);

    expect(useTranslation).toHaveBeenCalledWith('dashboard');
  });
});
```

### Integration Tests

**Playwright E2E Tests:**

```typescript
// e2e/dashboard-i18n.spec.ts
test.describe('Dashboard Table Headers Localization', () => {
  test('should display Polish headers when language is Polish', async ({ page }) => {
    await page.goto('/dashboard');

    // Switch to Polish
    await page.click('[data-testid="language-switcher"]');
    await page.click('[data-testid="lang-pl"]');

    // Verify table headers
    await expect(page.locator('th:has-text("Status")')).toBeVisible();
    await expect(page.locator('th:has-text("Repozytorium")')).toBeVisible();
    await expect(page.locator('th:has-text("Widoczność")')).toBeVisible();
    await expect(page.locator('th:has-text("Dostawca")')).toBeVisible();
    await expect(page.locator('th:has-text("PR-y")')).toBeVisible();
    await expect(page.locator('th:has-text("Ostatnia aktualizacja")')).toBeVisible();
  });

  test('should maintain sorting functionality after localization', async ({ page }) => {
    await page.goto('/dashboard');
    await page.click('[data-testid="lang-pl"]');

    // Click Status column to sort
    await page.click('th:has-text("Status")');

    // Verify sort indicator appears
    await expect(page.locator('th:has-text("Status") svg')).toBeVisible();
  });
});
```

### Manual Testing Checklist

**Languages to Test Manually:**

| Priority | Language | Reason                | Tester Notes                                   |
| -------- | -------- | --------------------- | ---------------------------------------------- |
| P0       | English  | Source language       | Verify no regression                           |
| P0       | Polish   | User-reported issue   | Reported in initial bug                        |
| P0       | Arabic   | RTL layout            | Check header alignment                         |
| P1       | Japanese | CJK characters        | Font rendering, column width                   |
| P1       | Hebrew   | RTL + complex script  | Bidirectional text handling                    |
| P1       | Thai     | Complex script        | Tone marks, character composition              |
| P2       | German   | Long compound words   | Column width overflow check                    |
| P2       | Finnish  | Complex pluralization | (Not applicable to headers, but good baseline) |

**Test Procedure:**

1. Navigate to Dashboard → Repository List View
2. Switch language using language picker
3. Verify all 6 headers translated correctly
4. Click each header to verify sorting works
5. Check responsive layout (mobile, tablet, desktop)
6. Screenshot for visual regression baseline

---

## 📦 Rollout Plan

### Pre-Deployment Checklist

- [ ] All 27 language files updated with `table.columns` keys
- [ ] `ListView.tsx` modified to use `useTranslation`
- [ ] Unit tests passing
- [ ] E2E tests passing (Playwright)
- [ ] Manual testing completed for P0 languages (3/27)
- [ ] Type checks passing (`npm run type-check`)
- [ ] Bundle size impact measured (<1KB expected)
- [ ] CI/CD validation checks passing

### Deployment Strategy

**Stage 1: Staging Environment (Week 1)**

- Deploy to staging
- Internal QA testing across all 27 languages
- Performance testing (translation load time)
- Visual regression testing (Percy/Chromatic)

**Stage 2: Canary Release (Week 2)**

- 10% traffic rollout via feature flag
- Monitor error rates, translation loading metrics
- Collect user feedback

**Stage 3: Full Production (Week 3)**

- 100% rollout if no issues in canary
- Update documentation
- Close original issue

### Feature Flag (Optional)

```typescript
// Example feature flag implementation
const useLocalizedTableHeaders = useFeatureFlag('localized-table-headers', true);

const headerText = useLocalizedTableHeaders ? t('table.columns.status') : 'Status'; // Fallback
```

---

## 🔧 Maintenance

### Adding New Languages in Future

When adding language #28:

1. Create `frontend/public/locales/{new-lang}/dashboard.json`
2. Copy structure from `en/dashboard.json`
3. Translate all keys including `table.columns.*`
4. Run validation: `ampel-i18n validate --lang {new-lang}`
5. Test in UI manually

### Updating Column Headers

If new columns are added to `ListView`:

1. Add English key to `en/dashboard.json` under `table.columns.{newColumn}`
2. Run batch translation script for all 26 languages
3. Update component to use `t('table.columns.{newColumn}')`
4. Update tests
5. Follow standard i18n PR process

### Translation Quality Issues

If users report translation issues:

1. File GitHub issue with `[i18n]` prefix
2. Include language code, incorrect translation, suggested correction
3. Update specific language file
4. Optional: Re-run DeepL translation for comparison
5. Deploy as hotfix if critical

### CI/CD Integration

**GitHub Actions Workflow:**

```yaml
# .github/workflows/i18n-validation.yml
name: i18n Validation

on: [pull_request]

jobs:
  validate-translations:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Check for missing dashboard.table.columns keys
        run: |
          REQUIRED_KEYS=(status repository visibility provider prs lastUpdated)
          LANGUAGES=(en ar cs da de en-GB es-ES es-MX fi fr he hi it ja ko nl no pl pt-BR ru sr sv th tr vi zh-CN zh-TW)

          for LANG in "${LANGUAGES[@]}"; do
            FILE="frontend/public/locales/$LANG/dashboard.json"
            for KEY in "${REQUIRED_KEYS[@]}"; do
              if ! jq -e ".table.columns.$KEY" "$FILE" > /dev/null; then
                echo "ERROR: Missing key table.columns.$KEY in $LANG"
                exit 1
              fi
            done
          done

          echo "✅ All required keys present in all languages"
```

---

## 📊 Success Metrics

### Quantitative Metrics

- **Translation Coverage:** 100% (10 keys × 27 languages = 270 translations)
- **Components Updated:** 2 (ListView.tsx, PRListView.tsx)
- **Test Coverage:** >95% (unit + integration)
- **Bundle Size Impact:** <2KB (negligible)
- **Performance:** <10ms translation lookup overhead
- **Error Rate:** 0 missing key errors in production

### Qualitative Metrics

- **User Satisfaction:** No complaints about untranslated headers or filters
- **Consistency:** All table headers and filter options match language switcher selection
- **Maintainability:** New contributors can easily add columns/filters using established pattern

---

## 🎯 Timeline Estimate

| Phase | Task                              | Effort | Dependencies |
| ----- | --------------------------------- | ------ | ------------ |
| 1     | Update English source             | 10 min | None         |
| 2A    | Update ListView component         | 10 min | Phase 1      |
| 2B    | Update PRListView component       | 10 min | Phase 1      |
| 3     | Generate 26 language translations | 45 min | Phase 1      |
| 4     | Update all language files         | 60 min | Phase 3      |
| 5A    | Update ListView tests             | 10 min | Phase 2A     |
| 5B    | Update PRListView tests           | 15 min | Phase 2B     |
| 6     | Validation & QA                   | 15 min | Phase 4, 5   |
| 7     | Code review & PR                  | 45 min | All phases   |

**Total Effort:** ~3.5 hours (single developer)

**Recommended Approach:**

- Assign to developer familiar with i18n workflow
- Complete in single PR to avoid partial states
- Include screenshots in PR for visual verification

---

## 📚 Appendix A: Complete Translation Matrix

### Part 1: Table Column Translations by Language

| Language Code                   | Status     | Repository    | Visibility        | Provider         | PRs         | Last Updated          |
| ------------------------------- | ---------- | ------------- | ----------------- | ---------------- | ----------- | --------------------- |
| **en** (English)                | Status     | Repository    | Visibility        | Provider         | PRs         | Last Updated          |
| **en-GB** (English UK)          | Status     | Repository    | Visibility        | Provider         | PRs         | Last Updated          |
| **ar** (Arabic)                 | الحالة     | المستودع      | الرؤية            | المزود           | طلبات السحب | آخر تحديث             |
| **cs** (Czech)                  | Stav       | Repozitář     | Viditelnost       | Poskytovatel     | PR          | Poslední aktualizace  |
| **da** (Danish)                 | Status     | Repository    | Synlighed         | Udbyder          | PRs         | Senest opdateret      |
| **de** (German)                 | Status     | Repository    | Sichtbarkeit      | Anbieter         | PRs         | Zuletzt aktualisiert  |
| **es-ES** (Spanish-Spain)       | Estado     | Repositorio   | Visibilidad       | Proveedor        | PRs         | Última actualización  |
| **es-MX** (Spanish-Mexico)      | Estado     | Repositorio   | Visibilidad       | Proveedor        | PRs         | Última actualización  |
| **fi** (Finnish)                | Tila       | Repositorio   | Näkyvyys          | Palveluntarjoaja | PRs         | Viimeksi päivitetty   |
| **fr** (French)                 | Statut     | Dépôt         | Visibilité        | Fournisseur      | PRs         | Dernière mise à jour  |
| **he** (Hebrew)                 | סטטוס      | מאגר          | נראות             | ספק              | בקשות משיכה | עודכן לאחרונה         |
| **hi** (Hindi)                  | स्थिति     | रिपॉजिटरी     | दृश्यता           | प्रदाता          | PRs         | अंतिम अपडेट           |
| **it** (Italian)                | Stato      | Repository    | Visibilità        | Provider         | PR          | Ultimo aggiornamento  |
| **ja** (Japanese)               | ステータス | リポジトリ    | 可視性            | プロバイダー     | PR          | 最終更新日            |
| **ko** (Korean)                 | 상태       | 리포지토리    | 가시성            | 공급자           | PR          | 마지막 업데이트       |
| **nl** (Dutch)                  | Status     | Repository    | Zichtbaarheid     | Provider         | PRs         | Laatst bijgewerkt     |
| **no** (Norwegian)              | Status     | Repository    | Synlighet         | Leverandør       | PRs         | Sist oppdatert        |
| **pl** (Polish)                 | Status     | Repozytorium  | Widoczność        | Dostawca         | PR-y        | Ostatnia aktualizacja |
| **pt-BR** (Portuguese-Brazil)   | Status     | Repositório   | Visibilidade      | Provedor         | PRs         | Última atualização    |
| **ru** (Russian)                | Статус     | Репозиторий   | Видимость         | Провайдер        | PR          | Последнее обновление  |
| **sr** (Serbian)                | Статус     | Репозиторијум | Видљивост         | Провајдер        | PR-ови      | Последње ажурирање    |
| **sv** (Swedish)                | Status     | Repository    | Synlighet         | Leverantör       | PRs         | Senast uppdaterad     |
| **th** (Thai)                   | สถานะ      | ที่เก็บ       | การมองเห็น        | ผู้ให้บริการ     | PRs         | อัปเดตล่าสุด          |
| **tr** (Turkish)                | Durum      | Depo          | Görünürlük        | Sağlayıcı        | PR'ler      | Son Güncelleme        |
| **vi** (Vietnamese)             | Trạng thái | Kho lưu trữ   | Khả năng hiển thị | Nhà cung cấp     | PRs         | Cập nhật lần cuối     |
| **zh-CN** (Chinese-Simplified)  | 状态       | 仓库          | 可见性            | 提供商           | PR          | 最后更新              |
| **zh-TW** (Chinese-Traditional) | 狀態       | 儲存庫        | 可見性            | 提供者           | PR          | 最後更新              |

**Notes:**

- Translations generated using DeepL API (primary) and Google Cloud Translation API (fallback)
- RTL languages (Arabic, Hebrew) require no special handling for table headers (CSS handles directionality)
- "PRs" kept as acronym in most languages (common technical term)
- Some languages localized "Repository" phonetically (e.g., Finnish: "Repositorio")

---

### Part 2: PR Filter Translations by Language

| Language Code                   | All PRs          | Ready      | Pending        | Blocked       |
| ------------------------------- | ---------------- | ---------- | -------------- | ------------- |
| **en** (English)                | All PRs          | Ready      | Pending        | Blocked       |
| **en-GB** (English UK)          | All PRs          | Ready      | Pending        | Blocked       |
| **ar** (Arabic)                 | جميع طلبات السحب | جاهز       | قيد الانتظار   | محظور         |
| **cs** (Czech)                  | Všechny PR       | Připraveno | Čekající       | Blokováno     |
| **da** (Danish)                 | Alle PRs         | Klar       | Afventer       | Blokeret      |
| **de** (German)                 | Alle PRs         | Bereit     | Ausstehend     | Blockiert     |
| **es-ES** (Spanish-Spain)       | Todos los PRs    | Listo      | Pendiente      | Bloqueado     |
| **es-MX** (Spanish-Mexico)      | Todos los PRs    | Listo      | Pendiente      | Bloqueado     |
| **fi** (Finnish)                | Kaikki PR:t      | Valmis     | Odottaa        | Estetty       |
| **fr** (French)                 | Tous les PRs     | Prêt       | En attente     | Bloqué        |
| **he** (Hebrew)                 | כל בקשות המשיכה  | מוכן       | ממתין          | חסום          |
| **hi** (Hindi)                  | सभी PRs          | तैयार      | लंबित          | अवरुद्ध       |
| **it** (Italian)                | Tutti i PR       | Pronto     | In attesa      | Bloccato      |
| **ja** (Japanese)               | すべてのPR       | 準備完了   | 保留中         | ブロック済み  |
| **ko** (Korean)                 | 모든 PR          | 준비 완료  | 대기 중        | 차단됨        |
| **nl** (Dutch)                  | Alle PRs         | Klaar      | In behandeling | Geblokkeerd   |
| **no** (Norwegian)              | Alle PRs         | Klar       | Venter         | Blokkert      |
| **pl** (Polish)                 | Wszystkie PR-y   | Gotowe     | Oczekujące     | Zablokowane   |
| **pt-BR** (Portuguese-Brazil)   | Todos os PRs     | Pronto     | Pendente       | Bloqueado     |
| **ru** (Russian)                | Все PR           | Готово     | Ожидание       | Заблокировано |
| **sr** (Serbian)                | Сви PR-ови       | Спремно    | На чекању      | Блокирано     |
| **sv** (Swedish)                | Alla PRs         | Klar       | Väntande       | Blockerad     |
| **th** (Thai)                   | PRs ทั้งหมด      | พร้อม      | รอดำเนินการ    | ถูกบล็อก      |
| **tr** (Turkish)                | Tüm PR'ler       | Hazır      | Bekliyor       | Engellendi    |
| **vi** (Vietnamese)             | Tất cả PRs       | Sẵn sàng   | Đang chờ       | Bị chặn       |
| **zh-CN** (Chinese-Simplified)  | 所有PR           | 就绪       | 待处理         | 已阻止        |
| **zh-TW** (Chinese-Traditional) | 所有PR           | 就緒       | 待處理         | 已阻止        |

**Notes:**

- "All PRs" variations: Some languages use "Todos" (all), others "Wszystkie" (all), maintaining "PRs" as technical term
- "Ready" vs "Prêt/Klar/Готово" - context-aware translations for "ready to merge" status
- "Pending" vs "En attente/Oczekujące/待處理" - conveys "work in progress" state
- "Blocked" vs "Bloqué/Zablokowane/已阻止" - universally understood as "cannot proceed"

---

## 📚 Appendix B: Related Components

### Other Components That May Need Similar Treatment

**Potential Future Candidates for Localization:**

1. **GridView.tsx** - Repository grid view (may have sortable columns)
2. **PRListView.tsx** - ~~Pull request list (has filter labels)~~ ✅ **INCLUDED IN THIS PLAN**
3. **Analytics.tsx** - Chart labels and axis titles
4. **RepositoriesPage.tsx** - Repository table columns
5. **Select All / Deselect All buttons** - Already in `dashboard.actions` namespace

**Recommendation:** Audit all `<th>`, `<thead>`, `<select>`, and dropdown components for hardcoded English text after this implementation is successful.

---

## 📚 Appendix C: Code Review Checklist

**For Reviewers:**

- [ ] English source file (`en/dashboard.json`) has correct structure
- [ ] All 27 language files include `table.columns` keys
- [ ] All 27 language files include `filters.prStatus` keys
- [ ] No missing keys (run `ampel-i18n validate`)
- [ ] ListView.tsx imports and uses `useTranslation('dashboard')`
- [ ] PRListView.tsx imports and uses `useTranslation('dashboard')`
- [ ] All 6 column headers use translation keys
- [ ] All 4 filter options use translation keys
- [ ] Tests mock `react-i18next` correctly (both components)
- [ ] No TypeScript errors
- [ ] No console warnings about missing translations
- [ ] Screenshots included for 2-3 representative languages (both views)
- [ ] Bundle size diff is acceptable (<2KB)
- [ ] Sorting functionality still works (manual QA)
- [ ] Filtering functionality still works (manual QA)

---

## 📞 Support & Questions

**Documentation Issues:**

- File issue with `[docs][i18n]` prefix
- Reference this document: `TABLE-COLUMN-HEADERS-LOCALIZATION.md`

**Implementation Questions:**

- See [DEVELOPER-GUIDE.md](./DEVELOPER-GUIDE.md) for general i18n workflow
- See [TRANSLATION_WORKFLOW.md](./TRANSLATION_WORKFLOW.md) for CLI tools

**Translation Quality:**

- File issue with `[i18n][quality]` prefix
- Include: language code, current text, suggested correction, context

---

**Document Version:** 1.1
**Last Updated:** 2026-01-17
**Author:** Ampel Development Team
**Status:** Ready for Implementation

---

## 📝 Changelog

### Version 1.1 (2026-01-17)

- ✨ Added PRListView filter dropdown localization (4 additional keys)
- 📊 Updated total translation count: 10 keys × 27 languages = 270 translations
- ⏱️ Updated timeline estimate: 2.5 → 3.5 hours
- 📚 Added Appendix A Part 2: PR Filter Translations matrix
- 🧪 Added PRListView test update guidance

### Version 1.0 (2026-01-17)

- 📄 Initial technical implementation plan
- 🎯 ListView table column headers localization
- 📊 6 keys × 27 languages = 162 translations
- ⏱️ 2.5 hour implementation estimate
