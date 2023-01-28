use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::jvec::JVec;
use crate::models::{FileInfo, Nfo, Thumb, Video};
use crate::sqlx::impl_sqlx_traits_for;
use crate::util::Id;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct MediaItem {
    /// "movie", "tvshow", "episode".
    pub type_: String,
    /// Unique ID.
    pub id: Id,
    /// Collection id
    pub collection_id: u32,
    /// Last modified (timestamp of newest file/directory).
    pub lastmodified: i64,
    /// Date added YYYY-MM-DD
    pub dateadded: String,
    /// Directory relative to the collection directory
    pub directory: Option<FileInfo>,
    /// Deleted?
    pub deleted: bool,

    /// Title.
    pub title: String,
    /// Year (movies only)
    pub year: Option<u32>,

    /// Nfo file.
    pub nfo_file: Option<FileInfo>,
    /// Info about this item from themoviedb / thetvdb, probably from nfo_file.
    pub nfo_info: Option<Nfo>,

    /// Thumbs, posters, fanart etc on the filesystem.
    pub thumbs: JVec<Thumb>,
    // pub subtitles: JVec<Subtitle>,

    /// Video file and info.
    pub video_file: Option<FileInfo>,
    pub video_info: Option<Video>,

    /// Episode specific.
    pub season: Option<u32>,
    /// Episode specific.
    pub episode: Option<u32>,
    /// Episode specific.
    pub tvshow_id: Option<Id>,
}
impl_sqlx_traits_for!(MediaItem);

impl MediaItem {
    pub async fn lookup_by(
        dbh: &mut db::TxnHandle<'_>,
        find: &db::FindItemBy<'_>,
    ) -> Result<Option<Box<MediaItem>>> {
        // Find the ID.
        let id = match find.is_only_id() {
            Some(id) => id,
            None => match db::lookup(dbh, &find).await? {
                Some(id) => id,
                None => return Ok(None),
            },
        };

        // Find the item in the database.
        let r = sqlx::query_as!(
            MediaItem,
            r#"
                SELECT id AS "id: Id",
                       type AS "type_",
                       collection_id AS "collection_id: u32",
                       lastmodified,
                       dateadded,
                       directory AS "directory?: FileInfo",
                       deleted AS "deleted!: bool",
                       title AS "title!: String",
                       year AS "year?: u32",
                       nfo_file AS "nfo_file?: FileInfo",
                       nfo_info AS "nfo_info?: Nfo",
                       thumbs AS "thumbs!: JVec<Thumb>",
                       video_file AS "video_file?: FileInfo",
                       video_info AS "video_info?: Video",
                       season AS "season?: u32",
                       episode AS "episode?: u32",
                       tvshow_id AS "tvshow_id?: Id"
                FROM mediaitems
                WHERE id = ? AND (deleted = 0 OR deleted = ?)"#,
            id,
            find.deleted_too,
        )
        .fetch_optional(dbh)
        .await?;

        Ok(r.map(|r| Box::new(r)))
    }

    pub async fn insert(&self, txn: &mut db::TxnHandle<'_>) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO mediaitems(
                    type,
                    id,
                    collection_id,
                    lastmodified,
                    dateadded,
                    directory,
                    deleted,
                    title,
                    year,
                    nfo_file,
                    nfo_info,
                    thumbs,
                    video_file,
                    video_info,
                    season,
                    episode,
                    tvshow_id
                ) VALUES("movie", ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.collection_id,
            self.lastmodified,
            self.dateadded,
            self.directory,
            self.deleted,
            self.title,
            self.year,
            self.nfo_file,
            self.nfo_info,
            self.thumbs,
            self.video_file,
            self.video_info,
            self.season,
            self.episode,
            self.tvshow_id,
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }

    pub async fn update(&self, txn: &mut db::TxnHandle<'_>) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE mediaitems SET
                    collection_id = ?,
                    lastmodified = ?,
                    dateadded = ?,
                    directory = ?,
                    deleted = ?,
                    nfo_file = ?,
                    nfo_info = ?,
                    thumbs = ?,
                    video_file = ?,
                    video_info = ?,
                    season = ?,
                    episode = ?,
                    tvshow_id = ?
                WHERE id = ?"#,
            self.collection_id,
            self.lastmodified,
            self.dateadded,
            self.directory,
            self.deleted,
            self.nfo_file,
            self.nfo_info,
            self.thumbs,
            self.video_file,
            self.video_info,
            self.season,
            self.episode,
            self.tvshow_id,
            self.id
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }
}
