# Screen Reader Testing Guide - Git Diff Components

Complete guide for testing git diff components with screen readers.

---

## Supported Screen Readers

| Screen Reader | Platform    | Version Tested | Status                     |
| ------------- | ----------- | -------------- | -------------------------- |
| VoiceOver     | macOS 13+   | Built-in       | ✅ Fully Supported         |
| NVDA          | Windows 10+ | 2024.1+        | ✅ Fully Supported         |
| JAWS          | Windows 10+ | 2024+          | ⚠️ Compatible (not tested) |
| Narrator      | Windows 10+ | Built-in       | ⚠️ Compatible (not tested) |
| TalkBack      | Android     | Latest         | ⚠️ Mobile (partial)        |
| ChromeVox     | Chrome      | Extension      | ✅ Supported               |

---

## VoiceOver (macOS)

### Setup

```bash
# Enable VoiceOver
Cmd + F5

# Or: System Settings > Accessibility > VoiceOver > Enable

# Basic commands:
VO = Control + Option
VO + Right Arrow = Next item
VO + Left Arrow = Previous item
VO + Space = Activate
VO + A = Read from current position
```

### FilesChangedTab Announcements

#### Initial Load

```
"Search files, search field"
"Filter, combo box, All files selected"
"Unified, button, selected"
"Split, button"
"Expand All, button"
"Collapse All, button, dimmed"
"3 files changed"
"30 additions"
"5 deletions"
```

#### Searching

```
User types: "App"

VoiceOver announces:
"A, P, p"
[Screen updates]
"Showing 1 of 3 files"
```

#### Filtering

```
User activates filter select:
VO + Space on "Filter" button

VoiceOver announces:
"All files, selected"
"Added"
"Deleted"
"Modified"
"Binary"

User selects "Added":
VO + Down Arrow to "Added"
VO + Space to select

VoiceOver announces:
"Added, selected"
[Screen updates]
"Showing 1 of 3 files"
```

#### Expanding Files

```
User activates expand button:
VO + Space on file

VoiceOver announces:
"Expand file diff, button"
[Click]
"Collapse file diff, button, expanded"
[Diff content appears]
```

### DiffViewer Announcements

#### Multi-file Summary

```
"2 files changed, heading level 2"
"30 additions"
"5 deletions"
"Filter files, edit text"
"Modified, button"
"Added, button"
"Deleted, button"
```

#### File List

```
"src/components/App.tsx, heading level 4"
"Modified, group"
"TypeScript, group"
"10 additions"
"5 deletions"
[Click to expand]
"Expanded"
[Diff content]
```

### DiffFileItem Announcements

#### Collapsed State

```
"Expand file diff, button"
"App.tsx, heading"
"modified, group"
"TypeScript, group"
"src/components, text"
"10, text"
"5, text"
```

#### Expanded State

```
"Collapse file diff, button, expanded"
"App.tsx, heading"
"modified, group"
[Diff content begins]
"import React from react"
"Plus import useState from react"
"Minus return div Hello div"
[etc.]
```

#### Binary Files

```
"Expand file diff, button"
"image.png, heading"
"added, group"
"Binary, group"
[If expanded]
"Binary file - no diff available"
```

---

## NVDA (Windows)

### Setup

```bash
# Download from https://www.nvaccess.org/download/
# Install and run

# Basic commands:
NVDA = Insert (or Caps Lock if configured)
NVDA + Down Arrow = Next item
NVDA + Up Arrow = Previous item
Enter/Space = Activate
NVDA + Down Arrow (on button) = Read button state
```

### FilesChangedTab Announcements

#### Navigation Mode

```
"Search files, edit, blank"
"Filter, combo box, All files"
"Unified, button, pressed"
"Split, button, not pressed"
"Expand All, button"
"Collapse All, button, unavailable"
```

#### Forms Mode (in search input)

```
User types: "App"

NVDA announces:
"A"
"P"
"P"
[Announces each character as typed]
```

#### Button States

```
User activates "Expand All":
Space or Enter

NVDA announces:
"Expand All, button"
[Click]
"Expanded"
[Button becomes unavailable]
"Expand All, button, unavailable"
```

### DiffViewer Announcements

#### Summary Statistics

```
"Main region"
"2 files changed"
"30 additions"
"5 deletions"
```

#### File Filtering

```
"Modified, toggle button, not pressed"
[User presses]
"Modified, toggle button, pressed"
[Files filter]
```

### DiffFileItem Announcements

#### File Header

```
"Expand file diff, button, collapsed"
"App.tsx, heading level 2"
"modified, static text"
"TypeScript, static text"
"src/components, static text"
"10, static text"
"5, static text"
```

#### Expanded Diff

```
"Collapse file diff, button, expanded"
[Diff content]
"Line 1: import React from react"
"Line 2: Plus import useState from react"
"Line 3: export function App"
[etc.]
```

---

## Expected Announcements by Component

### Search Input

```
Component: <Input placeholder="Search files..." />

VoiceOver: "Search files, search field"
NVDA: "Search files, edit, blank"
Expected: Input type and purpose clear
```

### Filter Select

```
Component: <Select value="all">

VoiceOver: "Filter, combo box, All files selected"
NVDA: "Filter, combo box, All files"
Expected: Current selection announced
```

### View Mode Buttons

```
Component: <Button variant={viewMode === 'unified' ? 'default' : 'ghost'}>

VoiceOver: "Unified, button, selected"
NVDA: "Unified, button, pressed"
Expected: Selected state communicated
```

### Expand/Collapse Buttons

```
Component: <Button aria-label="Expand file diff" aria-expanded="false">

VoiceOver: "Expand file diff, button"
NVDA: "Expand file diff, button, collapsed"
Expected: Action and state clear
```

### File Status Badges

```
Component: <Badge variant="default">added</Badge>

VoiceOver: "added, group"
NVDA: "added, static text"
Expected: Status communicated
```

### Additions/Deletions Counters

```
Component: <span className="text-green-600">+10</span>

VoiceOver: "10, text"
NVDA: "10, static text"
Expected: Number announced (context from icon)
```

### Results Count

```
Component: <div>Showing {filtered} of {total} files</div>

VoiceOver: "Showing 1 of 3 files, text"
NVDA: "Showing 1 of 3 files"
Expected: Filter results announced
```

---

## ARIA Patterns Used

### Button with Icon and Label

```tsx
<Button aria-label="Expand file diff" aria-expanded={false}>
  <ChevronRight />
</Button>
```

**Announced as:**

- "Expand file diff, button"
- State: "collapsed" or "not expanded"

### Button with Text and Icon

```tsx
<Button>
  <ChevronsDown /> Expand All
</Button>
```

**Announced as:**

- "Expand All, button"
- Icon decorative, text provides label

### Select/Combo Box

```tsx
<Select>
  <SelectTrigger>
    <Filter /> <SelectValue />
  </SelectTrigger>
</Select>
```

**Announced as:**

- "Filter, combo box"
- Current value: "All files selected"

### Status Badge

```tsx
<Badge variant="default">modified</Badge>
```

**Announced as:**

- "modified" (grouped with file name)

---

## Testing Procedures

### VoiceOver Testing Procedure

1. **Enable VoiceOver:** Cmd + F5
2. **Navigate to page:** Open PR with Files Changed tab
3. **Start testing:** VO + A to begin reading
4. **Navigate elements:** VO + Right Arrow through each element
5. **Test interactions:**
   - VO + Space to activate buttons
   - Type in search input
   - Navigate select menu with arrows
6. **Verify announcements:** Match expected text above
7. **Test empty states:** Clear search, verify "No files" message
8. **Disable VoiceOver:** Cmd + F5

### NVDA Testing Procedure

1. **Start NVDA:** Desktop shortcut or Ctrl + Alt + N
2. **Navigate to page:** Open PR with Files Changed tab
3. **Start testing:** NVDA + Down Arrow to read page
4. **Navigate elements:** Down Arrow through each element
5. **Test interactions:**
   - Space/Enter to activate buttons
   - Type in search input
   - Navigate select menu with arrows
6. **Verify announcements:** Match expected text above
7. **Test forms mode:** Tab into input, verify typing announced
8. **Stop NVDA:** NVDA + Q

---

## Common Issues and Fixes

### Issue: Button not announced

**Symptom:** Screen reader skips button or announces as "button" without label

**Fix:**

```tsx
// Before
<Button><Icon /></Button>

// After
<Button aria-label="Descriptive action"><Icon /></Button>
```

### Issue: State not announced

**Symptom:** Expanded/collapsed state not communicated

**Fix:**

```tsx
// Before
<Button onClick={toggle}>...</Button>

// After
<Button onClick={toggle} aria-expanded={isExpanded}>...</Button>
```

### Issue: Results not announced

**Symptom:** Filtering doesn't announce updated results

**Fix:**

```tsx
// Add ARIA live region
<div aria-live="polite" aria-atomic="true">
  Showing {filtered} of {total} files
</div>
```

### Issue: Icon-only button unclear

**Symptom:** Screen reader announces icon alt text or nothing

**Fix:**

```tsx
// Before
<Button><X /></Button>

// After
<Button aria-label="Clear search"><X aria-hidden="true" /></Button>
```

---

## Verification Checklist

### FilesChangedTab

- [ ] Search input announced as "Search files, search field"
- [ ] Filter select announces current selection
- [ ] View mode buttons announce selected state
- [ ] Expand/Collapse All buttons announce disabled state
- [ ] File count announced: "Showing X of Y files"
- [ ] Empty state announced: "No files match your filters"

### DiffViewer

- [ ] Summary statistics announced: "X files changed"
- [ ] Filter buttons announce toggle state
- [ ] File headers announce as headings
- [ ] Status badges announced with file names
- [ ] Empty state announced: "No files changed"

### DiffFileItem

- [ ] Expand button announces: "Expand file diff"
- [ ] Collapse button announces: "Collapse file diff"
- [ ] Button announces expanded state: aria-expanded="true"
- [ ] File name announced as heading
- [ ] Status badge announced: "modified/added/deleted"
- [ ] Language badge announced: "TypeScript"
- [ ] Additions/deletions counters announced
- [ ] Binary file message announced clearly

---

## Advanced Testing

### Screen Reader Shortcuts

#### VoiceOver Rotor

```
VO + U = Open rotor
Left/Right arrows = Change category (headings, links, form controls)
Up/Down arrows = Navigate items
Enter = Jump to item
```

**Use to verify:**

- All headings listed
- All buttons listed
- All form controls listed

#### NVDA Elements List

```
NVDA + F7 = Open elements list
Tab between categories
Enter = Jump to item
```

**Use to verify:**

- All links listed
- All headings listed
- All form fields listed

### Reading Order Test

```
VoiceOver: VO + A = Read all
NVDA: NVDA + Down Arrow = Read next

Verify:
1. Content reads in logical order
2. No content skipped
3. No content read twice
4. Related items grouped together
```

---

## Resources

### VoiceOver

- [VoiceOver User Guide](https://support.apple.com/guide/voiceover/welcome/mac)
- [WebAIM VoiceOver Guide](https://webaim.org/articles/voiceover/)

### NVDA

- [NVDA User Guide](https://www.nvaccess.org/files/nvda/documentation/userGuide.html)
- [WebAIM NVDA Guide](https://webaim.org/articles/nvda/)

### ARIA Patterns

- [ARIA Authoring Practices Guide](https://www.w3.org/WAI/ARIA/apg/)
- [Button Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/)
- [Combobox Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/)

---

**Last Updated:** 2025-12-25
**Tested With:** VoiceOver (macOS 13), NVDA 2024.1
**WCAG Level:** AA Compliant
