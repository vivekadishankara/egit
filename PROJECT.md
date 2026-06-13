# eGit — Project Brief

A self-hosted Git forge built entirely in Rust, inspired by GitHub. Full-stack Leptos with a real Git backend via gitoxide (read) and git CLI (write).

---

## Stack

| Layer | Technology |
|---|---|
| Frontend framework | Leptos 0.8 (SSR + hydration) |
| Build tool | cargo-leptos |
| Styling | TailwindCSS v3 (CSS variable-based themes, `@layer components`) |
| Database | PostgreSQL (via sqlx 0.8, runtime queries) |
| Git read ops | gitoxide (`gix` 0.70) |
| Git write ops | `git` CLI (init, commit-tree, merge-tree, diff, update-ref) |
| Auth | Username/password (bcrypt hashed, session cookies) |
| Deployment | Docker / docker-compose |
| Git protocol | HTTPS only (smart HTTP) |

---

## V1 Feature Scope

### Implemented
- [x] User auth (register, login, logout, session management)
- [x] User profiles (avatar initial-letter fallback, bio, repo list with public/private visibility)
- [x] Repository creation (bare init with empty initial commit), deletion (owner-only, removes on-disk repo + DB row)
- [x] Repository browser (file tree, blob viewer with syntax highlighting via syntect)
- [x] README rendering (multiple filename variants, Markdown → HTML via pulldown-cmark)
- [x] Commit history (list with relative timestamps, single commit detail + diff via `git show`)
- [x] Pull requests (create, list with status filter, detail with body + diff, merge via `git merge-tree --write-tree`, close)
- [x] Full theme system (6 CSS variable themes: dark/light/dracula/nord/solarized/gruvbox, per-user DB, `data-theme` SSR)
- [x] Git smart HTTP protocol (`info/refs` with symref, `git-upload-pack`, `git-receive-pack` with Basic Auth)
- [x] Branch selector on overview/code/commits pages
- [x] PR counts on overview sidebar
- [x] Repo tab bar (Overview / Code / Commits / Pull Requests) with proper auth-aware rendering
- [x] Responsive nav bar with auth-aware links
- [x] Loading states (Suspense with text fallbacks), 404 page, error alerts

### Explicitly out of scope for v1
- Issues / labels / milestones
- CI / Actions pipelines
- SSH push support (HTTPS only)
- Organizations / teams
- Code review inline comments / PR discussions
- Webhooks
- Stars / forks
- Search (code or repo)
- Tags / releases
- Wiki
- Collaborator permissions (only repo owner has delete/merge authority)
- File editing via UI (all content changes go through `git push`)

---

## Architecture Decisions

### Server functions for all DB access
All database queries go through Leptos server functions (`#[server]`). No direct client-side DB access. This keeps the WASM bundle clean and all secrets server-side.

### Repository storage layout
Bare git repos stored on disk at a configured path:
```
{REPO_BASE_PATH}/{username}/{reponame}.git
```
Metadata (description, visibility, default branch, etc.) stored in PostgreSQL.

### Git operations
- **Read operations** (tree traversal, blob reading, commit log): `gix` 0.70
- **Write operations** (init bare, commit-tree, merge-tree, update-ref, diff, show): `git` CLI via `std::process::Command` — all directly on the bare repo, no worktree needed

### Default branch resolution
The `default_branch` column in `repositories` stores the initial value at creation time, but the actual branch name is resolved from the git repo at display time via `git::get_default_branch()`. This avoids stale links when a repo's HEAD points to a different branch (e.g. `master` vs `main`). The overview page, profile repo list, and branch selector all use runtime resolution.

### Auth
- Passwords hashed with `bcrypt`
- Sessions stored in PostgreSQL (`sessions` table); cookie name `egit_session`, 30-day expiry, HttpOnly + SameSite=Lax
- Session cookie is set/cleared via `leptos_axum::ResponseOptions::insert_header` inside server functions
- Reading request cookies in server functions: use `leptos_axum::extract::<axum::http::HeaderMap>().await` — **not** `expect_context::<RequestParts>()` (not available in leptos_axum 0.8)
- `ServerAction` generics use the PascalCase struct name emitted by `#[server]`, e.g. `ServerAction::<LoginUser>::new()` — not the snake_case fn name
- Auth-aware navbar uses a `Resource` to call `GetCurrentUser` on load and re-fetches after logout via `Effect`
- Git push authentication: HTTP Basic Auth against users table (checked in `git_routes.rs`)

### Theming
- Six CSS variable-based themes in `style/input.css`: dark, light, dracula, nord, solarized, gruvbox
- Theme preference stored per-user in DB
- Applied as `data-theme="..."` on `<html>` element via SSR
- Theme middleware in `main.rs` resolves from session before SSR shell renders
- User can change theme from profile settings (ThemeSwitcher component with optimistic DOM update + auto-submit)

### Tailwind CSS + custom tokens
- All component classes live inside `@layer components {}` in `style/input.css`
- CSS variable–based colors are set as plain CSS (`color: var(--color-accent)`) — **never** via `@apply` with token names like `text-text-primary` or `bg-bg-secondary`; Tailwind doesn't know those exist and will error
- `@apply` is only used for real Tailwind utilities (`flex`, `rounded-lg`, `text-sm`, etc.)
- Alert/overlay tints use `color-mix(in srgb, var(--color-danger) 10%, transparent)` instead of Tailwind opacity modifiers (`bg-danger/10`), which require a registered color
- Rust `class=` attributes use the component class names defined in `@layer components` (e.g. `text-accent`, `text-muted`, `bg-surface`, `bg-surface-secondary`) — not raw token names

### HTTPS Git protocol
Git smart HTTP (`/info/refs`, `/git-upload-pack`, `/git-receive-pack`) implemented as direct Axum routes in `src/git_routes.rs`, handled alongside Leptos SSR routes. Push authenticated via HTTP Basic Auth against the users table. Pull/clone is unauthenticated.

### Pull request merge strategy
PR merge uses `git merge-tree --write-tree` (computes merge result in bare repo) + `git commit-tree` (creates merge commit with two parents) + `git update-ref` (updates base branch). All operations on the bare repo, no worktree checkout needed.

### Repo page tab bar
Tab bar (Overview / Code / Commits / Pulls) appears on overview, commits, and pulls pages. TreePage and BlobPage are **missing** the tab bar — they show breadcrumbs only. The "Code" tab link goes to `/tree/{default_branch}`. Overview also only shows "Code" and "Commits" tabs when `has_commits` is true. The Pull Requests tab is always visible and always an `<a>` link (clickable even when active), so you can navigate back to the PR list from the PR detail page.

---

## Database Schema

```sql
-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    bio TEXT,
    avatar_url TEXT,
    theme TEXT NOT NULL DEFAULT 'dark',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Sessions
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Repositories
CREATE TABLE repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    is_private BOOLEAN NOT NULL DEFAULT false,
    default_branch TEXT NOT NULL DEFAULT 'main',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(owner_id, name)
);

-- Pull Requests
CREATE TABLE pull_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    body TEXT,
    head_branch TEXT NOT NULL,
    base_branch TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',  -- open | merged | closed
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

---

## Project Structure

```
egit/
├── Cargo.toml
├── Cargo.lock
├── docker-compose.yml
├── package.json                # TailwindCSS CLI npm dependency
├── tailwind.config.js
├── .env.example
├── .env
├── AGENTS.md
├── migrations/
│   ├── 001_users.sql
│   ├── 002_sessions.sql
│   ├── 003_repositories.sql
│   └── 004_pull_requests.sql
├── public/
│   └── (favicon.ico — referenced but absent)
├── style/
│   └── input.css               # Tailwind entry + @layer components + CSS variable themes
├── src/
│   ├── main.rs                 # Axum server setup, middleware, shell closure
│   ├── lib.rs                  # Crate root: feature-gated module exports
│   ├── app.rs                  # Leptos Router with all client routes
│   ├── auth.rs                 # Session cookie management, DB queries
│   ├── db.rs                   # PostgreSQL pool creation + migrations
│   ├── error.rs                # EgitError enum (defined but unused)
│   ├── git.rs                  # gix read wrappers + git CLI helpers
│   ├── git_routes.rs           # Git smart HTTP handlers (info/refs, upload-pack, receive-pack)
│   ├── components/
│   │   ├── mod.rs
│   │   ├── navbar.rs
│   │   ├── file_tree.rs
│   │   ├── diff_viewer.rs
│   │   ├── markdown.rs
│   │   ├── theme_switcher.rs
│   │   ├── repo_tab_bar.rs     # Tab bar + BranchSelector + url_encode_branch
│   │   ├── repo_header.rs      # RepoHeader with visibility badge + delete button
│   │   └── delete_repo_button.rs
│   ├── server/
│   │   ├── mod.rs
│   │   ├── repos.rs            # delete_repo server fn
│   │   └── prs.rs              # pull request server fns (create, list, get, merge, close, diff, counts)
│   └── pages/
│       ├── mod.rs
│       ├── home.rs             # Landing page
│       ├── auth/
│       │   ├── mod.rs          # Server fns: register, login, logout, get_current_user, set_theme
│       │   ├── login.rs
│       │   └── register.rs
│       ├── user/
│       │   ├── mod.rs
│       │   └── profile.rs
│       └── repo/
│           ├── mod.rs
│           ├── overview.rs     # README + stats + PR count
│           ├── tree.rs         # File browser
│           ├── blob.rs         # File viewer (syntect highlighting)
│           ├── commits.rs      # Commit log
│           ├── commit.rs       # Single commit detail + diff
│           ├── create.rs       # Create repo form
│           └── pulls/
│               ├── mod.rs
│               ├── list.rs
│               ├── new.rs
│               └── detail.rs
```

---

## Routes

### Axum (direct, non-Leptos)
| Method | Path | Handler | Auth |
|--------|------|---------|------|
| GET | `/{username}/{reponame}/info/refs` | `handle_info_refs` | Basic auth for receive-pack only |
| POST | `/{username}/{reponame}/git-upload-pack` | `handle_upload_pack` | None (read) |
| POST | `/{username}/{reponame}/git-receive-pack` | `handle_receive_pack` | Basic auth required |

### Leptos SSR (client-side router)
| Path | Component | Purpose |
|------|-----------|---------|
| `/` | `HomePage` | Landing page |
| `/login` | `LoginPage` | Sign-in form |
| `/register` | `RegisterPage` | Registration form |
| `/repos/new` | `CreateRepoPage` | New repo form |
| `/:username` | `ProfilePage` | User profile + repo list |
| `/:username/:reponame` | `RepoOverviewPage` | Repo overview with README |
| `/:username/:reponame/tree/:branch` | `TreePage` | Directory listing |
| `/:username/:reponame/tree/:branch/*path` | `TreePage` | Subdirectory listing |
| `/:username/:reponame/blob/:branch/*path` | `BlobPage` | File content view |
| `/:username/:reponame/commits` | `CommitsPage` | Commit log (default branch) |
| `/:username/:reponame/commits/:branch` | `CommitsPage` | Commit log (specific branch) |
| `/:username/:reponame/commit/:id` | `CommitPage` | Single commit detail + diff |
| `/:username/:reponame/pulls` | `PullListPage` | PR list (filterable by status) |
| `/:username/:reponame/pulls/new` | `NewPullPage` | New PR form |
| `/:username/:reponame/pulls/:pr_id` | `PullDetailPage` | PR detail + diff + merge/close |

---

## Key Crates (Cargo.toml additions beyond portfolio)

```toml
# Git
gix = { version = "0.70", default-features = false, features = [
    "basic", "extras", "blob-diff",
    "blocking-network-client",
    "blocking-http-transport-reqwest-rust-tls",
] }

# Database
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "time", "migrate"] }

# Auth
bcrypt = "0.15"
uuid = { version = "1", features = ["v4", "serde", "js"] }

# Markdown
pulldown-cmark = "0.11"

# Syntax highlighting
syntect = "5"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# HTTP / server
axum = { version = "0.8", features = ["macros"] }
axum-extra = { version = "0.10", features = ["cookie", "typed-header"] }
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = ["fs", "compression-gzip"] }

# Other
base64 = "0.22"
bytes = "1"
time = { version = "0.3", features = ["serde"] }
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
```

---

## Environment Variables

```env
DATABASE_URL=postgres://egit:password@localhost:5432/egit
REPO_BASE_PATH=/data/repos
SESSION_SECRET=change-me-in-production    # NOTE: defined but not actually used in code
LEPTOS_OUTPUT_NAME=egit
LEPTOS_SITE_ADDR=0.0.0.0:3000
LEPTOS_SITE_ROOT=site
```

---

## Implementation Order

1. ✅ **Project scaffold** — Cargo.toml, Leptos app shell, Tailwind, Docker/PostgreSQL setup
2. ✅ **DB + migrations** — sqlx pool, run migrations on startup
3. ✅ **Auth** — register, login, logout, session middleware; auth-aware navbar
4. ✅ **Theme system** — CSS variables, per-user theme, `data-theme` SSR
5. ✅ **Repo creation** — form, `git init_bare`, insert DB row
6. ✅ **HTTPS Git push** — Axum smart HTTP routes, Basic Auth
7. ✅ **Repo browser** — file tree via `gix`, blob viewer, syntax highlight
8. ✅ **README rendering** — detect README, render via pulldown-cmark
9. ✅ **Commit log + diff** — commit history page, single commit diff view
10. ✅ **Pull requests** — create, list, diff, merge (bare-repo merge-tree + commit-tree + update-ref), close, auth-gated
11. ✅ **User profiles** — avatar initial fallback, bio, public/private repo list
12. ✅ **Polish** — responsive nav, branch selector, tab bar, loading/error/404 states, error alerts

---

## How to Use This Document

Paste/upload this file at the start of every Claude session working on eGit. Always tell Claude:
- Which feature or area you're working on
- Which files are relevant to the current task
- Any decisions made that deviate from this document

At the end of each session, ask Claude: *"Update the PROJECT.md to reflect what was completed and any decisions that changed."*

For coding, refer to `AGENTS.md` in the repo root for precise conventions (cookie extraction pattern, `ServerAction` naming, styling rules, etc.).
