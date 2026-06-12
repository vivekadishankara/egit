# eGit — Project Brief

A self-hosted Git forge built entirely in Rust, inspired by GitHub. Full-stack Leptos with a real Git backend via gitoxide.

---

## Stack

| Layer | Technology |
|---|---|
| Frontend framework | Leptos 0.8 (SSR + hydration) |
| Build tool | cargo-leptos 0.3.6 |
| Styling | TailwindCSS v3 (CSS variable-based themes, `@layer components`) |
| Database | PostgreSQL (via sqlx, runtime queries) |
| Git backend | gitoxide (`gix` crate — pure Rust) |
| Auth | Username/password (bcrypt hashed, session cookies) |
| Deployment | Docker / docker-compose |
| Git protocol | HTTPS only (smart HTTP) |

---

## V1 Feature Scope

### In scope
- [x] User auth (register, login, logout, session management)
- [x] User profiles (avatar, bio, list of repos)
- [ ] Repository creation, deletion, basic settings
- [x] Repository browser (file tree, file viewer with syntax highlight)
- [x] README rendering (Markdown → HTML)
- [x] Commit history (list, individual commit diff)
- [ ] Pull requests (open, view diff, merge, close)
- [x] Full theme system (CSS variable themes stored in DB, `data-theme` on `<html>`)

### Explicitly out of scope for v1
- Issues / labels / milestones
- CI / Actions pipelines
- SSH push support (HTTPS only)
- Organizations / teams
- Code review inline comments
- Webhooks
- Stars / forks (data model yes, UI no)

---

## Architecture Decisions

### Server functions for all DB access
All database queries go through Leptos server functions (`#[server]`). No direct client-side DB access. This keeps the WASM bundle clean and all secrets server-side.

### Repository storage layout
Bare git repos stored on disk at a configured path:
```
/data/repos/{username}/{reponame}.git
```
Metadata (description, visibility, default branch, etc.) stored in PostgreSQL.

### Git operations via `gix`
Use the `gix` crate (gitoxide) for all read operations: file tree traversal, blob reading, commit log, diff generation. For write operations (init, receive-pack for HTTPS push), use `gix` where stable, fall back to `git2` if needed.

### Default branch resolution
The `default_branch` column in `repositories` stores the initial value at creation time, but the actual branch name is resolved from the git repo at display time via `git::get_default_branch()`. This avoids stale links when a repo's HEAD points to a different branch (e.g. `master` vs `main`). Both the repo overview page and the profile page repo list use this runtime resolution.

### Auth
- Passwords hashed with `bcrypt`
- Sessions stored in PostgreSQL (`sessions` table); cookie name `egit_session`, 30-day expiry, HttpOnly + SameSite=Lax
- Session cookie is set/cleared via `leptos_axum::ResponseOptions::insert_header` inside server functions
- Reading request cookies in server functions: use `leptos_axum::extract::<axum::http::HeaderMap>().await` — **not** `expect_context::<RequestParts>()` (not available in leptos_axum 0.8)
- `ServerAction` generics use the PascalCase struct name emitted by `#[server]`, e.g. `ServerAction::<LoginUser>::new()` — not the snake_case fn name
- Auth-aware navbar uses a `Resource` to call `GetCurrentUser` on load and re-fetches after logout

### Theming
- Six CSS variable-based themes (same pattern as portfolio project)
- Theme preference stored per-user in DB
- Applied as `data-theme="..."` on `<html>` element via SSR
- User can change theme from profile settings

### Tailwind CSS + custom tokens
- All component classes live inside `@layer components {}` in `style/input.css`
- CSS variable–based colors are set as plain CSS (`color: var(--color-accent)`) — **never** via `@apply` with token names like `text-text-primary` or `bg-bg-secondary`; Tailwind doesn't know those exist and will error
- `@apply` is only used for real Tailwind utilities (`flex`, `rounded-lg`, `text-sm`, etc.)
- Alert/overlay tints use `color-mix(in srgb, var(--color-danger) 10%, transparent)` instead of Tailwind opacity modifiers (`bg-danger/10`), which require a registered color
- Rust `class=` attributes use the component class names defined in `@layer components` (e.g. `text-accent`, `text-muted`, `bg-surface`, `bg-surface-secondary`) — not raw token names

### HTTPS Git protocol
Implement Git smart HTTP (`/info/refs`, `/git-upload-pack`, `/git-receive-pack`) as Axum routes alongside Leptos. Authenticate push via HTTP Basic Auth checked against the users table.

---

## Database Schema (planned)

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
    status TEXT NOT NULL DEFAULT 'open', -- open | merged | closed
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

---

## Project Structure (target)

```
egit/
├── Cargo.toml
├── Cargo.lock
├── docker-compose.yml
├── Dockerfile
├── .env.example
├── migrations/
│   ├── 001_users.sql
│   ├── 002_sessions.sql
│   ├── 003_repositories.sql
│   └── 004_pull_requests.sql
├── public/
│   └── favicon.ico
├── style/
│   └── main.css          # Tailwind entry + CSS variable theme definitions
├── src/
│   ├── main.rs           # Axum server setup, cargo-leptos entry
│   ├── lib.rs            # Leptos app root, router
│   ├── auth.rs           # Session middleware, auth helpers
│   ├── db.rs             # PostgreSQL pool setup
│   ├── git.rs            # gitoxide wrappers (repo open, tree, diff, log)
│   ├── components/
│   │   ├── mod.rs
│   │   ├── navbar.rs
│   │   ├── file_tree.rs
│   │   ├── diff_viewer.rs
│   │   ├── markdown.rs
│   │   └── theme_switcher.rs
│   └── pages/
│       ├── mod.rs
│       ├── home.rs           # Landing / dashboard
│       ├── auth/
│       │   ├── mod.rs
│       │   ├── login.rs
│       │   └── register.rs
│       ├── user/
│       │   ├── mod.rs
│       │   └── profile.rs
│       └── repo/
│           ├── mod.rs
│           ├── overview.rs   # README + stats
│           ├── tree.rs       # File browser
│           ├── blob.rs       # File viewer
│           ├── commits.rs    # Commit log
│           ├── commit.rs     # Single commit diff
│           └── pulls/
│               ├── mod.rs
│               ├── list.rs
│               ├── new.rs
│               └── detail.rs
```

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
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "time"] }

# Auth
bcrypt = "0.15"
uuid = { version = "1", features = ["v4", "js"] }

# Markdown
pulldown-cmark = "0.11"

# Syntax highlighting
syntect = "5"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## Environment Variables

```env
DATABASE_URL=postgres://egit:password@localhost:5432/egit
REPO_BASE_PATH=/data/repos
SESSION_SECRET=change-me-in-production
LEPTOS_OUTPUT_NAME=egit
LEPTOS_SITE_ADDR=0.0.0.0:3000
LEPTOS_SITE_ROOT=site
```

---

## Implementation Order (suggested)

1. ✅ **Project scaffold** — Cargo.toml, Leptos app shell, Tailwind, Docker/PostgreSQL setup
2. ✅ **DB + migrations** — sqlx pool, run migrations on startup
3. ✅ **Auth** — register, login, logout, session middleware; auth-aware navbar; `egit_stage_3.zip`
4. ✅ **Theme system** — CSS variables, per-user theme, `data-theme` SSR
5. ✅ **Repo creation** — form, `gix::init_bare`, insert DB row
6. ✅ **HTTPS Git push** — Axum smart HTTP routes, Basic Auth
7. ✅ **Repo browser** — file tree via `gix`, blob viewer, syntax highlight
8. ✅ **README rendering** — detect README.md, render via pulldown-cmark
9. ✅ **Commit log + diff** — commit history page, single commit diff view
10. **Pull requests** — create, list, diff between branches, merge, close
11. ✅ **User profiles** — avatar, bio, repo list
12. **Polish** — themes, responsive layout, empty states, error pages

---

## How to Use This Document

Paste this file (or upload it) at the start of every Claude session working on eGit. Always tell Claude:
- Which step from the implementation order you're on
- Which files are relevant to the current task
- Any decisions made that deviate from this document

At the end of each session, ask Claude: *"Update the PROJECT.md to reflect what was completed and any decisions that changed."*
