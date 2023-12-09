use anyhow::Result;
use orion::pwhash::{self, hash_password_verify, Password, PasswordHash};
use sqlx::prelude::*;

#[derive(Debug, FromRow)]
pub struct App {
    pub master_password_hash: String,
}

impl App {
    pub async fn set_master_password(&self) -> Result<()> {
        todo!()
    }
}

/// Hashes the given password and returns it as a String
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let pw = Password::from_slice(password.as_bytes())?;
    let hash = pwhash::hash_password(&pw, 3, 1 << 16)?;
    Ok(hash.unprotected_as_encoded().to_string())
}

/// Verifies the given password aginst the given hash string
pub fn verify_hash(password: &str, hash: &Option<String>) -> bool {
    match hash {
        Some(h) => {
            let hash = match PasswordHash::from_encoded(h) {
                Ok(p) => p,
                Err(_) => return false,
            };
            let input_password = Password::from_slice(password.as_bytes()).unwrap_or_default();
            hash_password_verify(&hash, &input_password).is_ok()
        }
        None => false,
    }
}
