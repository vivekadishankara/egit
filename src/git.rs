use gix::hash::ObjectId;
use std::path::{Path, PathBuf};

use crate::diff::DiffFile;

/// Initialize a new bare git repository at the given path.
pub fn init_bare(path: &Path) -> anyhow::Result<()> {
    use std::process::Command;

    std::fs::create_dir_all(path.parent().unwrap_or(path))?;
    gix::init_bare(path)?;

    // Explicitly set HEAD to refs/heads/main.
    std::fs::write(path.join("HEAD"), b"ref: refs/heads/main\n")?;

    // Create an initial empty commit so refs/heads/main exists as a valid ref.
    // Without it, git-upload-pack can't advertise a symref target, and the
    // client falls back to init.defaultBranch (typically "master").
    let empty_tree = Command::new("git")
        .args(["hash-object", "-t", "tree", "--stdin", "-w"])
        .env("GIT_DIR", path)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| anyhow::anyhow!("git hash-object: {e}"))?;

    if !empty_tree.status.success() {
        let stderr = String::from_utf8_lossy(&empty_tree.stderr);
        anyhow::bail!("empty tree creation failed: {stderr}");
    }

    let empty_oid = std::str::from_utf8(&empty_tree.stdout)
        .map_err(|_| anyhow::anyhow!("bad utf-8"))?
        .trim()
        .to_string();

    let output = Command::new("git")
        .args(["commit-tree", "-m", "Initial commit", &empty_oid])
        .env("GIT_DIR", path)
        .output()
        .map_err(|e| anyhow::anyhow!("git commit-tree: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("initial commit failed: {stderr}");
    }

    let commit_oid = std::str::from_utf8(&output.stdout)
        .map_err(|_| anyhow::anyhow!("bad utf-8"))?
        .trim()
        .to_string();

    Command::new("git")
        .args(["update-ref", "refs/heads/main", &commit_oid])
        .env("GIT_DIR", path)
        .output()
        .map_err(|e| anyhow::anyhow!("git update-ref: {e}"))?;

    tracing::info!("Initialized bare repo at {:?}", path);
    Ok(())
}

/// Build the on-disk path for a repository.
pub fn repo_path(repo_base: &str, username: &str, reponame: &str) -> PathBuf {
    PathBuf::from(repo_base)
        .join(username)
        .join(format!("{}.git", reponame))
}

/// Resolve a revision string ("HEAD" or a branch name) to a `gix::Commit`.
fn resolve_commit<'a>(
    repo: &'a gix::Repository,
    revision: &str,
) -> anyhow::Result<gix::Commit<'a>> {
    if revision.is_empty() || revision == "HEAD" {
        // Try HEAD first (e.g. "refs/heads/main")
        if let Ok(mut head) = repo.head() {
            if let Ok(c) = head.peel_to_commit_in_place() {
                return Ok(c.clone());
            }
        }
        // HEAD is unborn (branch name mismatch); try common branch names
        for name in ["main", "master"] {
            let branch = format!("refs/heads/{name}");
            if let Ok(mut reference) = repo.find_reference(&branch) {
                if let Ok(commit) = reference.peel_to_commit() {
                    return Ok(commit);
                }
            }
        }
        anyhow::bail!("Repository has no commits yet")
    } else {
        let mut reference = repo.find_reference(&format!("refs/heads/{revision}"))?;
        reference.peel_to_commit().map_err(Into::into)
    }
}

/// Navigate to a subdirectory tree by path components from a root tree.
fn navigate_to_tree<'a>(
    repo: &'a gix::Repository,
    path: &str,
    tree: &gix::Tree<'a>,
) -> anyhow::Result<gix::Tree<'a>> {
    if path.is_empty() {
        return Ok(tree.clone());
    }
    let parts: Vec<&str> = path.split('/').collect();
    let mut current = tree.clone();
    for part in &parts {
        let object_id = {
            let entry = current
                .iter()
                .filter_map(|e| e.ok())
                .find(|e| e.filename() == *part)
                .ok_or_else(|| anyhow::anyhow!("entry not found: {}", path))?;
            ObjectId::from(entry.oid())
        };
        current = repo.find_object(object_id)?.into_tree();
    }
    Ok(current)
}

/// Find the final entry (leaf) at the given path within a tree.
/// Returns the object id and whether it is a directory.
fn find_leaf_entry<'a>(
    repo: &'a gix::Repository,
    path: &str,
    tree: &gix::Tree<'a>,
) -> anyhow::Result<(ObjectId, bool)> {
    if path.is_empty() {
        anyhow::bail!("empty path");
    }
    let parts: Vec<&str> = path.split('/').collect();
    let mut current = tree.clone();

    for (i, part) in parts.iter().enumerate() {
        let entry = {
            current
                .iter()
                .filter_map(|e| e.ok())
                .find(|e| e.filename() == *part)
                .ok_or_else(|| anyhow::anyhow!("entry not found: {}", path))?
        };
        if i == parts.len() - 1 {
            let is_dir = entry.mode().is_tree();
            let object_id = ObjectId::from(entry.oid());
            return Ok((object_id, is_dir));
        }
        let object_id = ObjectId::from(entry.oid());
        current = repo.find_object(object_id)?.into_tree();
    }

    anyhow::bail!("empty path");
}

/// List entries in a directory at the given revision and path.
pub fn list_directory(
    repo_base: &str,
    username: &str,
    reponame: &str,
    revision: &str,
    path: &str,
) -> anyhow::Result<Vec<(String, bool)>> {
    let repo = gix::open(repo_path(repo_base, username, reponame))?;
    let commit = resolve_commit(&repo, revision)?;
    let tree = commit.tree()?;
    let target = navigate_to_tree(&repo, path, &tree)?;

    let mut entries: Vec<(String, bool)> = target
        .iter()
        .filter_map(|e| e.ok())
        .map(|e| {
            let name = e.filename().to_string();
            let is_dir = e.mode().is_tree();
            (name, is_dir)
        })
        .collect();

    entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(entries)
}

/// Read a file blob at the given revision and path.
/// Returns (content_bytes, file_extension).
pub fn read_file(
    repo_base: &str,
    username: &str,
    reponame: &str,
    revision: &str,
    path: &str,
) -> anyhow::Result<(Vec<u8>, String)> {
    let repo = gix::open(repo_path(repo_base, username, reponame))?;
    let commit = resolve_commit(&repo, revision)?;
    let tree = commit.tree()?;
    let (oid, is_dir) = find_leaf_entry(&repo, path, &tree)?;
    if is_dir {
        anyhow::bail!("expected file, found directory");
    }
    let obj = repo.find_object(oid)?;
    let blob = obj.into_blob();

    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();

    Ok((blob.data.to_vec(), ext))
}

/// Try to find and read a README file from the root of the default branch.
pub fn read_readme(
    repo_base: &str,
    username: &str,
    reponame: &str,
    branch: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let path = repo_path(repo_base, username, reponame);
    let repo = match gix::open(&path) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let revision = branch.unwrap_or("HEAD");
    let commit = match resolve_commit(&repo, revision) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    let tree = commit.tree()?;

    let candidates = [
        "README.md",
        "README",
        "Readme.md",
        "readme.md",
        "README.markdown",
        "README.txt",
    ];

    for name in &candidates {
        if let Some(entry) = tree
            .iter()
            .filter_map(|e| e.ok())
            .find(|e| e.filename() == *name)
        {
            if let Ok(obj) = entry.object() {
                let blob = obj.into_blob();
                let content = String::from_utf8_lossy(&blob.data).to_string();
                return Ok(Some(content));
            }
        }
    }

    Ok(None)
}

/// Resolve HEAD to the actual branch name (e.g. "master" or "main").
pub fn get_default_branch(repo_base: &str, username: &str, reponame: &str) -> Option<String> {
    let path = repo_path(repo_base, username, reponame);
    let repo = gix::open(&path).ok()?;

    // First try HEAD's symbolic target
    if let Ok(head) = repo.head() {
        if let Some(refname) = head.referent_name() {
            let short = refname.shorten().to_string();
            // Verify the branch actually exists
            if repo.find_reference(&format!("refs/heads/{short}")).is_ok() {
                return Some(short);
            }
        }
    }

    // HEAD is unborn; find the first branch that has commits
    for name in ["master", "main"] {
        let branch = format!("refs/heads/{name}");
        if repo.find_reference(&branch).is_ok() {
            return Some(name.to_string());
        }
    }

    None
}

/// Check if a repository has at least one commit.
pub fn has_commits(repo_base: &str, username: &str, reponame: &str) -> bool {
    let path = repo_path(repo_base, username, reponame);
    let Ok(repo) = gix::open(&path) else {
        return false;
    };
    if let Ok(mut head) = repo.head() {
        if head.peel_to_commit_in_place().is_ok() {
            return true;
        }
    }
    // HEAD is unborn; check all branches for commits
    if let Ok(platform) = repo.references() {
        if let Ok(iter) = platform.all() {
            for r in iter.flatten().filter_map(|mut r| r.peel_to_commit().ok()) {
                let _ = r;
                return true;
            }
        }
    }
    false
}

/// Full detail for a single commit including its diff.
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

/// Get the full detail and diff for a single commit.
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
    let diff = highlight_diff_files(diff);

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

/// Internal helper: get the raw diff for a commit via `git show`.
fn get_commit_diff_internal(git_dir: &std::path::Path, oid: &str) -> anyhow::Result<String> {
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

/// Information about a single commit in the log.
pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub author_name: String,
    pub author_email: String,
    pub message: String,
    pub timestamp: i64,
}

/// List all branches in a repository.
pub fn list_branches(repo_base: &str, username: &str, reponame: &str) -> Vec<String> {
    let path = repo_path(repo_base, username, reponame);
    let Ok(repo) = gix::open(&path) else {
        return Vec::new();
    };
    let Ok(platform) = repo.references() else {
        return Vec::new();
    };
    let Ok(iter) = platform.local_branches() else {
        return Vec::new();
    };
    iter.filter_map(|r| {
        let r = r.ok()?;
        Some(r.name().shorten().to_string())
    })
    .collect()
}

/// Get the commit log for a repository's branch.
pub fn get_commit_log(
    repo_base: &str,
    username: &str,
    reponame: &str,
    revision: &str,
) -> anyhow::Result<Vec<CommitInfo>> {
    let repo = gix::open(repo_path(repo_base, username, reponame))?;
    let commit = resolve_commit(&repo, revision)?;
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

/// Get the diff between two commits for a pull request.
pub fn get_pr_diff(
    repo_base: &str,
    username: &str,
    reponame: &str,
    head_oid: &str,
    base_oid: &str,
) -> anyhow::Result<Vec<DiffFile>> {
    let path = repo_path(repo_base, username, reponame);
    
    let output = std::process::Command::new("git")
        .args(["diff", &format!("{base_oid}...{head_oid}")])
        .env("GIT_DIR", &path)
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    let diff = crate::diff::parse_diff(&raw);
    Ok(highlight_diff_files(diff))
}

/// Apply syntax highlighting to all lines in a parsed diff.
fn highlight_diff_files(mut files: Vec<DiffFile>) -> Vec<DiffFile> {
    for file in &mut files {
        if file.status == "binary" {
            continue;
        }
        for hunk in &mut file.hunks {
            for line in &mut hunk.lines {
                if !line.content.is_empty() {
                    line.highlighted = crate::syntax::highlight_line(&line.content, &file.extension);
                }
            }
        }
    }
    files
}
