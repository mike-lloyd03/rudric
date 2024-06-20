use std::path::PathBuf;

use anyhow::Result;

pub fn default_config_dir() -> Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new()?;
    let config_dir = xdg_dirs.get_config_home();
    let rudric_config_dir = config_dir.join("rudric");

    std::fs::create_dir_all(rudric_config_dir.clone())?;

    Ok(rudric_config_dir)
}
