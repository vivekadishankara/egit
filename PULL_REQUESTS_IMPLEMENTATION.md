# Pull Requests Implementation Plan

## Current State

### Existing Infrastructure
- **Database**: `migrations/004_pull_requests.sql` creates `pull_requests` table with columns: id, repo_id, author_id, title, body, head_branch, base_branch, status, created_at, updated_at
- **Database types**: Uses `uuid::Uuid` for IDs, `time::OffsetDateTime` for timestamps
- **Server functions**: `src/server/prs.rs` — full CRUD for PRs
- **Pages** (stub implementations still):
  - `src/pages/repo/pulls/list.rs` — PullListPage
  - `src/pages/repo/pulls/new.rs` — NewPullPage  
  - `src/pages/repo/pulls/detail.rs` — PullDetailPage
- **RepoTabBar**: Now shows [Overview, Code, Commits, Pull Requests] (PR tab shown conditionally via `has_pull_requests` prop)
- **Overview sidebar**: PR sidebar with live counts on the repo overview page

### Project Architecture (from AGENTS.md)
- **Leptos 0.8** SSR+hydrate with `#[server]` functions for all DB access
- **Database**: PostgreSQL via sqlx (runtime queries)
- **Git**: `gix` 0.70 for read ops; `git` CLI for init/commit-tree/show/diff
- **Auth**: Cookies via `leptos_axum::extract::<HeaderMap>().await`
- **Styling**: Tailwind `@layer components` with CSS variables

---

## Implementation Status

### Step 1: Server Functions (`src/server/prs.rs`) ✅
**File**: `src/server/prs.rs`

Implemented functions:
- `create_pull_request(repo_id, title, body, head_branch, base_branch)` → `Uuid`
- `list_pull_requests(repo_id, status)` → `Vec<PullRequest>`
- `get_pull_request(pr_id)` → `PullRequestDetail`
- `get_repo_id_by_name(username, reponame)` → `Uuid`
- `merge_pull_request(pr_id, user_id)` — DB status update only (no git merge yet)
- `close_pull_request(pr_id)`
- `get_branch_list_for_pr(username, reponame)` → `Vec<String>`
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
- Added `has_pull_requests: bool` prop
- Added Pull Requests tab (clickable link or static span based on `active`)
- PR tab shown only when `has_pull_requests` is true
- All call sites updated: `overview.rs`, `tree.rs`, `blob.rs`, `commits.rs`

### Step 7: Implement Pull Requests List Page ❌ (stub only)
**File**: `src/pages/repo/pulls/list.rs`

Current: `"Pull requests — coming in step 10"` placeholder.

### Step 8: Helper Functions (`src/server/prs.rs`) ✅
- `get_repo_id_by_name` (already part of Step 1)
- `get_pull_request_counts` (added in Step 5)
- `has_pull_requests` (added in Step 5)

### Step 9: Implement New Pull Request Page ❌ (stub only)
**File**: `src/pages/repo/pulls/new.rs`

Current: `"New pull request — coming in step 10"` placeholder.

### Step 10: Implement Pull Request Details Page ❌ (stub only)
**File**: `src/pages/repo/pulls/detail.rs`

Current: `"Pull request detail — coming in step 10"` placeholder.

---

## Files Modified (cumulative)

| File | Changes |
|------|---------|
| `src/server/prs.rs` | All PR server functions, PR count helpers, cfg_attr for sqlx |
| `src/lib.rs` | Removed cfg gate from `pub mod server;` |
| `src/git.rs` | `get_pr_diff` using git CLI |
| `src/components/repo_tab_bar.rs` | `has_pull_requests` prop + PR tab |
| `src/pages/repo/overview.rs` | PR sidebar, repo_id+has_pull_requests in RepoInfo |
| `src/pages/repo/tree.rs` | `has_pull_requests={false}` added to RepoTabBar |
| `src/pages/repo/blob.rs` | `has_pull_requests={false}` added to RepoTabBar |
| `src/pages/repo/commits.rs` | `has_pull_requests={false}` added to RepoTabBar |

---

## Database Migration

The migration already exists (`migrations/004_pull_requests.sql`). Ensure it runs on startup or manually:

```bash
sqlx migrate run
```

---

## Next Steps

1. **Implement Pull List Page** (`src/pages/repo/pulls/list.rs`)
   - Fetch repo_id via `get_repo_id_by_name`
   - Use `list_pull_requests` to fetch PRs by status
   - Render PR card list with status badges
   - Needs `RepoTabBar` with `active="pulls"` and `has_pull_requests`

2. **Implement New PR Page** (`src/pages/repo/pulls/new.rs`)
   - Branch selection via `get_branch_list_for_pr`
   - Form with title, body, head_branch, base_branch
   - Submit via `create_pull_request`

3. **Implement PR Detail Page** (`src/pages/repo/pulls/detail.rs`)
   - Fetch PR via `get_pull_request`
   - Merge/Close buttons
   - Author info, timestamps, branch info
   - Diff display (use `get_pr_diff`)

4. **Implement actual git merge logic** in `merge_pull_request`
   - Currently only updates DB status
   - Needs to perform git merge in the bare repo

5. **Add diff view** between branches in PR detail page
   - Use `get_pr_diff` from `src/git.rs`

6. **User authentication check** (author/collaborator only for merge actions)

7. **Test full workflow**: create → list → view → merge/close

---

## Notes

- `ServerAction` uses PascalCase: `ServerAction::<MergePullRequest>::new()` if using form-based approach
- DB queries use `sqlx::query!` or `query_as!` for compile-time SQL checking
- All `#[server]` function struct types need `#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]` — NOT unconditional `#[derive(sqlx::FromRow)]` — because the module is always compiled (no cfg gate on `pub mod server`)
- `pub mod server;` in `src/lib.rs` is **not** behind `#[cfg(feature = "ssr")]` — the `#[server]` macro generates client-side stubs, so the module must be visible in hydrate builds
- `get_pr_diff` uses `git diff refs/heads/{base}...refs/heads/{head}` CLI — the three-dot syntax shows changes in head that aren't in base
- Merge operation requires actual git repository manipulation (consider using `git` CLI for merge)
