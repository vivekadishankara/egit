# eGit — Agent Instructions

## Quick start

```bash
# Copy env and fill in secrets
cp .env.example .env
# Start DB (PostgreSQL)
docker compose up -d db
# Run in dev mode (auto-reloads on code/tailwind changes)
cargo leptos watch
```

Compile takes ~30s cold, incr ~3s. WASM cross-compilation is automatic.

## Build/test/lint

The cargo-leptos CLI orchestrates the whole build (Rust SSR, WASM hydration, Tailwind). There is no separate tsc/lint/test runner. To do a release build:

```bash
npm install    # installs tailwindcss
cargo leptos build --release
```

There are no tests, no Clippy in CI, no typecheck step.

## Architecture

- **Leptos 0.8** SSR+hydrate. Single crate, two feature flags: `ssr` (server binary) and `hydrate` (WASM).
- App shell in `src/app.rs`, entrypoints: `src/main.rs` (server), WASM entry via `lib.rs` + cargo-leptos.
- All DB access through `#[server]` functions (WASM calls them over fetch). DB is PostgreSQL via sqlx.
- Axum routes in `src/main.rs`; Git smart HTTP in `src/git_routes.rs`.
- `style/input.css` — Tailwind entry plus all `@layer components` classes.

## Key conventions

### Styling
- All component classes live in `@layer components {}` in `style/input.css`.
- CSS variable colors: use `color: var(--color-*)` directly. NEVER in `@apply` — Tailwind doesn't know about them and will error.
- `@apply` only for real Tailwind utilities (`flex`, `rounded-lg`, `text-sm`, etc.).
- Rust `class=` attributes use component class names (e.g. `text-accent`, `bg-surface`, `border-theme`).
- Alert tints use `color-mix()` instead of Tailwind opacity modifiers.

### Auth patterns
- Cookie: `egit_session`, HttpOnly + SameSite=Lax, stored in PostgreSQL `sessions` table.
- Reading cookies in server functions: `leptos_axum::extract::<axum::http::HeaderMap>().await` — NOT `expect_context::<RequestParts>()`.
- `ServerAction` uses the PascalCase struct name from `#[server]`: `ServerAction::<LoginUser>::new()`, not the fn name.
- Logout re-fetches user via `Resource::refetch()` inside an `Effect`.

### Git
- Bare repos at `{REPO_BASE_PATH}/{username}/{reponame}.git`.
- `gix` for read ops (tree, blob, log, commit detail); `git` CLI for init/commit-tree/show/diff/merge.
- `init_bare` creates an empty initial commit and sets HEAD to `refs/heads/main`.
- Default branch resolved at display time via `get_default_branch()`.
- `get_pr_diff` uses `git diff refs/heads/{base}...refs/heads/{head}` (three-dot syntax).
- PR merge uses `git merge-tree --write-tree` + `git commit-tree` (with two parents) + `git update-ref` — all on the bare repo, no worktree needed.

### Themes
- Six themes defined in `style/input.css`: dark, light, dracula, nord, solarized, gruvbox.
- Set per-user in DB, applied SSR via `data-theme` on `<html>`.
- Theme middleware in `main.rs` resolves from session before SSR shell renders.

### Repo page tab bar
- Tab bar (Overview / Code / Commits / Pulls) appears on overview, commits, and pulls pages.
- TreePage and BlobPage are **missing** the tab bar — they show breadcrumbs only.
- The "Code" tab link goes to `/tree/{default_branch}`. Overview also only shows "Code" and "Commits" tabs when `has_commits` is true.
- The Pull Requests tab is always visible and always an `<a>` link (clickable even when active), so you can navigate back to the PR list from the PR detail page.

### Pull requests
- Server functions in `src/server/prs.rs`: create, list, get detail, merge, close, get branch list, get PR diff, get counts.
- `create_pull_request` rejects duplicate PRs for the same head+base (any status: open/merged/closed).
- `merge_pull_request` and `close_pull_request` verify the caller is the PR author via session auth.
- Redirect after PR creation uses `leptos_axum::redirect` inside the server function (same pattern as login/register).
- Pages: list (`/pulls`), new (`/pulls/new`), detail (`/pulls/:pr_id`).

## Server functions

Every `#[server]` fn:
- Routes under `/api` prefix
- Uses `expect_context::<PgPool>()` and `expect_context::<String>()` (repo base path) — provided in `main.rs::shell` closure
- Error type: `ServerFnError::new(...)`

## Dependencies

- `gix` 0.70 (gitoxide) — read-only git ops
- `sqlx` 0.8 — PostgreSQL, runtime queries
- `pulldown-cmark` 0.11 — README rendering
- `syntect` 5 — code syntax highlighting
- `bcrypt` 0.15 — password hashing
- `leptos_axum` 0.8 — SSR integration

## Deployment

- Docker multi-stage build: `cargo leptos build --release` in builder, then copy binary + `target/site`.
- Production expects: `DATABASE_URL`, `REPO_BASE_PATH`, `SESSION_SECRET`.
