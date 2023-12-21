use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Password};
use orion::aead;
use sqlx::SqlitePool;

use crate::{db, prompt};

use super::{session::SessionToken, user::User};

pub struct App {
    pub db: SqlitePool,
    pub master_key: aead::SecretKey,
}

impl App {
    pub async fn new(check_session: bool) -> Result<Self> {
        if !db::exists().await? {
            bail!("Vault not found at {}", db::db_path()?)
        }

        let db = db::connect().await?;

        if check_session {
            if let Ok(st) = SessionToken::from_env() {
                let master_key = st.into_master_key(&db).await?;
                return Ok(Self { db, master_key });
            }
        };

        let input_password = prompt::read_password()?;
        let user = Self::authenticate_user(&db, &input_password).await?;
        let master_key = user.master_key(&input_password)?;

        Ok(Self { db, master_key })
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
