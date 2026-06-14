use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCreated {
    pub username: String,
    pub reponame: String,
}

/// Server function: create a new repository.
#[server(CreateRepo, "/api")]
pub async fn create_repo(
    name: String,
    description: Option<String>,
    is_private: bool,
) -> Result<RepoCreated, ServerFnError> {
    use crate::server::session;
    use crate::server::git;
    use axum::http::HeaderMap;
    use sqlx::PgPool;
    use std::path::PathBuf;

    let pool = expect_context::<PgPool>();
    let repo_base: String = expect_context::<String>();

    let headers: HeaderMap = leptos_axum::extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = session::session_id_from_headers(&headers);
    let session = session::get_session(&pool, session_id.as_deref())
        .await
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;

    let name = name.trim().to_lowercase();
    if name.is_empty() || name.len() > 100 {
        return Err(ServerFnError::new("Repository name must be 1–100 characters"));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(ServerFnError::new(
            "Repository name may only contain letters, numbers, hyphens, and underscores",
        ));
    }

    let repo_path = PathBuf::from(&repo_base)
        .join(&session.username)
        .join(format!("{}.git", name));

    if repo_path.exists() {
        return Err(ServerFnError::new("Repository already exists"));
    }

    git::init_bare(&repo_path).map_err(|e| {
        ServerFnError::new(format!("Failed to initialize git repository: {e}"))
    })?;

    let description = description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());

    sqlx::query!(
        r#"
        INSERT INTO repositories (owner_id, name, description, is_private)
        VALUES ($1, $2, $3, $4)
        "#,
        session.user_id,
        name,
        description,
        is_private
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") {
            let _ = std::fs::remove_dir_all(&repo_path);
            ServerFnError::new("You already have a repository with this name")
        } else {
            let _ = std::fs::remove_dir_all(&repo_path);
            ServerFnError::new(format!("Database error: {e}"))
        }
    })?;

    leptos_axum::redirect(&format!("/{}/{}", session.username, name));
    Ok(RepoCreated {
        username: session.username,
        reponame: name,
    })
}

#[component]
pub fn CreateRepoPage() -> impl IntoView {
    let create = ServerAction::<CreateRepo>::new();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(created)) = create.value().get() {
            navigate(
                &format!("/{}/{}", created.username, created.reponame),
                Default::default(),
            );
        }
    });

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <h1 class="auth-title">"Create a new repository"</h1>

                {move || {
                    create.value().get().and_then(|r| r.err()).map(|e| view! {
                        <div class="alert-error">{e.to_string()}</div>
                    })
                }}

                <ActionForm action=create>
                    <div class="form-group">
                        <label class="form-label" for="name">"Repository name"</label>
                        <input
                            class="form-input"
                            type="text"
                            id="name"
                            name="name"
                            required
                            autocomplete="off"
                            placeholder="e.g. my-project"
                            minlength="1"
                            maxlength="100"
                            pattern="[a-zA-Z0-9_\\-]+"
                        />
                        <p class="form-hint">"Letters, numbers, hyphens, and underscores only."</p>
                    </div>

                    <div class="form-group">
                        <label class="form-label" for="description">"Description"</label>
                        <textarea
                            class="form-input"
                            id="description"
                            name="description"
                            rows="3"
                            placeholder="A short description of your repository"
                        ></textarea>
                    </div>

                    <div class="form-group">
                        <label class="flex items-center gap-2 cursor-pointer">
                            <input type="hidden" name="is_private" value="false"/>
                            <input
                                type="checkbox"
                                id="is_private"
                                name="is_private"
                                value="true"
                                class="rounded border-theme bg-surface-secondary text-accent"
                            />
                            <span class="text-sm">"Private repository"</span>
                        </label>
                        <p class="form-hint ml-6">
                            "Only you and collaborators can see private repositories."
                        </p>
                    </div>

                    <button
                        class="btn-primary w-full"
                        type="submit"
                        disabled=move || create.pending().get()
                    >
                        {move || if create.pending().get() { "Creating repository…" } else { "Create repository" }}
                    </button>
                </ActionForm>
            </div>
        </div>
    }
}
