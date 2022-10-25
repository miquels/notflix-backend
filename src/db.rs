/// Database model.

use anyhow::Result;
use serde::Serialize;
use sqlx::sqlite::SqlitePool;

pub type DbHandle = SqlitePool;

#[derive(Serialize, sqlx::FromRow, Debug)]
pub struct Item {
    pub name: String,
    pub votes: Option<i64>,
    pub year: Option<i64>,
    pub genre: String,
    pub rating: Option<f32>,
    pub nfotime: i64,
    pub firstvideo: i64,
    pub lastvideo: i64,
}

pub async fn get_item(handle: &DbHandle, name: &str) -> Option<Item> {
    sqlx::query_as!(
        Item,
        "
    SELECT * from items where name = ?
        ",
        name
    )
    .fetch_one(handle)
    .await
    .ok()
}

pub async fn get_items(handle: &DbHandle) -> Result<Vec<Item>> {
    let items = sqlx::query_as!(
        Item,
        "SELECT * from items",
    )
    .fetch_all(handle)
    .await?;
    Ok(items)
}

pub async fn connect_db(db: &str) -> Result<DbHandle> {
    Ok(SqlitePool::connect(db).await?)
}

