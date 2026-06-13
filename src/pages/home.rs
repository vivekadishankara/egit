use leptos::prelude::*;

use crate::pages::auth::get_current_user;

#[component]
pub fn HomePage() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());

    view! {
        <Suspense fallback=|| view! { <div class="hero"><p class="hero-subtitle">"Loading…"</p></div> }>
            {move || {
                user.get().map(|result| {
                    match result {
                        Ok(Some(u)) => view! {
                            <div class="hero">
                                <h1 class="hero-title">"Welcome, " {u.username.clone()}</h1>
                                <p class="hero-subtitle">"Your Git forge dashboard"</p>
                                <div class="hero-actions">
                                    <a href="/repos/new" class="btn-primary">"New repository"</a>
                                    <a href=format!("/{}", u.username) class="btn-secondary">"Your profile"</a>
                                </div>
                            </div>
                        }.into_any(),
                        _ => view! {
                            <div class="hero">
                                <h1 class="hero-title">"Welcome to eGit"</h1>
                                <p class="hero-subtitle">"A self-hosted Git forge built in Rust"</p>
                                <div class="hero-actions">
                                    <a href="/register" class="btn-primary">"Get started"</a>
                                    <a href="/login" class="btn-secondary">"Sign in"</a>
                                </div>
                            </div>
                        }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
}
