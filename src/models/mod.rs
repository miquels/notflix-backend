mod episode;
mod fileinfo;
mod mediainfo;
mod misc;
mod movie;
mod nfo;
mod session;
mod tvshow;
mod uniqueids;
mod user;

pub use episode::Episode;
pub use fileinfo::FileInfo;
pub use mediainfo::MediaInfo;
pub use misc::*;
pub use movie::Movie;
pub use nfo::{NfoBase, NfoMovie};
pub use session::Session;
pub use tvshow::{TVShow, Season};
pub use uniqueids::UniqueIds;
pub use user::{User, UpdateUser};

use async_trait::async_trait;
use anyhow::Result;

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
    async fn lookup_by(dbh: &mut db::TxnHandle<'_>, find: &db::FindItemBy<'_>) -> Result<Option<Box<Self>>>;
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

    async fn lookup_by(dbh: &mut db::TxnHandle<'_>, find: &db::FindItemBy<'_>) -> Result<Option<Box<Self>>> {
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

    async fn lookup_by(dbh: &mut db::TxnHandle<'_>, find: &db::FindItemBy<'_>) -> Result<Option<Box<Self>>> {
        Self::lookup_by(dbh, find, true).await
    }

    async fn insert(&mut self, txn: &mut db::TxnHandle<'_>) -> anyhow::Result<()> {
        self.insert(txn).await
    }

    async fn update(&self, txn: &mut db::TxnHandle<'_>) -> anyhow::Result<()> {
        self.update(txn).await
    }
}

// helper function.
fn is_default<'a, T>(t: &'a T) -> bool
where
    T: Default,
    T: PartialEq,
{
    *t == T::default()
}
