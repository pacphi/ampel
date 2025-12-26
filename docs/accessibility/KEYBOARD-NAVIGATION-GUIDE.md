# Keyboard Navigation Guide - Git Diff Components

Complete guide for keyboard-only navigation through git diff components.

---

## Quick Reference

| Action               | Keyboard Shortcut                         |
| -------------------- | ----------------------------------------- |
| Navigate forward     | `Tab`                                     |
| Navigate backward    | `Shift + Tab`                             |
| Activate button/link | `Enter` or `Space`                        |
| Open select menu     | `Enter` or `Space`                        |
| Navigate menu items  | `Arrow Up/Down`                           |
| Select menu item     | `Enter`                                   |
| Close menu           | `Escape`                                  |
| Clear search         | Click clear button or select all + delete |

---

## FilesChangedTab Component

### Tab Order

1. **Search input** - Filter files by name
2. **Clear search button** (if search has text) - Clear search query
3. **Filter select** - Filter by file status (all/added/deleted/modified/binary)
4. **Unified view button** - Switch to unified diff view
5. **Split view button** - Switch to split diff view
6. **Expand all button** - Expand all file diffs
7. **Collapse all button** - Collapse all file diffs
8. **File navigation sidebar** - Jump to specific files
9. **File diff items** (for each file):
   - Expand/collapse button
   - (when expanded) Diff content becomes keyboard navigable

### Detailed Navigation Steps

#### 1. Search for Files

```
Tab to search input
Type: "App.tsx"
Result: Only matching files shown
Clear: Tab to X button, press Enter
```

**Expected behavior:**

- Input gains focus ring
- Typing filters files in real-time
- Results count updates: "Showing 1 of 3 files"
- Clear button appears when text entered

#### 2. Filter by Status

```
Tab to filter select
Press: Enter or Space
Navigate: Arrow Down to "Added"
Select: Enter
```

**Expected behavior:**

- Select opens dropdown
- Current selection highlighted
- Arrow keys move through options
- Enter selects, Escape cancels

#### 3. Switch View Mode

```
Tab to Unified button
Press: Enter or Space
Result: View switches to unified mode
```

**Expected behavior:**

- Button shows active state (filled background)
- Other view mode button shows inactive state
- Diff re-renders in selected mode

#### 4. Expand All Files

```
Tab to "Expand All" button
Press: Enter or Space
Result: All file diffs expand
```

**Expected behavior:**

- All files expand simultaneously
- "Expand All" becomes disabled
- "Collapse All" becomes enabled
- Focus remains on button

#### 5. Collapse All Files

```
Tab to "Collapse All" button
Press: Enter or Space
Result: All file diffs collapse
```

**Expected behavior:**

- All files collapse simultaneously
- "Collapse All" becomes disabled
- "Expand All" becomes enabled
- Focus remains on button

#### 6. Navigate Files

```
Tab through file list
Each file has expand/collapse button
Press: Enter or Space to toggle
```

**Expected behavior:**

- Each file header is focusable
- Expand/collapse icon indicates state
- Diff content appears/disappears smoothly
- Focus indicator clearly visible

---

## DiffViewer Component

### Tab Order (Multi-file View)

1. **Filter input** - Filter files by name
2. **Modified filter button** - Show only modified files
3. **Added filter button** - Show only added files
4. **Deleted filter button** - Show only deleted files
5. **File items** - Each file header is clickable

### Tab Order (Single File View)

1. **Diff content** - Scrollable content area
2. (No other interactive elements in view-only mode)

### Keyboard Actions

#### Filter Files by Name

```
Tab to filter input
Type: "component"
Result: Only files matching "component" shown
```

#### Filter by Status

```
Tab to "Modified" button
Press: Enter or Space
Result: Only modified files shown, button highlighted
Press again: Filter removed
```

#### Toggle File Expansion

```
Tab to file header
Press: Enter or Space
Result: Diff content expands/collapses
```

---

## DiffFileItem Component

### Tab Order

1. **Expand/collapse button** - Toggle diff visibility
2. (When expanded) **Diff content** - Scrollable diff lines

### Keyboard Actions

#### Expand File Diff

```
Tab to expand button (chevron right icon)
Press: Enter or Space
Result: Diff content appears
Button shows: Chevron down icon
ARIA: aria-expanded="true"
```

#### Collapse File Diff

```
Tab to collapse button (chevron down icon)
Press: Enter or Space
Result: Diff content hides
Button shows: Chevron right icon
ARIA: aria-expanded="false"
```

#### Navigate Diff Content

```
Once expanded, use:
Arrow keys: Scroll through diff
Page Up/Down: Jump by page
Home/End: Jump to start/end
```

---

## Common Keyboard Patterns

### Focus Indicators

All interactive elements show clear focus indicators:

```css
/* Focus ring using Tailwind */
focus-visible:outline-none
focus-visible:ring-2
focus-visible:ring-ring
focus-visible:ring-offset-2
```

**Visual appearance:**

- 2px colored ring around element
- 2px offset from element edge
- High contrast (4.5:1 minimum)
- Visible in light and dark mode

### Disabled States

Disabled buttons are not keyboard focusable:

```tsx
<Button disabled={expandedFiles.size === 0}>Collapse All</Button>
```

**Expected behavior:**

- Button appears grayed out
- Tab skips over disabled buttons
- Screen readers announce "disabled"

### Focus Management

Focus is managed automatically:

- **After expand:** Focus stays on toggle button
- **After search:** Focus stays in input
- **After filter:** Focus stays on filter button
- **After keyboard navigation:** Focus moves to next element in tab order

---

## Accessibility Best Practices

### Do's

✅ **DO** use Tab and Shift+Tab for navigation
✅ **DO** use Enter or Space to activate buttons
✅ **DO** expect clear focus indicators
✅ **DO** expect logical tab order (left-to-right, top-to-bottom)
✅ **DO** expect disabled elements to be skipped

### Don'ts

❌ **DON'T** rely on mouse-only interactions
❌ **DON'T** expect click events on non-focusable elements
❌ **DON'T** expect keyboard shortcuts without standard patterns
❌ **DON'T** expect tab order to skip around randomly

---

## Testing Checklist

### Manual Keyboard Testing

- [ ] Tab through all elements in order
- [ ] Activate all buttons with Enter
- [ ] Activate all buttons with Space
- [ ] Verify focus indicators visible
- [ ] Verify no focus traps (can Tab in and out)
- [ ] Verify disabled buttons skipped
- [ ] Verify Enter works on inputs
- [ ] Verify Escape works on select menus
- [ ] Test in light mode
- [ ] Test in dark mode

### Browser Testing

- [ ] Chrome (Tab, Enter, Space)
- [ ] Firefox (Tab, Enter, Space)
- [ ] Safari (Tab, Enter, Space)
- [ ] Edge (Tab, Enter, Space)

### Operating System Testing

- [ ] Windows (Tab, Enter, Space)
- [ ] macOS (Tab, Enter, Space)
- [ ] Linux (Tab, Enter, Space)

---

## Troubleshooting

### Issue: Can't tab to element

**Solution:** Check if element is:

1. Focusable (`<button>`, `<input>`, `<a>`, or `tabindex="0"`)
2. Not disabled
3. Not hidden (`display: none` or `visibility: hidden`)

### Issue: Focus indicator not visible

**Solution:** Check browser settings:

1. Some browsers hide focus for mouse users
2. Use `focus-visible` instead of `focus`
3. Verify high contrast mode compatibility

### Issue: Stuck in component (keyboard trap)

**Solution:** This is a WCAG violation:

1. Should always be able to Tab out
2. Should always be able to Shift+Tab out
3. Report as accessibility bug

### Issue: Button doesn't activate with Space

**Solution:** Ensure button is real `<button>` element:

```tsx
// Good
<button onClick={...}>Click</button>

// Bad - needs custom Space handler
<div onClick={...}>Click</div>
```

---

## Advanced Patterns

### Skip Links (Future Enhancement)

```tsx
<a href="#main-content" className="sr-only focus:not-sr-only">
  Skip to main content
</a>
```

Allows keyboard users to skip navigation and jump to content.

### Keyboard Shortcuts (Future Enhancement)

```tsx
// Potential shortcuts
Cmd/Ctrl + F: Focus search
Cmd/Ctrl + E: Expand all
Cmd/Ctrl + C: Collapse all
```

### Roving Tabindex (Future Enhancement)

For file lists, only one file in tab order at a time:

- Arrow keys move between files
- Tab moves to next component
- More efficient for long lists

---

## Resources

### WCAG 2.1 Guidelines

- [2.1.1 Keyboard](https://www.w3.org/WAI/WCAG21/Understanding/keyboard.html)
- [2.1.2 No Keyboard Trap](https://www.w3.org/WAI/WCAG21/Understanding/no-keyboard-trap.html)
- [2.4.3 Focus Order](https://www.w3.org/WAI/WCAG21/Understanding/focus-order.html)
- [2.4.7 Focus Visible](https://www.w3.org/WAI/WCAG21/Understanding/focus-visible.html)

### Browser Documentation

- [Chrome Keyboard Navigation](https://support.google.com/chrome/answer/157179)
- [Firefox Keyboard Shortcuts](https://support.mozilla.org/en-US/kb/keyboard-shortcuts-perform-firefox-tasks-quickly)
- [Safari Keyboard Shortcuts](https://support.apple.com/guide/safari/keyboard-shortcuts-cpsh003)

---

**Last Updated:** 2025-12-25
**Maintained By:** QE Team
**WCAG Level:** AA Compliant
