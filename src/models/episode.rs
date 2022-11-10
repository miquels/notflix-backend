use anyhow::Result;
use serde::Serialize;
use futures_util::TryStreamExt;

use crate::db::Db;
use super::nfo::build_struct;
use super::{Rating, Thumb, UniqueId, Actor};
use super::{J, JV, FileInfo, NfoBase, is_default};

#[derive(Serialize, serde::Deserialize, Clone, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct Episode {
    // Common.
    pub id: i64,
    #[serde(skip_serializing)]
    pub collection_id: i64,
    pub tvshow_id: i64,
    #[serde(skip_serializing)]
    pub directory: sqlx::types::Json<FileInfo>,
    #[serde(skip)]
    pub lastmodified: i64,
    #[serde(skip_serializing_if = "is_default")]
    pub dateadded: Option<String>,
    #[serde(skip_serializing)]
    pub nfofile: Option<sqlx::types::Json<FileInfo>>,

    // Common, from filesystem scan.
    #[serde(skip_serializing_if = "is_default")]
    pub thumbs: sqlx::types::Json<Vec<Thumb>>,

    // Common NFO.
    #[serde(flatten)]
    pub nfo_base: NfoBase,

    // Detail NFO (Episodes)
    #[serde(skip_serializing_if = "is_default")]
    pub aired: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub runtime: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub displayseason: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub displayepisode: Option<u32>,

    // Episode
    #[serde(skip_serializing)]
    pub video: sqlx::types::Json<FileInfo>,
    pub season: u32,
    pub episode: u32,
}

impl Episode {
    pub async fn select_one(db: &Db, episode_id: i64) -> Option<Episode> {
        let mut v = Episode::select(db, None, None, Some(episode_id)).await?;
        v.pop()
    }

    pub async fn select(db: &Db, tvshow_id: Option<i64>, season_id: Option<i64>, episode_id: Option<i64>) -> Option<Vec<Episode>> {
        let mut rows = sqlx::query!(
            r#"
                SELECT i.id AS "id: i64",
                       i.collection_id AS "collection_id: i64",
                       i.directory AS "directory!: J<FileInfo>",
                       i.lastmodified,
                       i.dateadded,
                       i.nfofile AS "nfofile?: J<FileInfo>",
                       i.thumbs AS "thumbs!: JV<Thumb>",
                       i.title, i.plot, i.tagline,
                       i.ratings AS "ratings!: JV<Rating>",
                       i.uniqueids AS "uniqueids!: JV<UniqueId>",
                       i.actors AS "actors!: JV<Actor>",
                       i.credits AS "credits!: JV<String>",
                       i.directors AS "directors!: JV<String>",
                       m.tvshow_id,
                       m.video AS "video!: J<FileInfo>",
                       m.season AS "season: u32",
                       m.episode AS "episode: u32",
                       m.aired,
                       m.runtime AS "runtime: u32",
                       m.displayseason AS "displayseason: u32",
                       m.displayepisode AS "displayepisode: u32"
                FROM mediaitems i
                JOIN episodes m ON (m.mediaitem_id = i.id)
                WHERE (m.tvshow_id = ? OR ? IS NULL)
                  AND (m.season = ? OR ? IS NULL)
                  AND (i.id = ? OR ? IS NULL)
                  AND i.deleted = 0"#,
                tvshow_id,
                tvshow_id,
                season_id,
                season_id,
                episode_id,
                episode_id
        )
        .fetch(&db.handle);

        let mut episodes = Vec::new();
        while let Some(row) = rows.try_next().await.ok().flatten() {
            let ep = build_struct!(Episode, row,
                id, collection_id, directory, lastmodified, dateadded, nfofile, thumbs,
                nfo_base.title, nfo_base.plot, nfo_base.tagline, nfo_base.ratings,
                nfo_base.uniqueids, nfo_base.actors, nfo_base.credits, nfo_base.directors,
                tvshow_id, video, season, episode, aired, runtime, displayseason, displayepisode)?;
            episodes.push(ep);
        }
        Some(episodes)
    }

    pub async fn insert(&mut self, db: &Db) -> Result<()> {
        self.id = sqlx::query!(
            r#"
                INSERT INTO mediaitems(
                    collection_id,
                    directory,
                    lastmodified,
                    dateadded,
                    thumbs,
                    nfofile,
                    type,
                    title,
                    plot,
                    tagline,
                    ratings,
                    uniqueids,
                    actors,
                    credits,
                    directors
                ) VALUES(?, ?, ?, ?, "episode", ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
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
        .execute(&db.handle)
        .await?
        .last_insert_rowid();

        sqlx::query!(
            r#"
                INSERT INTO episodes(
                    mediaitem_id,
                    tvshow_id,
                    aired,
                    runtime,
                    season,
                    episode,
                    displayseason,
                    displayepisode
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.tvshow_id,
            self.aired,
            self.runtime,
            self.season,
            self.episode,
            self.displayseason,
            self.displayepisode,
        )
        .execute(&db.handle)
        .await?;

        Ok(())
    }

    pub async fn update(&mut self, db: &Db) -> Result<()> {
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
        .execute(&db.handle)
        .await?;

        sqlx::query!(
            r#"
                UPDATE episodes SET
                    tvshow_id = ?,
                    video = ?,
                    aired = ?,
                    runtime = ?,
                    season = ?,
                    episode = ?,
                    displayseason = ?,
                    displayepisode = ?
                WHERE mediaitem_id = ?"#,
            self.tvshow_id,
            self.video,
            self.aired,
            self.runtime,
            self.season,
            self.episode,
            self.displayseason,
            self.displayepisode,
            self.id
        )
        .execute(&db.handle)
        .await?;

        Ok(())
    }
}

