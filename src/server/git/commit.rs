use std::path::Path;

use crate::diff::DiffFile;

use super::repo_path;

pub struct CommitDetail {
    pub id: String,
    pub short_id: String,
    pub author_name: String,
    pub author_email: String,
    pub message: String,
    pub message_body: String,
    pub timestamp: i64,
    pub diff: Vec<DiffFile>,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub author_name: String,
    pub author_email: String,
    pub message: String,
    pub timestamp: i64,
}

pub fn get_commit_detail(
    repo_base: &str,
    username: &str,
    reponame: &str,
    commit_id: &str,
) -> anyhow::Result<CommitDetail> {
    let path = repo_path(repo_base, username, reponame);
    let repo = gix::open(&path)?;
    let oid: gix::hash::ObjectId = commit_id.parse()?;

    let walk = repo.rev_walk([oid]).all()?;
    let entry = walk
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Commit not found"))??;

    let commit_obj = entry.object()?;
    let msg = commit_obj.message_raw_sloppy().to_string();
    let author = commit_obj.author()?;
    let time = commit_obj.time()?;
    let hex = entry.id.to_string();

    let (first_line, body) = match msg.split_once('\n') {
        Some((f, b)) => (f.to_string(), b.trim().to_string()),
        None => (msg.clone(), String::new()),
    };

    let raw_diff = get_commit_diff_internal(&path, &hex)?;
    let diff = crate::diff::parse_diff(&raw_diff);
    let diff = super::diff_ops::highlight_diff_files(diff);

    let files_changed = diff.len();
    let insertions: u32 = diff.iter().map(|f| f.stats.additions).sum();
    let deletions: u32 = diff.iter().map(|f| f.stats.deletions).sum();

    Ok(CommitDetail {
        id: hex.clone(),
        short_id: hex[..7].to_string(),
        author_name: author.name.to_string(),
        author_email: author.email.to_string(),
        message: first_line,
        message_body: body,
        timestamp: time.seconds,
        diff,
        files_changed,
        insertions: insertions as usize,
        deletions: deletions as usize,
    })
}

fn get_commit_diff_internal(git_dir: &Path, oid: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["show", oid, "--format="])
        .env("GIT_DIR", git_dir)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git show failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn get_commit_log(
    repo_base: &str,
    username: &str,
    reponame: &str,
    revision: &str,
) -> anyhow::Result<Vec<CommitInfo>> {
    let repo = gix::open(repo_path(repo_base, username, reponame))?;
    let commit = super::resolve_commit(&repo, revision)?;
    let oid = commit.id().detach();

    let walk = repo.rev_walk([oid]).all()?;

    let mut commits = Vec::new();
    for info in walk {
        let info = info?;
        let commit_obj = info.object()?;
        let msg = commit_obj.message_raw_sloppy().to_string();
        let first_line = msg.lines().next().unwrap_or(&msg).to_string();
        let author = commit_obj.author()?;
        let time = commit_obj.time()?;
        let hex = info.id.to_string();

        commits.push(CommitInfo {
            id: hex.clone(),
            short_id: hex[..7].to_string(),
            author_name: author.name.to_string(),
            author_email: author.email.to_string(),
            message: first_line,
            timestamp: time.seconds,
        });
    }

    Ok(commits)
}
