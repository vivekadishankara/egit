# Pull Requests Implementation Plan

## Current State

### Existing Infrastructure
- **Database**: `migrations/004_pull_requests.sql` creates `pull_requests` table with columns: id, repo_id, author_id, title, body, head_branch, base_branch, status, created_at, updated_at
- **Pages** (stub implementations):
  - `src/pages/repo/pulls/list.rs` — PullListPage
  - `src/pages/repo/pulls/new.rs` — NewPullPage  
  - `src/pages/repo/pulls/detail.rs` — PullDetailPage
- **Tabs**: RepoTabBar in `src/components/repo_tab_bar.rs` currently shows only [Overview, Code, Commits]

### Project Architecture (from AGENTS.md)
- **Leptos 0.8** SSR+hydrate with `#[server]` functions for all DB access
- **Database**: PostgreSQL via sqlx (runtime queries)
- **Git**: `gix` 0.70 for read ops; `git` CLI for init/commit-tree/show
- **Auth**: Cookies via `leptos_axum::extract::<HeaderMap>().await`
- **Styling**: Tailwind `@layer components` with CSS variables

---

## Implementation Steps

### Step 1: Add Server Functions (`src/lib.rs` + `src/lib.rs` or new file)

```rust
// Add to src/lib.rs under #[cfg(feature = "ssr")]
pub mod server;
```

Create `src/server/prs.rs` with:

```rust
use leptos::prelude::*;
use sqlx::{PgPool, PgRow};
use crate::git;

#[server]
pub async fn create_pull_request(
    repo_id: i64,
    title: String,
    body: Option<String>,
    head_branch: String,
    base_branch: String,
) -> Result<i64, ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    sqlx::query_scalar!(
        "INSERT INTO pull_requests (repo_id, title, body, head_branch, base_branch, status) VALUES ($1, $2, $3, $4, $5, 'open') RETURNING id",
        repo_id, title, body, head_branch, base_branch
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB error: {e}")))
}

#[server]
pub async fn list_pull_requests(repo_id: i64, status: Option<String>) -> Result<Vec<PullRequest>, ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    let status_filter = status.unwrap_or_else(|| "open".to_string());
    
    let prs = sqlx::query_as!(
        PullRequest,
        r#"
        SELECT 
            pr.id,
            pr.repo_id,
            pr.author_id,
            u.username as author_name,
            pr.title,
            pr.body,
            pr.head_branch,
            pr.base_branch,
            pr.status,
            pr.created_at,
            pr.updated_at
        FROM pull_requests pr
        JOIN users u ON u.id = pr.author_id
        WHERE pr.repo_id = $1 AND pr.status = $2
        ORDER BY pr.created_at DESC
        "#,
        repo_id,
        status_filter
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB error: {e}")))?;
    
    Ok(prs)
}

#[server]
pub async fn get_pull_request(pr_id: i64) -> Result<PullRequestDetail, ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    let pr = sqlx::query_as!(
        PullRequestDetail,
        r#"
        SELECT 
            pr.id,
            pr.repo_id,
            r.name as repo_name,
            u.username as author_name,
            pr.title,
            pr.body,
            pr.head_branch,
            pr.base_branch,
            pr.status,
            pr.created_at,
            pr.updated_at,
            pr.merged_at,
            pr.merged_by_id
        FROM pull_requests pr
        JOIN users u ON u.id = pr.author_id
        JOIN repositories r ON r.id = pr.repo_id
        WHERE pr.id = $1
        "#,
        pr_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB error: {e}")))?;
    
    Ok(pr)
}

#[server]
pub async fn merge_pull_request(pr_id: i64, user_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    // First get PR to verify it exists and is open
    let pr = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM pull_requests WHERE id = $1 AND status = 'open'"
    )
    .bind(pr_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB error: {e}")))?
    .ok_or_else(|| ServerFnError::new("Pull request not found or already merged/closed"))?;
    
    // Update PR status
    sqlx::query!(
        "UPDATE pull_requests SET status = 'merged', merged_at = NOW(), merged_by_id = $2 WHERE id = $1",
        pr_id, user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB error: {e}")))?;
    
    // TODO: Actually merge the branches in git repo
    // This requires getting repo path from repo_id, then using gix or git CLI
    
    Ok(())
}

#[server]
pub async fn close_pull_request(pr_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    sqlx::query!(
        "UPDATE pull_requests SET status = 'closed', updated_at = NOW() WHERE id = $1",
        pr_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB error: {e}")))?;
    
    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct PullRequest {
    pub id: i64,
    pub repo_id: i64,
    pub author_id: i64,
    pub author_name: String,
    pub title: String,
    pub body: Option<String>,
    pub head_branch: String,
    pub base_branch: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(sqlx::FromRow)]
pub struct PullRequestDetail {
    pub id: i64,
    pub repo_id: i64,
    pub repo_name: String,
    pub author_id: i64,
    pub author_name: String,
    pub title: String,
    pub body: Option<String>,
    pub head_branch: String,
    pub base_branch: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub merged_at: Option<chrono::NaiveDateTime>,
    pub merged_by_id: Option<i64>,
}
```

### Step 2: Update `src/lib.rs` to add PR module

```rust
// In src/lib.rs, add under #[cfg(feature = "ssr")]
#[cfg(feature = "ssr")]
pub mod server;
#[cfg(feature = "ssr")]
pub mod server::prs;
```

### Step 3: Add Git Operations (if not already present)

Check `src/git.rs` for diff/merge functions. If missing, add:

```rust
// In src/git.rs
pub async fn get_pr_diff(
    repo_base: &str,
    username: &str,
    reponame: &str,
    head_branch: &str,
    base_branch: &str,
) -> Result<String, anyhow::Error> {
    let repo_path = format!("{}/{}/{}.git", repo_base, username, reponame);
    let repo = gix::open(&repo_path)?;
    
    // Get commit range diff
    let diff = repo.diff(
        gix::dict::CMFHashes::default(),
        gix::worktree::state::Delta::default(),
        &gix::commit::Id::from_str(&head_branch)?.into(),
        &gix::commit::Id::from_str(&base_branch)?.into(),
        &Default::default(),
    )?;
    
    Ok(diff.to_string())
}
```

### Step 4: Update `src/lib.rs` Routes

```rust
// In src/app.rs or lib.rs router, add:
<Route path="/:username/:reponame/pulls" view={PullListPage} />
<Route path="/:username/:reponame/pulls/new" view={NewPullPage} />
<Route path="/:username/:reponame/pulls/:pr_id" view={PullDetailPage} />
```

### Step 5: Update Repo Overview Page

**File**: `src/pages/repo/overview.rs`

#### Add "Pull Requests" Tab to RepoTabBar

Add PR tabs prop:

```rust
view! {
    <RepoTabBar
        active="overview"
        owner={owner.clone()}
        name={name.clone()}
        default_branch={default_branch.clone()}
        has_commits={has_commits}
        current_branch={branch().unwrap_or_default()}
        // Add this:
        has_pull_requests={true}  // or check if PRs exist
    />
}
```

#### Add Sidebar to Overview

```rust
// After RepoTabBar, add:
view! {
    <div class="flex gap-6">
        <div class="w-64 shrink-0">
            <div class="card mb-6">
                <div class="px-4 py-3 border-b border-theme">
                    <h2 class="font-medium text-muted text-sm">"Pull Requests"</h2>
                </div>
                <div class="p-2">
                    <a href=format!("/{owner}/{name}/pulls/new") class="flex items-center gap-2 px-3 py-2 text-sm text-text hover:bg-surface-secondary rounded transition-colors">
                        <span>"+"</span>
                        <span>"New pull request"</span>
                    </a>
                    <a href=format!("/{owner}/{name}/pulls?status=open") class="flex items-center justify-between px-3 py-2 text-sm text-text hover:bg-surface-secondary rounded transition-colors">
                        <span>"Open"</span>
                        <span class="text-xs text-muted">"0"</span>
                    </a>
                    <a href=format!("/{owner}/{name}/pulls?status=merged") class="flex items-center justify-between px-3 py-2 text-sm text-text hover:bg-surface-secondary rounded transition-colors">
                        <span>"Merged"</span>
                        <span class="text-xs text-muted">"0"</span>
                    </a>
                    <a href=format!("/{owner}/{name}/pulls?status=closed") class="flex items-center justify-between px-3 py-2 text-sm text-text hover:bg-surface-secondary rounded transition-colors">
                        <span>"Closed"</span>
                        <span class="text-xs text-muted">"0"</span>
                    </a>
                </div>
            </div>
        </div>
        
        <div class="flex-1">
            {/* existing overview content */}
        </div>
    </div>
}
```

### Step 6: Update RepoTabBar Component

**File**: `src/components/repo_tab_bar.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTabBarProps {
    active: String,
    owner: String,
    name: String,
    default_branch: String,
    has_commits: bool,
    current_branch: String,
    has_pull_requests: bool,  // ADD THIS
}

#[component]
pub fn RepoTabBar(
    active: &'static str,
    owner: String,
    name: String,
    default_branch: String,
    has_commits: bool,
    current_branch: String,
    has_pull_requests: bool,  // ADD THIS
) -> impl IntoView {
    let branch = if current_branch.is_empty() {
        default_branch.clone()
    } else {
        current_branch.clone()
    };
    let encoded_branch = url_encode_branch(&branch);

    let tab_class = |tab: &str| {
        if tab == active {
            "px-4 py-2 text-sm font-medium border-b-2 border-accent text-accent"
        } else {
            "px-4 py-2 text-sm text-muted no-underline border-b-2 border-transparent hover:text-text hover:border-text"
        }
    };

    view! {
        <div class="flex gap-1 items-baseline border-b border-theme mb-6">
            {/* ... existing overview tab ... */}
            
            {/* ... existing code/commits tabs ... */}
            
            {/* ADD pull requests tab */}
            {if has_pull_requests {
                if active == "pulls" {
                    view! {
                        <span class={tab_class("pulls")}>
                            "Pull requests"
                        </span>
                    }.into_any()
                } else {
                    view! {
                        <a href=format!("/{owner}/{name}/pulls") class={tab_class("pulls")}>
                            "Pull requests"
                        </a>
                    }.into_any()
                }
            } else {
                view! {}.into_any()
            }}
        </div>
    }
}
```

### Step 7: Implement Pull Requests List Page

**File**: `src/pages/repo/pulls/list.rs`

```rust
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use crate::server::prs::list_pull_requests;

#[derive(Debug, Clone)]
struct PRCardProps {
    pr: PullRequest,
    owner: String,
    reponame: String,
}

#[component]
fn PRCard(pr: PullRequest, owner: String, reponame: String) -> impl IntoView {
    let status_class = match pr.status.as_str() {
        "open" => "px-2 py-0.5 text-xs rounded-full bg-success/10 text-success border border-success/20",
        "merged" => "px-2 py-0.5 text-xs rounded-full bg-merged/10 text-merged border border-merged/20",
        "closed" => "px-2 py-0.5 text-xs rounded-full bg-danger/10 text-danger border border-danger/20",
        _ => "px-2 py-0.5 text-xs rounded-full bg-muted/10 text-muted border border-muted/20",
    };

    let badge_text = match pr.status.as_str() {
        "open" => "Open",
        "merged" => "Merged",
        "closed" => "Closed",
        _ => "Unknown",
    };

    view! {
        <a href=format!("/{}/{}/pulls/{}", owner, reponame, pr.id) class="block p-4 mb-3 rounded-lg border border-theme bg-surface hover:bg-surface-secondary transition-colors">
            <div class="flex items-start justify-between mb-2">
                <span class="font-medium text-text text-base line-clamp-1">{pr.title}</span>
                <span class={status_class}>{badge_text}</span>
            </div>
            <div class="text-sm text-muted flex items-center gap-3">
                <span>by <span class="text-accent">{pr.author_name}</span></span>
                <span class="text-xs">•</span>
                <span>{pr.created_at.format("%Y-%m-%d")}</span>
            </div>
            <div class="text-xs text-muted mt-2 flex items-center gap-4">
                <span class="flex items-center gap-1">
                    <span class="text-accent">{pr.head_branch}</span>
                    <span>→</span>
                    <span>{pr.base_branch}</span>
                </span>
            </div>
        </a>
    }
}

#[component]
pub fn PullListPage() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let query = use_query_map();
    
    let owner = move || {
        params.get().get("username").map(|s| s.to_string()).unwrap_or_default()
    };
    let reponame = move || {
        params.get().get("reponame").map(|s| s.to_string()).unwrap_or_default()
    };
    
    let repo_resource = Resource::new(
        move || (owner(), reponame()),
        |(o, r)| async move {
            crate::server::prs::get_repo_id_for_pr(o, r).await  // Need to add this function
        },
    );
    
    let status = move || query.get().get("status").map(|s| s.to_string()).unwrap_or_else(|| "open".to_string());
    
    let prs_resource = Resource::new(
        move || (repo_resource.get().and_then(|r| r.ok()).map(|repo| repo.id), status()),
        |(repo_id, status)| async move {
            if let Some(id) = repo_id {
                list_pull_requests(id, Some(status)).await
            } else {
                Err(ServerFnError::new("Repo not found"))
            }
        },
    );

    view! {
        <div class="container">
            <Suspense fallback=|| view! { <p class="text-muted">"Loading..."</p> }>
                {move || {
                    prs_resource.get().map(|result| match result {
                        Ok(prs) => {
                            view! {
                                <>
                                    <h1 class="page-title">"Pull requests"</h1>
                                    <div class="flex gap-1 mb-6">
                                        <a href="?status=open" class={pr_tab_class("open", &status())}>Open</a>
                                        <a href="?status=merged" class={pr_tab_class("merged", &status())}>Merged</a>
                                        <a href="?status=closed" class={pr_tab_class("closed", &status())}>Closed</a>
                                    </div>
                                    <div class="pr-list">
                                        {prs.into_iter().map(|pr| {
                                            let owner = owner();
                                            let reponame = reponame();
                                            view! { <PRCard pr reponame owner /> }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </>
                            }.into_any()
                        }
                        Err(e) => view! { <div class="alert-error">{e.to_string()}</div> }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

fn pr_tab_class(active: &str, current: &str) -> &'static str {
    if active == current {
        "px-4 py-2 text-sm font-medium border-b-2 border-accent text-accent"
    } else {
        "px-4 py-2 text-sm text-muted no-underline border-b-2 border-transparent hover:text-text hover:border-text"
    }
}
```

### Step 8: Update Server Module to Add Helper Functions

```rust
// In src/server/prs.rs, add helper to get repo_id

#[server(GetRepoIdForPR, "/api")]
pub async fn get_repo_id_for_pr(username: String, reponame: String) -> Result<i64, ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    let repo_id = sqlx::query_scalar!(
        "SELECT r.id FROM repositories r JOIN users u ON u.id = r.owner_id WHERE r.name = $1 AND u.username = $2",
        reponame, username
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB error: {e}")))?;
    
    Ok(repo_id)
}
```

### Step 9: Implement New Pull Request Page

**File**: `src/pages/repo/pulls/new.rs`

```rust
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::server::prs::{create_pull_request, get_branch_list};

#[component]
pub fn NewPullPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.get().get("username").map(|s| s.to_string()).unwrap_or_default();
    let reponame = move || params.get().get("reponame").map(|s| s.to_string()).unwrap_or_default();
    
    let branches_resource = Resource::new(
        move || (owner(), reponame()),
        |(o, r)| async move { get_branch_list(o, r).await },
    );
    
    let (title, set_title) = create_signal(String::new());
    let (body, set_body) = create_signal(String::new());
    let (head_branch, set_head_branch) = create_signal(String::new());
    let (base_branch, set_base_branch) = create_signal(String::new());
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (success, set_success) = create_signal::<Option<String>>(None);
    
    let submit = move |ev: leptos::prelude::FormDataEvent| {
        ev.prevent_default();
        
        // Get repo_id first
        let repo_id_future = crate::server::prs::get_repo_id_for_pr(owner(), reponame())
            .and_then(move |repo_id| {
                create_pull_request(
                    repo_id,
                    title.get_untracked(),
                    Some(body.get_untracked()),
                    head_branch.get_untracked(),
                    base_branch.get_untracked(),
                )
            });
        
        spawn_local(async move {
            match repo_id_future.await {
                Ok(pr_id) => {
                    set_success.set(Some(format!("Pull request #{pr_id} created successfully!")));
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                    set_success.set(None);
                }
            }
        });
    };

    view! {
        <div class="container">
            <h1 class="page-title">"Create pull request"</h1>
            
            <div class="card max-w-2xl mt-4">
                <form on:submit=submit class="p-4">
                    <div class="mb-4">
                        <label class="block text-sm font-medium text-text mb-2">"Title"</label>
                        <input
                            type="text"
                            bind:value=title
                            class="w-full px-3 py-2 rounded-md border border-theme bg-surface text-text focus:outline-none focus:border-accent"
                            placeholder="Summary of changes"
                        />
                    </div>
                    
                    <div class="mb-4">
                        <label class="block text-sm font-medium text-text mb-2">"Description"</label>
                        <textarea
                            bind:value=body
                            class="w-full px-3 py-2 rounded-md border border-theme bg-surface text-text focus:outline-none focus:border-accent min-h-[100px]"
                            placeholder="Describe your changes in detail..."
                        />
                    </div>
                    
                    <div class="mb-4 grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-text mb-2">"Compare"</label>
                            <Suspense fallback=|| view! { <span class="text-sm text-muted">"Loading branches..."</span> }>
                                {move || {
                                    branches_resource.get().map(|result| match result {
                                        Ok(branches) => {
                                            view! {
                                                <select
                                                    bind:value=head_branch
                                                    class="w-full px-3 py-2 rounded-md border border-theme bg-surface text-text focus:outline-none focus:border-accent"
                                                >
                                                    {branches.iter().map(|b| {
                                                        view! {
                                                            <option value=b>{b}</option>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </select>
                                            }.into_any()
                                        }
                                        Err(e) => {
                                            view! { <span class="text-danger">"Error loading branches"</span> }.into_any()
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                        
                        <div>
                            <label class="block text-sm font-medium text-text mb-2">"into"</label>
                            <Suspense fallback=|| view! { <span class="text-sm text-muted">"Loading branches..."</span> }>
                                {move || {
                                    branches_resource.get().map(|result| match result {
                                        Ok(branches) => {
                                            let default = branches.first().map(|b| b.clone()).unwrap_or_default();
                                            view! {
                                                <select
                                                    bind:value=base_branch
                                                    class="w-full px-3 py-2 rounded-md border border-theme bg-surface text-text focus:outline-none focus:border-accent"
                                                >
                                                    {branches.iter().map(|b| {
                                                        view! {
                                                            <option value=b>{b}</option>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </select>
                                            }.into_any()
                                        }
                                        Err(e) => {
                                            view! { <span class="text-danger">"Error loading branches"</span> }.into_any()
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                    </div>
                    
                    {error.get().map(|e| {
                        view! { <div class="alert-error mb-4">{e}</div> }
                    })}
                    
                    {success.get().map(|s| {
                        view! { <div class="alert-success mb-4">{s}</div> }
                    })}
                    
                    <div class="flex gap-2 mt-4">
                        <button type="submit" class="btn-primary">
                            "Create pull request"
                        </button>
                        <a href=format!("/{}", owner()) class="btn-secondary">
                            "Cancel"
                        </a>
                    </div>
                </form>
            </div>
        </div>
    }
}
```

### Step 10: Implement Pull Request Details Page

**File**: `src/pages/repo/pulls/detail.rs`

```rust
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::server::prs::{get_pull_request, merge_pull_request, close_pull_request};

#[component]
pub fn PullDetailPage() -> impl IntoView {
    let params = use_params_map();
    let pr_id = move || {
        params.get().get("pr_id").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0)
    };
    
    let pr_resource = Resource::new(
        move || pr_id(),
        |id| async move { get_pull_request(id).await },
    );
    
    let (merge_error, set_merge_error) = create_signal::<Option<String>>(None);
    let (close_error, set_close_error) = create_signal::<Option<String>>(None);
    
    let handle_merge = move |_| {
        let pr_id = pr_id();
        spawn_local(async move {
            match merge_pull_request(pr_id, /* user_id */ 1).await {  // TODO: get actual user_id
                Ok(_) => {
                    set_merge_error.set(None);
                    // Show success toast
                }
                Err(e) => {
                    set_merge_error.set(Some(e.to_string()));
                }
            }
        });
    };
    
    let handle_close = move |_| {
        let pr_id = pr_id();
        spawn_local(async move {
            match close_pull_request(pr_id).await {
                Ok(_) => {
                    set_close_error.set(None);
                    // Show success toast
                }
                Err(e) => {
                    set_close_error.set(Some(e.to_string()));
                }
            }
        });
    };

    view! {
        <div class="container">
            <Suspense fallback=|| view! { <p class="text-muted">"Loading..."</p> }>
                {move || {
                    pr_resource.get().map(|result| match result {
                        Ok(pr) => {
                            let is_open = pr.status == "open";
                            let is_merged = pr.status == "merged";
                            let is_closed = pr.status == "closed";
                            
                            let merge_button = if is_open {
                                view! {
                                    <button on:click=handle_merge class="btn-primary ml-2">
                                        "Merge pull request"
                                    </button>
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            };
                            
                            let close_button = if is_open {
                                view! {
                                    <button on:click=handle_close class="btn-danger ml-2">
                                        "Close pull request"
                                    </button>
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            };
                            
                            view! {
                                <div>
                                    <h1 class="page-title">{pr.title}</h1>
                                    
                                    <div class="card flex items-center gap-2 mb-6">
                                        <span class={format!("px-2 py-0.5 text-xs rounded-full {}", status_class(&pr.status))}>
                                            {pr.status}
                                        </span>
                                        {merge_button}
                                        {close_button}
                                    </div>
                                    
                                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                                        <div class="lg:col-span-2">
                                            <div class="card p-4 mb-6">
                                                <h3 class="font-medium text-text mb-3">"Description"</h3>
                                                <div class="prose prose-invert max-w-none">
                                                    {pr.body.unwrap_or_default()}
                                                </div>
                                            </div>
                                            
                                            <div class="card p-4 mb-6">
                                                <h3 class="font-medium text-text mb-3">"Branches"</h3>
                                                <div class="flex items-center gap-2 text-sm">
                                                    <span class="text-accent">{pr.head_branch}</span>
                                                    <span>"→"</span>
                                                    <span>{pr.base_branch}</span>
                                                </div>
                                            </div>
                                            
                                            <div class="card p-4">
                                                <h3 class="font-medium text-text mb-3">"Commit diff"</h3>
                                                <div class="text-muted text-sm">
                                                    "Diff between {pr.head_branch} and {pr.base_branch} — coming soon"
                                                </div>
                                            </div>
                                        </div>
                                        
                                        <div class="lg:col-span-1">
                                            <div class="card p-4 mb-6">
                                                <h3 class="font-medium text-text mb-4">"Details"</h3>
                                                <div class="space-y-3 text-sm">
                                                    <div>
                                                        <span class="text-muted block mb-1">"Author"</span>
                                                        <span class="text-accent">{pr.author_name}</span>
                                                    </div>
                                                    <div>
                                                        <span class="text-muted block mb-1">"Created"</span>
                                                        <span>{pr.created_at.format("%Y-%m-%d %H:%M")}</span>
                                                    </div>
                                                    {pr.merged_at.map(|t| {
                                                        view! {
                                                            <div>
                                                                <span class="text-muted block mb-1">"Merged"</span>
                                                                <span>{t.format("%Y-%m-%d %H:%M")}</span>
                                                            </div>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                    
                                    {merge_error.get().map(|e| {
                                        view! { <div class="alert-error mt-4">{e}</div> }
                                    })}
                                    
                                    {close_error.get().map(|e| {
                                        view! { <div class="alert-error mt-4">{e}</div> }
                                    })}
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! { <div class="alert-error">{e}</div> }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

fn status_class(status: &str) -> &'static str {
    match status {
        "open" => "bg-success/10 text-success border border-success/20",
        "merged" => "bg-merged/10 text-merged border border-merged/20",
        "closed" => "bg-danger/10 text-danger border border-danger/20",
        _ => "bg-muted/10 text-muted border border-muted/20",
    }
}
```

---

## Database Migration

The migration already exists (`migrations/004_pull_requests.sql`). Ensure it runs on startup or manually:

```bash
# Run migration
sqlx migrate run
```

---

## Routes Integration

Add to `src/app.rs` or `src/lib.rs` router:

```rust
<Route path="/:username/:reponame/pulls" view={pages::repo::pulls::list::PullListPage} />
<Route path="/:username/:reponame/pulls/new" view={pages::repo::pulls::new::NewPullPage} />
<Route path="/:username/:reponame/pulls/:pr_id" view={pages::repo::pulls::detail::PullDetailPage} />
```

---

## Next Steps

1. Add `get_repo_id_for_pr` helper function to session module
2. Add `status_class` helper for PR status badges
3. Test branch selection in new PR form
4. Implement actual git merge logic in `merge_pull_request`
5. Add diff view between branches
6. Implement user authentication check (author/collaborator only)
7. Add toast notifications for success/error
8. Test full workflow: create → list → view → merge/close

---

## Notes

- `ServerAction` uses PascalCase: `ServerAction::<MergePullRequest>::new()` if using form-based approach
- DB queries must use `sqlx::query!` or `query_as!` for compile-time SQL checking
- All git operations should use `gix` crate for safety
- Merge operation requires actual git repository manipulation (consider using `git` CLI for merge)
