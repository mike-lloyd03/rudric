use anyhow::{bail, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};

mod cli;
mod db;
mod types;

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
        cli::Command::Create => todo!(),
        cli::Command::Get => todo!(),
        cli::Command::Update => todo!(),
        cli::Command::Delete => todo!(),
    }

    Ok(())
}
