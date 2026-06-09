use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use super::RegisterUser;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register = ServerAction::<RegisterUser>::new();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(())) = register.value().get() {
            navigate("/", Default::default());
        }
    });

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <h1 class="auth-title">"Create your account"</h1>

                {move || {
                    register.value().get().and_then(|r| r.err()).map(|e| view! {
                        <div class="alert-error">{e.to_string()}</div>
                    })
                }}

                <ActionForm action=register>
                    <div class="form-group">
                        <label class="form-label" for="username">"Username"</label>
                        <input
                            class="form-input"
                            type="text"
                            id="username"
                            name="username"
                            required
                            autocomplete="username"
                            placeholder="e.g. octocat"
                            minlength="3"
                            maxlength="39"
                            pattern="[a-zA-Z0-9\\-]+"
                        />
                        <p class="form-hint">"Letters, numbers, and hyphens only. 3–39 characters."</p>
                    </div>

                    <div class="form-group">
                        <label class="form-label" for="email">"Email address"</label>
                        <input
                            class="form-input"
                            type="email"
                            id="email"
                            name="email"
                            required
                            autocomplete="email"
                            placeholder="you@example.com"
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
                            autocomplete="new-password"
                            placeholder="At least 8 characters"
                            minlength="8"
                        />
                    </div>

                    <button
                        class="btn-primary w-full"
                        type="submit"
                        disabled=move || register.pending().get()
                    >
                        {move || if register.pending().get() { "Creating account…" } else { "Create account" }}
                    </button>
                </ActionForm>

                <p class="auth-footer">
                    "Already have an account? "
                    <a href="/login" class="auth-link">"Sign in"</a>
                </p>
            </div>
        </div>
    }
}
