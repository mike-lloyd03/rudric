use clap::{Parser, Subcommand};

use crate::types::renv::ShellType;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Init,
    Create {
        name: String,
        /// Set the secret description
        #[arg(short = 'd', long)]
        description: Option<String>,
    },
    Get {
        name: String,
        /// Output the secret in json format
        #[arg(long)]
        json: bool,
    },
    Edit {
        name: String,
    },
    Delete {
        name: String,
    },
    List,
    Session,
    Env {
        shell: Option<ShellType>,
    },
}
