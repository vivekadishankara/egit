use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::components::markdown::Markdown;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub description: Option<String>,
    pub owner_name: String,
    pub is_private: bool,
    pub readme_content: Option<String>,
    pub has_commits: bool,
    pub default_branch: String,
}

#[server(GetRepoOverview, "/api")]
pub async fn get_repo_overview(
    username: String,
    reponame: String,
) -> Result<RepoInfo, ServerFnError> {
    use sqlx::PgPool;

    let pool = expect_context::<PgPool>();
    let repo_base: String = expect_context::<String>();

    let row = sqlx::query!(
        r#"
        SELECT r.name, r.description, r.is_private, u.username as owner_name
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

    let readme_content = crate::git::read_readme(&repo_base, &username, &reponame)
        .ok()
        .flatten();

    let has_commits = crate::git::has_commits(&repo_base, &username, &reponame);
    let default_branch = crate::git::get_default_branch(&repo_base, &username, &reponame)
        .unwrap_or_else(|| "HEAD".to_string());

    Ok(RepoInfo {
        name: row.name,
        description: row.description,
        owner_name: row.owner_name,
        is_private: row.is_private,
        readme_content,
        has_commits,
        default_branch,
    })
}

#[component]
pub fn RepoOverviewPage() -> impl IntoView {
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

    let repo = Resource::new(
        move || (username(), reponame()),
        |(u, r)| async move { get_repo_overview(u, r).await },
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
                            let default_branch = info.default_branch.clone();

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

                                    <div class="flex gap-1 border-b border-theme mb-6">
                                        <span class="px-4 py-2 text-sm font-medium border-b-2 border-accent text-accent">
                                            "Overview"
                                        </span>
                                        {if has_commits {
                                            view! {
                                                <>
                                                    <a
                                                        href=format!("/{owner}/{name}/tree/{}", default_branch)
                                                        class="px-4 py-2 text-sm text-muted no-underline border-b-2 border-transparent hover:text-text hover:border-text"
                                                    >
                                                        "Code"
                                                    </a>
                                                    <a
                                                        href=format!("/{owner}/{name}/commits/{default_branch}")
                                                        class="px-4 py-2 text-sm text-muted no-underline border-b-2 border-transparent hover:text-text hover:border-text"
                                                    >
                                                        "Commits"
                                                    </a>
                                                </>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>

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
