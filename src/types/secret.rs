use orion::kex;
use sqlx::{prelude::*, SqlitePool};

use anyhow::{Context, Result};

#[derive(Debug, FromRow)]
pub struct Secret {
    pub id: Option<i64>,
    pub name: String,
    pub value: Vec<u8>,
    pub description: Option<String>,
}
#[derive(Debug)]
pub struct ClearTextSecret {
    pub name: String,
    pub value: String,
    pub description: Option<String>,
}

impl ClearTextSecret {
    pub fn new(name: &str, value: &str, description: Option<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            description,
        }
    }

    pub fn to_encrypted(&self, key: &kex::SecretKey) -> Result<Secret> {
        let encrypted_bytes =
            orion::aead::seal(key, self.value.as_bytes()).context("failed to seal input string")?;

        Ok(Secret {
            id: None,
            name: self.name.clone(),
            value: encrypted_bytes,
            description: self.description.clone(),
        })
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

    pub fn to_cleartext(&self, key: &kex::SecretKey) -> Result<ClearTextSecret> {
        let cleartext_value_bytes = orion::aead::open(key, &self.value)?;
        let cleartext_value = std::str::from_utf8(&cleartext_value_bytes)?;

        Ok(ClearTextSecret {
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
