/// Keep the database up to date.
///
/// This is where we put data scraped from the filesystem into
/// the database.

use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePool;

use crate::collections::Collection;
use crate::models::{Movie, UniqueId, UniqueIds};
use crate::kodifs::KodiFS;
use crate::util::SystemTimeToUnixTime;

pub type DbHandle = SqlitePool;

pub async fn connect_db(db: &str) -> Result<DbHandle> {
    Ok(SqlitePool::connect(db).await?)
}

pub struct Db {
    pub handle: DbHandle,
}

impl Db {
    pub async fn connect(db: &str) -> Result<Db> {
        let db = Db{ handle: SqlitePool::connect(db).await? };
        db.set_mediaitem_sequence().await?;
        Ok(db)
    }

    async fn set_mediaitem_sequence(&self) -> Result<()> {
        let mut txn = self.handle.begin().await?;

        let r: Option<i64> = sqlx::query!(
            r#"
                SELECT seq as "seq!: i64" FROM sqlite_sequence WHERE name = 'mediaitems'
            "#
        )
        .fetch_optional(&mut txn)
        .await?
        .map(|row| row.seq);

        match r {
            Some(r) if r < 1000 => {
                sqlx::query!("UPDATE sqlite_sequence SET seq = 1000 WHERE name = 'mediaitems'")
                    .execute(&mut txn)
                    .await?;
            },
            Some(_) => {},
            None => {
                sqlx::query!("INSERT INTO sqlite_sequence(name, seq) VALUES('mediaitems', 1000)")
                    .execute(&mut txn)
                    .await?;
            }
        }

        txn.commit().await?;

        Ok(())
    }

    // Update one movie.
    pub async fn update_movie(&self, coll: &Collection, name: &str) -> Result<()> {

        // Try to get the movie from the database by collection id and directory name.
        let by = FindItemBy::directory(coll.collection_id, name, false);
        let mut oldmovie = Movie::lookup_by(self, &by).await;

        // If not found, read the NFO file to get the uniqueids, and search for that.
        if oldmovie.is_none() {
            log::trace!("Db::update_movie: not found by name in db: {}", name);

            // Open the movies NFO file to read the unqiqueids.
            if let Some(mv) = Movie::scan_directory(coll, name, None, true).await {

                // Try to find the movie in the db by uniqueid.
                let by = FindItemBy::uniqueids(&mv.nfo_base.uniqueids, true);
                if let Some(oldmv) = Movie::lookup_by(self, &by).await {
                    log::trace!("Db::update_movie: found movie in db by uniqueid");
                    oldmovie = Some(oldmv);
                } else {
                    // Not in the db, but perhaps we did have it before,
                    // and we remembered the ID it had then.
                    if let Some(id) = self.lookup(&by).await {
                        log::trace!("Db::update_movie:: found movie id in db by uniqueid");
                        let mut mv = Box::new(Movie::default());
                        mv.id = id;
                        oldmovie = Some(mv);
                    }
                }
            }
        }
        let old_lastmodified = oldmovie.as_ref().map(|m| m.lastmodified).unwrap_or(0);

        // Now scan the movie directory.
        let mut movie = match Movie::scan_directory(coll, name, oldmovie, false).await {
            Some(mv) => mv,
            None => {
                // FIXME This is an error, but non-fatal, the transaction was not aborted.
                // So log an error and return "success".
                log::error!("db::update_movie: failed to scan directory {}", name);
                return Ok(());
            },
        };

        // insert or update?
        if movie.id == 0 {
            // No ID yet, so it doesn't exist in the database.
            log::debug!("Db::update_movie: adding new movie to the db: {}", name);
            movie.insert(self).await
                .with_context(|| format!("failed to insert db for {}", name))?;
        } else if movie.lastmodified > old_lastmodified && old_lastmodified != 0 {
            // There was an update, so update the database.
            log::debug!("Db::update_movie: updating movie in the db: {}", name);
            movie.update(self).await
                .with_context(|| format!("failed to update db for {}", name))?;
        } else {
            log::trace!("Db::update_movie: no update needed for: {}", name);
        }

        if let Some(nfofile) = movie.nfofile.as_ref() {
            if nfofile.modified.unixtime_ms() > old_lastmodified {
                let uids = UniqueIds::new(movie.id);
                uids.update(self, &movie.nfo_base.uniqueids).await
                    .with_context(|| format!("failed to update uniqueids table for {}", name))?;
            }
        }

        Ok(())
    }
/*
    // Update a collection of movies.
    pub async fn update_movie_collection(&self, coll: &Collection) -> Result<()> {

        // Get a list of directories from the filesystem.
        let mut names = Vec::new();
        let mut dirs = scandirs::read_dir(&coll.directory, false, &mut names).await;
        if dirs.len() == 0 {
            log::error!("Db:update_movie_collection: empty dir: {}", coll.directory);
            return Ok(());
        }


    // Lookup a movie or tvshow in the database and return it's ID.
    pub async fn lookup(&self, by: &FindItemBy<'_>) -> Option<i64> {
        let id = sqlx::query!(
            r#"
                SELECT  i.id
                FROM mediaitems i
                WHERE (? IS NULL OR collection_id = ?)
                  AND (id = ? OR json_extract(directory, '$.path') = ? OR title = ?)"#,
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
    pub deleted_too: bool,
}

impl<'a> FindItemBy<'a> {

    pub fn new() -> FindItemBy<'a> {
        FindItemBy::default()
    }

    pub fn id(id: i64, deleted_too: bool) -> FindItemBy<'a> {
        FindItemBy { id: Some(id), deleted_too, ..FindItemBy::default() }
    }

    pub fn uniqueids(uids: &'a [UniqueId], deleted_too: bool) -> FindItemBy<'a> {
        FindItemBy { uniqueids: uids, deleted_too, ..FindItemBy::default() }
    }

    pub fn directory(coll_id: u32, dir: &'a str, deleted_too: bool) -> FindItemBy<'a> {
        FindItemBy {
            collection_id: Some(coll_id as i64),
            directory: Some(dir),
            deleted_too,
            ..FindItemBy::default() }
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
