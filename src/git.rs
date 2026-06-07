use std::path::Path;

/// Initialize a new bare git repository at the given path.
pub fn init_bare(path: &Path) -> anyhow::Result<()> {
    // TODO: implement in step 5 (repo creation)
    std::fs::create_dir_all(path)?;
    tracing::info!("Init bare repo at {:?}", path);
    Ok(())
}
