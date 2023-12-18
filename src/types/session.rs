use std::{env, fmt::Display};

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use orion::kex::SecretKey;
use sqlx::SqlitePool;

use crate::crypto;

pub struct SessionToken(String);

impl SessionToken {
    pub fn from_env() -> Result<Self> {
        Ok(Self(env::var("RUDRIC_SESSION")?))
    }

    pub async fn new(db: &SqlitePool, derived_key: SecretKey) -> Result<Self> {
        let session_key = SecretKey::default();
        let session_key_bytes = session_key.unprotected_as_bytes();

        let encrypted_derived_key =
            crypto::encrypt_bytes(&session_key, derived_key.unprotected_as_bytes())?;
        let session_id = sqlx::types::Uuid::new_v4();

        sqlx::query!(
            "insert into session_tokens (id, key) values (?, ?)",
            session_id,
            session_key_bytes
        )
        .execute(db)
        .await?;

        let session_token = [session_id.as_bytes(), encrypted_derived_key.as_slice()].concat();

        Ok(Self(STANDARD_NO_PAD.encode(session_token)))
    }

    pub async fn into_derived_key(self, db: &SqlitePool) -> Result<SecretKey> {
        let session_token_bytes = STANDARD_NO_PAD.decode(self.0)?;

        if session_token_bytes.len() < 16 {
            bail!("Invalid session token")
        }
        let (session_id, encrypted_derived_key) = session_token_bytes.split_at(16);

        struct Session {
            key: Vec<u8>,
        }
        let session = sqlx::query_as!(
            Session,
            "select key from session_tokens where id = ?",
            session_id,
        )
        .fetch_one(db)
        .await?;

        let session_key = SecretKey::from_slice(session.key.as_slice())?;

        let decrypted_derived_key = crypto::decrypt_bytes(&session_key, encrypted_derived_key)?;

        SecretKey::from_slice(&decrypted_derived_key)
            .context("Failed to create SecretKey from decrypted key")
    }
}

impl Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
