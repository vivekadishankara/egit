use super::repo_path;

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
    let commit = match super::resolve_commit(&repo, revision) {
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
