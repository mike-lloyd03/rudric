use anyhow::{anyhow, bail, Context, Result};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};

pub async fn init() -> Result<SqlitePool> {
    let db_url = db_url()?;

    println!("Creating database {}", db_path()?);
    match Sqlite::create_database(&db_url).await {
        Ok(_) => println!("Create db success"),
        Err(error) => bail!("error: {}", error),
    }

    let db = connect().await?;
    sqlx::migrate!().run(&db).await?;
    Ok(db)
}

/// Checks if the db file exists and runs the latest migrations on it to ensure it is ready for use
pub async fn exists() -> Result<bool> {
    let db_url = db_url()?;

    if !Sqlite::database_exists(&db_url).await? {
        return Ok(false);
    }

    let db = connect().await?;
    sqlx::migrate!().run(&db).await?;

    Ok(true)
}

pub fn db_url() -> Result<String> {
    Ok(format!("sqlite://{}", db_path()?))
}

pub fn default_config_dir() -> Result<String> {
    let xdg_dirs = xdg::BaseDirectories::new()?;
    let path = xdg_dirs.create_config_directory("rudric")?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or(anyhow!("Failed to create config dir"))
}

pub fn db_path() -> Result<String> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("rudric")?;
    let db_file = xdg_dirs.place_config_file("data.db")?;
    Ok(db_file
        .to_str()
        .context("Failed to get database filepath")?
        .to_string())
}

pub async fn connect() -> Result<SqlitePool> {
    Ok(SqlitePool::connect(&db_url()?).await?)
}
