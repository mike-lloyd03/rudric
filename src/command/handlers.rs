use std::{
    io::{stdout, Read},
    path::Path,
};

use anyhow::{bail, Result};
use clap::CommandFactory;
use clap_complete::{generate, shells};
use tabled::{
    settings::{style::BorderColor, Color, Style},
    Table, Tabled,
};

use crate::{
    config::Config,
    crypto, db,
    io::edit_text,
    prompt,
    types::{
        app::App,
        renv::Renv,
        secret::{ClearSecret, Secret},
        session::{SessionKey, SessionToken},
        shell_type::ShellType,
        user::{self, User},
    },
};

use super::cli::{Cli, SessionArgs, SessionCmd};

pub async fn handle_init(config_dir: &Path) -> Result<()> {
    if db::exists(config_dir).await? {
        bail!(
            "A database already exists at {}",
            db::db_path(config_dir).to_string_lossy()
        );
    }

    let master_password: String = prompt::set_password("Set master password")?;

    let user = user::User::new(&master_password)?;

    let db = db::init(config_dir).await?;

    user.store(&db).await?;

    Ok(())
}

pub async fn handle_create(
    config_dir: &Path,
    name: String,
    description: Option<String>,
) -> Result<()> {
    let app = App::new(config_dir, true).await?;

    let mut value = String::new();
    std::io::stdin().read_to_string(&mut value).unwrap();

    if value.is_empty() {
        let value_from_editor = edit_text(b"", Some(&name))?;
        value = std::str::from_utf8(&value_from_editor)?.to_string();
        if value.is_empty() {
            bail!("Canceled")
        }
    }

    let sec = ClearSecret::new(&name, &value, description);
    let encrypted = sec.to_encrypted(&app.master_key)?;
    encrypted.store(&app.db).await?;

    println!("Created secret {name}");

    Ok(())
}

pub async fn handle_get(config_dir: &Path, name: String, json: bool) -> Result<()> {
    let app = App::new(config_dir, true).await?;

    let sec = Secret::get(&app.db, &name).await?;
    let cleartext = sec.to_cleartext(&app.master_key)?;

    if json {
        println!("{}", cleartext.to_json()?)
    } else {
        println!("{}", cleartext.value)
    }

    Ok(())
}

pub async fn handle_edit(config_dir: &Path, name: String, description: bool) -> Result<()> {
    let app = App::new(config_dir, true).await?;

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

    Ok(())
}

pub async fn handle_delete(config_dir: &Path, name: String) -> Result<()> {
    let app = App::new(config_dir, true).await?;

    let sec = Secret::get(&app.db, &name).await?;

    let prompt_msg = format!("Delete secret '{}'?", sec.name);
    let confirm = prompt::confirm(&prompt_msg, false)?;

    if confirm {
        sec.delete(&app.db).await?;
        println!("Done");
    } else {
        println!("Canceled");
    }

    Ok(())
}

pub async fn handle_rename(
    config_dir: &Path,
    name: String,
    new_name: Option<String>,
) -> Result<()> {
    let app = App::new(config_dir, true).await?;

    let mut sec = Secret::get(&app.db, &name).await?;

    let new_name = match new_name {
        Some(s) => s,
        None => prompt::input("Enter new secret name")?,
    };

    let prompt_msg = format!("Rename secret '{}' to '{}'?", sec.name, new_name.clone());
    if prompt::confirm(&prompt_msg, true)? {
        sec.rename(&app.db, &new_name).await?;
        println!("Done");
    } else {
        println!("Canceled");
    }

    Ok(())
}

pub async fn handle_list(config_dir: &Path) -> Result<()> {
    let app = App::new(config_dir, true).await?;

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

    Ok(())
}

pub async fn handle_session(config_dir: &Path, session_cmd: SessionArgs) -> Result<()> {
    match session_cmd.command {
        Some(SessionCmd::End) => {
            let app = App::new(config_dir, true).await?;

            if let Ok(st) = SessionToken::from_env() {
                let (session_key_id, _) = st.split_id()?;
                let session_key = SessionKey::get(&app.db, &session_key_id).await?;
                session_key.delete(&app.db).await?;
            } else {
                bail!("Session token not found")
            }
        }
        _ => {
            let app = App::new(config_dir, false).await?;
            let config = Config::load(config_dir)?;

            let session_token =
                SessionToken::new(&app.db, app.master_key, config.session_lifetime).await?;
            println!("{session_token}");
        }
    }

    Ok(())
}

pub async fn handle_env(
    config_dir: &Path,
    shell: Option<ShellType>,
    file: Option<String>,
) -> Result<()> {
    let app = App::new(config_dir, true).await?;
    let config = Config::load(config_dir)?;

    let path = Path::new(".renv");
    let renv = Renv::load(&app, path).await?;
    let shell = shell.unwrap_or(config.default_shell.unwrap_or_default());

    println!("{}", renv.to_shell(shell));

    Ok(())
}

pub async fn handle_change_password(config_dir: &Path) -> Result<()> {
    let app = App::new(config_dir, true).await?;
    let new_password = prompt::set_password("Enter new master password")?;
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
    Ok(())
}

pub fn handle_generate_completions(shell: ShellType) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = "rudric";
    match shell {
        ShellType::Bash => generate(shells::Bash, &mut cmd, bin_name, &mut stdout()),
        ShellType::Fish => generate(shells::Fish, &mut cmd, bin_name, &mut stdout()),
        ShellType::Zsh => generate(shells::Zsh, &mut cmd, bin_name, &mut stdout()),
        _ => bail!("Provided shell is not supported"),
    };

    Ok(())
}
