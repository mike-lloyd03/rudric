use colored_json::to_colored_json_auto;
use orion::aead;
use serde::Serialize;
use sqlx::{prelude::*, Sqlite, SqlitePool};

use anyhow::{anyhow, Context, Result};

use crate::crypto;

pub const SECRET_NOT_FOUND: &str = "Secret not found";

#[derive(Debug, FromRow)]
pub struct Secret {
    pub id: Option<i64>,
    pub name: String,
    pub value: Vec<u8>,
    pub description: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct ClearSecret {
    pub id: Option<i64>,
    pub name: String,
    pub value: String,
    pub description: Option<String>,
}

impl Secret {
    pub async fn get(db: &SqlitePool, name: &str) -> Result<Self> {
        sqlx::query_as!(Self, "select * from secrets where name = ?", name)
            .fetch_one(db)
            .await
            .map_err(|e| {
                if e.to_string().contains("no rows returned") {
                    anyhow!(SECRET_NOT_FOUND)
                } else {
                    anyhow!(e)
                }
            })
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

    pub async fn update<'a, E>(&self, executor: E) -> Result<()>
    where
        E: Executor<'a, Database = Sqlite>,
    {
        sqlx::query!(
            "update secrets set name = ?, value = ?, description = ? where id = ?",
            self.name,
            self.value,
            self.description,
            self.id
        )
        .execute(executor)
        .await
        .context("Failed to update secret")?;

        Ok(())
    }

    pub fn to_cleartext(&self, key: &aead::SecretKey) -> Result<ClearSecret> {
        let cleartext_value_bytes = crypto::decrypt(key, &self.value)?;
        let cleartext_value = std::str::from_utf8(&cleartext_value_bytes)?;

        Ok(ClearSecret {
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

    pub async fn rename(&mut self, db: &SqlitePool, new_name: &str) -> Result<()> {
        sqlx::query!(
            "update secrets set name = ? where name = ?",
            new_name,
            self.name
        )
        .execute(db)
        .await
        .map(|_| ())
        .context("Failed to rename secret")?;

        self.name = new_name.to_string();

        Ok(())
    }
}

impl ClearSecret {
    pub fn new(name: &str, value: &str, description: Option<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            value: value.into(),
            description,
        }
    }

    pub fn to_encrypted(&self, key: &aead::SecretKey) -> Result<Secret> {
        let encrypted_bytes = crypto::encrypt(key, self.value.as_bytes())?;

        Ok(Secret {
            id: self.id,
            name: self.name.clone(),
            value: encrypted_bytes,
            description: self.description.clone(),
        })
    }

    pub fn to_json(&self) -> Result<String> {
        to_colored_json_auto(&self).context("Failed to format secret as json")
    }
}
