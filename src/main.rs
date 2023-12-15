use anyhow::{bail, Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Password};
use io::edit_text;
use sqlx::SqlitePool;
use types::{
    secret::{ClearTextSecret, Secret},
    user::{self, User},
};

mod cli;
mod crypto;
mod db;
mod io;
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

            let master_password: String = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Set a master password")
                .with_confirmation("Confirm password", "Passwords do not match")
                .interact()?;

            let user = user::User::new(&master_password)?;

            let db = db::init().await?;

            user.store(&db).await?;
        }
        cli::Command::Create { name, description } => {
            let db = db::connect().await?;
            let input_password = read_password("Enter master password")?;
            let user = authenticate_user(&db, &input_password).await?;
            let key = user.derive_key(&input_password)?;
            let value = edit_text(b"")?;
            let value_bytes = std::str::from_utf8(&value)?;

            let sec = ClearTextSecret::new(&name, value_bytes, description);
            let encrypted = sec.to_encrypted(&key)?;
            if let Err(e) = encrypted.store(&db).await {
                eprintln!("{}", e);
            }
        }
        cli::Command::Get { name, json } => {
            let db = db::connect().await?;
            let input_password = read_password("Enter master password")?;
            let user = authenticate_user(&db, &input_password).await?;
            let key = user.derive_key(&input_password)?;

            let sec = Secret::get(&db, &name).await?;
            let cleartext = sec.to_cleartext(&key)?;

            if json {
                println!("{}", cleartext.to_json()?)
            } else {
                println!("{}", cleartext.value)
            }
        }
        cli::Command::Edit { name } => {
            let db = db::connect().await?;
            let input_password = read_password("Enter master password")?;
            let user = authenticate_user(&db, &input_password).await?;
            let key = user.derive_key(&input_password)?;

            let mut sec = Secret::get(&db, &name).await?;
            let clear_text = crypto::decrypt_bytes(&key, &sec.value)?;

            let new_contents = edit_text(&clear_text)?;

            if new_contents == clear_text {
                println!("Secret not changed. Aborting...")
            } else {
                let new_encrypted = crypto::encrypt_bytes(&key, &new_contents)?;
                sec.value = new_encrypted;
                sec.update(&db).await?;

                println!("Updated secret {}", sec.name);
            }
        }
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

fn read_password(prompt: &str) -> Result<String> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .report(false)
        .interact()
        .context("Failed to read user input")
}

async fn authenticate_user(db: &SqlitePool, password: &str) -> Result<User> {
    let user = user::User::load(db).await?;

    if user.authenticate(password) {
        Ok(user)
    } else {
        bail!("Invalid master password")
    }
}
