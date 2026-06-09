use axum::http::HeaderMap;
use axum_extra::extract::cookie::{Cookie, SameSite};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub const SESSION_COOKIE: &str = "egit_session";
pub const SESSION_DURATION_DAYS: i64 = 30;

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: Uuid,
    pub username: String,
    pub theme: String,
}

/// Look up a session by cookie value. Returns None if missing / expired.
pub async fn get_session(pool: &PgPool, session_id: Option<&str>) -> Option<Session> {
    let session_id = session_id?;
    let session_uuid = Uuid::parse_str(session_id).ok()?;
    let now = OffsetDateTime::now_utc();

    let row = sqlx::query!(
        r#"
        SELECT s.user_id, u.username, u.theme
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.id = $1 AND s.expires_at > $2
        "#,
        session_uuid,
        now
    )
    .fetch_optional(pool)
    .await
    .ok()??;

    Some(Session {
        user_id: row.user_id,
        username: row.username,
        theme: row.theme,
    })
}

/// Create a new session row and return its UUID as a string.
pub async fn create_session(pool: &PgPool, user_id: Uuid) -> anyhow::Result<String> {
    let expires_at = OffsetDateTime::now_utc() + Duration::days(SESSION_DURATION_DAYS);

    let row = sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, expires_at)
        VALUES ($1, $2)
        RETURNING id
        "#,
        user_id,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id.to_string())
}

/// Delete a session row (logout).
pub async fn delete_session(pool: &PgPool, session_id: &str) -> anyhow::Result<()> {
    if let Ok(id) = Uuid::parse_str(session_id) {
        sqlx::query!("DELETE FROM sessions WHERE id = $1", id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Build a Set-Cookie header value for a new session.
pub fn make_session_cookie(session_id: &str) -> String {
    let expires = OffsetDateTime::now_utc() + Duration::days(SESSION_DURATION_DAYS);
    Cookie::build((SESSION_COOKIE, session_id.to_string()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .expires(expires)
        .to_string()
}

/// Build a Set-Cookie header value that clears the session cookie.
pub fn clear_session_cookie() -> String {
    Cookie::build((SESSION_COOKIE, ""))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::seconds(0))
        .to_string()
}

/// Extract the session cookie value from request headers.
pub fn session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    cookie_header
        .split(';')
        .map(|s| s.trim())
        .find_map(|part| {
            let (name, value) = part.split_once('=')?;
            if name.trim() == SESSION_COOKIE {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
}
