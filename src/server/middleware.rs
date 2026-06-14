use axum::{extract::Request, middleware, response::IntoResponse};

use super::session;

#[derive(Clone)]
pub struct ResolvedTheme(pub String);

pub async fn theme_middleware(
    pool: sqlx::PgPool,
    mut req: Request,
    next: middleware::Next,
) -> impl IntoResponse {
    let sid = session::session_id_from_headers(req.headers());
    let theme = match session::get_session(&pool, sid.as_deref()).await {
        Some(s) => s.theme,
        None => "dark".to_string(),
    };
    req.extensions_mut().insert(ResolvedTheme(theme));
    next.run(req).await
}
