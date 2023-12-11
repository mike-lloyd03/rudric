use anyhow::{bail, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};
use types::secret::{ClearTextSecret, Secret};

mod cli;
mod db;
mod types;

const SECRET_KEY: &str = "abcdefghijklmnopqrstuvwxyzqwerty";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Init => {
            // TODO: Check if db is already initialized.

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
                .interact_text()
                .unwrap();

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
                .interact_text()
                .unwrap();

            if master_password1 != master_password2 {
                bail!("Passwords do not match.")
            }

            let db = db::init().await?;

            let app = types::app::App::new(&db);
            app.set_master_password(&master_password1).await;
        }
        cli::Command::Create {
            name,
            value,
            description,
        } => {
            let db = db::connect().await?;
            let sec = ClearTextSecret::new(&name, &value, description);
            let encrypted = sec.to_encrypted(SECRET_KEY)?;
            if let Err(e) = encrypted.store(&db).await {
                eprintln!("{}", e);
            }
        }
        cli::Command::Get { name } => {
            let db = db::connect().await?;
            let sec = Secret::get(&db, &name).await?;
            let cleartext = sec.to_cleartext(SECRET_KEY)?;

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
