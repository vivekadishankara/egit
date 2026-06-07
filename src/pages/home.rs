use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="hero">
            <h1 class="hero-title">"Welcome to eGit"</h1>
            <p class="hero-subtitle">"A self-hosted Git forge built in Rust"</p>
            <div class="hero-actions">
                <a href="/register" class="btn-primary">"Get started"</a>
                <a href="/login" class="btn-secondary">"Sign in"</a>
            </div>
        </div>
    }
}
