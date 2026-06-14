use std::path::Path;

pub fn init_bare(path: &Path) -> anyhow::Result<()> {
    use std::process::Command;

    std::fs::create_dir_all(path.parent().unwrap_or(path))?;
    gix::init_bare(path)?;

    std::fs::write(path.join("HEAD"), b"ref: refs/heads/main\n")?;

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
