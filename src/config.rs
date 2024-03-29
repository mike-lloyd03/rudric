use std::{fs, path::Path};

use anyhow::Result;
use duration_str::deserialize_option_duration_time;
use serde::Deserialize;

use crate::types::shell_type::ShellType;

#[derive(Default, Deserialize)]
pub struct Config {
    pub default_shell: Option<ShellType>,
    #[serde(default, deserialize_with = "deserialize_option_duration_time")]
    pub session_lifetime: Option<time::Duration>,
    pub renv_filename: Option<String>,
}

impl Config {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let config_file_path = config_dir.join("config.toml");

        let config_string = match fs::read_to_string(config_file_path) {
            Ok(c) => c,
            Err(_) => {
                return Ok(Self::default());
            }
        };

        Ok(toml::from_str(&config_string)?)
    }
}

#[cfg(test)]
mod session_tests {
    use std::{fs::File, io::Write};

    use super::*;
    use anyhow::Result;
    use time::Duration;

    #[test]
    fn test_load_config() -> Result<()> {
        let test_dir = "testdata/test_config1";
        std::fs::create_dir_all(test_dir)?;

        let config_path = test_dir.to_string() + "/config.toml";
        let mut file = File::create(config_path)?;
        file.write_all(b"default_shell = \"fish\"\nsession_lifetime = \"6h\"")?;

        let config = Config::load(Path::new(test_dir))?;

        assert_eq!(config.default_shell, Some(ShellType::Fish));
        assert_eq!(config.session_lifetime, Some(Duration::hours(6)));

        std::fs::remove_dir_all(test_dir)?;

        Ok(())
    }

    #[test]
    fn test_load_config_missing_field() -> Result<()> {
        let test_dir = "testdata/test_config2";
        std::fs::create_dir_all(test_dir)?;

        let config_path = test_dir.to_string() + "/config.toml";
        let mut file = File::create(config_path)?;
        file.write_all(b"default_shell = \"fish\"")?;

        let config = Config::load(Path::new(test_dir))?;

        assert_eq!(config.default_shell, Some(ShellType::Fish));
        assert_eq!(config.session_lifetime, None);

        std::fs::remove_dir_all(test_dir)?;

        Ok(())
    }
}
