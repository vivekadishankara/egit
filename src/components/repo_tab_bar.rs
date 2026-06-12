use leptos::prelude::*;
use serde::{Deserialize, Serialize};

fn url_encode_branch(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTabMeta {
    pub default_branch: String,
    pub has_commits: bool,
}

#[server(GetRepoTabMeta, "/api")]
pub async fn get_repo_tab_meta(
    username: String,
    reponame: String,
) -> Result<RepoTabMeta, ServerFnError> {
    let repo_base: String = expect_context::<String>();
    let default_branch = crate::git::get_default_branch(&repo_base, &username, &reponame)
        .unwrap_or_else(|| "HEAD".to_string());
    let has_commits = crate::git::has_commits(&repo_base, &username, &reponame);
    Ok(RepoTabMeta { default_branch, has_commits })
}

#[server(GetBranchList, "/api")]
pub async fn get_branch_list(
    username: String,
    reponame: String,
) -> Result<Vec<String>, ServerFnError> {
    let repo_base: String = expect_context::<String>();
    Ok(crate::git::list_branches(&repo_base, &username, &reponame))
}

#[component]
pub fn BranchSelector(
    owner: String,
    name: String,
    current_branch: String,
    redirect_to: &'static str,
) -> impl IntoView {
    let owner1 = owner.clone();
    let name1 = name.clone();
    let branches = Resource::new(
        move || (owner1.clone(), name1.clone()),
        |(u, r)| async move { get_branch_list(u, r).await },
    );

    let owner2 = owner.clone();
    let name2 = name.clone();
    let current_branch2 = current_branch.clone();
    view! {
        <div class="relative inline-block">
            <Suspense fallback=move || {
                view! { <span class="text-sm text-muted">{current_branch.clone()}</span> }
            }>
                {move || {
                    let owner = owner2.clone();
                    let name = name2.clone();
                    let current = current_branch2.clone();
                    branches.get().map(|result| match result {
                        Ok(list) => {
                            view! {
                                <details class="relative inline-block">
                                    <summary class="text-sm px-3 py-1.5 rounded-md border border-theme bg-surface-secondary text-text cursor-pointer list-none">
                                        {current.clone()}
                                        <span class="ml-1 text-xs text-muted">"▼"</span>
                                    </summary>
                                    <div class="absolute top-full left-0 mt-0.5 rounded-md border border-theme bg-surface shadow-lg z-10 min-w-[180px] max-h-60 overflow-y-auto">
                                        {list.into_iter().map(|b| {
                                            let url = format!("/{owner}/{name}{redirect_to}{}", url_encode_branch(&b));
                                            view! {
                                                <a
                                                    href=url
                                                    class="block px-3 py-2 text-sm text-text hover:bg-surface-secondary no-underline"
                                                >
                                                    {b.clone()}
                                                </a>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </details>
                            }.into_any()
                        }
                        Err(_) => {
                            view! { <span class="text-sm text-muted">{current.clone()}</span> }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn RepoTabBar(
    active: &'static str,
    owner: String,
    name: String,
    default_branch: String,
    has_commits: bool,
) -> impl IntoView {
    let tab_class = |tab: &str| {
        if tab == active {
            "px-4 py-2 text-sm font-medium border-b-2 border-accent text-accent"
        } else {
            "px-4 py-2 text-sm text-muted no-underline border-b-2 border-transparent hover:text-text hover:border-text"
        }
    };

    view! {
        <div class="flex gap-1 items-baseline border-b border-theme mb-6">
            {if active == "overview" {
                view! {
                    <span class={tab_class("overview")}>
                        "Overview"
                    </span>
                }.into_any()
            } else {
                view! {
                    <a href=format!("/{owner}/{name}") class={tab_class("overview")}>
                        "Overview"
                    </a>
                }.into_any()
            }}
            {has_commits.then(|| {
                if active == "code" {
                    view! {
                        <span class={tab_class("code")}>
                            "Code"
                        </span>
                    }.into_any()
                } else {
                    view! {
                        <a
                            href=format!("/{owner}/{name}/tree/{default_branch}")
                            class={tab_class("code")}
                        >
                            "Code"
                        </a>
                    }.into_any()
                }
            })}
            {has_commits.then(|| {
                if active == "commits" {
                    view! {
                        <span class={tab_class("commits")}>
                            "Commits"
                        </span>
                    }.into_any()
                } else {
                    view! {
                        <a
                            href=format!("/{owner}/{name}/commits/{default_branch}")
                            class={tab_class("commits")}
                        >
                            "Commits"
                        </a>
                    }.into_any()
                }
            })}
        </div>
    }
}
