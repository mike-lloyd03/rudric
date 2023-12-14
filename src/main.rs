use anyhow::{bail, Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input, Password};
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
            let value = read_input("Enter secret value")?;

            let sec = ClearTextSecret::new(&name, &value, description);
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
            sec.edit(&db, &key).await?;
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

fn read_input(prompt: &str) -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .report(false)
        .interact_text()
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
