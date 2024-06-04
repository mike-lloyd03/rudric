use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

use dialoguer::console::Term;
use rudric::{
    command::{
        cli::{Cli, Command},
        handlers::*,
    },
    utils::default_config_dir,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Ignore SIGINT so we can handle it ourselves
    ctrlc::set_handler(move || {}).expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();

    let config_dir = match cli.config_dir {
        Some(c) => {
            let p = PathBuf::from(c);
            if !p.exists() {
                bail!("The provided config directory does not exist")
            }
            p
        }
        None => default_config_dir()?,
    };

    match cli.command {
        Command::Init => handle_init(&config_dir).await,
        Command::Create {
            name,
            description,
            stdin,
            file,
        } => handle_create(&config_dir, name, description, stdin, file).await,
        Command::Get { name, json } => handle_get(&config_dir, name, json).await,
        Command::Edit { name, description } => handle_edit(&config_dir, name, description).await,
        Command::Delete { name } => handle_delete(&config_dir, name).await,
        Command::Rename { name, new_name } => handle_rename(&config_dir, name, new_name).await,
        Command::List => handle_list(&config_dir).await,
        Command::Session(session_cmd) => handle_session(&config_dir, session_cmd).await,
        Command::Env { shell, file } => handle_env(&config_dir, shell, file).await,
        Command::ChangePassword => handle_change_password(&config_dir).await,
        Command::GenerateCompletions { shell } => handle_generate_completions(shell),
    }
    .map_err(|e| {
        let _ = Term::stdout().show_cursor();
        e
    })
}
