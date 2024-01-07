use std::path::PathBuf;

use anyhow::Result;

pub fn default_config_dir() -> Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new()?;
    let config_dir = xdg_dirs.get_config_home();

    Ok(config_dir.join("rudric"))
}
