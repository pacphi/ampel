# Lint Quality Report

**Generated**: 2025-12-22
**Branch**: `feature/path-to-production`
**Status**: ✅ PASSING (all 108 issues fixed)

---

## Summary

| Category              | Errors | Warnings | Total  |
| --------------------- | ------ | -------- | ------ |
| Backend (Rust)        | 31     | 1        | 32     |
| Frontend (TypeScript) | 1      | 58       | 59     |
| **Total**             | **32** | **59**   | **85** |

---

## Backend Issues (Rust)

### 1. Configuration Errors (`unexpected-cfgs`) — 4 issues

**Location**: `crates/ampel-api/src/observability.rs`
**Severity**: Error
**Description**: Feature `openapi` is used but not declared in `Cargo.toml`

| Line | Code                                                         |
| ---- | ------------------------------------------------------------ |
| 96   | `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` |
| 104  | `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` |
| 110  | `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` |
| 117  | `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` |

**Fix**: Add `openapi` feature to `crates/ampel-api/Cargo.toml` or remove conditional attributes.

---

### 2. Unused Imports (`unused-imports`) — 1 issue

**Location**: `crates/ampel-api/src/observability.rs:89`
**Severity**: Error
**Code**: `use sea_orm::DatabaseConnection;`

**Fix**: Remove the unused import.

---

### 3. Type Mismatch Errors (`E0308`) — 3 issues

**File**: `crates/ampel-api/src/observability.rs`

| Line | Expected         | Found     | Context                  |
| ---- | ---------------- | --------- | ------------------------ |
| 17   | `MetricKindMask` | `Matcher` | `.idle_timeout()` method |

**File**: `crates/ampel-worker/tests/metrics_collection_tests.rs`

| Line | Expected | Found | Context                                       |
| ---- | -------- | ----- | --------------------------------------------- |
| 609  | `i32`    | `i64` | `create_test_pull_request()` number parameter |

**Fix**: Use correct types or add explicit conversions (`.try_into().unwrap()`).

---

### 4. Trait Bound Errors (`E0277`) — 16 issues

**Location**: `crates/ampel-api/src/observability.rs`
**Description**: Structs missing `utoipa::ToSchema` / `utoipa::PartialSchema` implementations

**Affected structs**:

- `HealthResponse` (lines 30, 31, 69)
- `ComponentHealth` (lines 45, 46, 47, 51, 52, 53, 57, 58, 59)
- `MetricsResponse` (line 71)
- `HealthStatus` (multiple references)
- `ComponentName` (multiple references)

**Fix**: Add `#[derive(utoipa::ToSchema)]` to affected structs, or add `openapi` feature to conditionally enable.

---

### 5. Missing Trait Import (`E0599`) — 6 issues

**Location**: `crates/ampel-worker/tests/metrics_collection_tests.rs`
**Description**: `.count()` method requires `PaginatorTrait` in scope

| Lines   |
| ------- |
| 268-271 |
| 293-296 |
| 619-621 |
| 693-695 |

**Fix**: Add `use sea_orm::PaginatorTrait;` at top of file.

---

### 6. Type Annotation Needed (`E0282`) — 3 issues

**Location**: `crates/ampel-worker/tests/metrics_collection_tests.rs`
**Description**: Compiler cannot infer type for `.count()` result

| Lines   |
| ------- |
| 268-272 |
| 293-296 |
| 619-622 |
| 693-696 |

**Fix**: Add explicit type annotation or ensure `PaginatorTrait` is imported.

---

## Frontend Issues (TypeScript)

### 1. TypeScript Strict Typing (`@typescript-eslint/no-explicit-any`) — 58 warnings

**Description**: Use of `any` type undermines TypeScript's type safety

**Affected Files**:

| File                                           | Count | Lines                                               |
| ---------------------------------------------- | ----- | --------------------------------------------------- |
| `src/components/dashboard/GridView.test.tsx`   | 1     | 6                                                   |
| `src/components/dashboard/ListView.test.tsx`   | 1     | 6                                                   |
| `src/components/dashboard/PRCard.test.tsx`     | 1     | 6                                                   |
| `src/components/dashboard/PRListView.test.tsx` | 12    | 73, 95, 159, 205, 254, 299, 350, 393, 451, 499, 530 |
| `src/pages/Repositories.test.tsx`              | 44    | 73-589 (multiple)                                   |

**Fix**: Replace `any` with proper types:

```typescript
// Instead of:
const mock = vi.fn() as any;

// Use:
const mock = vi.fn() as Mock<typeof actualFunction>;
// or
const mock = vi.fn<[], ReturnType>();
```

---

### 2. Parsing Error — 1 error

**Location**: `src/pages/Settings.test.tsx:43:47`
**Description**: Syntax error - `','` expected

**Fix**: Review and correct the syntax at line 43.

---

## Priority Action Items

### Critical (Blocking Build)

1. **Fix observability.rs metrics exporter API**
   - Change `Matcher::Full(...)` to `MetricKindMask::...`
   - File: `crates/ampel-api/src/observability.rs:17`

2. **Fix utoipa schema derives**
   - Add `openapi` feature to `crates/ampel-api/Cargo.toml`
   - Or derive `ToSchema` unconditionally on structs

3. **Fix test file imports**
   - Add `use sea_orm::PaginatorTrait;` to `metrics_collection_tests.rs`
   - Fix `i64` to `i32` conversion at line 609

4. **Fix Settings.test.tsx parsing error**
   - Syntax error at line 43

### High (Type Safety)

5. **Replace `any` types in test files** (58 occurrences)
   - Create proper mock types
   - Use `vi.fn()` with type parameters

### Low (Code Quality)

6. **Remove unused import**
   - `sea_orm::DatabaseConnection` in `observability.rs:89`

---

## Files Requiring Changes

| File                                                    | Issues | Severity |
| ------------------------------------------------------- | ------ | -------- |
| `crates/ampel-api/src/observability.rs`                 | 21     | Critical |
| `crates/ampel-api/Cargo.toml`                           | 1      | Critical |
| `crates/ampel-worker/tests/metrics_collection_tests.rs` | 10     | Critical |
| `frontend/src/pages/Settings.test.tsx`                  | 1      | Critical |
| `frontend/src/pages/Repositories.test.tsx`              | 44     | High     |
| `frontend/src/components/dashboard/PRListView.test.tsx` | 12     | High     |
| `frontend/src/components/dashboard/*.test.tsx`          | 3      | High     |

---

## Recommended Fix Order

1. `crates/ampel-api/src/observability.rs` — fixes 21 backend errors
2. `crates/ampel-worker/tests/metrics_collection_tests.rs` — fixes 10 backend errors
3. `frontend/src/pages/Settings.test.tsx` — fixes parsing error
4. Frontend test files — fixes 58 `any` type warnings

**Estimated effort**: 1-2 hours for critical fixes, 2-4 hours for all fixes
