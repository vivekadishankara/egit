use leptos::prelude::LeptosOptions;
use sqlx::PgPool;

#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub pool: PgPool,
    pub leptos_options: LeptosOptions,
    pub repo_base_path: String,
}
