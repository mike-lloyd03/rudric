use clap::{Parser, Subcommand};

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
        value: String,
        #[arg(short = 'd')]
        description: Option<String>,
    },
    Get {
        name: String,
    },
    Edit,
    Delete {
        name: String,
    },
    List,
}
