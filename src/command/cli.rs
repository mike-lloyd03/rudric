use clap::{Args, Parser, Subcommand};

use crate::types::shell_type::ShellType;

/// Store secrets in an encrypted state on disk
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Specify a subcommand
    #[command(subcommand)]
    pub command: Command,

    /// Specify an alternate config directory
    #[arg(short, long)]
    pub config_dir: Option<String>,
}

/// The subcommand to execute
#[derive(Subcommand)]
pub enum Command {
    /// Set a master password and initialize the database
    Init,

    /// Create a new secret
    Create {
        /// The name of the secret (must be unique)
        name: String,

        /// Set the secret description
        #[arg(short = 'd', long)]
        description: Option<String>,
    },

    /// Fetch a secret value
    Get {
        /// The name of the secret
        name: String,

        /// Output the secret in json format
        #[arg(long)]
        json: bool,
    },

    /// Edit an existing secret. Will open the secret value in $EDITOR or $VISUAL
    Edit {
        /// The name of the secret
        name: String,

        /// Edit the secret description
        #[arg(short = 'd', long)]
        description: bool,
    },

    /// Delete a secret. Asks for confirmation
    Delete {
        /// The name of the secret
        name: String,
    },

    /// Rename a secret
    Rename {
        /// The current name of the secret
        name: String,

        /// The new name of the secret. If blank, the user will be prompted for a new name
        new_name: Option<String>,
    },

    /// List all secrets
    List,

    /// Create a new session token. Setting this token as `RUDRIC_SESSION` in the environment will
    /// prevent the user from being prompted for the password each time the program is invoked.
    Session(SessionArgs),

    /// Read from the .renv file (or an alternate file) and set the specified variables in the environment.
    /// This command generates the shell code necessary to set the requested
    /// environment variables. The output of this command must be sourced with something like
    /// `rudric env | source`
    Env {
        /// The shell format to use. Defaults to bash but an alternate can be specified in the config
        /// file.
        shell: Option<ShellType>,

        /// Use an alternate environment file
        #[arg(short, long)]
        file: Option<String>,
    },

    /// Generate shell completions
    GenerateCompletions {
        /// The shell to generate completions for
        shell: ShellType,
    },

    /// Change the master password for the vault
    ChangePassword,
}

#[derive(Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub command: Option<SessionCmd>,
}

#[derive(Subcommand)]
pub enum SessionCmd {
    /// Create a new session token
    New,
    /// Invalidates the current session token
    End,
}
