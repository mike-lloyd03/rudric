use clap::ValueEnum;
use serde::Deserialize;

#[derive(Clone, Default, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    #[default]
    Bash,
    Fish,
    Zsh,
    Nu,
}
