use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Password};
use orion::kex::{self, SecretKey};
use sqlx::SqlitePool;

use crate::db;

use super::{session::SessionToken, user::User};

pub struct App {
    pub db: SqlitePool,
    pub derived_key: kex::SecretKey,
}

impl App {
    pub async fn new(check_session: bool) -> Result<Self> {
        let db = db::connect().await?;

        if check_session {
            if let Ok(st) = SessionToken::from_env() {
                let derived_key = st.into_derived_key(&db).await?;
                return Ok(Self { db, derived_key });
            }
        };

        let input_password = Self::read_password()?;
        let user = Self::authenticate_user(&db, &input_password).await?;
        let derived_key = user.derive_key(&input_password)?;

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
