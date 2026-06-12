use std::path::Path;

/// Initialize a new bare git repository at the given path.
pub fn init_bare(path: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap_or(path))?;
    gix::init_bare(path)?;
    tracing::info!("Initialized bare repo at {:?}", path);
    Ok(())
}
