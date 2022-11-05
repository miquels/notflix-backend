/// Keep the database up to date.
///
/// This is where we put data scraped from the filesystem into
/// the database.

use anyhow::Result;
use sqlx::sqlite::SqlitePool;

pub type DbHandle = SqlitePool;

pub async fn connect_db(db: &str) -> Result<DbHandle> {
    Ok(SqlitePool::connect(db).await?)
}

pub struct Db {
    pub handle: DbHandle,
}

/*
impl Db {
    pub async fn connect(db: &str) -> Result<DbHandle> {
        Ok(SqlitePool::connect(db).await?)
    }

    pub async fn scan(&self, coll: Collection) {
    }
}
*/
