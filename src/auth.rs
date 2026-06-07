use sqlx::PgPool;
use uuid::Uuid;

pub struct Session {
    pub user_id: Uuid,
    pub username: String,
    pub theme: String,
}

/// Extract session from request cookies. Returns None if unauthenticated.
pub async fn get_session(_pool: &PgPool, _session_id: Option<&str>) -> Option<Session> {
    // TODO: implement in step 3 (auth)
    None
}
