use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use super::super::overview::get_repo_overview;
use crate::components::repo_header::RepoHeader;
use crate::components::repo_tab_bar::RepoTabBar;
use crate::server::prs::{get_branch_list_for_pr, CreatePullRequest};

#[component]
pub fn NewPullPage() -> impl IntoView {
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
        |(u, r)| async move { get_repo_overview(u, r, None).await },
    );

    let branches = Resource::new(
        move || (username(), reponame()),
        |(u, r)| async move { get_branch_list_for_pr(u, r).await.ok() },
    );

    let create_pr = ServerAction::<CreatePullRequest>::new();

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
                            let repo_id = info.repo_id;
                            let branches_base = branches.clone();
                            let branches_head = branches;

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

                                    <div class="mt-6 max-w-2xl">
                                        <h2 class="text-lg font-semibold text-text mb-4">"New pull request"</h2>

                                        {move || {
                                            create_pr.value().get().and_then(|r| r.err()).map(|e| {
                                                view! { <div class="alert-error mb-4">{e.to_string()}</div> }
                                            })
                                        }}

                                        <ActionForm action=create_pr>
                                            <input type="hidden" name="repo_id" value=repo_id.to_string()/>
                                            <input type="hidden" name="username" value=owner.clone()/>
                                            <input type="hidden" name="reponame" value=name.clone()/>

                                            <div class="form-group">
                                                <label class="form-label" for="base_branch">"Base branch"</label>
                                                <select class="form-input" id="base_branch" name="base_branch" required>
                                                    <option value="" disabled selected>"Select base branch..."</option>
                                                    {move || branches_base.get().map(|list| {
                                                        let list = list.unwrap_or_default();
                                                        let default = default_branch.clone();
                                                        list.into_iter().map(|b| {
                                                            let is_default = b == default;
                                                            view! {
                                                                <option value=b.clone() selected=is_default>{b.clone()}</option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    })}
                                                </select>
                                            </div>

                                            <div class="form-group">
                                                <label class="form-label" for="head_branch">"Head branch"</label>
                                                <select class="form-input" id="head_branch" name="head_branch" required>
                                                    <option value="" disabled selected>"Select head branch..."</option>
                                                    {move || branches_head.get().map(|list| {
                                                        let list = list.unwrap_or_default();
                                                        list.into_iter().map(|b| {
                                                            view! {
                                                                <option value=b.clone()>{b.clone()}</option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    })}
                                                </select>
                                            </div>

                                            <div class="form-group">
                                                <label class="form-label" for="title">"Title"</label>
                                                <input
                                                    class="form-input"
                                                    type="text"
                                                    id="title"
                                                    name="title"
                                                    required
                                                    placeholder="Enter pull request title"
                                                />
                                            </div>

                                            <div class="form-group">
                                                <label class="form-label" for="body">"Description"</label>
                                                <textarea
                                                    class="form-input"
                                                    id="body"
                                                    name="body"
                                                    rows="8"
                                                    placeholder="Describe the changes in this pull request (optional)"
                                                ></textarea>
                                            </div>

                                            <div class="flex gap-3 mt-6">
                                                <button
                                                    class="btn-primary"
                                                    type="submit"
                                                    disabled=move || create_pr.pending().get()
                                                >
                                                    {move || if create_pr.pending().get() { "Creating..." } else { "Create pull request" }}
                                                </button>
                                                <a
                                                    href=format!("/{owner}/{name}/pulls")
                                                    class="btn-secondary no-underline"
                                                >
                                                    "Cancel"
                                                </a>
                                            </div>
                                        </ActionForm>
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
