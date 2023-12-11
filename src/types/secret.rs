use sqlx::{prelude::*, SqlitePool};

use anyhow::{bail, Context, Result};
use base64::{self, engine::general_purpose, Engine};

#[derive(Debug, FromRow)]
pub struct Secret {
    pub id: Option<i64>,
    pub name: String,
    pub value: String,
}
#[derive(Debug)]
pub struct ClearTextSecret {
    pub name: String,
    pub value: String,
}

impl ClearTextSecret {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn to_encrypted(&self, key: &str) -> Result<Secret> {
        if key.len() < 32 {
            bail!("Key length must be 32 bytes minimum");
        }

        let secret_key = orion::aead::SecretKey::from_slice(key.as_bytes())
            .context("failed to create secret key")?;
        let encrypted_bytes = orion::aead::seal(&secret_key, self.value.as_bytes())
            .context("failed to seal input string")?;
        let encoded: String = general_purpose::STANDARD_NO_PAD.encode(encrypted_bytes);

        Ok(Secret {
            id: None,
            name: self.name.clone(),
            value: encoded,
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

    pub async fn store(&self, db: &SqlitePool) -> Result<()> {
        match sqlx::query!(
            "insert into secrets (name, value) values (?, ?)",
            self.name,
            self.value
        )
        .execute(db)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => bail!(e),
        }
    }

    pub fn to_cleartext(&self, key: &str) -> Result<ClearTextSecret> {
        if key.len() < 32 {
            bail!("Key length must be 32 bytes minimum");
        }
        println!("Got bytes: {:?}", self.value);

        let secret_key = orion::aead::SecretKey::from_slice(key.as_bytes())
            .context("failed to create secret key")?;
        let decoded_value: Vec<u8> = general_purpose::STANDARD_NO_PAD.decode(&self.value)?;
        let cleartext_value_bytes = orion::aead::open(&secret_key, &decoded_value)?;
        let cleartext_value = std::str::from_utf8(&cleartext_value_bytes)?;

        Ok(ClearTextSecret {
            name: self.name.clone(),
            value: cleartext_value.to_string(),
        })
    }
}
