use anyhow::Result;
use sqlx::{prelude::FromRow, SqlitePool};

use crate::crypto;

#[derive(Debug, FromRow)]
pub struct User {
    pub master_password_hash: Vec<u8>,
    pub salt: Vec<u8>,
}

impl User {
    pub fn new(cleartext_password: &str) -> Result<Self> {
        let pwhash = crypto::hash_password(cleartext_password)?;
        let salt = orion::kdf::Salt::default();

        Ok(Self {
            master_password_hash: pwhash.unprotected_as_bytes().to_vec(),
            salt: salt.as_ref().to_vec(),
        })
    }

    pub async fn store(&self, db: &SqlitePool) -> Result<()> {
        Ok(sqlx::query!(
            "insert into user (master_password_hash, salt) values (?, ?)",
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
}
