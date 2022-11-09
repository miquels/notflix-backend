/// Keep the database up to date.
///
/// This is where we put data scraped from the filesystem into
/// the database.

use anyhow::{Context, Result, bail};
use futures_util::TryStreamExt;
use sqlx::sqlite::SqlitePool;

use crate::collections::Collection;
use crate::models::{FileInfo, Movie, UniqueId};
use crate::kodifs;

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

    // Update one movie.
    pub async fn update_movie(&self, coll: &Collection, name: &str) -> Result<()> {

        // First, get the movie from the database.
        let mut by = FindItemBy::new();
        by.directory = Some(name);
        let oldmovie = match Movie::lookup(self, &by).await {
            Some(mv) => mv,
            None => Movie::default(),
        };

        // Now scan the movie directory.
        let mut movie = match kodifs::update_movie(coll, name, &oldmovie).await {
            Some(mv) => mv,
            None => bail!("failed to scan directory {}", name),
        };

        // insert or update?
        if oldmovie.id == 0 {
            // No ID yet, so it doesn't exist in the database.
            movie.insert(self).await
                .with_context(|| format!("failed to insert db for {}", name))?;
        } else if movie.lastmodified > oldmovie.lastmodified && oldmovie.lastmodified != 0 {
            // There was an update, so update the database.
            movie.update(self).await
                .with_context(|| format!("failed to update db for {}", name))?;
        }

        Ok(())
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

// helper.
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
