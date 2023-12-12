use anyhow::{Context, Result};
use orion::{
    kdf, kex,
    pwhash::{self, hash_password_verify},
};

pub fn derive_key(password: &str, salt: &str) -> Result<kex::SecretKey> {
    let password = kdf::Password::from_slice(password.as_bytes())?;
    let salt = kdf::Salt::from_slice(salt.as_bytes())?;
    kdf::derive_key(&password, &salt, 3, 1 << 16, 32).context("Failed to derive key")
}

pub fn hash_password(password: &str) -> Result<pwhash::PasswordHash> {
    let password = pwhash::Password::from_slice(password.as_bytes())?;
    pwhash::hash_password(&password, 3, 1 << 16).context("Failed to hash password")
}

/// Hashes the given password and returns it as a String
pub fn hash_password_string(password: &str) -> anyhow::Result<String> {
    let pw = pwhash::Password::from_slice(password.as_bytes())?;
    let hash = pwhash::hash_password(&pw, 3, 1 << 16)?;
    Ok(hash.unprotected_as_encoded().to_string())
}

/// Verifies the given password aginst the given hash string
pub fn verify_hash(password: &str, hash: &Option<String>) -> bool {
    match hash {
        Some(h) => {
            let hash = match pwhash::PasswordHash::from_encoded(h) {
                Ok(p) => p,
                Err(_) => return false,
            };
            let input_password =
                pwhash::Password::from_slice(password.as_bytes()).unwrap_or_default();
            hash_password_verify(&hash, &input_password).is_ok()
        }
        None => false,
    }
}
