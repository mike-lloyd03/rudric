use clap::ValueEnum;
use serde::Deserialize;

#[derive(Debug, Clone, Default, ValueEnum, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    #[default]
    Bash,
    Fish,
    Zsh,
    Nu,
}
