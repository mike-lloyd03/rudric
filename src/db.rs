use anyhow::{bail, Result};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};

pub async fn init() -> Result<SqlitePool> {
    let db_url = db_url()?;

    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        println!("Creating database {}", db_url);
        match Sqlite::create_database(&db_url).await {
            Ok(_) => println!("Create db success"),
            Err(error) => bail!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    connect().await
}

pub fn db_url() -> Result<String> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("rudric")?;
    let db_file = xdg_dirs.place_config_file("data.db")?;
    let db_url = format!(
        "sqlite://{}",
        db_file.to_str().expect("db path should be utf-8")
    );

    Ok(db_url)
}

pub async fn connect() -> Result<SqlitePool> {
    Ok(SqlitePool::connect(&db_url()?).await?)
}
