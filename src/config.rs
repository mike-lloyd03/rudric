use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::types::shell_type::ShellType;

#[derive(Deserialize, Default)]
pub struct Config {
    pub default_shell: Option<ShellType>,
}

impl Config {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let config_file_path = config_dir.join("config.yaml");
        let config_file = match std::fs::File::open(config_file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to open config file: {e}");
                eprintln!("Using defaults");
                return Ok(Self::default());
            }
        };

        Ok(serde_yaml::from_reader(config_file)?)
    }
}
