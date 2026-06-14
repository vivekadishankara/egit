use gix::hash::ObjectId;
use std::path::PathBuf;

pub(crate) fn repo_path(repo_base: &str, username: &str, reponame: &str) -> PathBuf {
    PathBuf::from(repo_base)
        .join(username)
        .join(format!("{}.git", reponame))
}

pub(crate) fn resolve_commit<'a>(
    repo: &'a gix::Repository,
    revision: &str,
) -> anyhow::Result<gix::Commit<'a>> {
    if revision.is_empty() || revision == "HEAD" {
        if let Ok(mut head) = repo.head() {
            if let Ok(c) = head.peel_to_commit_in_place() {
                return Ok(c.clone());
            }
        }
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

pub(crate) fn find_leaf_entry<'a>(
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

pub mod branch;
pub mod blob;
pub mod commit;
pub mod diff_ops;
pub mod init;
pub mod readme;
pub mod tree;

pub use branch::{get_default_branch, has_commits, list_branches};
pub use blob::read_file;
pub use commit::{get_commit_detail, get_commit_log, CommitDetail, CommitInfo};
pub use diff_ops::get_pr_diff;
pub use init::init_bare;
pub use readme::read_readme;
pub use tree::list_directory;
