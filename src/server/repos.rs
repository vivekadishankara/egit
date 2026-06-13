use leptos::prelude::*;

#[server(DeleteRepo, "/api")]
pub async fn delete_repo(
    username: String,
    reponame: String,
) -> Result<(), ServerFnError> {
    use crate::auth;
    use axum::http::HeaderMap;
    use sqlx::PgPool;
    use std::path::PathBuf;

    let pool = expect_context::<PgPool>();
    let repo_base: String = expect_context::<String>();

    let headers: HeaderMap = leptos_axum::extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = auth::session_id_from_headers(&headers);
    let session = auth::get_session(&pool, session_id.as_deref())
        .await
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;

    let row = sqlx::query!(
        r#"
        SELECT r.id, r.owner_id
        FROM repositories r
        JOIN users u ON u.id = r.owner_id
        WHERE r.name = $1 AND u.username = $2
        "#,
        reponame,
        username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("Repository not found"))?;

    if row.owner_id != session.user_id {
        return Err(ServerFnError::new("Only the repository owner can delete this repository"));
    }

    let repo_path = PathBuf::from(&repo_base)
        .join(&username)
        .join(format!("{}.git", reponame));
    if repo_path.exists() {
        std::fs::remove_dir_all(&repo_path)
            .map_err(|e| ServerFnError::new(format!("Failed to delete repository on disk: {e}")))?;
    }

    sqlx::query!(
        "DELETE FROM repositories WHERE id = $1",
        row.id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    leptos_axum::redirect(&format!("/{}", session.username));
    Ok(())
}
