use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn RepoOverviewPage() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.get().get("username").unwrap_or_default();
    let reponame = move || params.get().get("reponame").unwrap_or_default();

    view! {
        <div class="container">
            <h1 class="page-title">
                {username} "/" <span class="text-accent">{reponame}</span>
            </h1>
            <p class="text-text-muted">"Repo overview — coming in step 7"</p>
        </div>
    }
}
