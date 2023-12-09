use sqlx::prelude::*;

#[derive(Debug, FromRow)]
pub struct Secret {
    pub id: i64,
    pub name: String,
    pub value: String,
}
