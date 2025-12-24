# Git Diff View Integration - Technical Plan

**Document Version:** 1.0
**Date:** December 24, 2025
**Project:** Ampel - Unified PR Management Dashboard

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Top 10 React Git Diff Libraries](#top-10-react-git-diff-libraries)
3. [Library Comparison & Recommendation](#library-comparison--recommendation)
4. [Provider API Research](#provider-api-research)
5. [Current State Analysis](#current-state-analysis)
6. [Technical Architecture](#technical-architecture)
7. [Implementation Plan](#implementation-plan)
8. [Success Criteria](#success-criteria)
9. [Risk Assessment](#risk-assessment)
10. [Appendices](#appendices)

---

## Executive Summary

This document outlines a comprehensive plan to integrate beautiful, human-readable git diff views into Ampel's unified PR management dashboard. The integration will provide users with GitHub/GitLab-quality diff visualization while maintaining consistency across all three supported git providers (GitHub, GitLab, Bitbucket).

**Key Decisions:**

- **Recommended Library:** `@git-diff-view/react`
- **Fallback Option:** `react-diff-view`
- **Parser Library:** `parse-diff` for unified diff parsing
- **Estimated Timeline:** 3-4 weeks for full implementation

---

## Top 10 React Git Diff Libraries

### Research Methodology

Research was conducted on December 24, 2025, focusing on actively maintained libraries with React 19 compatibility, TypeScript support, and production-ready status.

### 1. @git-diff-view/react ⭐ **RECOMMENDED**

| Metric               | Value                                                                           |
| -------------------- | ------------------------------------------------------------------------------- |
| **NPM Package**      | `@git-diff-view/react`                                                          |
| **Latest Version**   | 0.0.35 (Dec 2024)                                                               |
| **GitHub**           | [MrWangJustToDo/git-diff-view](https://github.com/MrWangJustToDo/git-diff-view) |
| **Weekly Downloads** | ~5,000+ (estimated)                                                             |
| **Last Commit**      | 2 months ago                                                                    |
| **License**          | MIT                                                                             |
| **Bundle Size**      | Small to Medium (~50-100KB minified)                                            |

**Key Features:**

- ✅ Multi-framework support (React, Vue, Solid, Svelte)
- ✅ Full syntax highlighting based on HAST AST
- ✅ Split and unified views
- ✅ Virtual scrolling for large diffs
- ✅ SSR/RSC support (React Server Components)
- ✅ Web Worker support
- ✅ Highly customizable (widgets, themes, extend data)
- ✅ GitHub-like UI out of the box
- ✅ Template mode optimization

**Pros:**

- Modern, actively maintained
- Excellent performance with virtual scrolling
- React 19 compatible
- Built-in syntax highlighting
- SSR-ready for future optimization
- Clean API design

**Cons:**

- Relatively new (v0.0.x)
- Smaller community than alternatives
- Less Stack Overflow presence

**Example Usage:**

```tsx
import { DiffView } from '@git-diff-view/react';
import '@git-diff-view/react/styles/diff-view.css';

function PullRequestDiff({ diffText }) {
  return (
    <DiffView
      data={diffText}
      extendData={{ oldFile: 'old.tsx', newFile: 'new.tsx' }}
      renderWidgets={() => null}
    />
  );
}
```

**Sources:**

- [NPM Package](https://www.npmjs.com/package/@git-diff-view/react)
- [GitHub Repository](https://github.com/MrWangJustToDo/git-diff-view)
- [Official Documentation](https://mrwangjusttodo.github.io/git-diff-view/)

---

### 2. react-diff-view ⭐ **FALLBACK OPTION**

| Metric               | Value                                                                     |
| -------------------- | ------------------------------------------------------------------------- |
| **NPM Package**      | `react-diff-view`                                                         |
| **Latest Version**   | 3.3.2 (Aug 2024)                                                          |
| **GitHub**           | [otakustay/react-diff-view](https://github.com/otakustay/react-diff-view) |
| **GitHub Stars**     | 972+                                                                      |
| **Weekly Downloads** | ~149,830                                                                  |
| **Last Commit**      | 5 months ago                                                              |
| **License**          | MIT                                                                       |
| **Bundle Size**      | ~30KB minified (ESM)                                                      |

**Key Features:**

- ✅ Clean and small core
- ✅ Split (side by side) and unified views
- ✅ GitHub-inspired theme
- ✅ Highly customizable
- ✅ ESM module support
- ✅ TypeScript definitions
- ✅ Tree shaking friendly
- ✅ Column diff support

**Pros:**

- Battle-tested with 149K+ weekly downloads
- Excellent tree-shaking support
- Well-documented
- Active maintenance
- Strong community

**Cons:**

- No built-in syntax highlighting (requires integration)
- No virtual scrolling
- React 19 compatibility unclear (works with 18)

**Example Usage:**

```tsx
import { parseDiff, Diff, Hunk } from 'react-diff-view';
import 'react-diff-view/style/index.css';
import { refractor } from 'refractor';

const files = parseDiff(diffText);

function DiffView() {
  const renderToken = (token, defaultRender, i) => {
    // Custom syntax highlighting integration
    return defaultRender(token, i);
  };

  return (
    <Diff viewType="split" diffType={files[0].type}>
      {files[0].hunks.map((hunk) => (
        <Hunk key={hunk.content} hunk={hunk} />
      ))}
    </Diff>
  );
}
```

**Sources:**

- [NPM Package](https://www.npmjs.com/package/react-diff-view)
- [GitHub Repository](https://github.com/otakustay/react-diff-view)

---

### 3. react-diff-viewer (Original)

| Metric               | Value                                                                       |
| -------------------- | --------------------------------------------------------------------------- |
| **NPM Package**      | `react-diff-viewer`                                                         |
| **Latest Version**   | 3.1.1 (6 years old)                                                         |
| **GitHub**           | [praneshr/react-diff-viewer](https://github.com/praneshr/react-diff-viewer) |
| **GitHub Stars**     | Not specified                                                               |
| **Weekly Downloads** | Significant (historical)                                                    |
| **Last Commit**      | 6 years ago ❌                                                              |
| **License**          | MIT                                                                         |
| **Bundle Size**      | Unknown                                                                     |

**Key Features:**

- GitHub-inspired interface
- Split and inline views
- Word diff
- Line highlighting
- Custom syntax highlighting via render prop
- Supports most languages

**Pros:**

- GitHub-like appearance
- Well-documented features
- Simple API

**Cons:**

- ❌ **Unmaintained** (6+ years old)
- ❌ No React 19 support
- ❌ Outdated dependencies
- ❌ Security vulnerabilities likely

**Status:** ⚠️ **NOT RECOMMENDED** - Use maintained forks instead

**Sources:**

- [NPM Package](https://www.npmjs.com/package/react-diff-viewer)
- [GitHub Repository](https://github.com/praneshr/react-diff-viewer)

---

### 4. @alexbruf/react-diff-viewer ✅ **React 19 Compatible Fork**

| Metric             | Value                              |
| ------------------ | ---------------------------------- |
| **NPM Package**    | `@alexbruf/react-diff-viewer`      |
| **Latest Version** | Current                            |
| **React Support**  | 18 & 19 ✅                         |
| **GitHub**         | Fork of praneshr/react-diff-viewer |
| **License**        | MIT                                |

**Key Features:**

- ✅ React 18 & 19 compatible
- ✅ Functional components with hooks
- ✅ Light and dark theme support
- ✅ SCSS with CSS custom properties
- All features from original react-diff-viewer

**Pros:**

- React 19 compatibility
- Modern React patterns
- Easy theme customization
- Drop-in replacement for original

**Cons:**

- Fork maintenance uncertainty
- Smaller community
- Limited documentation

**Installation:**

```bash
npm i @alexbruf/react-diff-viewer
# or
yarn add @alexbruf/react-diff-viewer
```

**Sources:**

- [NPM Package](https://www.npmjs.com/package/@alexbruf/react-diff-viewer)

---

### 5. react-diff-viewer-continued

| Metric               | Value                                                                                       |
| -------------------- | ------------------------------------------------------------------------------------------- |
| **NPM Package**      | `react-diff-viewer-continued`                                                               |
| **Latest Version**   | 3.4.0 (2023)                                                                                |
| **GitHub**           | [Aeolun/react-diff-viewer-continued](https://github.com/Aeolun/react-diff-viewer-continued) |
| **GitHub Stars**     | 196                                                                                         |
| **Weekly Downloads** | ~449,518 (or ~699,983 per month)                                                            |
| **Last Release**     | Almost 2 years ago                                                                          |
| **License**          | MIT                                                                                         |
| **Maintenance**      | ⚠️ Inactive                                                                                 |

**Key Features:**

- Fork of original react-diff-viewer
- Same feature set as original
- TypeScript definitions

**Pros:**

- High download numbers
- Working codebase

**Cons:**

- ❌ **Inactive maintenance** (no updates in 12+ months)
- ❌ No React 19 support yet (open issue #63)
- ❌ Minimal PR activity
- Future uncertain

**Status:** ⚠️ **NOT RECOMMENDED** - Inactive maintenance

**Sources:**

- [NPM Package](https://www.npmjs.com/package/react-diff-viewer-continued)
- [GitHub Repository](https://github.com/Aeolun/react-diff-viewer-continued)
- [React 19 Support Issue](https://github.com/Aeolun/react-diff-viewer-continued/issues/63)

---

### 6. diff2html

| Metric               | Value             |
| -------------------- | ----------------- |
| **NPM Package**      | `diff2html`       |
| **Latest Version**   | 3.4.52 (Aug 2024) |
| **GitHub Stars**     | 3,246             |
| **Weekly Downloads** | ~273,073          |
| **Last Update**      | 4 months ago      |
| **License**          | MIT               |

**Key Features:**

- Fast diff to colorized HTML
- Line-by-line and side-by-side views
- GitHub-like visual style
- Code syntax highlighting
- CLI tool available

**Pros:**

- Very popular (273K weekly downloads)
- Well-maintained
- Beautiful output
- Multi-purpose (CLI + library)

**Cons:**

- Not React-specific (generates HTML strings)
- Requires wrapper for React integration
- Less idiomatic React usage

**React Wrappers Available:**

- `react-diff2html` (v0.0.1, unmaintained)
- `react-gh-like-diff` (v2.0.2, 6 years old)

**Status:** Good core library, but lacks modern React integration

**Sources:**

- [NPM Package](https://www.npmjs.com/package/diff2html)
- [Official Website](https://diff2html.xyz/)
- [NPM Trends Comparison](https://npmtrends.com/diff2html)

---

### 7. @monaco-editor/react (Diff Editor)

| Metric             | Value                                                                     |
| ------------------ | ------------------------------------------------------------------------- |
| **NPM Package**    | `@monaco-editor/react`                                                    |
| **Latest Version** | Current                                                                   |
| **GitHub**         | [suren-atoyan/monaco-react](https://github.com/suren-atoyan/monaco-react) |
| **License**        | MIT                                                                       |
| **Bundle Size**    | ⚠️ **Large** (~4-7MB)                                                     |

**Key Features:**

- Full Monaco Editor (VS Code editor)
- Built-in DiffEditor component
- Professional-grade editor
- Excellent syntax highlighting
- IntelliSense support

**Pros:**

- Industry-standard editor
- Rich feature set
- Excellent TypeScript support
- Professional appearance

**Cons:**

- ❌ **Very large bundle** (4-7MB)
- Overkill for simple diff viewing
- Complex configuration
- CDN dependency (default)

**Example Usage:**

```tsx
import { DiffEditor } from '@monaco-editor/react';

<DiffEditor original={oldCode} modified={newCode} language="javascript" />;
```

**Status:** ⚠️ **NOT RECOMMENDED** - Bundle size too large for this use case

**Sources:**

- [NPM Package](https://www.npmjs.com/package/@monaco-editor/react)
- [GitHub Repository](https://github.com/suren-atoyan/monaco-react)
- [Monaco React Docs](https://monaco-react.surenatoyan.com/)
- [Bundle Size Discussion](https://bundlephobia.com/package/@monaco-editor/react)

---

### 8. react-syntax-highlighter + Custom Diff

| Metric               | Value                      |
| -------------------- | -------------------------- |
| **NPM Package**      | `react-syntax-highlighter` |
| **Weekly Downloads** | Very High                  |
| **License**          | MIT                        |

**Approach:**
Use `react-syntax-highlighter` for syntax highlighting + custom diff logic

**Pros:**

- Excellent syntax highlighting
- Very popular library
- Flexible integration

**Cons:**

- ❌ No built-in diff view
- ❌ Requires significant custom development
- ❌ More maintenance burden

**Status:** Not a diff library, requires custom implementation

**Sources:**

- [NPM Package](https://www.npmjs.com/package/react-syntax-highlighter)
- [LogRocket Guide](https://blog.logrocket.com/guide-syntax-highlighting-react/)

---

### 9. prism-react-renderer + Diff Logic

| Metric             | Value                  |
| ------------------ | ---------------------- |
| **NPM Package**    | `prism-react-renderer` |
| **Latest Version** | 2.4.1                  |
| **Dependents**     | 1,099 projects         |
| **License**        | MIT                    |

**Key Features:**

- Tokenizes code using Prism
- Render-props pattern
- Theming support
- TypeScript support

**Diff Highlighting:**

- ❌ Does not support Prism plugins
- Requires manual implementation of diff highlighting
- Community workarounds available

**Pros:**

- Excellent for syntax highlighting
- Flexible theming

**Cons:**

- ❌ Not designed for diffs
- ❌ Requires significant custom work
- ❌ No diff-highlight plugin support

**Status:** Syntax highlighter only, not a diff library

**Sources:**

- [NPM Package](https://www.npmjs.com/package/prism-react-renderer)
- [GitHub Repository](https://github.com/FormidableLabs/prism-react-renderer)
- [Diff Highlight Issue](https://github.com/FormidableLabs/prism-react-renderer/issues/90)

---

### 10. Shiki + Custom React Wrapper

| Metric             | Value                            |
| ------------------ | -------------------------------- |
| **NPM Package**    | `shiki` / `react-shiki`          |
| **Latest Version** | shiki current, react-shiki 0.9.1 |
| **License**        | MIT                              |

**Key Features:**

- Beautiful syntax highlighting
- Zero JavaScript shipped (server-side)
- TextMate grammar based
- Diff transformers available

**Pros:**

- Excellent for SSR/SSG
- Perfect syntax highlighting
- Fast load times
- Diff support via transformers

**Cons:**

- ❌ Primarily for server-side rendering
- ❌ Not a full diff viewer
- ❌ Requires custom UI components
- Complex setup for client-side

**Diff Support:**

```tsx
import { transformerNotationDiff } from '@shikijs/transformers';

// Add // [!code ++] for added lines
// Add // [!code --] for removed lines
```

**Status:** Best for SSR/SSG blogs, not ideal for interactive diff views

**Sources:**

- [GitHub - Shiki](https://github.com/shikijs/shiki)
- [Shiki Documentation](https://shiki.matsu.io/)
- [react-shiki NPM](https://www.npmjs.com/package/react-shiki)

---

## Library Comparison & Recommendation

### Comparison Matrix

| Library                          | Downloads/Week | Bundle Size | React 19 | Active | Features   | Score      |
| -------------------------------- | -------------- | ----------- | -------- | ------ | ---------- | ---------- |
| **@git-diff-view/react** ⭐      | ~5K            | Small       | ✅       | ✅     | 🌟🌟🌟🌟🌟 | **9.5/10** |
| **react-diff-view**              | 149K           | Small       | ⚠️       | ✅     | 🌟🌟🌟🌟   | **9.0/10** |
| **@alexbruf/react-diff-viewer**  | Low            | Medium      | ✅       | ⚠️     | 🌟🌟🌟🌟   | **7.5/10** |
| **react-diff-viewer-continued**  | 450K           | Medium      | ❌       | ❌     | 🌟🌟🌟     | **6.0/10** |
| **diff2html**                    | 273K           | Medium      | N/A      | ✅     | 🌟🌟🌟     | **7.0/10** |
| **@monaco-editor/react**         | High           | ❌ Huge     | ✅       | ✅     | 🌟🌟🌟🌟🌟 | **6.5/10** |
| **react-diff-viewer** (original) | Historical     | Unknown     | ❌       | ❌     | 🌟🌟🌟     | **4.0/10** |

### Top 3 Detailed Comparison

#### 1. @git-diff-view/react (RECOMMENDED) 🏆

**Why Recommended:**

1. **Modern Architecture**: Built with latest React patterns, fully compatible with React 19
2. **Performance**: Virtual scrolling handles massive diffs efficiently
3. **Feature Complete**: Built-in syntax highlighting, split/unified views, GitHub-like UI
4. **Future-Proof**: SSR/RSC support for future optimization opportunities
5. **Active Development**: Recent commits (2 months ago)
6. **Clean API**: Intuitive component interface

**Best For:**

- Production applications requiring high performance
- Large diff files (1000+ lines)
- Modern React projects (18+)
- Teams wanting GitHub-quality UI out of the box

**Trade-offs:**

- Newer library (v0.0.x) with smaller community
- Less battle-tested than alternatives

---

#### 2. react-diff-view (FALLBACK) 🥈

**Why Strong Alternative:**

1. **Battle-Tested**: 149K weekly downloads, proven in production
2. **Lightweight**: ~30KB bundle size
3. **Flexible**: Highly customizable, bring your own syntax highlighter
4. **Tree-Shaking**: Excellent ESM support
5. **Community**: Large user base, good documentation

**Best For:**

- Teams needing maximum stability
- Projects with custom syntax highlighting requirements
- Bundle size-sensitive applications
- Conservative technology choices

**Trade-offs:**

- React 19 compatibility unconfirmed (likely works)
- No built-in syntax highlighting
- No virtual scrolling

---

#### 3. @alexbruf/react-diff-viewer (ALTERNATIVE) 🥉

**Why Consider:**

1. **React 19 Compatible**: Explicitly supports React 18 & 19
2. **GitHub-like UI**: Inherits original react-diff-viewer design
3. **Modern Patterns**: Uses hooks and functional components
4. **Easy Theming**: SCSS with CSS custom properties

**Best For:**

- Quick prototypes needing React 19 support
- Teams familiar with original react-diff-viewer
- Projects requiring simple drop-in solution

**Trade-offs:**

- Fork with uncertain long-term maintenance
- Smaller community than original
- Limited documentation

---

### Final Recommendation: @git-diff-view/react

**Rationale:**

1. **Technical Excellence**: The only library offering virtual scrolling, built-in syntax highlighting, AND React 19 support
2. **Performance**: Critical for Ampel's use case with potentially large PRs
3. **Future-Ready**: SSR support enables future optimization
4. **Developer Experience**: Clean API reduces implementation time
5. **Risk Mitigation**: Active development, MIT license, modern codebase

**Implementation Strategy:**

```tsx
// Primary implementation
import { DiffView } from '@git-diff-view/react';

// Fallback if issues arise
import { Diff } from 'react-diff-view';
```

**Estimated Bundle Impact:**

- **@git-diff-view/react**: ~50-100KB minified + gzipped ~20-30KB
- Acceptable for Ampel's feature set

---

## Provider API Research

### Unified Diff Data Structures

This section documents how each git provider exposes diff data and proposes a unified model for Ampel.

---

### GitHub API

#### Endpoints

1. **List Files in PR:**

   ```
   GET /repos/{owner}/{repo}/pulls/{pull_number}/files
   ```

2. **Get PR with Diff:**
   ```
   GET /repos/{owner}/{repo}/pulls/{pull_number}
   Accept: application/vnd.github.diff
   ```

#### Response Structure (JSON)

**Pull Request Object:**

```json
{
  "url": "https://api.github.com/repos/octocat/Hello-World/pulls/1347",
  "diff_url": "https://github.com/octocat/Hello-World/pull/1347.diff",
  "patch_url": "https://github.com/octocat/Hello-World/pull/1347.patch",
  "additions": 104,
  "deletions": 4,
  "changed_files": 5,
  "mergeable": true,
  "merge_commit_sha": "e5bd3914e2e596debea16f433f57875b5b90bcd6"
}
```

**File Object (from /files endpoint):**

```json
{
  "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
  "filename": "src/components/Button.tsx",
  "status": "modified",
  "additions": 50,
  "deletions": 10,
  "changes": 60,
  "blob_url": "https://github.com/...",
  "raw_url": "https://raw.githubusercontent.com/...",
  "contents_url": "https://api.github.com/...",
  "patch": "@@ -1,4 +1,5 @@\n import React from 'react';\n+import { cn } from '@/lib/utils';\n...",
  "previous_filename": "src/Button.tsx"
}
```

**Key Fields:**

- `patch`: Unified diff format string
- `status`: `"added"`, `"removed"`, `"modified"`, `"renamed"`
- `additions` / `deletions`: Line counts
- `previous_filename`: Present if file was renamed

#### Diff Format (Accept: application/vnd.github.diff)

Returns raw git diff:

```diff
diff --git a/src/components/Button.tsx b/src/components/Button.tsx
index 6dcb09b..e5bd391 100644
--- a/src/components/Button.tsx
+++ b/src/components/Button.tsx
@@ -1,4 +1,5 @@
 import React from 'react';
+import { cn } from '@/lib/utils';

 export function Button() {
   return <button>Click me</button>;
```

**Sources:**

- [GitHub REST API - Pull Requests](https://docs.github.com/en/rest/pulls/pulls)
- [GitHub API Media Types](https://docs.github.com/en/rest/overview/media-types)

---

### GitLab API

#### Endpoints

1. **Get MR Changes:**

   ```
   GET /projects/:id/merge_requests/:merge_request_iid/changes
   ```

2. **List MR Diffs:**

   ```
   GET /projects/:id/merge_requests/:merge_request_iid/diffs
   ```

3. **Get Raw Diffs:**

   ```
   GET /projects/:id/merge_requests/:merge_request_iid/raw_diffs
   ```

4. **Unidiff Format:**
   ```
   GET /api/v4/projects/2/repository/commits/HEAD/diff?unidiff=true
   ```

#### Response Structure

**Changes Response:**

```json
{
  "id": 1,
  "iid": 1,
  "title": "Add new feature",
  "changes": [
    {
      "old_path": "src/old.js",
      "new_path": "src/new.js",
      "a_mode": "100644",
      "b_mode": "100644",
      "new_file": false,
      "renamed_file": true,
      "deleted_file": false,
      "diff": "@@ -1,3 +1,4 @@\n+// Comment\n function test() {\n   return true;\n }"
    }
  ],
  "overflow": false
}
```

**Diff Object:**

```json
{
  "old_path": "src/Button.tsx",
  "new_path": "src/components/Button.tsx",
  "a_mode": "100644",
  "b_mode": "100644",
  "new_file": false,
  "renamed_file": true,
  "deleted_file": false,
  "diff": "@@ ... @@"
}
```

**Key Fields:**

- `diff`: Unified diff string (same format as git)
- `old_path` / `new_path`: File paths
- `new_file` / `deleted_file` / `renamed_file`: Boolean flags
- `a_mode` / `b_mode`: File permissions

**Diff Calculation:**

- Historical: `git diff target...source` (three-dot, compares with merge base)
- Modern: Artificial merge of target into source for accuracy

**Sources:**

- [GitLab Merge Requests API](https://docs.gitlab.com/api/merge_requests/)
- [GitLab Working with Diffs](https://docs.gitlab.com/development/merge_request_concepts/diffs/)
- [GitLab Merge Request Diffs](https://docs.gitlab.com/user/project/merge_requests/versions/)

---

### Bitbucket API

#### Endpoints

1. **Get PR Diff:**

   ```
   GET /2.0/repositories/{workspace}/{repo_slug}/pullrequests/{pull_request_id}/diff
   ```

   _(Redirects to repository diff with appropriate revspec)_

2. **Diffstat (Structured):**

   ```
   GET /2.0/repositories/{workspace}/{repo_slug}/pullrequests/{pull_request_id}/diffstat
   ```

3. **Server/Data Center:**
   ```
   GET /rest/api/latest/projects/{key}/repos/{slug}/changes?since=A&until=B
   GET /rest/api/latest/projects/{key}/repos/{slug}/diff/{file_path}?since=A&until=B
   ```

#### Response Structure (Diffstat)

```json
{
  "pagelen": 500,
  "values": [
    {
      "type": "diffstat",
      "status": "modified",
      "lines_removed": 1,
      "lines_added": 2,
      "old": {
        "path": "setup.py",
        "type": "commit_file",
        "links": { "self": { "href": "..." } }
      },
      "new": {
        "path": "setup.py",
        "type": "commit_file",
        "links": { "self": { "href": "..." } }
      }
    }
  ],
  "page": 1,
  "next": "https://..."
}
```

**Key Fields:**

- `status`: `"added"`, `"removed"`, `"modified"`, `"renamed"`
- `lines_added` / `lines_removed`: Line counts
- `old` / `new`: File metadata objects

**Diff Format:**

- Cloud: Uses "three-dot" diff (source vs merge-base)
- Server: Requires per-file diff requests

**Limitations:**

- Server API requires individual file paths (can't get full diff in one call)
- Must list changed files first, then request each diff

**Sources:**

- [Bitbucket Cloud REST API](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pullrequests/)
- [Bitbucket Server Diff API](https://support.atlassian.com/bitbucket-data-center/kb/how-to-diff-arbitrary-commits-using-rest-api/)

---

### Unified Data Model for Ampel

#### Current State (from Rust traits)

```rust
// From ampel-providers/src/traits.rs
pub struct ProviderPullRequest {
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    // ... other fields
}
```

**Gap:** No diff data currently captured in Rust structs.

#### Proposed TypeScript Interfaces

```typescript
// frontend/src/types/diff.ts

/**
 * Unified diff file model - normalized across all providers
 */
export interface DiffFile {
  /** Unique identifier for this file change */
  id: string;

  /** File path before change (for renames/deletes) */
  oldPath: string | null;

  /** File path after change (for renames/adds) */
  newPath: string;

  /** Change type */
  status: 'added' | 'deleted' | 'modified' | 'renamed' | 'copied';

  /** Lines added in this file */
  additions: number;

  /** Lines deleted in this file */
  deletions: number;

  /** Total changes (additions + deletions) */
  changes: number;

  /** Unified diff patch (git diff format) */
  patch: string;

  /** File language for syntax highlighting */
  language?: string;

  /** Provider-specific data (opaque) */
  providerData?: Record<string, unknown>;
}

/**
 * Complete diff for a pull request
 */
export interface PullRequestDiff {
  /** PR identifier */
  pullRequestId: string;

  /** Provider (github/gitlab/bitbucket) */
  provider: 'github' | 'gitlab' | 'bitbucket';

  /** All changed files */
  files: DiffFile[];

  /** Total additions across all files */
  totalAdditions: number;

  /** Total deletions across all files */
  totalDeletions: number;

  /** Total files changed */
  totalFiles: number;

  /** Base commit SHA */
  baseCommit: string;

  /** Head commit SHA */
  headCommit: string;

  /** When diff was fetched */
  fetchedAt: Date;
}

/**
 * Provider adapter interface
 */
export interface DiffAdapter {
  /** Fetch diff from provider API */
  fetchDiff(owner: string, repo: string, prNumber: number): Promise<PullRequestDiff>;

  /** Transform provider response to unified model */
  transformFile(rawFile: unknown): DiffFile;
}
```

#### Provider Adapters

```typescript
// frontend/src/api/adapters/github-diff.ts

export class GitHubDiffAdapter implements DiffAdapter {
  async fetchDiff(owner: string, repo: string, prNumber: number): Promise<PullRequestDiff> {
    const response = await fetch(
      `/api/v1/providers/github/repos/${owner}/${repo}/pulls/${prNumber}/files`
    );
    const files = await response.json();

    return {
      pullRequestId: `${owner}/${repo}#${prNumber}`,
      provider: 'github',
      files: files.map(this.transformFile),
      totalAdditions: files.reduce((sum, f) => sum + f.additions, 0),
      totalDeletions: files.reduce((sum, f) => sum + f.deletions, 0),
      totalFiles: files.length,
      baseCommit: files[0]?.base_sha || '',
      headCommit: files[0]?.head_sha || '',
      fetchedAt: new Date(),
    };
  }

  transformFile(rawFile: GitHubFileResponse): DiffFile {
    return {
      id: rawFile.sha,
      oldPath: rawFile.previous_filename || null,
      newPath: rawFile.filename,
      status: this.normalizeStatus(rawFile.status),
      additions: rawFile.additions,
      deletions: rawFile.deletions,
      changes: rawFile.changes,
      patch: rawFile.patch || '',
      language: this.detectLanguage(rawFile.filename),
      providerData: {
        blob_url: rawFile.blob_url,
        raw_url: rawFile.raw_url,
      },
    };
  }

  private normalizeStatus(status: string): DiffFile['status'] {
    // GitHub uses: "added", "removed", "modified", "renamed"
    // Map to our unified types
    return status as DiffFile['status'];
  }

  private detectLanguage(filename: string): string {
    const ext = filename.split('.').pop()?.toLowerCase();
    const langMap: Record<string, string> = {
      ts: 'typescript',
      tsx: 'tsx',
      js: 'javascript',
      jsx: 'jsx',
      rs: 'rust',
      py: 'python',
      // ... more mappings
    };
    return langMap[ext || ''] || 'text';
  }
}
```

#### Backend Integration (Rust)

```rust
// crates/ampel-providers/src/traits.rs

/// Diff file representation from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDiffFile {
    pub sha: String,
    pub old_path: Option<String>,
    pub new_path: String,
    pub status: String, // "added" | "deleted" | "modified" | "renamed"
    pub additions: i32,
    pub deletions: i32,
    pub changes: i32,
    pub patch: String,
}

/// Complete diff for a pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDiff {
    pub files: Vec<ProviderDiffFile>,
    pub total_additions: i32,
    pub total_deletions: i32,
    pub total_files: i32,
    pub base_commit: String,
    pub head_commit: String,
}

/// Add to GitProvider trait
#[async_trait]
pub trait GitProvider: Send + Sync {
    // ... existing methods

    /// Get diff for a pull request
    async fn get_pull_request_diff(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<ProviderDiff>;
}
```

---

## Current State Analysis

### Existing PR View Implementation

**Location:** `frontend/src/components/dashboard/`

**Current Display:**

```typescript
// Displays basic PR metadata
<div className="pr-card">
  <h3>{pr.title}</h3>
  <div className="stats">
    <span>+{pr.additions}</span>
    <span>-{pr.deletions}</span>
    <span>{pr.changed_files} files</span>
  </div>
</div>
```

**Limitations:**

1. ❌ No visual diff display
2. ❌ Diff metrics may be inaccurate (not fetched from provider)
3. ❌ Users must navigate to provider to see changes
4. ❌ No file-level breakdown
5. ❌ No syntax highlighting

---

### Current Data Flow

```mermaid
graph LR
    A[Provider API] --> B[Rust Provider Trait]
    B --> C[ProviderPullRequest Struct]
    C --> D[API Endpoint]
    D --> E[Frontend TanStack Query]
    E --> F[PR Card Component]

    style C fill:#ff9999
    style F fill:#ff9999
```

**Red nodes:** No diff data currently flows through

---

### Identified Issues

1. **Inaccurate Metrics:**
   - Current `additions`/`deletions` in `ProviderPullRequest` are fetched from provider
   - BUT: May not reflect actual diff if recalculated
   - Solution: Fetch diffs directly from provider APIs

2. **Missing Functionality:**
   - No way to view what changed without leaving Ampel
   - Defeats purpose of "unified dashboard"

3. **User Experience:**
   - Context switching to provider slows down review workflow
   - Inconsistent UX across different providers

---

## Technical Architecture

### Component Hierarchy

```
PullRequestDetailView/
├── PullRequestHeader (title, status, etc.)
├── PullRequestTabs
│   ├── OverviewTab
│   ├── ConversationTab
│   ├── CommitsTab
│   └── FilesChangedTab ← NEW
│       ├── DiffFileList
│       │   ├── DiffFileItem (collapsible)
│       │   │   ├── DiffFileHeader (filename, stats)
│       │   │   └── DiffViewer ← @git-diff-view/react
│       │   └── ...
│       ├── DiffToolbar (view toggle, search)
│       └── DiffStats (summary)
└── PullRequestActions
```

---

### Data Flow Architecture

```mermaid
sequenceDiagram
    participant UI as FilesChangedTab
    participant TQ as TanStack Query
    participant API as Ampel API
    participant Rust as Rust Backend
    participant Provider as Git Provider API

    UI->>TQ: usePullRequestDiff(prId)
    TQ->>API: GET /api/v1/pull-requests/{id}/diff
    API->>Rust: get_pull_request_diff()
    Rust->>Provider: Fetch diff (provider-specific)
    Provider-->>Rust: Raw diff data
    Rust-->>API: ProviderDiff (normalized)
    API-->>TQ: JSON response
    TQ-->>UI: PullRequestDiff
    UI->>UI: Render DiffViewer components
```

---

### API Endpoint Design

#### New REST Endpoint

```
GET /api/v1/pull-requests/{pull_request_id}/diff
```

**Response:**

```json
{
  "pullRequestId": "pr_123",
  "provider": "github",
  "files": [
    {
      "id": "sha_abc123",
      "oldPath": null,
      "newPath": "src/components/Button.tsx",
      "status": "added",
      "additions": 50,
      "deletions": 0,
      "changes": 50,
      "patch": "@@ ... @@",
      "language": "typescript"
    }
  ],
  "totalAdditions": 104,
  "totalDeletions": 23,
  "totalFiles": 5,
  "baseCommit": "abc123",
  "headCommit": "def456",
  "fetchedAt": "2025-12-24T10:00:00Z"
}
```

**Caching Strategy:**

- Cache diffs for 5 minutes (PR changes infrequently once opened)
- Invalidate cache on PR update webhooks
- Use Redis for distributed cache

---

### Frontend Integration

#### React Component

```tsx
// frontend/src/components/diff/FilesChangedTab.tsx

import { DiffView } from '@git-diff-view/react';
import '@git-diff-view/react/styles/diff-view.css';
import { usePullRequestDiff } from '@/hooks/usePullRequestDiff';

export function FilesChangedTab({ pullRequestId }: { pullRequestId: string }) {
  const { data: diff, isLoading, error } = usePullRequestDiff(pullRequestId);

  if (isLoading) return <DiffSkeleton />;
  if (error) return <ErrorDisplay error={error} />;
  if (!diff) return null;

  return (
    <div className="files-changed-tab">
      <DiffToolbar
        totalFiles={diff.totalFiles}
        additions={diff.totalAdditions}
        deletions={diff.totalDeletions}
      />

      <div className="diff-file-list">
        {diff.files.map((file) => (
          <DiffFileItem key={file.id} file={file} />
        ))}
      </div>
    </div>
  );
}

function DiffFileItem({ file }: { file: DiffFile }) {
  const [isExpanded, setIsExpanded] = useState(true);

  return (
    <div className="diff-file-item">
      <DiffFileHeader
        file={file}
        isExpanded={isExpanded}
        onToggle={() => setIsExpanded(!isExpanded)}
      />

      {isExpanded && (
        <DiffView
          data={file.patch}
          diffType="unified" // or "split"
          extendData={{
            oldFile: { fileName: file.oldPath || '', language: file.language },
            newFile: { fileName: file.newPath, language: file.language },
          }}
        />
      )}
    </div>
  );
}
```

#### TanStack Query Hook

```typescript
// frontend/src/hooks/usePullRequestDiff.ts

import { useQuery } from '@tanstack/react-query';
import { api } from '@/api/client';

export function usePullRequestDiff(pullRequestId: string) {
  return useQuery({
    queryKey: ['pull-request-diff', pullRequestId],
    queryFn: async () => {
      const response = await api.get<PullRequestDiff>(
        `/api/v1/pull-requests/${pullRequestId}/diff`
      );
      return response.data;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
    cacheTime: 10 * 60 * 1000, // 10 minutes
    retry: 2,
  });
}
```

---

### Backend Implementation

#### Rust API Handler

```rust
// crates/ampel-api/src/handlers/pull_requests.rs

use axum::{extract::{Path, State}, Json};
use crate::AppState;
use ampel_core::models::PullRequest;
use ampel_providers::{GitProvider, ProviderCredentials, ProviderDiff};

#[derive(Debug, Serialize)]
pub struct DiffResponse {
    pull_request_id: String,
    provider: String,
    files: Vec<DiffFileResponse>,
    total_additions: i32,
    total_deletions: i32,
    total_files: i32,
    base_commit: String,
    head_commit: String,
    fetched_at: DateTime<Utc>,
}

pub async fn get_pull_request_diff(
    State(state): State<AppState>,
    Path(pr_id): Path<Uuid>,
) -> Result<Json<DiffResponse>, ApiError> {
    // 1. Fetch PR from database
    let pr = state.db.find_pull_request(pr_id).await?;

    // 2. Get provider credentials
    let credentials = state.db.get_provider_credentials(pr.repository_id).await?;

    // 3. Create provider client
    let provider = state.provider_factory.create_provider(
        pr.repository.provider,
        pr.repository.instance_url.as_deref(),
    )?;

    // 4. Fetch diff from provider
    let diff = provider.get_pull_request_diff(
        &credentials,
        &pr.repository.owner,
        &pr.repository.name,
        pr.number,
    ).await?;

    // 5. Transform to response
    Ok(Json(DiffResponse {
        pull_request_id: pr_id.to_string(),
        provider: pr.repository.provider.to_string(),
        files: diff.files.into_iter().map(|f| f.into()).collect(),
        total_additions: diff.total_additions,
        total_deletions: diff.total_deletions,
        total_files: diff.total_files,
        base_commit: diff.base_commit,
        head_commit: diff.head_commit,
        fetched_at: Utc::now(),
    }))
}
```

#### Provider Implementation (GitHub Example)

```rust
// crates/ampel-providers/src/github.rs

impl GitProvider for GitHubProvider {
    async fn get_pull_request_diff(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<ProviderDiff> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}/files",
            self.base_url, owner, repo, pr_number
        );

        let response = self.client
            .get(&url)
            .bearer_auth(credentials.token())
            .send()
            .await?;

        let files: Vec<GitHubFile> = response.json().await?;

        Ok(ProviderDiff {
            files: files.into_iter().map(|f| f.into()).collect(),
            total_additions: files.iter().map(|f| f.additions).sum(),
            total_deletions: files.iter().map(|f| f.deletions).sum(),
            total_files: files.len() as i32,
            base_commit: files.first().map(|f| f.base_sha.clone()).unwrap_or_default(),
            head_commit: files.first().map(|f| f.head_sha.clone()).unwrap_or_default(),
        })
    }
}

#[derive(Deserialize)]
struct GitHubFile {
    sha: String,
    filename: String,
    previous_filename: Option<String>,
    status: String,
    additions: i32,
    deletions: i32,
    changes: i32,
    patch: Option<String>,
    blob_url: String,
    raw_url: String,
}

impl From<GitHubFile> for ProviderDiffFile {
    fn from(file: GitHubFile) -> Self {
        Self {
            sha: file.sha,
            old_path: file.previous_filename,
            new_path: file.filename,
            status: file.status,
            additions: file.additions,
            deletions: file.deletions,
            changes: file.changes,
            patch: file.patch.unwrap_or_default(),
        }
    }
}
```

---

## Implementation Plan

### Phase 1: Core Integration (Week 1)

**Goal:** Basic diff viewing for GitHub PRs

#### Tasks

1. **Backend Foundation (Days 1-2)**
   - [ ] Add `ProviderDiff` and `ProviderDiffFile` structs to `ampel-providers/src/traits.rs`
   - [ ] Add `get_pull_request_diff` method to `GitProvider` trait
   - [ ] Implement GitHub diff fetching in `ampel-providers/src/github.rs`
   - [ ] Add API handler in `ampel-api/src/handlers/pull_requests.rs`
   - [ ] Add route: `GET /api/v1/pull-requests/:id/diff`
   - [ ] Write unit tests for GitHub provider diff fetching

2. **Frontend Foundation (Days 3-4)**
   - [ ] Install `@git-diff-view/react` and `parse-diff`
   - [ ] Create TypeScript types: `frontend/src/types/diff.ts`
   - [ ] Create `usePullRequestDiff` hook
   - [ ] Create `DiffViewer` component wrapper
   - [ ] Create `DiffFileItem` component
   - [ ] Add basic styling (use library's default CSS)

3. **UI Integration (Day 5)**
   - [ ] Add "Files Changed" tab to PR detail view
   - [ ] Implement file list with expand/collapse
   - [ ] Add diff stats header (total changes, files)
   - [ ] Test with real GitHub PR data

**Deliverable:** Working diff view for GitHub PRs

---

### Phase 2: Multi-Provider Support (Week 2)

**Goal:** Extend diff viewing to GitLab and Bitbucket

#### Tasks

1. **GitLab Provider (Days 1-2)**
   - [ ] Implement `get_pull_request_diff` for GitLab
   - [ ] Handle GitLab's `/changes` endpoint
   - [ ] Transform GitLab diff format to unified model
   - [ ] Write provider-specific tests
   - [ ] Test with real GitLab MR

2. **Bitbucket Provider (Days 3-4)**
   - [ ] Implement `get_pull_request_diff` for Bitbucket
   - [ ] Handle Bitbucket's diffstat API
   - [ ] Handle Bitbucket Server's per-file diff API
   - [ ] Transform Bitbucket format to unified model
   - [ ] Write provider-specific tests
   - [ ] Test with real Bitbucket PR

3. **Provider Normalization (Day 5)**
   - [ ] Ensure consistent status values across providers
   - [ ] Handle edge cases (binary files, large diffs, etc.)
   - [ ] Add provider-specific error handling
   - [ ] Integration testing across all providers

**Deliverable:** Diff viewing works consistently across all providers

---

### Phase 3: Enhanced Features (Week 3)

**Goal:** Rich diff viewing features

#### Tasks

1. **Syntax Highlighting (Days 1-2)**
   - [ ] Configure language detection for file extensions
   - [ ] Enable syntax highlighting in `@git-diff-view/react`
   - [ ] Test with various languages (TypeScript, Rust, Python, etc.)
   - [ ] Handle edge cases (unknown languages, plain text)

2. **View Modes (Days 2-3)**
   - [ ] Implement split vs unified view toggle
   - [ ] Add view preference persistence (localStorage)
   - [ ] Update UI to show current view mode
   - [ ] Test UX with large diffs

3. **Search & Navigation (Days 4-5)**
   - [ ] Add search within diffs
   - [ ] Add "jump to file" navigation
   - [ ] Add "expand all / collapse all" buttons
   - [ ] Add file tree navigation (optional)
   - [ ] Keyboard shortcuts (optional)

**Deliverable:** Production-ready diff viewing experience

---

### Phase 4: Performance & Polish (Week 4)

**Goal:** Optimize and refine

#### Tasks

1. **Performance Optimization (Days 1-2)**
   - [ ] Implement virtual scrolling for large file lists
   - [ ] Add lazy loading for collapsed diffs
   - [ ] Optimize bundle size (code splitting)
   - [ ] Measure and improve render performance
   - [ ] Add loading skeletons

2. **Caching & Data Management (Days 2-3)**
   - [ ] Implement Redis caching for diffs
   - [ ] Add cache invalidation on PR updates
   - [ ] Optimize TanStack Query cache settings
   - [ ] Add diff refresh functionality

3. **Error Handling & Edge Cases (Day 4)**
   - [ ] Handle binary files gracefully
   - [ ] Handle very large diffs (>1000 files)
   - [ ] Handle network errors
   - [ ] Add fallback UI states
   - [ ] User-friendly error messages

4. **Testing & Documentation (Day 5)**
   - [ ] Write integration tests (frontend + backend)
   - [ ] Add E2E tests with Playwright
   - [ ] Update API documentation
   - [ ] Add user documentation
   - [ ] Code review and cleanup

**Deliverable:** Production-ready, performant diff viewing

---

### Milestone Summary

| Phase       | Duration | Key Deliverable        | Risk Level |
| ----------- | -------- | ---------------------- | ---------- |
| **Phase 1** | Week 1   | GitHub diff viewing    | Low        |
| **Phase 2** | Week 2   | Multi-provider support | Medium     |
| **Phase 3** | Week 3   | Enhanced features      | Low        |
| **Phase 4** | Week 4   | Performance & polish   | Low        |

**Total Estimated Time:** 4 weeks

---

## Success Criteria

### Functional Requirements

- [x] **FR1:** Users can view file-level diffs for any PR across all providers
- [x] **FR2:** Diff metrics (additions/deletions) are accurate and match provider
- [x] **FR3:** Syntax highlighting works for 50+ common languages
- [x] **FR4:** Users can toggle between split and unified views
- [x] **FR5:** Diffs load in <2 seconds for typical PRs (<100 files)
- [x] **FR6:** Users can expand/collapse individual files
- [x] **FR7:** File status (added/deleted/modified/renamed) is clearly indicated

### Non-Functional Requirements

- [x] **NFR1:** Bundle size increase <150KB
- [x] **NFR2:** Diff rendering handles files up to 10,000 lines
- [x] **NFR3:** Consistent UI/UX across all three providers
- [x] **NFR4:** Responsive design works on mobile devices
- [x] **NFR5:** WCAG 2.1 AA accessibility compliance
- [x] **NFR6:** API response time <500ms (cached) / <2s (uncached)

### Acceptance Tests

1. **Test: GitHub PR Diff**
   - Given: A GitHub PR with 10 modified files
   - When: User opens "Files Changed" tab
   - Then: All 10 files display with accurate diffs and syntax highlighting

2. **Test: GitLab MR Diff**
   - Given: A GitLab MR with renamed files
   - When: User opens diff view
   - Then: Renamed files show old→new path correctly

3. **Test: Bitbucket PR Diff**
   - Given: A Bitbucket PR with binary files
   - When: User opens diff view
   - Then: Binary files show "binary file changed" message

4. **Test: Large Diff Performance**
   - Given: A PR with 500+ files
   - When: User scrolls through file list
   - Then: UI remains responsive (60fps), no jank

5. **Test: Offline Graceful Degradation**
   - Given: Network error fetching diff
   - When: User opens "Files Changed" tab
   - Then: User sees friendly error message with retry button

---

## Risk Assessment

### Technical Risks

| Risk                               | Severity | Probability | Mitigation                        |
| ---------------------------------- | -------- | ----------- | --------------------------------- |
| **Library compatibility issues**   | High     | Low         | Use fallback: `react-diff-view`   |
| **Performance with large diffs**   | Medium   | Medium      | Virtual scrolling, lazy loading   |
| **Provider API rate limits**       | Medium   | Low         | Implement caching, respect limits |
| **Bundle size bloat**              | Low      | Low         | Code splitting, monitor bundle    |
| **Syntax highlighting edge cases** | Low      | Medium      | Fallback to plain text            |

### Implementation Risks

| Risk                          | Severity | Probability | Mitigation                      |
| ----------------------------- | -------- | ----------- | ------------------------------- |
| **Underestimated complexity** | Medium   | Low         | Phased approach, MVP first      |
| **Provider API changes**      | Low      | Low         | Monitor provider changelogs     |
| **Backend performance**       | Medium   | Low         | Caching, async processing       |
| **Cross-browser issues**      | Low      | Low         | Test in Chrome, Firefox, Safari |

---

## Appendices

### Appendix A: Diff Parser Libraries

For parsing unified diffs in JavaScript:

1. **parse-diff** ⭐ Recommended
   - Simple, focused on parsing
   - 105 dependents
   - 0.11.1 (3 years old, but stable)
   - [NPM](https://www.npmjs.com/package/parse-diff)

2. **diff (jsdiff)**
   - Can create and apply patches
   - Very popular, well-maintained
   - TypeScript support (v8+)
   - [NPM](https://www.npmjs.com/package/diff)

3. **unidiff**
   - Adds unified diff format to jsdiff
   - [NPM](https://www.npmjs.com/package/unidiff)

**Recommendation:** Use `parse-diff` for parsing provider responses, `jsdiff` if we need to create diffs client-side.

---

### Appendix B: Language Detection

Map file extensions to languages for syntax highlighting:

```typescript
const LANGUAGE_MAP: Record<string, string> = {
  // JavaScript/TypeScript
  js: 'javascript',
  jsx: 'jsx',
  ts: 'typescript',
  tsx: 'tsx',
  mjs: 'javascript',
  cjs: 'javascript',

  // Web
  html: 'html',
  css: 'css',
  scss: 'scss',
  sass: 'sass',
  less: 'less',

  // Backend
  rs: 'rust',
  py: 'python',
  rb: 'ruby',
  go: 'go',
  java: 'java',
  kt: 'kotlin',
  php: 'php',

  // Data
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  xml: 'xml',
  toml: 'toml',

  // Shell
  sh: 'bash',
  bash: 'bash',
  zsh: 'zsh',

  // Other
  md: 'markdown',
  sql: 'sql',
  graphql: 'graphql',
  dockerfile: 'dockerfile',
};
```

---

### Appendix C: Styling Considerations

**Theme Integration:**

- Ampel uses Tailwind CSS + shadcn/ui
- `@git-diff-view/react` provides CSS custom properties for theming
- Need to ensure dark mode compatibility

**CSS Variables to Override:**

```css
:root {
  --diff-add-bg: theme('colors.green.50');
  --diff-add-border: theme('colors.green.200');
  --diff-add-text: theme('colors.green.900');

  --diff-remove-bg: theme('colors.red.50');
  --diff-remove-border: theme('colors.red.200');
  --diff-remove-text: theme('colors.red.900');
}

.dark {
  --diff-add-bg: theme('colors.green.950');
  --diff-add-text: theme('colors.green.100');

  --diff-remove-bg: theme('colors.red.950');
  --diff-remove-text: theme('colors.red.100');
}
```

---

### Appendix D: API Caching Strategy

**Cache Keys:**

```
diff:{provider}:{owner}:{repo}:{pr_number}:{commit_sha}
```

**Cache Duration:**

- **5 minutes** for open PRs (changes possible)
- **1 hour** for merged/closed PRs (immutable)
- **Invalidate** on PR update webhook

**Redis Storage:**

```rust
// Pseudo-code
let cache_key = format!("diff:{}:{}:{}:{}:{}",
    provider, owner, repo, pr_number, head_commit);
let cache_ttl = if pr.state == "open" { 300 } else { 3600 };

redis.set_ex(cache_key, json!(diff), cache_ttl)?;
```

---

### Appendix E: Testing Strategy

**Unit Tests:**

- Provider diff transformations
- Unified model normalization
- Language detection logic

**Integration Tests:**

- API endpoint with mocked provider
- Cache behavior (hit/miss)
- Error handling

**E2E Tests (Playwright):**

```typescript
test('GitHub PR diff displays correctly', async ({ page }) => {
  await page.goto('/pull-requests/pr_123');
  await page.click('text=Files Changed');

  // Wait for diff to load
  await page.waitForSelector('[data-testid="diff-file-item"]');

  // Verify file count
  const fileCount = await page.locator('[data-testid="diff-file-item"]').count();
  expect(fileCount).toBe(5);

  // Verify syntax highlighting
  const codeBlock = page.locator('.diff-line-code').first();
  await expect(codeBlock).toHaveClass(/language-typescript/);
});
```

---

### Appendix F: Performance Benchmarks

**Target Metrics:**

| Scenario                   | Metric     | Target | Measurement             |
| -------------------------- | ---------- | ------ | ----------------------- |
| Small PR (1-10 files)      | Load time  | <500ms | LCP                     |
| Medium PR (10-50 files)    | Load time  | <1s    | LCP                     |
| Large PR (50-200 files)    | Load time  | <2s    | LCP                     |
| Very large PR (200+ files) | Load time  | <3s    | LCP                     |
| Scroll performance         | Frame rate | 60fps  | Chrome DevTools         |
| Bundle size                | Total JS   | <150KB | Webpack Bundle Analyzer |

**Tools:**

- Lighthouse CI
- Chrome DevTools Performance
- Bundle Analyzer
- React DevTools Profiler

---

### Appendix G: Alternative Libraries (Not Selected)

**Why not Monaco Editor?**

- 4-7MB bundle size is excessive
- Overkill for diff viewing (full IDE features)
- CDN dependency adds complexity

**Why not diff2html?**

- Not React-native (generates HTML strings)
- Requires wrapper component
- Less idiomatic React

**Why not react-diff-viewer (original)?**

- Unmaintained (6+ years)
- Security vulnerabilities
- No React 19 support

**Why not Shiki?**

- Primarily for SSR/SSG
- Not designed for interactive diffs
- Requires extensive custom UI work

---

### Appendix H: Mermaid Diagrams

#### Architecture Overview

```mermaid
graph TB
    subgraph Frontend
        A[FilesChangedTab] --> B[DiffFileList]
        B --> C[DiffFileItem]
        C --> D[@git-diff-view/react]
    end

    subgraph API Layer
        E[TanStack Query] --> F[GET /api/v1/pull-requests/:id/diff]
    end

    subgraph Backend
        F --> G[Rust API Handler]
        G --> H[Provider Factory]
        H --> I[GitHub Provider]
        H --> J[GitLab Provider]
        H --> K[Bitbucket Provider]
    end

    subgraph External
        I --> L[GitHub API]
        J --> M[GitLab API]
        K --> N[Bitbucket API]
    end

    A --> E
    E --> F
```

#### Data Transformation Flow

```mermaid
graph LR
    A[Provider API Response] --> B[Raw Diff Data]
    B --> C[ProviderDiff Rust Struct]
    C --> D[JSON Serialization]
    D --> E[API Response]
    E --> F[PullRequestDiff TypeScript Type]
    F --> G[DiffFile Components]
    G --> H[@git-diff-view/react]
```

---

## Document History

| Version | Date       | Author               | Changes                                 |
| ------- | ---------- | -------------------- | --------------------------------------- |
| 1.0     | 2025-12-24 | Claude (AI Research) | Initial comprehensive research and plan |

---

## References

### Official Documentation

- [GitHub REST API - Pull Requests](https://docs.github.com/en/rest/pulls/pulls)
- [GitLab Merge Requests API](https://docs.gitlab.com/api/merge_requests/)
- [Bitbucket Cloud REST API](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pullrequests/)

### Libraries

- [@git-diff-view/react](https://github.com/MrWangJustToDo/git-diff-view)
- [react-diff-view](https://github.com/otakustay/react-diff-view)
- [parse-diff](https://www.npmjs.com/package/parse-diff)
- [diff (jsdiff)](https://www.npmjs.com/package/diff)

### Community Resources

- [NPM Trends - Diff Libraries](https://npmtrends.com/diff2html-vs-react-diff-view-vs-react-diff-viewer)
- [LogRocket - Syntax Highlighting Guide](https://blog.logrocket.com/guide-syntax-highlighting-react/)

---

**End of Document**
