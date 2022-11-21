use anyhow::Result;
use serde::Serialize;
use futures_util::TryStreamExt;
use poem_openapi::Object;

use crate::db;
use crate::jvec::JVec;
use super::nfo::build_struct;
use super::{Rating, Thumb, UniqueId, Actor};
use super::{FileInfo, NfoBase, is_default};

#[derive(Object, Serialize, serde::Deserialize, Clone, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct Episode {
    // Common.
    pub id: i64,
    #[oai(skip)]
    pub collection_id: i64,
    pub tvshow_id: i64,
    #[oai(skip)]
    pub directory: FileInfo,
    #[oai(skip)]
    pub deleted: bool,
    #[oai(skip)]
    pub lastmodified: i64,
    #[oai(skip_serializing_if = "is_default")]
    pub dateadded: Option<String>,
    #[oai(skip)]
    pub nfofile: Option<FileInfo>,

    // Common, from filesystem scan.
    #[oai(skip_serializing_if = "is_default")]
    pub thumbs: JVec<Thumb>,

    // Common NFO.
    #[oai(flatten)]
    pub nfo_base: NfoBase,

    // Detail NFO (Episodes)
    #[oai(skip_serializing_if = "is_default")]
    pub aired: Option<String>,
    #[oai(skip_serializing_if = "is_default")]
    pub runtime: Option<u32>,
    #[oai(skip_serializing_if = "is_default")]
    pub displayseason: Option<u32>,
    #[oai(skip_serializing_if = "is_default")]
    pub displayepisode: Option<u32>,

    // Episode
    #[oai(skip)]
    pub video: FileInfo,
    pub season: u32,
    pub episode: u32,
}

impl Episode {
    pub async fn select_one(dbh: &mut db::TxnHandle<'_>, episode_id: i64) -> Result<Option<Episode>> {
        let mut v = Episode::select(dbh, None, None, Some(episode_id)).await?;
        Ok(v.pop())
    }

    pub async fn select(dbh: &mut db::TxnHandle<'_>, tvshow_id: Option<i64>, season_id: Option<i64>, episode_id: Option<i64>) -> Result<Vec<Episode>> {
        let mut rows = sqlx::query!(
            r#"
                SELECT i.id AS "id!: i64",
                       i.collection_id AS "collection_id: i64",
                       i.directory AS "directory!: FileInfo",
                       i.deleted AS "deleted!: bool",
                       i.lastmodified,
                       i.dateadded,
                       i.nfofile AS "nfofile?: FileInfo",
                       i.thumbs AS "thumbs!: JVec<Thumb>",
                       i.title, i.plot, i.tagline,
                       i.ratings AS "ratings!: JVec<Rating>",
                       i.uniqueids AS "uniqueids!: JVec<UniqueId>",
                       i.actors AS "actors!: JVec<Actor>",
                       i.credits AS "credits!: JVec<String>",
                       i.directors AS "directors!: JVec<String>",
                       m.tvshow_id,
                       m.aired,
                       m.runtime AS "runtime: u32",
                       m.displayseason AS "displayseason: u32",
                       m.displayepisode AS "displayepisode: u32",
                       m.video AS "video!: FileInfo",
                       m.season AS "season: u32",
                       m.episode AS "episode: u32"
                FROM mediaitems i
                JOIN episodes m ON (m.mediaitem_id = i.id)
                WHERE (m.tvshow_id = ? OR ? IS NULL)
                  AND (m.season = ? OR ? IS NULL)
                  AND (i.id = ? OR ? IS NULL)
                  AND i.deleted = 0
                ORDER BY m.season, m.episode"#,
            tvshow_id,
            tvshow_id,
            season_id,
            season_id,
            episode_id,
            episode_id
        )
        .fetch(dbh);

        let mut episodes = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let ep = build_struct!(Episode, row,
                id, collection_id, directory, deleted, lastmodified, dateadded, nfofile, thumbs,
                nfo_base.title, nfo_base.plot, nfo_base.tagline, nfo_base.ratings,
                nfo_base.uniqueids, nfo_base.actors, nfo_base.credits, nfo_base.directors,
                tvshow_id, video, season, episode, aired, runtime, displayseason, displayepisode);
            episodes.push(ep);
        }
        Ok(episodes)
    }

    pub fn copy_nfo_from(&mut self, other: &Episode) {
        self.nfofile = other.nfofile.clone();
        self.nfo_base = other.nfo_base.clone();
        self.aired = other.aired.clone();
        self.runtime = other.runtime.clone();
        self.displayseason = other.displayseason;
        self.displayepisode = other.displayepisode;
    }

    pub async fn insert(&mut self, txn: &mut db::TxnHandle<'_>) -> Result<()> {
        self.id = sqlx::query!(
            r#"
                INSERT INTO mediaitems(
                    type,
                    collection_id,
                    directory,
                    lastmodified,
                    dateadded,
                    thumbs,
                    nfofile,
                    title,
                    plot,
                    tagline,
                    ratings,
                    uniqueids,
                    actors,
                    credits,
                    directors
                ) VALUES("episode", ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.collection_id,
            self.directory,
            self.lastmodified,
            self.dateadded,
            self.thumbs,
            self.nfofile,
            self.nfo_base.title,
            self.nfo_base.plot,
            self.nfo_base.tagline,
            self.nfo_base.ratings,
            self.nfo_base.uniqueids,
            self.nfo_base.actors,
            self.nfo_base.credits,
            self.nfo_base.directors
        )
        .execute(&mut *txn)
        .await?
        .last_insert_rowid();

        sqlx::query!(
            r#"
                INSERT INTO episodes(
                    mediaitem_id,
                    tvshow_id,
                    aired,
                    runtime,
                    displayseason,
                    displayepisode,
                    video,
                    season,
                    episode
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.tvshow_id,
            self.aired,
            self.runtime,
            self.displayseason,
            self.displayepisode,
            self.video,
            self.season,
            self.episode,
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }

    pub async fn update(&mut self, txn: &mut db::TxnHandle<'_>) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE mediaitems SET
                    collection_id = ?,
                    directory = ?,
                    lastmodified = ?,
                    dateadded = ?,
                    thumbs = ?,
                    nfofile = ?,
                    title = ?,
                    plot = ?,
                    tagline = ?,
                    ratings = ?,
                    uniqueids = ?,
                    actors = ?,
                    credits = ?,
                    directors = ?
                WHERE id = ?"#,
            self.collection_id,
            self.directory,
            self.lastmodified,
            self.dateadded,
            self.thumbs,
            self.nfofile,
            self.nfo_base.title,
            self.nfo_base.plot,
            self.nfo_base.tagline,
            self.nfo_base.ratings,
            self.nfo_base.uniqueids,
            self.nfo_base.actors,
            self.nfo_base.credits,
            self.nfo_base.directors,
            self.id
        )
        .execute(&mut *txn)
        .await?;

        sqlx::query!(
            r#"
                UPDATE episodes SET
                    tvshow_id = ?,
                    video = ?,
                    aired = ?,
                    runtime = ?,
                    displayseason = ?,
                    displayepisode = ?,
                    video = ?,
                    season = ?,
                    episode = ?
                WHERE mediaitem_id = ?"#,
            self.tvshow_id,
            self.video,
            self.aired,
            self.runtime,
            self.displayseason,
            self.displayepisode,
            self.video,
            self.season,
            self.episode,
            self.id
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }
}

