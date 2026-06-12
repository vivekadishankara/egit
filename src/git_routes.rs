use axum::{
    extract::{Path, Query},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use bytes::Bytes;
use serde::Deserialize;
use sqlx::PgPool;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Clone)]
pub struct GitSmartHttpState {
    pub pool: PgPool,
    pub repo_base_path: String,
}

#[derive(Deserialize)]
pub struct ServiceQuery {
    service: Option<String>,
}

/// Build the on-disk path to a bare repository from URL components.
/// The `reponame` may include a `.git` suffix (e.g. `myrepo.git`);
/// we strip it to look up the correct on-disk path.
fn repo_path(base: &str, username: &str, reponame: &str) -> PathBuf {
    let name = reponame.strip_suffix(".git").unwrap_or(reponame);
    PathBuf::from(base).join(username).join(format!("{}.git", name))
}

/// Common Git HTTP no-cache headers.
fn git_headers(content_type: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::EXPIRES,
        HeaderValue::from_static("Fri, 01 Jan 1980 00:00:00 GMT"),
    );
    headers.insert(http::header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(
        http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, max-age=0, must-revalidate"),
    );
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).unwrap(),
    );
    headers
}

/// GET /:username/:reponame.git/info/refs?service=git-upload-pack
/// GET /:username/:reponame.git/info/refs?service=git-receive-pack
pub async fn handle_info_refs(
    Extension(state): Extension<GitSmartHttpState>,
    Path((username, reponame)): Path<(String, String)>,
    Query(query): Query<ServiceQuery>,
    headers: HeaderMap,
) -> Response {
    let service = match query.service {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, "service query parameter required").into_response(),
    };

    let path = repo_path(&state.repo_base_path, &username, &reponame);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "Repository not found").into_response();
    }

    let (cmd, content_type, auth_required) = match service.as_str() {
        "git-upload-pack" => (
            "git-upload-pack",
            "application/x-git-upload-pack-advertisement",
            false,
        ),
        "git-receive-pack" => (
            "git-receive-pack",
            "application/x-git-receive-pack-advertisement",
            true,
        ),
        _ => return (StatusCode::BAD_REQUEST, "Unknown service").into_response(),
    };

    if auth_required && !verify_basic_auth(&state.pool, &headers).await {
        let mut unauth_headers = HeaderMap::new();
        unauth_headers.insert(
            http::header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm=\"eGit\""),
        );
        return (StatusCode::UNAUTHORIZED, unauth_headers, "Unauthorized").into_response();
    }

    let output = match Command::new(cmd)
        .arg("--advertise-refs")
        .arg(&path)
        .output()
        .await
    {
        Ok(out) if out.status.success() => out.stdout,
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::error!("{} --advertise-refs failed: {}", cmd, stderr);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Git command failed").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to execute {}: {}", cmd, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to execute git command: {}", e),
            )
                .into_response();
        }
    };

    (git_headers(content_type), output).into_response()
}

/// POST /:username/:reponame.git/git-upload-pack
pub async fn handle_upload_pack(
    Extension(state): Extension<GitSmartHttpState>,
    Path((username, reponame)): Path<(String, String)>,
    body: Bytes,
) -> Response {
    let path = repo_path(&state.repo_base_path, &username, &reponame);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "Repository not found").into_response();
    }

    let output = run_git_stateless("git-upload-pack", &path, &body).await;

    match output {
        Ok(data) => (git_headers("application/x-git-upload-pack-result"), data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// POST /:username/:reponame.git/git-receive-pack
pub async fn handle_receive_pack(
    Extension(state): Extension<GitSmartHttpState>,
    Path((username, reponame)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let path = repo_path(&state.repo_base_path, &username, &reponame);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "Repository not found").into_response();
    }

    if !verify_basic_auth(&state.pool, &headers).await {
        let mut unauth_headers = HeaderMap::new();
        unauth_headers.insert(
            http::header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm=\"eGit\""),
        );
        return (StatusCode::UNAUTHORIZED, unauth_headers, "Unauthorized").into_response();
    }

    let output = run_git_stateless("git-receive-pack", &path, &body).await;

    match output {
        Ok(data) => (git_headers("application/x-git-receive-pack-result"), data).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Run a Git `--stateless-rpc` subcommand, piping `input` to its stdin
/// and returning the captured stdout on success.
async fn run_git_stateless(
    cmd: &str,
    repo_path: &PathBuf,
    input: &[u8],
) -> Result<Vec<u8>, Response> {
    let mut child = Command::new(cmd)
        .arg("--stateless-rpc")
        .arg(repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            tracing::error!("Failed to spawn {}: {}", cmd, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to spawn {}", cmd),
            )
                .into_response()
        })?;

    use tokio::io::AsyncWriteExt;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input).await.map_err(|e| {
            tracing::error!("Failed to write to {} stdin: {}", cmd, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to communicate with {}", cmd),
            )
                .into_response()
        })?;
    }

    let output = child.wait_with_output().await.map_err(|e| {
        tracing::error!("{} wait error: {}", cmd, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{} failed", cmd),
        )
            .into_response()
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("{} --stateless-rpc failed: {}", cmd, stderr);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{} failed", cmd),
        )
            .into_response());
    }

    Ok(output.stdout)
}

/// Verify HTTP Basic Auth credentials against the users table.
async fn verify_basic_auth(pool: &PgPool, headers: &HeaderMap) -> bool {
    let auth_header = match headers.get(http::header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        Some(v) => v,
        None => return false,
    };

    let encoded = match auth_header.strip_prefix("Basic ") {
        Some(s) => s,
        None => return false,
    };

    use base64::Engine;

    let decoded = match base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
    {
        Some(s) => s,
        None => return false,
    };

    let (username, password) = match decoded.split_once(':') {
        Some((u, p)) => (u.to_string(), p.to_string()),
        None => return false,
    };

    let row = sqlx::query!(
        r#"SELECT id, password_hash FROM users WHERE username = $1"#,
        username
    )
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some(user)) => bcrypt::verify(&password, &user.password_hash).unwrap_or(false),
        _ => false,
    }
}
