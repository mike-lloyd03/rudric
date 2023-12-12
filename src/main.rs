use anyhow::{bail, Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};
use sqlx::SqlitePool;
use types::{
    secret::{ClearTextSecret, Secret},
    user::{self, User},
};

mod cli;
mod crypto;
mod db;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Init => {
            let db_exists = db::exists().await?;

            if db_exists {
                bail!("A database already exists at {}", db::db_path()?);
            }

            let master_password1: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Set a master password")
                .validate_with(|a: &String| -> Result<()> {
                    if a.len() >= 8 {
                        Ok(())
                    } else {
                        bail!("Password must be minimum 8 characters")
                    }
                })
                .report(false)
                .interact_text()?;

            let master_password2: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Re-enter")
                .validate_with(|a: &String| -> Result<()> {
                    if a == &master_password1 {
                        Ok(())
                    } else {
                        bail!("Passwords do not match")
                    }
                })
                .report(false)
                .interact_text()?;

            if master_password1 != master_password2 {
                bail!("Passwords do not match.")
            }

            let user = user::User::new(&master_password1)?;

            let db = db::init().await?;

            user.store(&db).await?;
        }
        cli::Command::Create {
            name,
            value,
            description,
        } => {
            let db = db::connect().await?;
            let input_password = read_password()?;
            let user = authenticate_user(&db, &input_password).await?;
            let key = user.derive_key(&input_password)?;

            let sec = ClearTextSecret::new(&name, &value, description);
            let encrypted = sec.to_encrypted(&key)?;
            if let Err(e) = encrypted.store(&db).await {
                eprintln!("{}", e);
            }
        }
        cli::Command::Get { name } => {
            let db = db::connect().await?;
            let input_password = read_password()?;
            let user = authenticate_user(&db, &input_password).await?;
            let key = user.derive_key(&input_password)?;

            let sec = Secret::get(&db, &name).await?;
            let cleartext = sec.to_cleartext(&key)?;

            println!("{}", cleartext.value)
        }
        cli::Command::Edit => todo!(),
        cli::Command::Delete { name } => {
            let db = db::connect().await?;
            let sec = Secret::get(&db, &name).await?;
            sec.delete(&db).await?;
        }
        cli::Command::List => {
            let db = db::connect().await?;
            let secrets = Secret::get_all(&db).await?;
            for secret in secrets {
                println!(
                    "{}\t\t{}",
                    secret.name,
                    secret.description.unwrap_or_default()
                )
            }
        }
    }

    Ok(())
}

fn read_password() -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter master password")
        .report(false)
        .interact_text()
        .context("Failed to read user password")
}

async fn authenticate_user(db: &SqlitePool, password: &str) -> Result<User> {
    let user = user::User::load(db).await?;

    if user.authenticate(password) {
        Ok(user)
    } else {
        bail!("Invalid master password")
    }
}
