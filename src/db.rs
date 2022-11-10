/// Keep the database up to date.
///
/// This is where we put data scraped from the filesystem into
/// the database.

use anyhow::{Context, Result, bail};
use sqlx::sqlite::SqlitePool;

use crate::collections::Collection;
use crate::models::{Movie, UniqueId, UniqueIds};
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

        // Try to get the movie from the database by collection id and directory name.
        let by = FindItemBy::directory(coll.collection_id, name);
        let mut oldmovie = Movie::lookup_by(self, &by).await.unwrap_or_else(|| Movie::default());

        // If not found, read the NFO file to get the uniqueids, and search for that.
        if oldmovie.id == 0 {
            // println!("movie not found by dirname"); 
            if let Some(mv) = kodifs::update_movie(coll, name, &oldmovie, true).await {
                // println!("succeeded in reading nfo: {:?}", mv.nfo_base.uniqueids);
                let by = FindItemBy::uniqueids(&mv.nfo_base.uniqueids);
                if let Some(id) = self.lookup(&by).await {
                    // println!("found existing movie via uniqueids: {id}");
                    let by = FindItemBy::id(id);
                    if let Some(oldmv) = Movie::lookup_by(self, &by).await {
                        // println!("Found oldmovie in database");
                        oldmovie = oldmv;
                        oldmovie.directory = mv.directory.clone();
                        oldmovie.lastmodified = 1;
                    } else {
                        // println!("movie not in database, but re-using id");
                        oldmovie.id = id;
                    }
                }
            }
        }

        // Now scan the movie directory.
        let mut movie = match kodifs::update_movie(coll, name, &oldmovie, false).await {
            Some(mv) => mv,
            None => bail!("failed to scan directory {}", name),
        };

        // insert or update?
        if oldmovie.id == 0 {
            // println!("INSERT movie");
            // No ID yet, so it doesn't exist in the database.
            movie.insert(self).await
                .with_context(|| format!("failed to insert db for {}", name))?;
            let uids = UniqueIds::new(movie.id);
            uids.update(self, &movie.nfo_base.uniqueids).await?;
        } else if movie.lastmodified > oldmovie.lastmodified && oldmovie.lastmodified != 0 {
            // println!("UPDATE movie");
            // There was an update, so update the database.
            movie.update(self).await
                .with_context(|| format!("failed to update db for {}", name))?;
            let uids = UniqueIds::new(movie.id);
            uids.update(self, &movie.nfo_base.uniqueids).await?;
        }

        Ok(())
    }

    pub async fn lookup(&self, by: &FindItemBy<'_>) -> Option<i64> {
        let id = sqlx::query!(
            r#"
                SELECT  i.id
                FROM mediaitems i
                WHERE (? IS NULL OR collection_id = ?)
                  AND (id = ? OR directory = ? OR title = ?)"#,
                by.collection_id,
                by.collection_id,
                by.id,
                by.directory,
                by.title,
        )
        .fetch_optional(&self.handle)
        .await
        .ok()
        .flatten()
        .map(|row| row.id);

        if id.is_some() {
            return id;
        }

        // OK now the UniqueId lookup.
        UniqueIds::get_mediaitem_id(self, &by.uniqueids).await
    }
}

#[derive(Default, Debug)]
pub struct FindItemBy<'a> {
    pub id: Option<i64>,
    pub collection_id: Option<i64>,
    pub directory: Option<&'a str>,
    pub title: Option<&'a str>,
    pub uniqueids: &'a [UniqueId],
}

impl<'a> FindItemBy<'a> {

    pub fn new() -> FindItemBy<'a> {
        FindItemBy::default()
    }

    pub fn id(id: i64) -> FindItemBy<'a> {
        FindItemBy { id: Some(id), ..FindItemBy::default() }
    }

    pub fn uniqueids(uids: &'a [UniqueId]) -> FindItemBy<'a> {
        FindItemBy { uniqueids: uids, ..FindItemBy::default() }
    }

    pub fn directory(coll_id: u32, dir: &'a str) -> FindItemBy<'a> {
        FindItemBy { collection_id: Some(coll_id as i64), directory: Some(dir), ..FindItemBy::default() }
    }

    pub(crate) fn is_only_id(&self) -> Option<i64> {
        if let Some(id) = self.id {
            if self.title.is_none() &&
               self.directory.is_none() &&
               self.uniqueids.len() == 0 {
                return Some(id);
            }
        }
        None
    }
}
