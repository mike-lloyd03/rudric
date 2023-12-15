use colored_json::to_colored_json_auto;
use orion::kex;
use serde::Serialize;
use sqlx::{prelude::*, SqlitePool};

use anyhow::{Context, Result};

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
        let encrypted_bytes = crypto::encrypt_bytes(key, self.value.as_bytes())?;

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
}
