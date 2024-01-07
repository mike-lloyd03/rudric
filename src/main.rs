use std::{io::stdout, path::Path};

use anyhow::{bail, Result};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use cli::{Cli, Session};
use tabled::{
    settings::{style::BorderColor, Color, Style},
    Table, Tabled,
};
use types::{
    renv::ShellType,
    secret::Secret,
    session::{SessionKey, SessionToken},
    user::{self, User},
};
use utils::set_password;

use crate::{
    io::edit_text,
    types::{app::App, renv::Renv, secret::ClearSecret},
};

mod cli;
mod crypto;
mod db;
mod io;
mod prompt;
mod types;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // let config_dir = cli.config_dir.unwrap_or(default_config_dir()?);

    match cli.command {
        cli::Command::Init => {
            if db::exists().await? {
                bail!("A database already exists at {}", db::db_path()?);
            }

            let master_password: String = prompt::set_master_password()?;

            let user = user::User::new(&master_password)?;

            let db = db::init().await?;

            user.store(&db).await?;
        }
        cli::Command::Create { name, description } => {
            let app = App::new(true).await?;

            let value = edit_text(b"", Some(&name))?;
            let value = std::str::from_utf8(&value)?;

            if value.is_empty() {
                bail!("Canceled")
            }

            let sec = ClearSecret::new(&name, value, description);
            let encrypted = sec.to_encrypted(&app.master_key)?;
            encrypted.store(&app.db).await?;

            println!("Created secret {name}")
        }
        cli::Command::Get { name, json } => {
            let app = App::new(true).await?;

            let sec = Secret::get(&app.db, &name).await?;
            let cleartext = sec.to_cleartext(&app.master_key)?;

            if json {
                println!("{}", cleartext.to_json()?)
            } else {
                println!("{}", cleartext.value)
            }
        }
        cli::Command::Edit { name, description } => {
            let app = App::new(true).await?;

            let mut sec = Secret::get(&app.db, &name).await?;

            if description {
                let old_desc = sec.description.unwrap_or_default();
                let new_desc = edit_text(old_desc.as_bytes(), Some(&sec.name))?;

                if new_desc != old_desc.as_bytes() {
                    if new_desc.is_empty() {
                        sec.description = None;
                    } else {
                        let new_desc = String::from_utf8(new_desc)?;
                        sec.description = Some(new_desc);
                    }
                    sec.update(&app.db).await?;
                    println!("Updated description for secret '{}'", sec.name);

                    return Ok(());
                } else {
                    println!("Secret not changed. Canceling...")
                }
            } else {
                let clear_text = crypto::decrypt(&app.master_key, &sec.value)?;

                let new_contents = edit_text(&clear_text, Some(&sec.name))?;

                if new_contents == clear_text {
                    println!("Secret not changed. Canceling...")
                } else {
                    let new_encrypted = crypto::encrypt(&app.master_key, &new_contents)?;
                    sec.value = new_encrypted;
                    sec.update(&app.db).await?;

                    println!("Updated secret '{}'", sec.name);
                }
            }
        }
        cli::Command::Delete { name } => {
            let app = App::new(true).await?;

            let sec = Secret::get(&app.db, &name).await?;

            let prompt_msg = format!("Delete secret '{}'?", sec.name);
            let confirm = prompt::confirm(&prompt_msg, false)?;

            if confirm {
                sec.delete(&app.db).await?;
                println!("Done");
            } else {
                println!("Canceled");
            }
        }
        cli::Command::Rename { name, mut new_name } => {
            let app = App::new(true).await?;

            let mut sec = Secret::get(&app.db, &name).await?;

            if new_name.is_none() {
                new_name = Some(prompt::input("Enter new secret name")?);
            }

            let prompt_msg = format!(
                "Rename secret '{}' to '{}'?",
                sec.name,
                new_name.clone().unwrap()
            );
            if prompt::confirm(&prompt_msg, true)? {
                sec.rename(&app.db, &new_name.unwrap()).await?;
                println!("Done");
            } else {
                println!("Canceled");
            }
        }
        cli::Command::List => {
            let app = App::new(true).await?;

            #[derive(Tabled)]
            struct SecretsTable {
                id: i64,
                name: String,
                description: String,
            }

            let secrets = Secret::get_all(&app.db).await?;
            let secrets_table = secrets.iter().map(|s| SecretsTable {
                id: s.id.unwrap_or_default(),
                name: s.name.clone(),
                description: s.description.clone().unwrap_or_default().trim().to_string(),
            });

            let table = Table::new(secrets_table)
                .with(Style::rounded())
                .with(BorderColor::filled(Color::FG_BLUE))
                .to_string();

            println!("{table}");
        }
        cli::Command::Session(session_cmd) => match session_cmd.command {
            Some(Session::End) => {
                let app = App::new(true).await?;

                if let Ok(st) = SessionToken::from_env() {
                    let (session_key_id, _) = st.split_id()?;
                    let session_key = SessionKey::get(&app.db, &session_key_id).await?;
                    session_key.delete(&app.db).await?;
                } else {
                    bail!("Session token not found")
                }
            }
            _ => {
                let app = App::new(false).await?;
                let session_token = SessionToken::new(&app.db, app.master_key).await?;
                println!("{session_token}");
            }
        },
        cli::Command::Env { shell } => {
            let app = App::new(true).await?;

            let path = Path::new(".renv");
            let renv = Renv::load(&app, path).await?;
            let shell = shell.unwrap_or_default();

            println!("{}", renv.to_shell(shell))
        }
        cli::Command::GenerateCompletions { shell } => {
            let mut cmd = Cli::command();
            let bin_name = "rudric";
            match shell {
                ShellType::Bash => generate(shells::Bash, &mut cmd, bin_name, &mut stdout()),
                ShellType::Fish => generate(shells::Fish, &mut cmd, bin_name, &mut stdout()),
                ShellType::Zsh => generate(shells::Zsh, &mut cmd, bin_name, &mut stdout()),
                _ => bail!("Provided shell is not supported"),
            };
        }
        cli::Command::ChangePassword => {
            let app = App::new(true).await?;
            let new_password = set_password("Enter new master password")?;
            let new_pwhash = crypto::hash_password(&new_password)?;
            let new_salt = crypto::generate_salt()?.as_ref().to_vec();
            let mut user = User::load(&app.db).await?;
            let new_key = crypto::derive_key(&new_password, &new_salt)?;

            let secrets = Secret::get_all(&app.db).await?;

            let mut tx = app.db.begin().await?;

            user.master_password_hash = new_pwhash.unprotected_as_encoded().to_string();
            user.salt = new_salt;
            user.update(&mut *tx).await?;

            for secret in secrets {
                let clear_secret = secret.to_cleartext(&app.master_key)?;
                let reencrypted_secret = clear_secret.to_encrypted(&new_key)?;
                reencrypted_secret.update(&mut *tx).await?;
            }

            tx.commit().await?;
        }
    }

    Ok(())
}
