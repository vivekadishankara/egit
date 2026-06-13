use leptos::prelude::*;
#[cfg(feature = "ssr")]
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PullRequest {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub title: String,
    pub body: Option<String>,
    pub head_branch: String,
    pub base_branch: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PullRequestDetail {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub author_id: Uuid,
    pub author_name: String,
    pub title: String,
    pub body: Option<String>,
    pub head_branch: String,
    pub base_branch: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[server]
pub async fn create_pull_request(
    repo_id: Uuid,
    title: String,
    body: Option<String>,
    head_branch: String,
    base_branch: String,
) -> Result<Uuid, ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    let pr_id = sqlx::query_scalar!(
        r#"
        INSERT INTO pull_requests (repo_id, title, body, head_branch, base_branch, status)
        VALUES ($1, $2, $3, $4, $5, 'open')
        RETURNING id
        "#,
        repo_id, title, body, head_branch, base_branch
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    
    Ok(pr_id)
}

#[server]
pub async fn list_pull_requests(repo_id: Uuid, status: Option<String>) -> Result<Vec<PullRequest>, ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    let status_filter = status.unwrap_or_else(|| "open".to_string());
    
    let prs = sqlx::query_as!(
        PullRequest,
        r#"
        SELECT 
            pr.id,
            pr.repo_id,
            pr.author_id,
            u.username as author_name,
            pr.title,
            pr.body,
            pr.head_branch,
            pr.base_branch,
            pr.status,
            pr.created_at,
            pr.updated_at
        FROM pull_requests pr
        JOIN users u ON u.id = pr.author_id
        WHERE pr.repo_id = $1 AND pr.status = $2
        ORDER BY pr.created_at DESC
        "#,
        repo_id,
        status_filter
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    
    Ok(prs)
}

#[server(GetPullRequest)]
pub async fn get_pull_request(pr_id: Uuid) -> Result<PullRequestDetail, ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    let pr = sqlx::query_as!(
        PullRequestDetail,
        r#"
        SELECT 
            pr.id,
            pr.repo_id,
            r.name as repo_name,
            pr.author_id,
            u.username as author_name,
            pr.title,
            pr.body,
            pr.head_branch,
            pr.base_branch,
            pr.status,
            pr.created_at,
            pr.updated_at
        FROM pull_requests pr
        JOIN users u ON u.id = pr.author_id
        JOIN repositories r ON r.id = pr.repo_id
        WHERE pr.id = $1
        "#,
        pr_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    
    Ok(pr)
}

#[server]
pub async fn get_repo_id_by_name(username: String, reponame: String) -> Result<Uuid, ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    let repo_id = sqlx::query_scalar!(
        r#"
        SELECT r.id FROM repositories r
        JOIN users u ON u.id = r.owner_id
        WHERE r.name = $1 AND u.username = $2
        "#,
        reponame, username
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    
    Ok(repo_id)
}

#[server]
pub async fn merge_pull_request(pr_id: Uuid, _user_id: Uuid) -> Result<(), ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    let exists = sqlx::query_scalar!(
        r#"
        SELECT id FROM pull_requests 
        WHERE id = $1 AND status = 'open'
        "#,
        pr_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    
    if exists.is_none() {
        return Err(ServerFnError::new("Pull request not found or already merged/closed"));
    }
    
    sqlx::query!(
        r#"
        UPDATE pull_requests 
        SET status = 'merged', 
            updated_at = NOW() 
        WHERE id = $1
        "#,
        pr_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    
    Ok(())
}

#[server]
pub async fn close_pull_request(pr_id: Uuid) -> Result<(), ServerFnError> {
    let pool = expect_context::<PgPool>();
    
    sqlx::query!(
        r#"
        UPDATE pull_requests 
        SET status = 'closed', 
            updated_at = NOW() 
        WHERE id = $1
        "#,
        pr_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    
    Ok(())
}

#[server]
pub async fn get_branch_list_for_pr(username: String, reponame: String) -> Result<Vec<String>, ServerFnError> {
    let repo_base: String = expect_context::<String>();
    let branches = crate::git::list_branches(&repo_base, &username, &reponame);
    Ok(branches)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestCounts {
    pub open: i64,
    pub merged: i64,
    pub closed: i64,
}

#[server(GetPullRequestCounts, "/api")]
pub async fn get_pull_request_counts(repo_id: Uuid) -> Result<PullRequestCounts, ServerFnError> {
    let pool = expect_context::<PgPool>();

    let open = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM pull_requests WHERE repo_id = $1 AND status = 'open'",
        repo_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .unwrap_or(0);

    let merged = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM pull_requests WHERE repo_id = $1 AND status = 'merged'",
        repo_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .unwrap_or(0);

    let closed = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM pull_requests WHERE repo_id = $1 AND status = 'closed'",
        repo_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .unwrap_or(0);

    Ok(PullRequestCounts { open, merged, closed })
}

#[server(HasPullRequests, "/api")]
pub async fn has_pull_requests(repo_id: Uuid) -> Result<bool, ServerFnError> {
    let pool = expect_context::<PgPool>();

    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM pull_requests WHERE repo_id = $1",
        repo_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .unwrap_or(0);

    Ok(count > 0)
}
