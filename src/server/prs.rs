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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PullRequestDetail {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub owner_name: String,
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
    username: String,
    reponame: String,
) -> Result<Uuid, ServerFnError> {
    use crate::auth;
    use axum::http::HeaderMap;

    let pool = expect_context::<PgPool>();
    let headers: HeaderMap = leptos_axum::extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = auth::session_id_from_headers(&headers);
    let session = auth::get_session(&pool, session_id.as_deref())
        .await
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;

    let existing = sqlx::query_scalar!(
        r#"
        SELECT status FROM pull_requests
        WHERE repo_id = $1 AND head_branch = $2 AND base_branch = $3
        LIMIT 1
        "#,
        repo_id, head_branch, base_branch
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    if let Some(status) = existing {
        let msg = match status.as_str() {
            "open" => "An open pull request already exists for these branches",
            "merged" => "A merged pull request already exists for these branches",
            "closed" => "A closed pull request already exists for these branches",
            _ => "A pull request already exists for these branches",
        };
        return Err(ServerFnError::new(msg));
    }

    let pr_id = sqlx::query_scalar!(
        r#"
        INSERT INTO pull_requests (repo_id, author_id, title, body, head_branch, base_branch, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'open')
        RETURNING id
        "#,
        repo_id, session.user_id, title, body, head_branch, base_branch
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    leptos_axum::redirect(&format!("/{username}/{reponame}/pulls/{pr_id}"));
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
            ou.username as owner_name,
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
        JOIN users ou ON ou.id = r.owner_id
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
pub async fn merge_pull_request(pr_id: Uuid) -> Result<(), ServerFnError> {
    use crate::auth;
    use axum::http::HeaderMap;

    let pool = expect_context::<PgPool>();
    let repo_base: String = expect_context::<String>();
    let headers: HeaderMap = leptos_axum::extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = auth::session_id_from_headers(&headers);
    let session = auth::get_session(&pool, session_id.as_deref())
        .await
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;

    let pr = sqlx::query!(
        r#"
        SELECT pr.repo_id, pr.author_id, pr.head_branch, pr.base_branch,
               r.name as repo_name, u.username as owner_name
        FROM pull_requests pr
        JOIN repositories r ON r.id = pr.repo_id
        JOIN users u ON u.id = r.owner_id
        WHERE pr.id = $1 AND pr.status = 'open'
        "#,
        pr_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("Pull request not found or already merged/closed"))?;

    if pr.author_id != session.user_id {
        return Err(ServerFnError::new("Only the author can merge this pull request"));
    }

    let repo_path = crate::git::repo_path(&repo_base, &pr.owner_name, &pr.repo_name);
    let base_ref = format!("refs/heads/{}", pr.base_branch);
    let head_ref = format!("refs/heads/{}", pr.head_branch);

    let merge_output = std::process::Command::new("git")
        .args(["merge-tree", "--write-tree", &base_ref, &head_ref])
        .env("GIT_DIR", &repo_path)
        .output()
        .map_err(|e| ServerFnError::new(format!("git merge-tree failed: {e}")))?;

    if !merge_output.status.success() {
        let stderr = String::from_utf8_lossy(&merge_output.stderr);
        return Err(ServerFnError::new(format!(
            "Merge conflict: {stderr}"
        )));
    }

    let merged_tree = String::from_utf8_lossy(&merge_output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    if merged_tree.is_empty() {
        return Err(ServerFnError::new("Merge produced no tree"));
    }

    let base_oid = std::process::Command::new("git")
        .args(["rev-parse", &base_ref])
        .env("GIT_DIR", &repo_path)
        .output()
        .map_err(|e| ServerFnError::new(format!("git rev-parse failed: {e}")))?;
    let base_oid = String::from_utf8_lossy(&base_oid.stdout).trim().to_string();

    let head_oid = std::process::Command::new("git")
        .args(["rev-parse", &head_ref])
        .env("GIT_DIR", &repo_path)
        .output()
        .map_err(|e| ServerFnError::new(format!("git rev-parse failed: {e}")))?;
    let head_oid = String::from_utf8_lossy(&head_oid.stdout).trim().to_string();

    let merge_msg = format!("Merge pull request #{} from {}/{}/{}", pr_id, pr.owner_name, pr.repo_name, pr.head_branch);

    let commit_output = std::process::Command::new("git")
        .args([
            "commit-tree",
            &merged_tree,
            "-p",
            &base_oid,
            "-p",
            &head_oid,
            "-m",
            &merge_msg,
        ])
        .env("GIT_DIR", &repo_path)
        .output()
        .map_err(|e| ServerFnError::new(format!("git commit-tree failed: {e}")))?;

    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        return Err(ServerFnError::new(format!("Failed to create merge commit: {stderr}")));
    }

    let merge_commit = String::from_utf8_lossy(&commit_output.stdout).trim().to_string();

    let update_status = std::process::Command::new("git")
        .args(["update-ref", &base_ref, &merge_commit])
        .env("GIT_DIR", &repo_path)
        .output()
        .map_err(|e| ServerFnError::new(format!("git update-ref failed: {e}")))?;

    if !update_status.status.success() {
        let stderr = String::from_utf8_lossy(&update_status.stderr);
        return Err(ServerFnError::new(format!("Failed to update branch ref: {stderr}")));
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
    use crate::auth;
    use axum::http::HeaderMap;

    let pool = expect_context::<PgPool>();
    let headers: HeaderMap = leptos_axum::extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let session_id = auth::session_id_from_headers(&headers);
    let session = auth::get_session(&pool, session_id.as_deref())
        .await
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;

    let author_id = sqlx::query_scalar!(
        r#"
        SELECT author_id FROM pull_requests 
        WHERE id = $1 AND status = 'open'
        "#,
        pr_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    .ok_or_else(|| ServerFnError::new("Pull request not found or already merged/closed"))?;

    if author_id != session.user_id {
        return Err(ServerFnError::new("Only the author can close this pull request"));
    }

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

#[server(GetPrDiff, "/api")]
pub async fn get_pr_diff(
    username: String,
    reponame: String,
    head_branch: String,
    base_branch: String,
) -> Result<Vec<crate::diff::DiffFile>, ServerFnError> {
    let repo_base: String = expect_context::<String>();
    crate::git::get_pr_diff(&repo_base, &username, &reponame, &head_branch, &base_branch)
        .map_err(|e| ServerFnError::new(format!("Failed to get diff: {e}")))
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
