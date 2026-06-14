use super::repo_path;

pub fn get_default_branch(repo_base: &str, username: &str, reponame: &str) -> Option<String> {
    let path = repo_path(repo_base, username, reponame);
    let repo = gix::open(&path).ok()?;

    if let Ok(head) = repo.head() {
        if let Some(refname) = head.referent_name() {
            let short = refname.shorten().to_string();
            if repo.find_reference(&format!("refs/heads/{short}")).is_ok() {
                return Some(short);
            }
        }
    }

    for name in ["master", "main"] {
        let branch = format!("refs/heads/{name}");
        if repo.find_reference(&branch).is_ok() {
            return Some(name.to_string());
        }
    }

    None
}

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
