use leptos::prelude::*;

use crate::components::delete_repo_button::DeleteRepoButton;

#[component]
pub fn RepoHeader(
    owner: String,
    name: String,
    is_private: bool,
    desc: Option<String>,
    link_to: Option<String>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between mb-6">
            <h1 class="page-title !mb-0">
                {if let Some(url) = link_to {
                    view! {
                        <a href=url class="no-underline">
                            <span class="text-accent">{owner.clone()}</span>
                            <span class="text-muted">"/"</span>
                            <span class="text-accent">{name.clone()}</span>
                        </a>
                    }.into_any()
                } else {
                    view! {
                        <span class="text-accent">{owner.clone()}</span>
                        <span class="text-muted">"/"</span>
                        <span class="text-accent">{name.clone()}</span>
                    }.into_any()
                }}
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
            <DeleteRepoButton owner=owner.clone() reponame=name.clone()/>
        </div>
        {desc.map(|d| {
            view! { <p class="text-muted mb-4">{d}</p> }
        })}
    }
}
