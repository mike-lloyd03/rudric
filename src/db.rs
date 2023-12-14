use anyhow::{bail, Context, Result};
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

pub async fn exists() -> Result<bool> {
    let db_url = db_url()?;

    Ok(Sqlite::database_exists(&db_url).await?)
}

pub fn db_url() -> Result<String> {
    Ok(format!("sqlite://{}", db_path()?))
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
