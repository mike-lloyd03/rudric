use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Password};
use orion::kex;
use sqlx::SqlitePool;

use crate::db;

use super::{session::SessionToken, user::User};

pub struct App {
    pub db: SqlitePool,
    pub derived_key: kex::SecretKey,
}

impl App {
    pub async fn new() -> Result<Self> {
        let db = db::connect().await?;
        let derived_key = match SessionToken::from_env() {
            Ok(st) => st.into_derived_key(&db).await?,
            Err(_) => {
                let input_password = Self::read_password()?;
                let user = Self::authenticate_user(&db, &input_password).await?;
                user.derive_key(&input_password)?
            }
        };

        Ok(Self { db, derived_key })
    }

    pub fn read_password() -> Result<String> {
        Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter master password")
            .report(false)
            .interact()
            .context("Failed to read user input")
    }

    pub async fn authenticate_user(db: &SqlitePool, password: &str) -> Result<User> {
        let user = User::load(db).await?;

        if user.authenticate(password) {
            Ok(user)
        } else {
            bail!("Invalid master password")
        }
    }
}
