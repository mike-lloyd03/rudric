// #![deny(missing_docs)]
use clap::{Args, Parser, Subcommand};

use crate::types::shell_type::ShellType;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(short, long)]
    pub config_dir: Option<String>,
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
        /// Edit the secret description
        #[arg(short = 'd', long)]
        description: bool,
    },
    Delete {
        name: String,
    },
    Rename {
        name: String,
        new_name: Option<String>,
    },
    List,
    Session(SessionArgs),
    Env {
        shell: Option<ShellType>,
    },
    GenerateCompletions {
        shell: ShellType,
    },
    ChangePassword,
}

#[derive(Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub command: Option<SessionCmd>,
}

#[derive(Subcommand)]
pub enum SessionCmd {
    New,
    End,
}
