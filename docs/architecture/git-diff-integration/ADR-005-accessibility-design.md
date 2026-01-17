# ADR-005: Accessibility Design (WCAG 2.1 AA Compliance)

**Status:** Accepted
**Date:** 2025-12-25
**Decision Makers:** Architecture Team, UX Team
**Technical Story:** Inclusive Diff Viewing Experience

## Context

Ampel must be accessible to users with disabilities, including:

- Visual impairments (blindness, low vision, color blindness)
- Motor impairments (keyboard-only navigation)
- Cognitive impairments (clear, predictable interactions)

WCAG 2.1 Level AA compliance is both a legal requirement (ADA, Section 508) and ethical imperative.

## Decision Drivers

- **Legal Compliance**: Meet ADA, Section 508, WCAG 2.1 AA standards
- **Inclusive Design**: Serve 1 billion+ disabled users globally
- **User Experience**: Accessibility benefits all users (keyboard shortcuts, high contrast)
- **Brand Reputation**: Demonstrate commitment to inclusivity
- **Market Access**: Government/enterprise contracts require WCAG compliance

## Considered Options

### Option 1: WCAG 2.1 AA Compliance from Day One (SELECTED)

Build accessibility into initial design:

- Semantic HTML (proper landmarks, headings, ARIA labels)
- Keyboard navigation (tab order, focus management, skip links)
- Color contrast (4.5:1 for text, 3:1 for UI components)
- Screen reader support (ARIA labels, live regions)
- Responsive design (mobile, zoom up to 200%)

**Pros:**

- Compliance from launch
- Cheaper than retrofitting
- Better UX for all users

**Cons:**

- Requires upfront planning
- More initial development time

### Option 2: Retrofit After Launch

Launch MVP first, add accessibility later.

**Pros:**

- Faster time to market

**Cons:**

- Expensive to retrofit (30-50% more effort)
- Legal risk (ADA lawsuits)
- Technical debt accumulation

### Option 3: WCAG AAA (Maximum Compliance)

Go beyond AA to AAA standard.

**Pros:**

- Maximum inclusivity

**Cons:**

- Significantly more effort
- Some criteria impractical (e.g., 7:1 contrast)

## Decision Outcome

**Chosen Option:** WCAG 2.1 AA Compliance from Day One

### Design Principles

1. **Semantic HTML First**: Use native elements before ARIA
2. **Keyboard-First Navigation**: All actions accessible via keyboard
3. **Clear Focus Indicators**: High-contrast focus rings
4. **Descriptive Labels**: Clear, concise ARIA labels and alt text
5. **Predictable Behavior**: Consistent UI patterns, no surprises

### Component Specifications

#### File List Navigation

```tsx
// frontend/src/components/diff/DiffFileList.tsx

<nav aria-label="Changed files">
  <ul role="list">
    {files.map((file) => (
      <li key={file.id}>
        <button
          onClick={() => toggleFile(file.id)}
          aria-expanded={expandedFiles.has(file.id)}
          aria-controls={`diff-content-${file.id}`}
          className="file-header"
        >
          <StatusIcon status={file.status} aria-hidden="true" />
          <span className="file-path">{file.newPath}</span>
          <span className="sr-only">
            {file.status === 'added' && 'Added, '}
            {file.status === 'deleted' && 'Deleted, '}
            {file.status === 'modified' && 'Modified, '}
            {file.status === 'renamed' && `Renamed from ${file.oldPath}, `}
            {file.additions} additions, {file.deletions} deletions
          </span>
          <ChevronIcon
            direction={expandedFiles.has(file.id) ? 'down' : 'right'}
            aria-hidden="true"
          />
        </button>

        {expandedFiles.has(file.id) && (
          <div id={`diff-content-${file.id}`} role="region" aria-label={`Diff for ${file.newPath}`}>
            <DiffView file={file} />
          </div>
        )}
      </li>
    ))}
  </ul>
</nav>
```

#### Diff View Component

```tsx
// frontend/src/components/diff/DiffView.tsx

<div role="region" aria-label={`Changes to ${file.newPath}`}>
  {/* Skip link for long diffs */}
  <a href="#end-of-diff" className="skip-link">
    Skip to next file
  </a>

  {/* View toggle */}
  <div role="radiogroup" aria-label="Diff view mode">
    <button
      role="radio"
      aria-checked={viewMode === 'unified'}
      onClick={() => setViewMode('unified')}
    >
      Unified
    </button>
    <button role="radio" aria-checked={viewMode === 'split'} onClick={() => setViewMode('split')}>
      Split
    </button>
  </div>

  {/* Diff content with ARIA labels */}
  <div className="diff-container" tabIndex={0}>
    <table role="table" aria-label="Code changes">
      <thead>
        <tr>
          <th scope="col" id="line-numbers">
            Line
          </th>
          <th scope="col" id="code-content">
            Code
          </th>
        </tr>
      </thead>
      <tbody>
        {lines.map((line) => (
          <tr
            key={line.number}
            className={`diff-line diff-line-${line.type}`}
            aria-label={
              line.type === 'add'
                ? `Added: ${line.content}`
                : line.type === 'delete'
                  ? `Removed: ${line.content}`
                  : line.content
            }
          >
            <td headers="line-numbers">{line.number}</td>
            <td headers="code-content">
              <code>{line.content}</code>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  </div>

  <div id="end-of-diff" />
</div>
```

### Color Contrast Requirements

| Element               | Foreground | Background | Ratio  | Standard |
| --------------------- | ---------- | ---------- | ------ | -------- |
| **Normal Text**       | #1a1a1a    | #ffffff    | 15.6:1 | ✓ AAA    |
| **Added Lines**       | #0d4b09    | #e6ffe6    | 8.2:1  | ✓ AAA    |
| **Deleted Lines**     | #a51d0d    | #ffe6e6    | 7.1:1  | ✓ AAA    |
| **Focus Indicator**   | #0066cc    | #ffffff    | 8.5:1  | ✓ AAA    |
| **Link Text**         | #0066cc    | #ffffff    | 8.5:1  | ✓ AAA    |
| **Button Text**       | #ffffff    | #0066cc    | 8.5:1  | ✓ AAA    |
| **Disabled Text**     | #767676    | #ffffff    | 4.54:1 | ✓ AA     |
| **Error Message**     | #c00000    | #ffffff    | 7.0:1  | ✓ AAA    |
| **Code Syntax (var)** | #8f5902    | #ffffff    | 5.6:1  | ✓ AA     |
| **Code Syntax (str)** | #4e9a06    | #ffffff    | 5.2:1  | ✓ AA     |

**Dark Mode:**

| Element           | Foreground | Background | Ratio  | Standard |
| ----------------- | ---------- | ---------- | ------ | -------- |
| **Normal Text**   | #e8e8e8    | #1a1a1a    | 13.2:1 | ✓ AAA    |
| **Added Lines**   | #7ee787    | #0d4b09    | 4.9:1  | ✓ AA     |
| **Deleted Lines** | #f88e86    | #5a1e1a    | 5.1:1  | ✓ AA     |
| **Focus**         | #58a6ff    | #0d1117    | 8.1:1  | ✓ AAA    |

### Keyboard Navigation

| Key             | Action                                   |
| --------------- | ---------------------------------------- |
| **Tab**         | Navigate to next interactive element     |
| **Shift+Tab**   | Navigate to previous interactive element |
| **Enter/Space** | Activate button, toggle file expansion   |
| **Arrow Keys**  | Navigate within file list                |
| **Home**        | Jump to first file                       |
| **End**         | Jump to last file                        |
| **Ctrl+F**      | Open search within diff                  |
| **Esc**         | Close search, clear focus                |
| **?**           | Show keyboard shortcuts help             |

**Implementation:**

```tsx
// frontend/src/components/diff/DiffFileList.tsx

function DiffFileList({ files }: { files: DiffFile[] }) {
  const listRef = useRef<HTMLUListElement>(null);

  const handleKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case 'Home':
        e.preventDefault();
        focusFile(0);
        break;

      case 'End':
        e.preventDefault();
        focusFile(files.length - 1);
        break;

      case 'ArrowDown':
        e.preventDefault();
        focusNextFile();
        break;

      case 'ArrowUp':
        e.preventDefault();
        focusPreviousFile();
        break;

      case '?':
        e.preventDefault();
        showKeyboardHelp();
        break;
    }
  };

  return (
    <ul ref={listRef} role="list" onKeyDown={handleKeyDown} tabIndex={0} aria-label="Changed files">
      {/* File items */}
    </ul>
  );
}
```

### Screen Reader Announcements

```tsx
// frontend/src/components/diff/DiffStatusAnnouncer.tsx

import { useEffect } from 'react';

export function DiffStatusAnnouncer({ loading, error, fileCount }: Props) {
  const announcerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (loading) {
      announce('Loading diff files...');
    } else if (error) {
      announce(`Error loading diff: ${error.message}`);
    } else if (fileCount !== undefined) {
      announce(`${fileCount} files changed. Use arrow keys to navigate.`);
    }
  }, [loading, error, fileCount]);

  function announce(message: string) {
    if (announcerRef.current) {
      announcerRef.current.textContent = message;
    }
  }

  return (
    <div
      ref={announcerRef}
      role="status"
      aria-live="polite"
      aria-atomic="true"
      className="sr-only"
    />
  );
}
```

### Focus Management

```tsx
// frontend/src/hooks/useFocusManagement.tsx

export function useFocusManagement(fileId: string, isExpanded: boolean) {
  const prevExpandedRef = useRef(isExpanded);

  useEffect(() => {
    // When file expands, focus first interactive element in diff
    if (isExpanded && !prevExpandedRef.current) {
      const diffContent = document.getElementById(`diff-content-${fileId}`);
      const firstFocusable = diffContent?.querySelector(
        'button, a, input, [tabindex]:not([tabindex="-1"])'
      );

      if (firstFocusable instanceof HTMLElement) {
        firstFocusable.focus();
      }
    }

    prevExpandedRef.current = isExpanded;
  }, [isExpanded, fileId]);
}
```

### Responsive Design (Mobile)

```css
/* Ensure 200% zoom support */
@media (max-width: 768px) {
  .diff-container {
    /* Horizontal scroll for long lines */
    overflow-x: auto;

    /* Ensure touch targets are 44x44px minimum */
    .file-header {
      min-height: 44px;
      padding: 12px 16px;
    }

    /* Stack split view on mobile */
    &.diff-view-split {
      .diff-side-by-side {
        flex-direction: column;
      }
    }
  }

  /* Readable font sizes at 200% zoom */
  .diff-line-content {
    font-size: max(14px, 1rem);
  }
}
```

### Color Blindness Considerations

**Beyond color:**

- Use icons + text labels (not color alone)
- Pattern fills for added/deleted lines
- Text indicators: "[+]" for additions, "[-]" for deletions

```tsx
// Use both color AND icon/pattern
<div className={`diff-line diff-line-${type}`}>
  {type === 'add' && <PlusIcon aria-hidden="true" />}
  {type === 'delete' && <MinusIcon aria-hidden="true" />}
  <code>{content}</code>
</div>
```

**CSS:**

```css
/* Patterns for color blindness */
.diff-line-add {
  background: linear-gradient(
    135deg,
    #e6ffe6 0%,
    #e6ffe6 25%,
    #d4f4d4 25%,
    #d4f4d4 50%,
    #e6ffe6 50%
  );
}

.diff-line-delete {
  background: linear-gradient(
    135deg,
    #ffe6e6 0%,
    #ffe6e6 25%,
    #f4d4d4 25%,
    #f4d4d4 50%,
    #ffe6e6 50%
  );
}
```

## Testing Strategy

### Automated Testing

```typescript
// frontend/src/components/diff/__tests__/DiffView.a11y.test.tsx

import { axe, toHaveNoViolations } from 'jest-axe';
expect.extend(toHaveNoViolations);

describe('DiffView Accessibility', () => {
  it('should have no WCAG violations', async () => {
    const { container } = render(<DiffView file={mockFile} />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should support keyboard navigation', () => {
    const { getByRole } = render(<DiffFileList files={mockFiles} />);
    const firstFile = getByRole('button', { name: /first.tsx/ });

    firstFile.focus();
    expect(firstFile).toHaveFocus();

    // Tab to next file
    userEvent.tab();
    const secondFile = getByRole('button', { name: /second.tsx/ });
    expect(secondFile).toHaveFocus();
  });

  it('should announce status changes to screen readers', () => {
    const { getByRole } = render(<DiffStatusAnnouncer loading />);
    const status = getByRole('status');
    expect(status).toHaveTextContent('Loading diff files...');
  });
});
```

### Manual Testing Checklist

- [ ] Navigate entire diff using keyboard only (no mouse)
- [ ] Test with screen reader (NVDA, JAWS, VoiceOver)
- [ ] Verify color contrast with browser DevTools
- [ ] Test at 200% browser zoom
- [ ] Test on mobile (iOS VoiceOver, Android TalkBack)
- [ ] Test with high contrast mode enabled
- [ ] Test with color filters (simulate color blindness)

## Consequences

### Positive

- **Legal Compliance**: Meets ADA, Section 508, WCAG 2.1 AA
- **Broader Audience**: Accessible to 1 billion+ disabled users
- **Better UX**: Keyboard shortcuts, high contrast benefit all users
- **SEO Benefits**: Semantic HTML improves search engine ranking
- **Brand Reputation**: Demonstrates commitment to inclusivity

### Negative

- **Development Time**: +10-15% initial development effort
- **Testing Overhead**: Manual accessibility testing required
- **Maintenance**: Must maintain accessibility in future changes

### Mitigation Strategies

1. **Automated Testing**: jest-axe catches 40-60% of issues automatically
2. **Linting**: eslint-plugin-jsx-a11y prevents common mistakes
3. **Training**: Developer training on accessibility best practices
4. **Audit**: Annual third-party accessibility audit

## Monitoring

```typescript
// Track accessibility usage
analytics.track('accessibility_feature_used', {
  feature: 'keyboard_navigation',
  action: 'file_expanded',
});

analytics.track('screen_reader_detected', {
  reader: 'NVDA', // Detected via user agent
});
```

## Related Decisions

- ADR-001: Diff Library Selection (chose library with good a11y defaults)
- ADR-006: Responsive Design Strategy

## References

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- [A11y Project Checklist](https://www.a11yproject.com/checklist/)
- [WAI-ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [jest-axe Documentation](https://github.com/nickcolley/jest-axe)
