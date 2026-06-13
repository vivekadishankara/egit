use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use super::overview::get_repo_overview;
use crate::components::repo_header::RepoHeader;
use crate::components::repo_tab_bar::{BranchSelector, RepoTabBar};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitLogEntry {
    pub id: String,
    pub short_id: String,
    pub author_name: String,
    pub author_email: String,
    pub message: String,
    pub timestamp: i64,
}

#[server(GetCommitLog, "/api")]
pub async fn get_commit_log(
    username: String,
    reponame: String,
    revision: String,
) -> Result<Vec<CommitLogEntry>, ServerFnError> {
    let repo_base: String = expect_context::<String>();
    let raw = crate::git::get_commit_log(&repo_base, &username, &reponame, &revision)
        .map_err(|e| ServerFnError::new(format!("Failed to read commit log: {e}")))?;
    Ok(raw
        .into_iter()
        .map(|c| CommitLogEntry {
            id: c.id,
            short_id: c.short_id,
            author_name: c.author_name,
            author_email: c.author_email,
            message: c.message,
            timestamp: c.timestamp,
        })
        .collect())
}

fn format_timestamp(ts: i64) -> String {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let diff = now - ts;
    if diff < 60 {
        format!("{diff}s ago")
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 2592000 {
        format!("{}d ago", diff / 86400)
    } else {
        time::OffsetDateTime::from_unix_timestamp(ts)
            .map(|dt| format!("{:04}-{:02}-{:02}", dt.year(), dt.month() as u8, dt.day()))
            .unwrap_or_default()
    }
}

#[component]
pub fn CommitsPage() -> impl IntoView {
    let params = use_params_map();

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
    let branch = move || {
        params
            .get()
            .get("branch")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "HEAD".to_string())
    };

    let repo = Resource::new(
        move || (username(), reponame()),
        |(u, r)| async move { get_repo_overview(u, r, None).await },
    );

    let commits = Resource::new(
        move || (username(), reponame(), branch()),
        |(u, r, b)| async move { get_commit_log(u, r, b).await },
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
                            let default_branch = info.default_branch.clone();
                            let has_commits = info.has_commits;

                            view! {
                                <div>
                                    <RepoHeader
                                        owner={owner.clone()}
                                        name={name.clone()}
                                        is_private={is_private}
                                        desc={desc.clone()}
                                        link_to={None}
                                    />

                                    {has_commits.then(|| {
                                        view! {
                                            <BranchSelector
                                                owner={owner.clone()}
                                                name={name.clone()}
                                                current_branch={branch()}
                                                redirect_to="/commits/"
                                            />
                                        }
                                    })}

                                    <RepoTabBar
                                        active="commits"
                                        owner={owner.clone()}
                                        name={name.clone()}
                                        default_branch={default_branch.clone()}
                                        has_commits={has_commits}
                                        current_branch={branch()}

                                    />

                                    <Suspense fallback=|| view! { <p class="text-muted">"Loading commits..."</p> }>
                                        {move || {
                                            commits.get().map(|result| match result {
                                                Ok(list) => {
                                                    view! {
                                                        <div class="card">
                                                    {list.into_iter().map(|entry| {
                                                                 let sid = entry.short_id.clone();
                                                                 let id = entry.id.clone();
                                                                 let aname = entry.author_name.clone();
                                                                 let uname = username();
                                                                 let rname = reponame();
                                                                 let msg = entry.message.clone();
                                                                 let ts = entry.timestamp;
                                                                 view! {
                                                                     <a
                                                                         href=format!("/{uname}/{rname}/commit/{id}")
                                                                         class="flex items-center justify-between py-3 px-4 border-b border-theme last:border-b-0 hover:bg-surface-secondary no-underline"
                                                                     >
                                                                         <div class="flex-1 min-w-0">
                                                                             <div class="text-sm font-medium text-text truncate">
                                                                                 {msg}
                                                                             </div>
                                                                             <div class="text-xs text-muted mt-1">
                                                                                 <span class="font-mono">{sid}</span>
                                                                                 " by "
                                                                                 <span>{aname}</span>
                                                                             </div>
                                                                         </div>
                                                                         <div class="text-xs text-muted whitespace-nowrap ml-4">
                                                                             {format_timestamp(ts)}
                                                                         </div>
                                                                     </a>
                                                                 }
                                                            }).collect::<Vec<_>>()}
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
