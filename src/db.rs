use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};

pub async fn init(config_dir: &Path) -> Result<SqlitePool> {
    let db_url = db_url(config_dir)?;
    let db_path = db_path(config_dir);

    println!("Creating database {}", db_path.to_string_lossy());

    match Sqlite::create_database(&db_url).await {
        Ok(_) => println!("Create db success"),
        Err(error) => bail!("error: {}", error),
    }

    let perms = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(db_path, perms)?;

    let db = connect(config_dir).await?;
    sqlx::migrate!().run(&db).await?;
    Ok(db)
}

/// Checks if the db file exists and runs the latest migrations on it to ensure it is ready for use
pub async fn exists(config_dir: &Path) -> Result<bool> {
    let db_url = db_url(config_dir)?;

    if !Sqlite::database_exists(&db_url).await? {
        return Ok(false);
    }

    let db = connect(config_dir).await?;
    sqlx::migrate!().run(&db).await?;

    Ok(true)
}

pub fn db_url(config_dir: &Path) -> Result<String> {
    Ok(format!(
        "sqlite://{}",
        db_path(config_dir).to_string_lossy()
    ))
}

pub fn db_path(config_dir: &Path) -> PathBuf {
    config_dir.join("data.db")
}

pub async fn connect(config_dir: &Path) -> Result<SqlitePool> {
    Ok(SqlitePool::connect(&db_url(config_dir)?).await?)
}
