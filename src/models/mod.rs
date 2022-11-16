mod movie;
mod tvshow;
mod episode;
mod misc;
mod nfo;
mod fileinfo;
mod uniqueids;

pub use movie::Movie;
pub use tvshow::{TVShow, Season};
pub use episode::Episode;
pub use fileinfo::FileInfo;
pub use uniqueids::UniqueIds;
pub use misc::*;
pub use nfo::{NfoBase, NfoMovie};

use async_trait::async_trait;

use crate::db;
use crate::util::SystemTimeToUnixTime;

#[async_trait]
pub trait MediaItem {
    fn id(&self) -> i64;
    fn set_id(&mut self, id: i64);
    fn set_collection_id(&mut self, id: i64);
    fn uniqueids(&self) -> &'_ [UniqueId];
    fn lastmodified(&self) -> i64;
    fn nfo_lastmodified(&self) -> Option<i64>;
    fn undelete(&mut self);
    async fn lookup_by(dbh: &mut db::TxnHandle<'_>, find: &db::FindItemBy<'_>) -> Option<Box<Self>>;
    async fn insert(&mut self, txn: &mut db::TxnHandle<'_>) -> anyhow::Result<()>;
    async fn update(&self, txn: &mut db::TxnHandle<'_>) -> anyhow::Result<()>;
}

#[async_trait]
impl MediaItem for Movie {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = id;
    }

    fn set_collection_id(&mut self, id: i64) {
        self.collection_id = id;
    }

    fn uniqueids(&self) -> &'_ [UniqueId] {
        self.nfo_base.uniqueids.0.as_ref()
    }

    fn lastmodified(&self) -> i64 {
        self.lastmodified
    }

    fn nfo_lastmodified(&self) -> Option<i64> {
        self.nfofile.as_ref().map(|m| m.modified.unixtime_ms())
    }

    fn undelete(&mut self) {
        if self.deleted {
            self.deleted = false;
            self.lastmodified = 0;
        }
    }

    async fn lookup_by(dbh: &mut db::TxnHandle<'_>, find: &db::FindItemBy<'_>) -> Option<Box<Self>> {
        Self::lookup_by(dbh, find).await
    }

    async fn insert(&mut self, txn: &mut db::TxnHandle<'_>) -> anyhow::Result<()> {
        self.insert(txn).await
    }

    async fn update(&self, txn: &mut db::TxnHandle<'_>) -> anyhow::Result<()> {
        self.update(txn).await
    }
}

#[async_trait]
impl MediaItem for TVShow {
    fn id(&self) -> i64 {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = id;
    }

    fn set_collection_id(&mut self, id: i64) {
        self.collection_id = id;
    }

    fn uniqueids(&self) -> &'_ [UniqueId] {
        self.nfo_base.uniqueids.0.as_ref()
    }

    fn lastmodified(&self) -> i64 {
        self.lastmodified
    }

    fn nfo_lastmodified(&self) -> Option<i64> {
        self.nfofile.as_ref().map(|m| m.modified.unixtime_ms())
    }

    fn undelete(&mut self) {
        if self.deleted {
            self.deleted = false;
            self.lastmodified = 0;
        }
    }

    async fn lookup_by(dbh: &mut db::TxnHandle<'_>, find: &db::FindItemBy<'_>) -> Option<Box<Self>> {
        Self::lookup_by(dbh, find).await
    }

    async fn insert(&mut self, txn: &mut db::TxnHandle<'_>) -> anyhow::Result<()> {
        self.insert(txn).await
    }

    async fn update(&self, txn: &mut db::TxnHandle<'_>) -> anyhow::Result<()> {
        self.update(txn).await
    }
}

type J<T> = sqlx::types::Json<T>;
type JV<T> = sqlx::types::Json<Vec<T>>;

// helper function.
fn is_default<'a, T>(t: &'a T) -> bool
where
    T: Default,
    T: PartialEq,
{
    *t == T::default()
}
