use leptos::prelude::*;

use crate::components::theme_switcher::ThemeSwitcher;
use crate::pages::auth::{get_current_user, LogoutUser};

#[component]
pub fn Navbar() -> impl IntoView {
    let current_user = Resource::new(|| (), |_| get_current_user());
    let logout = ServerAction::<LogoutUser>::new();

    // Re-fetch user after logout
    Effect::new(move |_| {
        if logout.value().get().is_some() {
            current_user.refetch();
        }
    });

    view! {
        <nav class="navbar">
            <div class="navbar-inner">
                <a href="/" class="navbar-brand">
                    <svg class="w-7 h-7" viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
                        <defs>
                            <linearGradient id="ringBlue" x1="0%" y1="0%" x2="100%" y2="100%">
                                <stop offset="0%" stop-color="#4d95ff"/>
                                <stop offset="100%" stop-color="#2270ea"/>
                            </linearGradient>
                        </defs>
                        <rect width="1024" height="1024" fill="transparent"/>
                        <circle cx="512" cy="512" r="350" fill="none" stroke="url(#ringBlue)" stroke-width="110"/>
                        <path d="M286 409 L370 378 H615 C623 378 626 382 626 389 V419 H753 C744 460 707 486 630 497 C588 503 570 530 570 590 H648 V646 H582 C568 603 462 603 448 646 H362 V590 C362 590 445 571 445 526 C445 487 389 459 286 451 Z" fill="#3f87f5" stroke="#75adff" stroke-width="2"/>
                        <rect x="375" y="392" width="40" height="16" fill="#82b7ff" opacity="0.8"/>
                    </svg>
                    <span class="text-accent font-bold text-xl ml-2">"eGit"</span>
                </a>

                <div class="navbar-links">
                    <Suspense fallback=|| view! { <span class="navbar-link-placeholder"/> }>
                        {move || {
                            current_user.get().map(|result| {
                                match result {
                                    Ok(Some(user)) => view! {
                                        <div class="navbar-user">
                                            <ThemeSwitcher current_theme=user.theme.clone()/>
                                            <a
                                                href="/repos/new"
                                                class="btn-primary text-sm"
                                                title="Create a new repository"
                                            >
                                                "+ New"
                                            </a>
                                            <a
                                                href=format!("/{}", user.username)
                                                class="navbar-link font-medium"
                                            >
                                                {user.username.clone()}
                                            </a>
                                            <ActionForm action=logout>
                                                <button type="submit" class="btn-secondary text-sm">
                                                    "Sign out"
                                                </button>
                                            </ActionForm>
                                        </div>
                                    }.into_any(),
                                    _ => view! {
                                        <div class="navbar-user">
                                            <a href="/login" class="navbar-link">"Sign in"</a>
                                            <a href="/register" class="btn-primary text-sm">"Sign up"</a>
                                        </div>
                                    }.into_any(),
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>
        </nav>
    }
}
