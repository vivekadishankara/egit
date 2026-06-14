use crate::diff::DiffFile;

use super::repo_path;

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

pub fn highlight_diff_files(mut files: Vec<DiffFile>) -> Vec<DiffFile> {
    for file in &mut files {
        if file.status == "binary" {
            continue;
        }
        for hunk in &mut file.hunks {
            for line in &mut hunk.lines {
                if !line.content.is_empty() {
                    line.highlighted = crate::server::syntax::highlight_line(&line.content, &file.extension);
                }
            }
        }
    }
    files
}
