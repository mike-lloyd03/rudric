use std::{fs, path::Path};

use colored_json::to_colored_json_auto;
use mktemp::Temp;
use orion::kex;
use serde::Serialize;
use sqlx::{prelude::*, SqlitePool};

use anyhow::{bail, Context, Result};

use crate::crypto;

#[derive(Debug, FromRow)]
pub struct Secret {
    pub id: Option<i64>,
    pub name: String,
    pub value: Vec<u8>,
    pub description: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct ClearTextSecret {
    pub id: Option<i64>,
    pub name: String,
    pub value: String,
    pub description: Option<String>,
}

impl ClearTextSecret {
    pub fn new(name: &str, value: &str, description: Option<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            value: value.into(),
            description,
        }
    }

    pub fn to_encrypted(&self, key: &kex::SecretKey) -> Result<Secret> {
        let encrypted_bytes = crypto::encrypt_bytes(key, &self.value.as_bytes())?;

        Ok(Secret {
            id: None,
            name: self.name.clone(),
            value: encrypted_bytes,
            description: self.description.clone(),
        })
    }

    pub fn to_json(&self) -> Result<String> {
        to_colored_json_auto(&self).context("Failed to format secret as json")
    }
}

impl Secret {
    pub async fn get(db: &SqlitePool, name: &str) -> Result<Self> {
        sqlx::query_as!(Self, "select * from secrets where name = ?", name)
            .fetch_one(db)
            .await
            .context("Failed to fetch secret from database")
    }

    pub async fn get_all(db: &SqlitePool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "select * from secrets")
            .fetch_all(db)
            .await
            .context("Failed to fetch all secrets from database")
    }

    pub async fn store(&self, db: &SqlitePool) -> Result<()> {
        sqlx::query!(
            "insert into secrets (name, value, description) values (?, ?, ?)",
            self.name,
            self.value,
            self.description
        )
        .execute(db)
        .await
        .map(|_| ())
        .context("Failed to store secret")
    }

    pub async fn update(&self, db: &SqlitePool) -> Result<()> {
        sqlx::query!(
            "update secrets set name = ?, value = ?, description = ? where id = ?",
            self.name,
            self.value,
            self.description,
            self.id
        )
        .execute(db)
        .await
        .map(|r| {
            println!("{:?}", r);
            ()
        })
        .context("Failed to update secret")
    }

    pub fn to_cleartext(&self, key: &kex::SecretKey) -> Result<ClearTextSecret> {
        let cleartext_value_bytes = crypto::decrypt_bytes(key, &self.value)?;
        let cleartext_value = std::str::from_utf8(&cleartext_value_bytes)?;

        Ok(ClearTextSecret {
            id: self.id,
            name: self.name.clone(),
            value: cleartext_value.to_string(),
            description: self.description.clone(),
        })
    }

    pub async fn delete(&self, db: &SqlitePool) -> Result<()> {
        sqlx::query!("delete from secrets where name = ?", self.name)
            .execute(db)
            .await
            .map(|_| ())
            .context("Failed to delete secret")
    }

    pub async fn edit(&mut self, db: &SqlitePool, key: &kex::SecretKey) -> Result<()> {
        let clear_text = crypto::decrypt_bytes(key, &self.value)?;

        let secret_file = Temp::new_file()?;

        fs::write(secret_file.as_path(), &clear_text).expect("failed to write file");
        edit_file(secret_file.as_path())?;

        let new_contents = fs::read(secret_file.as_path()).unwrap();

        if new_contents == clear_text {
            println!("Secret not changed. Aborting...")
        } else {
            let new_encrypted = crypto::encrypt_bytes(key, &new_contents)?;
            self.value = new_encrypted;
            self.update(db).await?;

            println!("Updated secret {}", self.name);
        }

        Ok(())
    }
}

// Opens a file in the user's preferred text editor.
fn edit_file(file: &Path) -> Result<()> {
    let editor = get_editor()?;
    std::process::Command::new(editor)
        .arg(file)
        .status()
        .map(|_| ())
        .context("Failed to edit file")
}

/// Gets the user's prefered text editor from `VISUAL` or `EDITOR` variables. Returns an error
/// if neither variables are defined.
pub fn get_editor() -> Result<String> {
    if let Ok(r) = std::env::var("VISUAL") {
        return Ok(r);
    }
    if let Ok(r) = std::env::var("EDITOR") {
        return Ok(r);
    }
    bail!("Could not determine preferred editor. Define VISUAL or EDITOR in the environment and try again.")
}
