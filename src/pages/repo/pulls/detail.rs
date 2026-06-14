use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_navigate};
use time::OffsetDateTime;
use uuid::Uuid;

use super::super::overview::get_repo_overview;
use crate::components::diff_viewer::DiffViewer;
use crate::components::markdown::Markdown;
use crate::components::repo_header::RepoHeader;
use crate::components::repo_tab_bar::RepoTabBar;
use crate::pages::auth::get_current_user;
use crate::server::prs::{
    get_pr_diff, get_pull_request, ClosePullRequest, MergePullRequest,
};

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

fn status_badge(status: &str, large: bool) -> impl IntoView + use<'_> {
    let color = match status {
        "open" => "var(--color-success)",
        "merged" => "var(--color-accent)",
        "closed" => "var(--color-danger)",
        _ => "var(--color-text-muted)",
    };
    let size = if large { "px-3 py-1 text-sm" } else { "px-2 py-0.5 text-xs" };
    view! {
        <span
            class=format!("inline-flex items-center gap-1.5 {size} font-medium rounded-full")
            style=format!("color: {color}; border: 1px solid color-mix(in srgb, {color} 30%, transparent); background-color: color-mix(in srgb, {color} 10%, transparent);")
        >
            <span class="inline-block w-2.5 h-2.5 rounded-full" style=format!("background-color: {color};")></span>
            {status}
        </span>
    }
}

fn short_id(id: Uuid) -> String {
    id.to_string()[..8].to_string()
}

#[component]
pub fn PullDetailPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

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
    let pr_id = move || {
        params
            .get()
            .get("pr_id")
            .and_then(|s| Uuid::parse_str(&s).ok())
    };

    let repo = Resource::new(
        move || (username(), reponame()),
        |(u, r)| async move { get_repo_overview(u, r, None).await },
    );

    let pr_detail = Resource::new(
        move || pr_id(),
        |id| async move {
            match id {
                Some(id) => get_pull_request(id).await.ok(),
                None => None,
            }
        },
    );

    let diff = Resource::new(
        move || pr_id(),
        |id| async move {
            match id {
                Some(pr_id) => get_pr_diff(pr_id).await.ok(),
                None => None,
            }
        },
    );

    let current_user = Resource::new(|| (), |_| get_current_user());

    let merge_action = ServerAction::<MergePullRequest>::new();
    let close_action = ServerAction::<ClosePullRequest>::new();

    let navigate_merge = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(())) = merge_action.value().get() {
            navigate_merge(&format!("/{}/{}/pulls", username(), reponame()), Default::default());
        }
    });

    let navigate_close = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(())) = close_action.value().get() {
            navigate_close(&format!("/{}/{}/pulls", username(), reponame()), Default::default());
        }
    });

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

                            view! {
                                <div>
                                    <RepoHeader
                                        owner={owner.clone()}
                                        name={name.clone()}
                                        is_private={is_private}
                                        desc={desc.clone()}
                                        link_to={None}
                                    />

                                    <RepoTabBar
                                        active="pulls"
                                        owner={owner.clone()}
                                        name={name.clone()}
                                        default_branch={default_branch.clone()}
                                        has_commits={has_commits}
                                        current_branch={default_branch.clone()}
                                    />

                                    <Suspense fallback=|| view! { <p class="text-muted">"Loading pull request..."</p> }>
                                        {move || {
                                            pr_detail.get().map(|detail| match detail {
                                                Some(pr) => {
                                                    let pr_author = pr.author_name.clone();
                                                    let is_open = pr.status == "open";
                                                    let is_closed = pr.status == "closed";

                                                    let is_author = current_user.get()
                                                        .and_then(|r| r.ok())
                                                        .flatten()
                                                        .map(|u| u.username == pr_author)
                                                        .unwrap_or(false);

                                                    view! {
                                                        <div class="mt-6">
                                                            <div class="flex items-center gap-3 mb-2">
                                                                <span class="text-lg text-muted font-mono">
                                                                    {format!("#{}", short_id(pr.id))}
                                                                </span>
                                                                {status_badge(&pr.status, false)}
                                                            </div>

                                                            <h2 class="text-2xl font-semibold text-text mb-4">
                                                                {pr.title.clone()}
                                                            </h2>

                                                            <div class="flex items-center gap-2 text-sm text-muted mb-4">
                                                                <span class="font-medium text-text">{pr.author_name.clone()}</span>
                                                                <span>"opened this pull request"</span>
                                                                {format_pr_time(pr.created_at)}
                                                                <span class="mx-1">"·"</span>
                                                                <span class="font-mono">{pr.head_branch.clone()}</span>
                                                                <span>"→"</span>
                                                                <span class="font-mono">{pr.base_branch.clone()}</span>
                                                            </div>

                                            {is_open.then(|| {
                                                let pr_id_val = pr.id;
                                                let pr_id_for_merge = pr_id_val;
                                                let pr_id_for_close = pr_id_val;
                                                view! {
                                                    <div class="flex gap-2 mb-6">
                                                        {is_author.then(|| {
                                                            view! {
                                                                <ActionForm action=merge_action>
                                                                    <input type="hidden" name="pr_id" value=pr_id_for_merge.to_string()/>
                                                                    <button
                                                                        type="submit"
                                                                        class="btn-primary text-sm"
                                                                        disabled=move || merge_action.pending().get()
                                                                    >
                                                                        {move || if merge_action.pending().get() { "Merging..." } else { "Merge pull request" }}
                                                                    </button>
                                                                </ActionForm>
                                                                <ActionForm action=close_action>
                                                                    <input type="hidden" name="pr_id" value=pr_id_for_close.to_string()/>
                                                                    <button
                                                                        type="submit"
                                                                        class="btn-secondary text-sm"
                                                                        disabled=move || close_action.pending().get()
                                                                    >
                                                                        {move || if close_action.pending().get() { "Closing..." } else { "Close" }}
                                                                    </button>
                                                                </ActionForm>
                                                            }.into_any()
                                                        })}
                                                    </div>
                                                }
                                            })}

                                                            {pr.body.as_ref().map(|body| {
                                                                view! {
                                                                    <div class="card mb-6">
                                                                        <div class="px-4 py-3 border-b border-theme">
                                                                            <span class="text-sm font-medium text-muted">
                                                                                {pr.author_name.clone()}
                                                                                " commented "
                                                                                {format_pr_time(pr.created_at)}
                                                                            </span>
                                                                        </div>
                                                                        <div class="p-4">
                                                                            <Markdown content=body.clone()/>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            })}

                                                            {(!is_closed).then(|| {
                                                                let diff_state = diff.clone();
                                                                view! {
                                                                    <div class="mt-8">
                                                                        <h3 class="text-sm font-medium text-muted mb-3 uppercase tracking-wide">
                                                                            "Changes"
                                                                        </h3>
                                                                        <Suspense fallback=|| view! { <p class="text-muted">"Loading diff..."</p> }>
                                                                             {move || {
                                                                                 diff_state.get().map(|diff_files| {
                                                                                     match diff_files {
                                                                                         Some(files) if !files.is_empty() => {
                                                                                             view! { <DiffViewer files=files.clone()/> }.into_any()
                                                                                         }
                                                                                         _ => {
                                                                                             view! { <p class="text-sm text-muted">"No changes between these branches."</p> }.into_any()
                                                                                         }
                                                                                     }
                                                                                 })
                                                                             }}
                                                                        </Suspense>
                                                                    </div>
                                                                }
                                                            })}
                                                        </div>
                                                    }.into_any()
                                                }
                                                None => {
                                                    view! { <div class="alert-error">"Failed to load pull request."</div> }.into_any()
                                                }
                                            })
                                        }}
                                    </Suspense>
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
