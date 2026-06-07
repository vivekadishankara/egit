use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="navbar">
            <div class="navbar-inner">
                <a href="/" class="navbar-brand">
                    <span class="text-accent font-bold text-xl">"eGit"</span>
                </a>
                <div class="navbar-links">
                    // TODO: show user menu or login link based on session
                    <a href="/login" class="navbar-link">"Sign in"</a>
                    <a href="/register" class="btn-primary text-sm">"Sign up"</a>
                </div>
            </div>
        </nav>
    }
}
