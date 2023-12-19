use std::{env, fmt::Display};

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD_NO_PAD as b64, Engine};
use orion::kex::SecretKey;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::crypto;

pub struct SessionToken(String);

impl SessionToken {
    pub fn from_env() -> Result<Self> {
        Ok(Self(env::var("RUDRIC_SESSION")?))
    }

    pub async fn new(db: &SqlitePool, derived_key: SecretKey) -> Result<Self> {
        let session_id = sqlx::types::Uuid::new_v4();

        let session_key = SecretKey::default();
        let session_key_bytes = session_key.unprotected_as_bytes();

        let expire_time =
            (time::OffsetDateTime::now_utc() + time::Duration::hours(8)).unix_timestamp();
        let expire_bytes = expire_time.to_be_bytes();

        let key_time_concat = [&expire_bytes, derived_key.unprotected_as_bytes()].concat();

        let encrypted_derived_key = crypto::encrypt_bytes(&session_key, &key_time_concat)?;

        Self::insert(db, session_id, session_key_bytes, expire_time).await?;

        let session_token = [session_id.as_bytes(), encrypted_derived_key.as_slice()].concat();

        Ok(Self(b64.encode(session_token)))
    }

    async fn insert(db: &SqlitePool, id: Uuid, session_key: &[u8], timestamp: i64) -> Result<()> {
        sqlx::query!(
            "insert into session_tokens (id, key, expire_time) values (?, ?, ?)",
            id,
            session_key,
            timestamp
        )
        .execute(db)
        .await
        .map(|_| ())
        .context("Failed to insert session key")
    }

    async fn delete(db: &SqlitePool, id: &[u8]) -> Result<()> {
        sqlx::query!("delete from session_tokens where id = ?", id)
            .execute(db)
            .await
            .map(|_| ())
            .context("Failed to delete session key")
    }

    async fn delete_expired(db: &SqlitePool) -> Result<()> {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        sqlx::query!("delete from session_tokens where expire_time < ?", now)
            .execute(db)
            .await
            .map(|_| ())
            .context("Failed to delete expired session key")
    }

    async fn get(db: &SqlitePool, id: &[u8]) -> Result<Vec<u8>> {
        struct Session {
            key: Vec<u8>,
        }

        let session = sqlx::query_as!(Session, "select key from session_tokens where id = ?", id,)
            .fetch_one(db)
            .await?;

        Ok(session.key)
    }

    pub async fn into_derived_key(self, db: &SqlitePool) -> Result<SecretKey> {
        let session_token_bytes = b64.decode(self.0)?;

        if session_token_bytes.len() < 16 {
            bail!("Invalid session token")
        }

        let (session_id, encrypted_key_time) = session_token_bytes.split_at(16);

        let session_key_bytes = match Self::get(db, session_id).await {
            Ok(s) => s,
            Err(_) => bail!("Invalid session token"),
        };
        let session_key = SecretKey::from_slice(&session_key_bytes)?;

        let decrypted_key_time = crypto::decrypt_bytes(&session_key, encrypted_key_time)?;

        let (timestamp_bytes, decrypted_derived_key) = decrypted_key_time.split_at(8);

        let timestamp = i64::from_be_bytes(timestamp_bytes.try_into()?);

        let expire_time = time::OffsetDateTime::from_unix_timestamp(timestamp)?;

        if expire_time < time::OffsetDateTime::now_utc() {
            println!(
                "{} is less than {}",
                expire_time,
                time::OffsetDateTime::now_utc()
            );
            Self::delete(db, session_id).await?;
            bail!("Session key has expired");
        }

        if let Err(e) = Self::delete_expired(db).await {
            eprintln!("Error deleting expired session tokens: {e}");
        }

        SecretKey::from_slice(decrypted_derived_key)
            .context("Failed to create SecretKey from decrypted key")
    }
}

impl Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn generate_key_with_expire_time() -> Result<Vec<u8>> {
    let key = SecretKey::default();
    let key_bytes = key.unprotected_as_bytes();

    let expire_time = (time::OffsetDateTime::now_utc() + time::Duration::hours(8)).unix_timestamp();
    let expire_bytes = expire_time.to_be_bytes();

    let key_with_time = [&expire_bytes, key_bytes].concat();

    todo!()
}
