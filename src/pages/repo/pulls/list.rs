use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use time::OffsetDateTime;

use super::super::overview::get_repo_overview;
use crate::components::repo_tab_bar::RepoTabBar;
use crate::server::prs::list_pull_requests;

fn format_pr_time(dt: OffsetDateTime) -> String {
    let now = OffsetDateTime::now_utc();
    let diff = now - dt;
    let seconds = diff.whole_seconds();
    if seconds < 60 {
        format!("{seconds}s ago")
    } else if seconds < 3600 {
        format!("{}m ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h ago", seconds / 3600)
    } else if seconds < 2592000 {
        format!("{}d ago", seconds / 86400)
    } else {
        format!("{:04}-{:02}-{:02}", dt.year(), dt.month() as u8, dt.day())
    }
}

fn status_badge(status: String) -> impl IntoView {
    let color = match status.as_str() {
        "open" => "var(--color-success)",
        "merged" => "var(--color-accent)",
        "closed" => "var(--color-danger)",
        _ => "var(--color-text-muted)",
    };
    view! {
        <span
            class="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium rounded-full"
            style=format!("color: {color}; border: 1px solid color-mix(in srgb, {color} 30%, transparent); background-color: color-mix(in srgb, {color} 10%, transparent);")
        >
            <span class="inline-block w-2 h-2 rounded-full" style=format!("background-color: {color};")></span>
            {status}
        </span>
    }
}

#[component]
pub fn PullListPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();

    let username = move || {
        params
            .get()
            .get("username")
            .map(|s| s.to_string())
            .unwrap_or_default()
    };
    let reponame = move || {
        params
            .get()
            .get("reponame")
            .map(|s| s.to_string())
            .unwrap_or_default()
    };
    let status = move || {
        query
            .get()
            .get("status")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "open".to_string())
    };

    let repo = Resource::new(
        move || (username(), reponame()),
        |(u, r)| async move { get_repo_overview(u, r, None).await },
    );

    let repo_id = move || {
        repo.get().and_then(|r| r.ok()).map(|info| info.repo_id)
    };

    let prs = Resource::new(
        move || (repo_id(), status()),
        |(id, s)| async move {
            if let Some(id) = id {
                list_pull_requests(id, Some(s)).await.ok()
            } else {
                None
            }
        },
    );

    view! {
        <div class="container">
            <Suspense fallback=|| view! { <p class="text-muted">"Loading..."</p> }>
                {move || {
                    repo.get().map(|result| match result {
                        Ok(info) => {
                            let name = info.name.clone();
                            let owner = info.owner_name.clone();
                            let desc = info.description.clone();
                            let is_private = info.is_private;
                            let has_commits = info.has_commits;
                            let default_branch = info.default_branch.clone();
                            let current_status = status();

                            view! {
                                <div>
                                    <h1 class="page-title">
                                        <span class="text-accent">{owner.clone()}</span>
                                        <span class="text-muted">"/"</span>
                                        <span class="text-accent">{name.clone()}</span>
                                        {if is_private {
                                            view! {
                                                <span class="ml-2 px-2 py-0.5 text-xs rounded-full border border-theme text-muted">
                                                    "Private"
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </h1>

                                    {desc.as_ref().map(|d| {
                                        view! { <p class="text-muted mb-4">{d.clone()}</p> }
                                    })}

                                    <RepoTabBar
                                        active="pulls"
                                        owner={owner.clone()}
                                        name={name.clone()}
                                        default_branch={default_branch.clone()}
                                        has_commits={has_commits}
                                        current_branch={default_branch.clone()}
                                    />

                                    <div class="flex gap-6">
                                        <div class="w-48 shrink-0">
                                            <div class="flex flex-col gap-0.5">
                                                <a
                                                    href=format!("/{owner}/{name}/pulls?status=open")
                                                    class=format!(
                                                        "flex items-center justify-between px-3 py-2 text-sm rounded transition-colors no-underline {}",
                                                        if current_status == "open" {
                                                            "bg-surface-tertiary text-text font-medium"
                                                        } else {
                                                            "text-muted hover:text-text hover:bg-surface-secondary"
                                                        }
                                                    )
                                                >
                                                    <span>"Open"</span>
                                                </a>
                                                <a
                                                    href=format!("/{owner}/{name}/pulls?status=merged")
                                                    class=format!(
                                                        "flex items-center justify-between px-3 py-2 text-sm rounded transition-colors no-underline {}",
                                                        if current_status == "merged" {
                                                            "bg-surface-tertiary text-text font-medium"
                                                        } else {
                                                            "text-muted hover:text-text hover:bg-surface-secondary"
                                                        }
                                                    )
                                                >
                                                    <span>"Merged"</span>
                                                </a>
                                                <a
                                                    href=format!("/{owner}/{name}/pulls?status=closed")
                                                    class=format!(
                                                        "flex items-center justify-between px-3 py-2 text-sm rounded transition-colors no-underline {}",
                                                        if current_status == "closed" {
                                                            "bg-surface-tertiary text-text font-medium"
                                                        } else {
                                                            "text-muted hover:text-text hover:bg-surface-secondary"
                                                        }
                                                    )
                                                >
                                                    <span>"Closed"</span>
                                                </a>
                                            </div>
                                        </div>

                                        <div class="flex-1">
                                            <Suspense fallback=|| view! { <p class="text-muted">"Loading pull requests..."</p> }>
                                                {move || {
                                                    let owner = owner.clone();
                                                    let name = name.clone();
                                                    prs.get().map(|list| match list {
                                                        Some(pr_list) => {
                                                            if pr_list.is_empty() {
                                                                view! {
                                                                    <div class="card">
                                                                        <p class="text-muted text-sm">
                                                                            "No pull requests found."
                                                                        </p>
                                                                        <a
                                                                            href=format!("/{owner}/{name}/pulls/new")
                                                                            class="btn-primary mt-4 inline-block no-underline"
                                                                        >
                                                                            "New pull request"
                                                                        </a>
                                                                    </div>
                                                                }.into_any()
                                                            } else {
                                                                view! {
                                                                    <div class="card">
                                                                        {pr_list.into_iter().map(|pr| {
                                                                            let pr_id = pr.id;
                                                                            let title = pr.title.clone();
                                                                            let author = pr.author_name.clone();
                                                                            let status_str = pr.status.clone();
                                                                            let head = pr.head_branch.clone();
                                                                            let base = pr.base_branch.clone();
                                                                            let created = pr.created_at;
                                                                            let owner = owner.clone();
                                                                            let name = name.clone();

                                                                            view! {
                                                                                <a
                                                                                    href=format!("/{owner}/{name}/pulls/{pr_id}")
                                                                                    class="flex items-start justify-between py-3 px-4 border-b border-theme last:border-b-0 hover:bg-surface-secondary no-underline"
                                                                                >
                                                                                    <div class="flex-1 min-w-0">
                                                                                        <div class="flex items-center gap-2">
                                                                                            <div class="text-sm font-medium text-text truncate">
                                                                                                {title}
                                                                                            </div>
                                                                                            {status_badge(status_str)}
                                                                                        </div>
                                                                                        <div class="text-xs text-muted mt-1">
                                                                                            "by "
                                                                                            <span class="font-medium">{author}</span>
                                                                                            " in "
                                                                                            <span class="font-mono">{head.clone()}</span>
                                                                                            " → "
                                                                                            <span class="font-mono">{base.clone()}</span>
                                                                                            " opened "
                                                                                            {format_pr_time(created)}
                                                                                        </div>
                                                                                    </div>
                                                                                </a>
                                                                            }
                                                                        }).collect::<Vec<_>>()}
                                                                    </div>
                                                                }.into_any()
                                                            }
                                                        }
                                                        None => {
                                                            view! { <div class="alert-error">"Failed to load pull requests."</div> }.into_any()
                                                        }
                                                    })
                                                }}
                                            </Suspense>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(e) => {
                            view! { <div class="alert-error">{e.to_string()}</div> }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
