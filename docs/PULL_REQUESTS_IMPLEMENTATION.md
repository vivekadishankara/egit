# Pull Requests Implementation Plan

## Current State

### Existing Infrastructure
- **Database**: `migrations/004_pull_requests.sql` creates `pull_requests` table with columns: id, repo_id, author_id, title, body, head_branch, base_branch, status, created_at, updated_at
- **Database types**: Uses `uuid::Uuid` for IDs, `time::OffsetDateTime` for timestamps
- **Server functions**: `src/server/prs.rs` — full CRUD for PRs
- **Pages**:
  - `src/pages/repo/pulls/list.rs` — PullListPage
  - `src/pages/repo/pulls/new.rs` — NewPullPage
  - `src/pages/repo/pulls/detail.rs` — PullDetailPage
- **RepoTabBar**: Shows [Overview, Code, Commits, Pull Requests] (PR tab is unconditional, always clickable)
- **Overview sidebar**: PR sidebar with live counts on the repo overview page

### Project Architecture (from AGENTS.md)
- **Leptos 0.8** SSR+hydrate with `#[server]` functions for all DB access
- **Database**: PostgreSQL via sqlx (runtime queries)
- **Git**: `gix` 0.70 for read ops; `git` CLI for init/commit-tree/show/diff/merge
- **Auth**: Cookies via `leptos_axum::extract::<HeaderMap>().await`
- **Styling**: Tailwind `@layer components` with CSS variables

---

## Implementation Status

### Step 1: Server Functions (`src/server/prs.rs`) ✅
**File**: `src/server/prs.rs`

Implemented functions:
- `create_pull_request(repo_id, title, body, head_branch, base_branch, username, reponame)` → `Uuid` (extracts `author_id` from session; rejects duplicate PRs for same head+base regardless of status; redirects to detail page via `leptos_axum::redirect`)
- `list_pull_requests(repo_id, status)` → `Vec<PullRequest>`
- `get_pull_request(pr_id)` → `PullRequestDetail` (includes `owner_name` field)
- `get_repo_id_by_name(username, reponame)` → `Uuid`
- `merge_pull_request(pr_id)` — auth check (author only), performs actual git merge using `git merge-tree --write-tree` + `git commit-tree` + `git update-ref`
- `close_pull_request(pr_id)` — auth check (author only)
- `get_branch_list_for_pr(username, reponame)` → `Vec<String>`
- `get_pr_diff(username, reponame, head_branch, base_branch)` → `String` (server fn wrapper around `crate::git::get_pr_diff`)
- `get_pull_request_counts(repo_id)` → `PullRequestCounts { open, merged, closed }`
- `has_pull_requests(repo_id)` → `bool`

**Note**: `sqlx` imports and `sqlx::FromRow` derives are gated behind `#[cfg(feature = "ssr")]` so the module compiles for both SSR and hydrate.

### Step 2: Update `src/lib.rs` to add PR module ✅
- `pub mod server;` (no cfg gate — needed for hydrate to see server fn types)
- `#[cfg(feature = "ssr")] pub use server::prs;`

### Step 3: Add Git Operations (`src/git.rs`) ✅
**File**: `src/git.rs`

`get_pr_diff(repo_base, username, reponame, head_branch, base_branch)` → `Result<String>`

Uses `git diff` CLI (not gix) — follows the existing pattern in `get_commit_diff_internal`.

### Step 4: Routes (`src/app.rs`) ✅
PR routes already registered:
- `/:username/:reponame/pulls` → `PullListPage`
- `/:username/:reponame/pulls/new` → `NewPullPage`
- `/:username/:reponame/pulls/:pr_id` → `PullDetailPage`

### Step 5: Overview Page Sidebar (`src/pages/repo/overview.rs`) ✅
- Added `repo_id: Uuid` and `has_pull_requests: bool` to `RepoInfo`
- `get_repo_overview` now queries PR count to populate `has_pull_requests`
- Added PR sidebar with:
  - "New pull request" link
  - Open / Merged / Closed links with live counts from `get_pull_request_counts`
  - Flex layout: sidebar (w-64) on the left, existing content on the right

### Step 6: RepoTabBar Component (`src/components/repo_tab_bar.rs`) ✅
- Added Pull Requests tab (always an `<a>` link, even when active)
- PR tab is unconditional (always visible in the tab bar)
- `has_pull_requests` prop removed from `RepoTabBar`
- All call sites updated: `overview.rs`, `tree.rs`, `blob.rs`, `commits.rs`, `list.rs`

### Step 7: Implement Pull Requests List Page ✅
**File**: `src/pages/repo/pulls/list.rs`

Full `PullListPage` implementation:
- Reads `username`/`reponame` from URL params, `status` from `?status=` query (defaults to `"open"`)
- Fetches repo overview via `get_repo_overview` for tab bar meta, description, private badge
- Fetches PR list via `list_pull_requests(repo_id, status)`
- Left sidebar with "New pull request" button at top, then Open / Merged / Closed filter links (active state highlighted)
- PR card list with title, status badge (`--color-success` / `--color-accent` / `--color-danger`), author, branch info (`head → base`), and relative timestamp via `format_pr_time`
- `<RepoTabBar active="pulls">` with repo header (owner/name, private badge, description)

### Step 8: Helper Functions (`src/server/prs.rs`) ✅
- `get_repo_id_by_name` (already part of Step 1)
- `get_pull_request_counts` (added in Step 5)
- `has_pull_requests` (added in Step 5)

### Step 9: Implement New Pull Request Page ✅
**File**: `src/pages/repo/pulls/new.rs`

Full `NewPullPage` implementation:
- Branch selection via `get_branch_list_for_pr` (base defaults to repo default branch)
- `<ActionForm>` with hidden `repo_id`, `username`, `reponame`, base/head branch `<select>` dropdowns, title `<input>`, body `<textarea>`
- `ServerAction::<CreatePullRequest>` submit with pending state and error display
- Redirect to PR detail page on success (via `leptos_axum::redirect` in server function)
- Duplicate PR detection: `create_pull_request` rejects creation when any PR (open/merged/closed) already exists with the same `head_branch` + `base_branch` for the repo

### Step 10: Implement Pull Request Details Page ✅
**File**: `src/pages/repo/pulls/detail.rs`

Full `PullDetailPage` implementation:
- Reads `username`/`reponame`/`pr_id` from URL params
- Fetches repo overview via `get_repo_overview` for tab bar meta, description, private badge
- Fetches PR detail via `get_pull_request(pr_id)`
- Fetches diff via `get_pr_diff` (shown for open/merged PRs, displayed via `DiffViewer`)
- Shows: PR number (`#short_uuid`), title, status badge, author, timestamp, branch info (`head → base`)
- Body rendered as markdown via `Markdown` component
- Merge/Close buttons (visible only for open PRs when current user is the author)
- `ServerAction::<MergePullRequest>` and `ServerAction::<ClosePullRequest>` with `ActionForm`
- Redirects to PR list on successful merge/close
- Author check via `get_current_user()` comparing usernames (client-side show/hide) + server-side auth verification

---

## Files Modified (cumulative)

| File | Changes |
|------|---------|
| `src/server/prs.rs` | All PR server functions, PR count helpers, cfg_attr for sqlx; `merge_pull_request` now does actual git merge; auth checks on merge/close; `owner_name` in PullRequestDetail; `GetPrDiff` server fn; duplicate check covers all statuses |
| `src/lib.rs` | Removed cfg gate from `pub mod server;` |
| `src/git.rs` | `get_pr_diff` using git CLI |
| `src/components/repo_tab_bar.rs` | PR tab always an `<a>` link, removed `has_pull_requests` prop, removed conditional span |
| `src/pages/repo/overview.rs` | PR sidebar, repo_id+has_pull_requests in RepoInfo |
| `src/pages/repo/tree.rs` | Removed `has_pull_requests={false}` from RepoTabBar |
| `src/pages/repo/blob.rs` | Removed `has_pull_requests={false}` from RepoTabBar |
| `src/pages/repo/commits.rs` | Removed `has_pull_requests={false}` from RepoTabBar |
| `src/pages/repo/pulls/list.rs` | Full PullListPage implementation with tab bar, status filter, PR cards; "New pull request" button in sidebar |
| `src/pages/repo/pulls/new.rs` | Full NewPullPage implementation with branch selects, form, hidden username/reponame, error display |
| `src/pages/repo/pulls/detail.rs` | Full PullDetailPage implementation with PR info, body (markdown), merge/close buttons, diff view |
| `src/server/prs.rs` | `create_pull_request` now extracts `author_id` from session and rejects duplicate PRs (any status) for same head+base |

---

## Database Migration

The migration already exists (`migrations/004_pull_requests.sql`). Ensure it runs on startup or manually:

```bash
sqlx migrate run
```

---

## Notes

- `ServerAction` uses PascalCase: `ServerAction::<MergePullRequest>::new()` if using form-based approach
- DB queries use `sqlx::query!` or `query_as!` for compile-time SQL checking
- All `#[server]` function struct types need `#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]` — NOT unconditional `#[derive(sqlx::FromRow)]` — because the module is always compiled (no cfg gate on `pub mod server`)
- `pub mod server;` in `src/lib.rs` is **not** behind `#[cfg(feature = "ssr")]` — the `#[server]` macro generates client-side stubs, so the module must be visible in hydrate builds
- `get_pr_diff` uses `git diff refs/heads/{base}...refs/heads/{head}` CLI — the three-dot syntax shows changes in head that aren't in base
- Merge uses `git merge-tree --write-tree` to compute the merge tree, `git commit-tree` to create a merge commit with two parents, and `git update-ref` to update the base branch ref — all on the bare repo without needing a worktree
- Redirect after PR creation is done server-side via `leptos_axum::redirect` (same pattern as login/register), avoiding client-side `Effect`+`navigate` race conditions
