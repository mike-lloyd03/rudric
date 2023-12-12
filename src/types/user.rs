use anyhow::{bail, Result};
use orion::kex;
use sqlx::{prelude::FromRow, SqlitePool};

use crate::crypto;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64, // This will always be 1 to ensure only one record can be added to the table
    pub master_password_hash: String,
    pub salt: Vec<u8>,
}

impl User {
    pub fn new(cleartext_password: &str) -> Result<Self> {
        let pwhash = crypto::hash_password(cleartext_password)?;
        let salt = orion::kdf::Salt::default();

        Ok(Self {
            id: 1,
            master_password_hash: pwhash.unprotected_as_encoded().to_string(),
            salt: salt.as_ref().to_vec(),
        })
    }

    pub async fn store(&self, db: &SqlitePool) -> Result<()> {
        Ok(sqlx::query!(
            "insert into user (id, master_password_hash, salt) values (1, ?, ?)",
            self.master_password_hash,
            self.salt
        )
        .execute(db)
        .await
        .map(|_| ())?)
    }

    pub async fn load(db: &SqlitePool) -> Result<Self> {
        Ok(sqlx::query_as!(Self, "select * from user limit 1")
            .fetch_one(db)
            .await?)
    }

    pub fn authenticate(&self, password: &str) -> bool {
        crypto::verify_hash(password, &self.master_password_hash)
    }

    pub fn derive_key(&self, password: &str) -> Result<kex::SecretKey> {
        if !crypto::verify_hash(password, &self.master_password_hash) {
            bail!("Invalid master password")
        }

        crypto::derive_key(password, &self.salt)
    }
}
