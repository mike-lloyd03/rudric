use sqlx::{prelude::*, SqlitePool};

use anyhow::{bail, Context, Result};

#[derive(Debug, FromRow)]
pub struct Secret {
    pub id: Option<i64>,
    pub name: String,
    pub value: String,
}

impl Secret {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            id: None,
            name: name.into(),
            value: value.into(),
        }
    }

    fn encrypt_value(&self, key: &str) -> Result<String> {
        let secret_key = orion::aead::SecretKey::from_slice(key.as_bytes())
            .context("failed to create secret key")?;
        let encrypted_bytes = orion::aead::seal(&secret_key, self.value.as_bytes())
            .context("failed to seal input string")?;
        let encrypted_str = std::str::from_utf8(&encrypted_bytes)
            .context("failed to convert encrypted string to utf8")?;
        Ok(encrypted_str.into())
    }

    pub async fn store(&self, db: &SqlitePool, key: &str) -> Result<()> {
        let encrypted_value = self.encrypt_value(key)?;

        match sqlx::query!(
            "insert into secrets (name, value) values (?, ?)",
            self.name,
            encrypted_value
        )
        .execute(db)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => bail!(e),
        }
    }
}
