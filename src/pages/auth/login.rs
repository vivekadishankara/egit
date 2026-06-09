use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use super::LoginUser;

#[component]
pub fn LoginPage() -> impl IntoView {
    let login = ServerAction::<LoginUser>::new();
    let navigate = use_navigate();

    // Redirect on success
    Effect::new(move |_| {
        if let Some(Ok(())) = login.value().get() {
            navigate("/", Default::default());
        }
    });

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <h1 class="auth-title">"Sign in to eGit"</h1>

                {move || {
                    login.value().get().and_then(|r| r.err()).map(|e| view! {
                        <div class="alert-error">{e.to_string()}</div>
                    })
                }}

                <ActionForm action=login>
                    <div class="form-group">
                        <label class="form-label" for="username_or_email">"Username or email"</label>
                        <input
                            class="form-input"
                            type="text"
                            id="username_or_email"
                            name="username_or_email"
                            required
                            autocomplete="username"
                            placeholder="Enter your username or email"
                        />
                    </div>

                    <div class="form-group">
                        <label class="form-label" for="password">"Password"</label>
                        <input
                            class="form-input"
                            type="password"
                            id="password"
                            name="password"
                            required
                            autocomplete="current-password"
                            placeholder="Enter your password"
                        />
                    </div>

                    <button
                        class="btn-primary w-full"
                        type="submit"
                        disabled=move || login.pending().get()
                    >
                        {move || if login.pending().get() { "Signing in…" } else { "Sign in" }}
                    </button>
                </ActionForm>

                <p class="auth-footer">
                    "New to eGit? "
                    <a href="/register" class="auth-link">"Create an account"</a>
                </p>
            </div>
        </div>
    }
}
