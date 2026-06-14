use std::path::Path;

use super::repo_path;

pub fn read_file(
    repo_base: &str,
    username: &str,
    reponame: &str,
    revision: &str,
    path: &str,
) -> anyhow::Result<(Vec<u8>, String)> {
    let repo = gix::open(repo_path(repo_base, username, reponame))?;
    let commit = super::resolve_commit(&repo, revision)?;
    let tree = commit.tree()?;
    let (oid, is_dir) = super::find_leaf_entry(&repo, path, &tree)?;
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
