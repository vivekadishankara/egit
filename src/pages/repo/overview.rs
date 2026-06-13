use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use serde::{Deserialize, Serialize};

use crate::components::markdown::Markdown;
use crate::components::repo_header::RepoHeader;
use crate::components::repo_tab_bar::{BranchSelector, RepoTabBar};
use crate::server::prs::get_pull_request_counts;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub description: Option<String>,
    pub owner_name: String,
    pub is_private: bool,
    pub readme_content: Option<String>,
    pub has_commits: bool,
    pub default_branch: String,
    pub repo_id: Uuid,
    pub has_pull_requests: bool,
}

#[server(GetRepoOverview, "/api")]
pub async fn get_repo_overview(
    username: String,
    reponame: String,
    branch: Option<String>,
) -> Result<RepoInfo, ServerFnError> {
    use sqlx::PgPool;

    let pool = expect_context::<PgPool>();
    let repo_base: String = expect_context::<String>();

    let row = sqlx::query!(
        r#"
        SELECT r.id, r.name, r.description, r.is_private, u.username as owner_name
        FROM repositories r
        JOIN users u ON u.id = r.owner_id
        WHERE r.name = $1 AND u.username = $2
        "#,
        reponame,
        username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("Repository not found"))?;

    let readme_content = crate::git::read_readme(&repo_base, &username, &reponame, branch.as_deref())
        .ok()
        .flatten();

    let has_commits = crate::git::has_commits(&repo_base, &username, &reponame);
    let default_branch = crate::git::get_default_branch(&repo_base, &username, &reponame)
        .unwrap_or_else(|| "HEAD".to_string());

    let pr_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM pull_requests WHERE repo_id = $1",
        row.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .unwrap_or(0);

    Ok(RepoInfo {
        name: row.name,
        description: row.description,
        owner_name: row.owner_name,
        is_private: row.is_private,
        readme_content,
        has_commits,
        default_branch,
        repo_id: row.id,
        has_pull_requests: pr_count > 0,
    })
}

#[component]
pub fn RepoOverviewPage() -> impl IntoView {
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
    let branch = move || {
        query.get().get("branch").map(|s| s.to_string())
    };

    let repo = Resource::new(
        move || (username(), reponame(), branch()),
        |(u, r, b)| async move { get_repo_overview(u, r, b).await },
    );

    let pr_counts = Resource::new(
        move || (username(), reponame()),
        |(u, r)| async move {
            let repo = get_repo_overview(u, r, None).await.ok()?;
            get_pull_request_counts(repo.repo_id).await.ok()
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
                            let readme = info.readme_content.clone();
                            let is_private = info.is_private;
                            let has_commits = info.has_commits;
                            let _has_pull_requests = info.has_pull_requests;
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

                                    {has_commits.then(|| {
                                        view! {
                                            <BranchSelector
                                                owner={owner.clone()}
                                                name={name.clone()}
                                                current_branch={branch().unwrap_or_else(|| default_branch.clone())}
                                                redirect_to="?branch="
                                            />
                                        }
                                    })}

                                    <RepoTabBar
                                        active="overview"
                                        owner={owner.clone()}
                                        name={name.clone()}
                                        default_branch={default_branch.clone()}
                                        has_commits={has_commits}
                                        current_branch={branch().unwrap_or_default()}

                                    />

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
                                                        {move || pr_counts.get().map(|c| {
                                                            view! { <span class="text-xs text-muted">{c.as_ref().map(|c| c.open.to_string()).unwrap_or_else(|| "0".to_string())}</span> }
                                                        })}
                                                    </a>
                                                    <a href=format!("/{owner}/{name}/pulls?status=merged") class="flex items-center justify-between px-3 py-2 text-sm text-text hover:bg-surface-secondary rounded transition-colors">
                                                        <span>"Merged"</span>
                                                        {move || pr_counts.get().map(|c| {
                                                            view! { <span class="text-xs text-muted">{c.as_ref().map(|c| c.merged.to_string()).unwrap_or_else(|| "0".to_string())}</span> }
                                                        })}
                                                    </a>
                                                    <a href=format!("/{owner}/{name}/pulls?status=closed") class="flex items-center justify-between px-3 py-2 text-sm text-text hover:bg-surface-secondary rounded transition-colors">
                                                        <span>"Closed"</span>
                                                        {move || pr_counts.get().map(|c| {
                                                            view! { <span class="text-xs text-muted">{c.as_ref().map(|c| c.closed.to_string()).unwrap_or_else(|| "0".to_string())}</span> }
                                                        })}
                                                    </a>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="flex-1">
                                            {match &readme {
                                                Some(content) => {
                                                    view! {
                                                        <div class="card">
                                                            <div class="text-sm font-medium text-muted mb-2 border-b border-theme pb-2">
                                                                "README.md"
                                                            </div>
                                                            <Markdown content=content.clone()/>
                                                        </div>
                                                    }.into_any()
                                                }
                                                None => {
                                                    if has_commits {
                                                        view! {
                                                            <div class="card">
                                                                <p class="text-muted text-sm">
                                                                    "No README found for this repository."
                                                                </p>
                                                                <a
                                                                    href=format!("/{owner}/{name}/tree/{default_branch}")
                                                                    class="btn-primary mt-4 inline-block no-underline"
                                                                >
                                                                    "Browse files"
                                                                </a>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="card">
                                                                <p class="text-muted text-sm">
                                                                    "This repository is empty."
                                                                </p>
                                                            </div>
                                                        }.into_any()
                                                    }
                                                }
                                            }}
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
