/// Keep the database up to date.
///
/// This is where we put data scraped from the filesystem into
/// the database.
///
use std::collections::HashMap;
use std::io::ErrorKind;

use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePool;

use crate::collections::Collection;
use crate::jvec::JVec;
use crate::kodifs::{self, scandirs};
use crate::models::{MediaItem, UniqueId, UniqueIds};
use crate::util::{Id, SystemTimeToUnixTime};

pub type DbHandle = SqlitePool;
pub type TxnHandle<'a> = sqlx::Transaction<'a, sqlx::Sqlite>;

enum ItemType {
    Movie,
    TVShow,
}

#[derive(Clone)]
pub struct Db {
    pub handle: DbHandle,
}

impl Db {
    pub async fn connect(db: &str) -> Result<Db> {
        let db = Db { handle: SqlitePool::connect(db).await? };
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
            },
        }

        txn.commit().await?;

        Ok(())
    }

    // Update one movie.
    pub async fn update_mediaitem(
        &self,
        coll: &Collection,
        name: &str,
        txn: &mut TxnHandle<'_>,
    ) -> Result<Option<Id>> {
        let mut need_update = false;

        // Try to get the item from the database by collection id and directory name.
        log::trace!("Db::update_mediaitem: database lookup for {}", name);
        let by = FindItemBy::directory(coll.collection_id, name, false);
        let mut db_item = MediaItem::lookup_by(&mut *txn, &by).await?;
        log::trace!("Db::update_mediaitem: found: {:?}", db_item.is_some());

        // If not found, read the NFO file to get the uniqueids, and search for that.
        if db_item.is_none() {
            log::trace!("Db::update_mediaitem: not found by name in db: {}", name);

            // Open the mediaitem's NFO file to read the unqiqueids.
            if let Some(mv) = kodifs::scan_mediaitem_dir(coll, name, None, true).await {
                // Try to find the mediaitem in the db by uniqueid.
                if let Some(nfo_info) = mv.nfo_info.as_ref() {
                    let by = FindItemBy::uniqueids(&nfo_info.uniqueids, true);
                    if let Some(mut oldmv) = MediaItem::lookup_by(&mut *txn, &by).await? {
                        log::trace!("Db::update_mediaitem: found item in db by uniqueid");
                        oldmv.collection_id = coll.collection_id;
                        db_item = Some(oldmv);
                        need_update = true;
                    } else {
                        // Not in the db, but perhaps we did have it before,
                        // and we remembered the ID it had then.
                        if let Some(id) = lookup(&mut *txn, &by).await? {
                            log::trace!("Db::update_mediaitem:: found item id in db by uniqueid");
                            let mut mv = Box::new(MediaItem::default());
                            mv.id = id;
                            mv.collection_id = coll.collection_id;
                            db_item = Some(mv);
                            need_update = true;
                        }
                    }
                }
            }
        }
        if let Some(db_item) = db_item.as_mut() {
            if db_item.deleted {
                db_item.deleted = false;
                db_item.lastmodified = 0;
            }
        }
        let old_lastmodified = db_item.as_ref().map(|m| m.lastmodified).unwrap_or(0);
        let is_new = db_item.is_none();

        // Now scan the item's directory.
        let item = match kodifs::scan_mediaitem_dir(coll, name, db_item, false).await {
            Some(mv) => mv,
            None => {
                // FIXME This is an error, but non-fatal, the transaction was not aborted.
                // So log an error and return "success".
                log::error!("db::update_mediaitem: failed to scan directory {}", name);
                return Ok(None);
            },
        };

        // insert or update?
        if is_new {
            log::debug!("Db::update_mediaitem: adding new item to the db: {}", name);
            item.insert(&mut *txn)
                .await
                .with_context(|| format!("failed to insert db for {}", name))?;
        } else if item.lastmodified > old_lastmodified || need_update {
            // There was an update, so update the database.
            log::debug!("Db::update_mediaitem: updating item in the db: {}", name);
            item.update(&mut *txn)
                .await
                .with_context(|| format!("failed to update db for {}", name))?;
        } else {
            log::trace!("Db::update_mediaitem: no update needed for: {}", name);
        }

        log::trace!("checking UniqueIds..");
        if let Some(nfo_lastmodified) = item.nfo_file.map(|f| f.modified) {
            if nfo_lastmodified.unixtime_ms() > old_lastmodified {
                log::trace!("updating UniqueIds..");
                let uids = UniqueIds::new(item.id);
                let empty = JVec::new();
                let orig_uids = item.nfo_info.as_ref().map(|n| &n.uniqueids).unwrap_or(&empty);
                uids.update(&mut *txn, orig_uids)
                    .await
                    .with_context(|| format!("failed to update uniqueids table for {}", name))?;
            }
        }

        log::trace!("done.");
        Ok(Some(item.id))
    }

    // Update a collection of movies / tvshows.
    //
    // Returns Ok if we can commit, error if not.
    pub async fn update_collection(&self, coll: &Collection) -> Result<()> {
        let r = async {
            let mut txn = self.handle.begin().await?;
            match self.do_update_collection(coll, &mut txn).await {
                Ok(()) => Ok(txn.commit().await?),
                Err(e) => {
                    let _ = txn.rollback().await;
                    return Err(e)?;
                },
            }
        }
        .await
        .map_err(|e: anyhow::Error| e);

        if let Err(e) = r {
            log::error!("Db::update_collection({}): {}", coll.directory, e);
            return Err(e)?;
        }

        Ok(())
    }

    async fn do_update_collection(&self, coll: &Collection, txn: &mut TxnHandle<'_>) -> Result<()> {
        // Get a list of directories from the filesystem.
        log::debug!("update_collection: scanning directory {}", coll.directory);
        let mut dirs = scandirs::scan_directories(coll, true).await;
        if dirs.len() == 0 {
            log::error!("Db:update_collection: empty dir: {}", coll.directory);
            return Ok(());
        }

        #[derive(Default)]
        struct DbItem {
            id: Id,
            dir: String,
            lastmodified: i64,
            keep: bool,
        }

        // Get a list of items from the database.
        log::debug!("update_collection: scanning database");
        let mut items = sqlx::query_as!(
            DbItem,
            r#"
                SELECT id AS "id!: Id",
                       json_extract(directory, '$.path') AS "dir!: String",
                       lastmodified,
                       0 AS "keep!: bool"
                FROM mediaitems
                WHERE collection_id = ?
                  AND deleted != 1"#,
            coll.collection_id
        )
        .fetch_all(&mut *txn)
        .await?;

        // Put the items from the database in a HashMap
        let mut map = items.drain(..).map(|m| (m.id, m)).collect::<HashMap<_, _>>();

        // For each item in the database.
        log::debug!("update_collection: starting loop over db items ({})", map.len());
        for (_id, dbitem) in map.iter_mut() {
            // Remove from the list of filesystem directories.
            dirs.remove(&dbitem.dir);

            // Get the last modified stamp of the files in the directory.
            match scandirs::scan_directory(coll, &dbitem.dir, true).await {
                Ok(ts) => {
                    if ts <= dbitem.lastmodified {
                        // Not modified.
                        log::trace!("update_collection: no change: {}", dbitem.dir);
                        dbitem.keep = true;
                        continue;
                    }
                    log::trace!("update_collection: modified: {}", dbitem.dir);
                },
                Err(e) => {
                    if e.kind() != ErrorKind::NotFound {
                        log::error!("Db:update_collection: {}: {}", dbitem.dir, e);
                    } else {
                        log::trace!("update_collection: removed: {}", dbitem.dir);
                    }
                    continue;
                },
            }

            // Ok, we have to do a full rescan of this item.
            // self.update_movie will only return an error for SQL errors.
            if self.update_mediaitem(coll, &dbitem.dir, &mut *txn).await?.is_some() {
                // successfully updated.
                dbitem.keep = true;
            }
        }

        // Now first loop over all the directories in the filesystem
        // for which there was no database entry yet.
        log::trace!("adding new directories ({})", dirs.len());
        for dir in dirs.keys() {
            log::trace!("adding {}", dir);
            if let Some(id) = self.update_mediaitem(coll, dir, &mut *txn).await? {
                map.remove(&id);
            }
        }

        // Finally set the deleted flag on all State::Deleted entries.
        log::trace!("setting deleted flags ({})", map.iter().filter(|(_, v)| !v.keep).count());
        for (id, dbitem) in map.iter().filter(|(_, item)| !item.keep) {
            log::trace!("update_collection: marking as deleted: {}", dbitem.dir);
            sqlx::query!(
                r#"
                    UPDATE mediaitems
                    SET deleted = 1
                    WHERE id = ?"#,
                *id
            )
            .execute(&mut *txn)
            .await?;
        }

        Ok(())
    }

    // Lookup a movie or tvshow in the database and return it's ID.
    pub async fn lookup(&self, by: &FindItemBy<'_>) -> Result<Option<Id>> {
        let mut txn = self.handle.begin().await?;
        let res = lookup(&mut txn, by).await?;
        txn.commit().await?;
        Ok(res)
    }
}

// Lookup a movie or tvshow in the database and return it's ID.
pub async fn lookup(txn: &mut TxnHandle<'_>, by: &FindItemBy<'_>) -> Result<Option<Id>> {
    let id = sqlx::query!(
        r#"
            SELECT  i.id AS "id!: Id"
            FROM mediaitems i
            WHERE (? IS NULL OR collection_id = ?)
              AND (id = ? OR json_extract(directory, '$.path') = ? OR title = ?)"#,
        by.collection_id,
        by.collection_id,
        by.id,
        by.directory,
        by.title,
    )
    .fetch_optional(&mut *txn)
    .await?
    .map(|row| row.id);

    if id.is_some() {
        return Ok(id);
    }

    // OK now the UniqueId lookup.
    UniqueIds::get_mediaitem_id(txn, &by.uniqueids).await
}

#[derive(Default, Debug)]
pub struct FindItemBy<'a> {
    pub id: Option<Id>,
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

    pub fn id(id: Id, deleted_too: bool) -> FindItemBy<'a> {
        FindItemBy {
            id: Some(id),
            deleted_too,
            ..FindItemBy::default()
        }
    }

    pub fn uniqueids(uids: &'a [UniqueId], deleted_too: bool) -> FindItemBy<'a> {
        FindItemBy {
            uniqueids: uids,
            deleted_too,
            ..FindItemBy::default()
        }
    }

    pub fn directory(coll_id: u32, dir: &'a str, deleted_too: bool) -> FindItemBy<'a> {
        FindItemBy {
            collection_id: Some(coll_id as i64),
            directory: Some(dir),
            deleted_too,
            ..FindItemBy::default()
        }
    }

    pub(crate) fn is_only_id(&self) -> Option<Id> {
        if let Some(id) = self.id {
            if self.title.is_none() && self.directory.is_none() && self.uniqueids.len() == 0 {
                return Some(id);
            }
        }
        None
    }
}
