use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.get().get("username").unwrap_or_default();

    view! {
        <div class="container">
            <h1 class="page-title">{username}</h1>
            // TODO: implement in step 11
            <p class="text-muted">"Profile coming soon"</p>
        </div>
    }
}
