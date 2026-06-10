pub mod login;
pub mod register;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthError(pub String);

/// Server function: register a new user.
#[server(RegisterUser, "/api")]
pub async fn register_user(
    username: String,
    email: String,
    password: String,
) -> Result<(), ServerFnError> {
    use crate::auth;
    use bcrypt::{hash, DEFAULT_COST};
    use leptos_axum::ResponseOptions;
    use sqlx::PgPool;

    let pool = expect_context::<PgPool>();

    // Basic validation
    if username.len() < 3 || username.len() > 39 {
        return Err(ServerFnError::new("Username must be 3–39 characters"));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(ServerFnError::new(
            "Username may only contain letters, numbers, and hyphens",
        ));
    }
    if password.len() < 8 {
        return Err(ServerFnError::new("Password must be at least 8 characters"));
    }

    let password_hash = hash(password.as_bytes(), DEFAULT_COST)
        .map_err(|e| ServerFnError::new(format!("Hash error: {e}")))?;

    let user = sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        username.to_lowercase(),
        email.to_lowercase(),
        password_hash
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") {
            ServerFnError::new("Username or email already taken")
        } else {
            ServerFnError::new(format!("Database error: {e}"))
        }
    })?;

    let session_id = auth::create_session(&pool, user.id)
        .await
        .map_err(|e| ServerFnError::new(format!("Session error: {e}")))?;

    let response = expect_context::<ResponseOptions>();
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&auth::make_session_cookie(&session_id))
            .map_err(|e| ServerFnError::new(format!("Cookie error: {e}")))?,
    );

    leptos_axum::redirect("/");
    Ok(())
}

/// Server function: log in an existing user.
#[server(LoginUser, "/api")]
pub async fn login_user(
    username_or_email: String,
    password: String,
) -> Result<(), ServerFnError> {
    use crate::auth;
    use bcrypt::verify;
    use leptos_axum::ResponseOptions;
    use sqlx::PgPool;

    let pool = expect_context::<PgPool>();

    let user = sqlx::query!(
        r#"
        SELECT id, password_hash
        FROM users
        WHERE username = $1 OR email = $1
        "#,
        username_or_email.to_lowercase()
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("Invalid username or password"))?;

    let valid = verify(password.as_bytes(), &user.password_hash)
        .map_err(|e| ServerFnError::new(format!("Verify error: {e}")))?;

    if !valid {
        return Err(ServerFnError::new("Invalid username or password"));
    }

    let session_id = auth::create_session(&pool, user.id)
        .await
        .map_err(|e| ServerFnError::new(format!("Session error: {e}")))?;

    let response = expect_context::<ResponseOptions>();
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&auth::make_session_cookie(&session_id))
            .map_err(|e| ServerFnError::new(format!("Cookie error: {e}")))?,
    );

    leptos_axum::redirect("/");
    Ok(())
}

/// Server function: log out the current user.
#[server(LogoutUser, "/api")]
pub async fn logout_user() -> Result<(), ServerFnError> {
    use crate::auth;
    use axum::http::HeaderMap;
    use leptos_axum::ResponseOptions;
    use sqlx::PgPool;

    let pool = expect_context::<PgPool>();
    let headers: HeaderMap = leptos_axum::extract().await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = auth::session_id_from_headers(&headers);

    if let Some(sid) = session_id {
        let _ = auth::delete_session(&pool, &sid).await;
    }

    let response = expect_context::<ResponseOptions>();
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&auth::clear_session_cookie())
            .map_err(|e| ServerFnError::new(format!("Cookie error: {e}")))?,
    );

    leptos_axum::redirect("/");
    Ok(())
}

/// Server function: get the currently logged-in user info.
#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<CurrentUser>, ServerFnError> {
    use crate::auth;
    use axum::http::HeaderMap;
    use sqlx::PgPool;

    let pool = expect_context::<PgPool>();
    let headers: HeaderMap = leptos_axum::extract().await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = auth::session_id_from_headers(&headers);

    let session = auth::get_session(&pool, session_id.as_deref()).await;

    Ok(session.map(|s| CurrentUser {
        username: s.username,
        theme: s.theme,
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub username: String,
    pub theme: String,
}

/// The six supported themes (kept in sync with input.css `[data-theme=...]` blocks).
pub const THEMES: &[(&str, &str)] = &[
    ("dark",      "Dark"),
    ("light",     "Light"),
    ("dracula",   "Dracula"),
    ("nord",      "Nord"),
    ("solarized", "Solarized"),
    ("gruvbox",   "Gruvbox"),
];

/// Server function: persist a new theme for the current user.
#[server(SetTheme, "/api")]
pub async fn set_theme(theme: String) -> Result<(), ServerFnError> {
    use crate::auth;
    use axum::http::HeaderMap;
    use sqlx::PgPool;

    // Validate the theme name before touching the DB.
    if !THEMES.iter().any(|(id, _)| *id == theme) {
        return Err(ServerFnError::new("Unknown theme"));
    }

    let pool = expect_context::<PgPool>();
    let headers: HeaderMap = leptos_axum::extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = auth::session_id_from_headers(&headers);
    let session = auth::get_session(&pool, session_id.as_deref())
        .await
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;

    sqlx::query!(
        "UPDATE users SET theme = $1 WHERE id = $2",
        theme,
        session.user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB error: {e}")))?;

    Ok(())
}
