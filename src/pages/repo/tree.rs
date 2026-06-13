use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::components::file_tree::{FileTree, TreeEntry};
use crate::components::repo_tab_bar::{url_encode_branch, BranchSelector, RepoTabBar, get_repo_tab_meta};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryDto {
    name: String,
    is_dir: bool,
}

#[server(GetTreeEntries, "/api")]
pub async fn get_tree_entries(
    username: String,
    reponame: String,
    revision: String,
    path: String,
) -> Result<Vec<EntryDto>, ServerFnError> {
    let repo_base: String = expect_context::<String>();
    let raw = crate::git::list_directory(&repo_base, &username, &reponame, &revision, &path)
        .map_err(|e| ServerFnError::new(format!("Failed to read repository: {e}")))?;
    Ok(raw
        .into_iter()
        .map(|(name, is_dir)| EntryDto { name, is_dir })
        .collect())
}

#[component]
pub fn TreePage() -> impl IntoView {
    let params = use_params_map();

    let username = move || {
        params
            .get()
            .get("username")
            .unwrap_or_default()
            .to_string()
    };
    let reponame = move || {
        params
            .get()
            .get("reponame")
            .unwrap_or_default()
            .to_string()
    };
    let branch = move || {
        params
            .get()
            .get("branch")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "HEAD".to_string())
    };
    let path = move || {
        params
            .get()
            .get("path")
            .map(|s| s.trim_start_matches('/').to_string())
            .unwrap_or_default()
    };

    let entries = Resource::new(
        move || (username(), reponame(), branch(), path()),
        |(u, r, b, p)| async move { get_tree_entries(u, r, b, p).await },
    );

    let repo_meta = Resource::new(
        move || (username(), reponame()),
        |(u, r)| async move { get_repo_tab_meta(u, r).await },
    );

    let parent_url = move || {
        let u = username();
        let r = reponame();
        let b = url_encode_branch(&branch());
        let p = path();
        if p.is_empty() {
            format!("/{u}/{r}/tree/{b}")
        } else {
            let parent = match p.rsplit_once('/') {
                Some((head, _)) => head,
                None => "",
            };
            if parent.is_empty() {
                format!("/{u}/{r}/tree/{b}")
            } else {
                format!("/{u}/{r}/tree/{b}/{parent}")
            }
        }
    };

    view! {
        <div class="container">
            <h1 class="page-title">
                <a
                    href=format!("/{}/{}/tree/{}", username(), reponame(), url_encode_branch(&branch()))
                    class="no-underline"
                >
                    <span class="text-accent">{username()}</span>
                    <span class="text-muted">"/"</span>
                    <span class="text-accent">{reponame()}</span>
                </a>
            </h1>

            <Suspense fallback=|| view! { <p class="text-muted">"Loading..."</p> }>
                {move || {
                    repo_meta.get().map(|result| match result {
                        Ok(meta) => {
                            view! {
                                <>
                                    {meta.description.as_ref().map(|d| {
                                        view! { <p class="text-muted mb-4">{d.clone()}</p> }
                                    })}
                                    <BranchSelector
                                        owner={username()}
                                        name={reponame()}
                                        current_branch={branch()}
                                        redirect_to="/tree/"
                                    />
                                    <RepoTabBar
                                        active="code"
                                        owner={username()}
                                        name={reponame()}
                                        default_branch={meta.default_branch}
                                        has_commits={meta.has_commits}
                                        current_branch={branch()}
                                        has_pull_requests={false}
                                    />
                                </>
                            }.into_any()
                        }
                        Err(e) => {
                            view! { <div class="alert-error">{e.to_string()}</div> }.into_any()
                        }
                    })
                }}
            </Suspense>

            <Suspense fallback=|| view! { <p class="text-muted">"Loading..."</p> }>
                {move || {
                    entries.get().map(|result| match result {
                        Ok(list) => {
                            let tree_entries: Vec<TreeEntry> = list
                                .into_iter()
                                .map(|dto| TreeEntry {
                                    name: dto.name,
                                    is_dir: dto.is_dir,
                                })
                                .collect();

                            if tree_entries.is_empty() {
                                view! {
                                    <p class="text-muted text-sm">"This directory is empty."</p>
                                }
                                .into_any()
                            } else {
                                view! {
                                    <FileTree
                                        entries=tree_entries
                                        username=username()
                                        reponame=reponame()
                                        branch=branch()
                                        current_path=path()
                                    />
                                }
                                .into_any()
                            }
                        }
                        Err(e) => {
                            view! { <div class="alert-error">{e.to_string()}</div> }.into_any()
                        }
                    })
                }}
            </Suspense>

            <div class="mt-4">
                <a href=parent_url() class="btn-secondary text-sm no-underline">
                    "← Back"
                </a>
            </div>
        </div>
    }
}
