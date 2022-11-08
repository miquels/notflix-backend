/// Keep the database up to date.
///
/// This is where we put data scraped from the filesystem into
/// the database.

use anyhow::Result;
use futures_util::TryStreamExt;
use sqlx::sqlite::SqlitePool;

use crate::models::{FileInfo, UniqueId};

pub type DbHandle = SqlitePool;

pub async fn connect_db(db: &str) -> Result<DbHandle> {
    Ok(SqlitePool::connect(db).await?)
}

pub struct Db {
    pub handle: DbHandle,
}

impl Db {
    pub async fn connect(db: &str) -> Result<Db> {
        Ok(Db {
            handle: SqlitePool::connect(db).await?,
        })
    }

    // TODO: if multiple matches, return the one we trust most (a match on 'id'
    //       has 100% trust, ofcourse).
    //       Later, return a Vec of (id, matched_on, trust) instead of just one value.
    pub async fn lookup(&self, by: &FindItemBy<'_>) -> Option<i64> {
        // If we match on id, return right away.
        // It's basically just a test 'is this entry in the db'.
        if let Some(id) = by.id {
            let row = sqlx::query!(
                r#"
                    SELECT i.id
                    FROM mediaitems i
                    WHERE i.id == ?"#,
                id
            )
            .fetch_one(&self.handle)
            .await
            .ok();
            if row.is_some() {
                return Some(id);
            }
        }

        #[derive(sqlx::FromRow)]
        struct Result {
            id: i64,
            directory: sqlx::types::Json<FileInfo>,
            title: Option<String>,
            pub uniqueids: sqlx::types::Json<Vec<UniqueId>>,
        }
        let mut rows = sqlx::query_as!(
            Result,
            r#"
                SELECT  i.id,
                        i.directory AS "directory: _",
                        i.title, 
                        i.uniqueids AS "uniqueids: _"
                FROM mediaitems i
                WHERE i.collection_id = ? OR ? IS NULL"#,
                by.collection_id,
                by.collection_id,
        )
        .fetch(&self.handle);

        // Inspect each row. Could do this in SQL, but we might want to
        // compare directory and/or title in a fuzzy way.
        while let Some(row) = rows.try_next().await.unwrap_or(None) {
            let mut res = false;
            res |= by.id.map(|x| x == row.id).unwrap_or(false);
            res |= by.imdb.map(|x| has_uid(&row.uniqueids, "imdb", x)).unwrap_or(false);
            res |= by.tmdb.map(|x| has_uid(&row.uniqueids, "tmdb", x)).unwrap_or(false);
            res |= by.tvdb.map(|x| has_uid(&row.uniqueids, "tvdb", x)).unwrap_or(false);
            res |= by.directory.map(|x| x == row.directory.path).unwrap_or(false);
            let title = row.title.as_ref().map(|p| p.as_str());
            res |= by.title.is_some() && by.title == title;
            if res {
                return Some(row.id);
            }
        }
        None
    }
}

fn has_uid(uids: &Vec<UniqueId>, idtype: &str, id: &str) -> bool {
    for uid in uids {
        let uid_idtype = uid.idtype.as_ref().map(|s| s.as_str()).unwrap_or("");
        if uid_idtype == idtype && uid.id == id {
            return true;
        }
    }
    false
}

#[derive(Default)]
pub struct FindItemBy<'a> {
    pub id: Option<i64>,
    pub collection_id: Option<i64>,
    pub imdb: Option<&'a str>,
    pub tmdb: Option<&'a str>,
    pub tvdb: Option<&'a str>,
    pub title: Option<&'a str>,
    pub directory: Option<&'a str>,
}

impl<'a> FindItemBy<'a> {

    pub fn new() -> FindItemBy<'a> {
        FindItemBy::default()
    }

    pub(crate) fn is_only_id(&self) -> Option<i64> {
        if let Some(id) = self.id {
            if self.imdb.is_none() &&
                self.imdb.is_none() &&
                self.tmdb.is_none() &&
                self.tvdb.is_none() &&
                self.title.is_none() &&
                self.directory.is_none() {
                return Some(id);
            }
        }
        None
    }
}
