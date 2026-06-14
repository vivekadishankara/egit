# GitHub-Style PR Diff Viewer — Implementation Plan

## Goal

Replace the current flat, syntax-unaware diff display with a structured, syntax-highlighted, GitHub-style diff view on both the PR detail page and the commit detail page (they share the `DiffViewer` component).

## Design Decisions

| Decision | Choice |
|----------|--------|
| Syntect theme | `base16-ocean.dark` (existing hardcoded theme, as used in blob viewer) |
| File collapse default | All expanded |
| Inline word diff | Yes — highlight changed words within matched `-`/`+` line pairs |
| Collapse mechanism | Leptos reactive signal per file |

---

## Step 1 — `src/syntax.rs` (NEW)

Extract shared syntax highlighting utilities out of `blob.rs`.

```rust
#[cfg(feature = "ssr")]
pub fn syntax_set() -> &'static syntect::parsing::SyntaxSet

#[cfg(feature = "ssr")]
pub fn theme_set() -> &'static syntect::highlighting::ThemeSet

#[cfg(feature = "ssr")]
pub fn highlight_line(code: &str, ext: &str) -> String
```

- `syntax_set()` / `theme_set()` — identical to the `OnceLock`-backed statics currently in `blob.rs`.
- `highlight_line()` — uses `syntect::easy::HighlightLines::new(syntax, theme)`, calls `highlight_line()` on the input, then `syntect::html::styled_line_to_highlighted_html(&ranges, IncludeBackground::No)` to produce per-line inline-styled HTML.
- Theme is `base16-ocean.dark` (hardcoded), same as blob viewer.

---

## Step 2 — `src/diff.rs` (NEW)

### Data structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    pub old_path: String,
    pub new_path: String,
    pub status: String,        // "added", "deleted", "modified", "binary"
    pub extension: String,     // for syntect lookup (inferred from new_path)
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub header: String,        // e.g. "@@ -1,5 +1,7 @@"
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: String,     // "add", "delete", "context"
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,       // raw text (without leading + or -)
    pub highlighted: String,   // pre-rendered syntax HTML (server-side)
    pub inline_diff: Option<String>,  // word-level diff HTML if applicable
}
```

### Parser: `parse_diff(raw: &str) -> Vec<DiffFile>`

A state-machine parser on the unified diff format:

1. **File detection** — lines starting with `diff --git a/... b/...` start a new `DiffFile`.
   - Parse `--- a/path` / `+++ b/path` for old/new paths.
   - Detect `/dev/null` for added/deleted files.
   - Infer `extension` from the new path's file suffix.
   - Detect binary files (no `---`/`+++` lines after `diff --git` → skip).

2. **Hunk detection** — lines starting with `@@ -a,b +c,d @@` start a new `DiffHunk`.
   - Parse old start, old count, new start, new count, and the header text.

3. **Line classification** — within a hunk, each line is one of:
   - Context (space prefix) → `line_type: "context"`, increments both old/new line counters
   - Deletion (`-` prefix, but not `---`) → `line_type: "delete"`, increments old counter
   - Addition (`+` prefix, but not `+++`) → `line_type: "add"`, increments new counter

4. **Stats** — count `+` and `-` lines per file (excluding `+++`/`---`).

### Inline word diff logic

Within each hunk, after initial parsing, match consecutive `-` / `+` line pairs (they represent the same logical line with changes):

1. Collect aligned pairs: iterate through the hunk lines, matching each `delete` with the next `add` (or vice versa), handling cases where there's a 1:1 match.
2. For each matched pair, compute a word-level diff:
   - Split both old and new content on word boundaries (whitespace and punctuation).
   - Run a simple LCS-based diff on the token sequences.
   - Build an HTML string: unchanged tokens as-is, deleted tokens wrapped in `<span class="diff-word-del">`, added tokens wrapped in `<span class="diff-word-add">`.
3. Store the result in `inline_diff` for the `add` line. Set it to `None` for unmatched lines.

For unmatched `delete` lines (no corresponding `add`), set `inline_diff = None`. Unmatched `add` lines get `inline_diff` from their matched `delete` partner.

---

## Step 3 — `src/git.rs` — Modify `get_pr_diff`

```rust
pub fn get_pr_diff(
    repo_base: &str, username: &str, reponame: &str,
    head_branch: &str, base_branch: &str,
) -> anyhow::Result<Vec<DiffFile>>
```

- Keep the `git diff refs/heads/{base}...refs/heads/{head}` command unchanged.
- After capturing stdout, call `parse_diff(&raw)` to get `Vec<DiffFile>`.
- For each `DiffLine` in each file, call `highlight_line(&line.content, &file.extension)` (server-side only, behind `#[cfg(feature = "ssr")]`).
- Attach `inline_diff` HTML from the word-diff step.
- Return the structured data.

Also modify `get_commit_detail` and its internal helper to return `Vec<DiffFile>` instead of raw string:

```rust
pub struct CommitDetail {
    // ... existing fields ...
    pub diff: Vec<DiffFile>,   // was String
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,      // computed from parsed diff
}
```

---

## Step 4 — `src/server/prs.rs` — Update `get_pr_diff` server fn

```rust
#[server(GetPrDiff, "/api")]
pub async fn get_pr_diff(
    username: String, reponame: String,
    head_branch: String, base_branch: String,
) -> Result<Vec<DiffFile>, ServerFnError>
```

Change return type from `Result<String>` to `Result<Vec<DiffFile>>`.

Also update `get_commit_detail` server function in `src/pages/repo/commit.rs`:

```rust
#[server(GetCommitDetail, "/api")]
pub async fn get_commit_detail(
    username: String, reponame: String, commit_id: String,
) -> Result<CommitDetail, ServerFnError>
```

Where `CommitDetail.diff` is now `Vec<DiffFile>`.

---

## Step 5 — `src/components/diff_viewer.rs` — Rewrite

### Props

```rust
#[component]
pub fn DiffViewer(#[prop(into)] files: Vec<DiffFile>) -> impl IntoView
```

### Rendered structure

```html
<div class="diff-viewer">
  <!-- For each file -->
  <div class="diff-file">
    <!-- File header (always visible) -->
    <div class="diff-file-header">
      <div class="diff-file-header-left">
        <span>{file_status_icon}</span>
        <span class="diff-file-path">{file.new_path}</span>
      </div>
      <div class="diff-file-header-right">
        <span class="diff-stat-add">+{file.stats.additions}</span>
        <span class="diff-stat-del">-{file.stats.deletions}</span>
        <div class="diff-bar">
          <div class="diff-bar-add" style="width: {pct_add}%"></div>
          <div class="diff-bar-del" style="width: {pct_del}%"></div>
        </div>
      </div>
    </div>

    <!-- File body (collapsible) -->
    <div class="diff-body" style="display: ...">
      <!-- For each hunk -->
      <div class="diff-hunk-header">
        <span>{hunk.header}</span>
      </div>
      <!-- For each line -->
      <div class="diff-line diff-line-{line.line_type}">
        <span class="diff-line-num-old">{line.old_lineno}</span>
        <span class="diff-line-num-new">{line.new_lineno}</span>
        <div
          class="diff-line-content"
          inner_html={line.inline_diff.clone().unwrap_or(line.highlighted)}
        />
      </div>
    </div>
  </div>
</div>
```

### Behavior

- **Collapse/expand**: A reactive `RwSignal<Vec<bool>>` tracks collapsed state per file (indexed by file position). Clicking the file header toggles the corresponding entry. `.diff-body` is conditionally rendered using `style:display`.
- **Line numbers**: Old line number column is blank for pure additions; new line number column is blank for pure deletions.
- **Syntax highlighting**: Rendered via `inner_html` on the `.diff-line-content` div. Content column uses `inline_diff` when available (for matched +/- pairs), falling back to `highlighted`.
- **Binary files**: Show file header with "(binary)" label and no body.
- **Empty state**: Handled by caller (shows "No changes between these branches.").
- **Loading state**: Handled by caller's `<Suspense>`.

---

## Step 6 — `style/input.css` — Updated diff styles

Replace the existing `.diff-viewer` block (lines 273–299) with expanded styles:

```css
/* ── Diff viewer ── */

/* Outer container */
.diff-viewer {
    @apply rounded-md border border-theme overflow-hidden;
    background-color: var(--color-bg-secondary);
}

/* File sections */
.diff-file + .diff-file {
    @apply border-t border-theme;
}
.diff-file-header {
    @apply flex items-center justify-between px-4 py-2 text-sm cursor-pointer select-none;
    background-color: var(--color-bg-tertiary, var(--color-bg-secondary));
}
.diff-file-header:hover {
    background-color: color-mix(in srgb, var(--color-bg-tertiary, var(--color-bg-secondary)) 90%, var(--color-text-muted));
}
.diff-file-header-left {
    @apply flex items-center gap-2 min-w-0;
}
.diff-file-header-left .diff-file-icon {
    color: var(--color-text-muted);
    @apply flex-shrink-0;
}
.diff-file-path {
    @apply font-mono text-sm truncate;
    color: var(--color-text);
}
.diff-file-header-right {
    @apply flex items-center gap-2 flex-shrink-0;
}
.diff-stat-add {
    color: var(--color-success);
    @apply text-xs font-medium;
}
.diff-stat-del {
    color: var(--color-danger);
    @apply text-xs font-medium;
}

/* Mini bar */
.diff-bar {
    @apply inline-flex w-16 h-2 rounded-full overflow-hidden;
    background-color: var(--color-border);
}
.diff-bar-add {
    background-color: var(--color-success);
    @apply h-full;
}
.diff-bar-del {
    background-color: var(--color-danger);
    @apply h-full;
}

/* Hunk header */
.diff-hunk-header {
    @apply px-4 py-0.5 text-xs font-mono;
    background-color: color-mix(in srgb, var(--color-accent) 8%, transparent);
    color: var(--color-accent);
    border-top: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
}

/* Single line row */
.diff-line {
    @apply flex text-sm font-mono leading-relaxed;
    min-height: 1.5rem;
}

/* Line numbers */
.diff-line-num-old,
.diff-line-num-new {
    @apply w-[52px] min-w-[52px] px-2 py-0 text-right text-xs select-none;
    color: var(--color-text-muted);
    border-right: 1px solid var(--color-border);
    vertical-align: top;
}
.diff-line-num-old {
    background-color: color-mix(in srgb, var(--color-bg-secondary) 98%, black);
}
.diff-line-num-new {
    background-color: color-mix(in srgb, var(--color-bg-secondary) 98%, black);
}

/* Line backgrounds */
.diff-line-add {
    background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
}
.diff-line-del {
    background-color: color-mix(in srgb, var(--color-danger) 15%, transparent);
}
.diff-line-add .diff-line-num-new {
    background-color: color-mix(in srgb, var(--color-success) 20%, transparent);
    border-right: 1px solid var(--color-border);
}
.diff-line-del .diff-line-num-old {
    background-color: color-mix(in srgb, var(--color-danger) 20%, transparent);
    border-right: 1px solid var(--color-border);
}

/* Line content */
.diff-line-content {
    @apply px-4 py-0 whitespace-pre flex-1;
    min-width: 0;
}
.diff-line-content [style*="background"] {
    /* syntect inline background styles interfere with diff line backgrounds */
    background: transparent !important;
}

/* Inline word diff */
.diff-word-add {
    background-color: color-mix(in srgb, var(--color-success) 40%, transparent);
    border-radius: 2px;
}
.diff-word-del {
    background-color: color-mix(in srgb, var(--color-danger) 40%, transparent);
    border-radius: 2px;
}
```

---

## Step 7 — Wire up callers

### `src/pages/repo/pulls/detail.rs`

- `diff` resource: type naturally changes as the server function's return type changes. No structural changes needed.
- The `<DiffViewer diff={...}/>` call works automatically with the new prop type.

### `src/pages/repo/commit.rs`

- `CommitDetail.diff` field changes from `String` to `Vec<DiffFile>`.
- `diff_state.get()` now yields `Option<Vec<DiffFile>>` instead of `Option<String>`.
- `DiffViewer` call updates automatically.
- `files_changed`, `insertions`, `deletions` summary fields can be kept as-is (server can still compute them from the parsed diff).

### `src/pages/repo/blob.rs`

- Remove the three duplicated functions (`syntax_set`, `theme_set`, `highlight`).
- Import `use crate::syntax::{syntax_set, theme_set, highlight_line}` (or keep the existing `highlight()` fn that highlights a whole string — move it to `syntax.rs` too).

---

## Files Changed Summary

| File | Action | Lines |
|------|--------|-------|
| `src/syntax.rs` | **NEW** | ~50 |
| `src/diff.rs` | **NEW** | ~180 |
| `src/git.rs` | Modify `get_pr_diff`, `get_commit_detail` return types, add parsing call | ~30 |
| `src/server/prs.rs` | Modify `get_pr_diff` server fn return type | ~5 |
| `src/pages/repo/commit.rs` | Update `CommitDetail.diff` type | ~5 |
| `src/components/diff_viewer.rs` | **REWRITE** | ~90 |
| `src/pages/repo/pulls/detail.rs` | Type adaptation (implicit) | ~0 |
| `src/pages/repo/blob.rs` | Remove duplicated syntax functions, import from `syntax.rs` | ~15 |
| `style/input.css` | **REWRITE** diff styles section | ~100 |

---

## Edge Cases Handled

| Case | Handling |
|------|----------|
| Binary files | Detected by parser (`status: "binary"`), file header shown, body hidden |
| New files | Old path is `/dev/null`, no old line numbers, all lines are `add` |
| Deleted files | New path is `/dev/null`, no new line numbers, all lines are `delete` |
| Renamed files | Detected via `rename from`/`rename to` in raw diff, shown with rename icon |
| Empty diff | Caller checks `Vec::is_empty()` before rendering DiffViewer |
| Large files | No special handling (full diff loaded) — could add lazy loading later |
| Theme clash | Syntect inline styles set only foreground color + font style; CSS overrides background via `!important` on `.diff-line-content` |
