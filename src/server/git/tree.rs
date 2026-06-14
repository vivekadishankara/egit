use gix::hash::ObjectId;

use super::repo_path;

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

pub fn list_directory(
    repo_base: &str,
    username: &str,
    reponame: &str,
    revision: &str,
    path: &str,
) -> anyhow::Result<Vec<(String, bool)>> {
    let repo = gix::open(repo_path(repo_base, username, reponame))?;
    let commit = super::resolve_commit(&repo, revision)?;
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
