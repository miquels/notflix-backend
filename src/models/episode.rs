use anyhow::Result;
use serde::Serialize;
use crate::db::DbHandle;
use super::misc::{FileInfo, Rating, Thumb, Fanart, UniqueId, Actor};
use super::{SqlU32, SqlU64, is_default};

#[derive(Serialize, serde::Deserialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct Episode {
    // Common.
    pub id: SqlU64,
    pub collection_id: SqlU64,
    #[serde(skip_serializing)]
    pub directory: sqlx::types::Json<FileInfo>,
    #[serde(skip_serializing_if = "is_default")]
    pub dateadded: Option<String>,
    #[serde(skip_serializing)]
    pub nfofile: Option<sqlx::types::Json<FileInfo>>,

    // Common, from filesystem scan.
    #[serde(skip_serializing_if = "is_default")]
    pub thumb: sqlx::types::Json<Vec<Thumb>>,
    #[serde(skip_serializing_if = "is_default")]
    pub fanart: sqlx::types::Json<Vec<Fanart>>,

    // Basic NFO
    #[serde(skip_serializing_if = "is_default")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub plot: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub tagline: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub rating: sqlx::types::Json<Vec<Rating>>,
    #[serde(skip_serializing_if = "is_default")]
    pub uniqueids: sqlx::types::Json<Vec<UniqueId>>,
    #[serde(skip_serializing_if = "is_default")]
    pub actors: sqlx::types::Json<Vec<Actor>>,
    #[serde(skip_serializing_if = "is_default")]
    pub credits: sqlx::types::Json<Vec<String>>,
    #[serde(skip_serializing_if = "is_default")]
    pub directors: sqlx::types::Json<Vec<String>>,

    // Detail NFO (Episodes)
    #[serde(skip_serializing_if = "is_default")]
    pub aired: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub runtime: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    pub displayseason: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    pub displayepisode: Option<SqlU32>,

    // Episode
    #[serde(skip_serializing)]
    pub video: sqlx::types::Json<FileInfo>,
    pub season: SqlU32,
    pub episode: SqlU32,
}

impl Episode {
    pub async fn select_one(dbh: &DbHandle, id: SqlU64) -> Option<Episode> {
        sqlx::query_as!(
            Episode,
            r#"
                SELECT i.id, i.collection_id,
                       i.directory AS "directory: _",
                       i.nfofile AS "nfofile: _",
                       i.title, i.plot, i.tagline,
                       i.dateadded,
                       i.rating AS "rating: _",
                       i.thumb AS "thumb: _",
                       i.fanart AS "fanart: _",
                       i.uniqueids AS "uniqueids: _",
                       i.actors AS "actors: _",
                       i.credits AS "credits: _",
                       i.directors AS "directors: _",
                       m.video AS "video: _",
                       m.season, m.episode,
                       m.aired, m.runtime,
                       m.displayseason, m.displayepisode
                FROM mediaitems i
                JOIN episodes m ON (m.mediaitem_id = i.id)
                WHERE i.id = ? AND i.deleted = 0"#,
                id
        )
        .fetch_one(dbh)
        .await
        .ok()
    }

    pub async fn insert(&mut self, dbh: &DbHandle) -> Result<()> {
        self.id = sqlx::query!(
            r#"
                INSERT INTO mediaitems(
                    collection_id,
                    directory,
                    dateadded,
                    thumb,
                    fanart,
                    nfofile,
                    type,
                    title,
                    plot,
                    tagline,
                    rating,
                    uniqueids,
                    actors,
                    credits,
                    directors
                ) VALUES(?, ?, ?, "episode", ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.collection_id,
            self.directory,
            self.dateadded,
            self.thumb,
            self.fanart,
            self.nfofile,
            self.title,
            self.plot,
            self.tagline,
            self.rating,
            self.uniqueids,
            self.actors,
            self.credits,
            self.directors
        )
        .execute(dbh)
        .await?
        .last_insert_rowid();

        sqlx::query!(
            r#"
                INSERT INTO episodes(
                    mediaitem_id,
                    aired,
                    runtime,
                    season,
                    episode,
                    displayseason,
                    displayepisode
                ) VALUES(?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.aired,
            self.runtime,
            self.season,
            self.episode,
            self.displayseason,
            self.displayepisode,
        )
        .execute(dbh)
        .await?;

        Ok(())
    }

    pub async fn update(&mut self, dbh: &DbHandle) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE mediaitems SET
                    collection_id = ?,
                    directory = ?,
                    dateadded = ?,
                    thumb = ?,
                    fanart = ?,
                    nfofile = ?,
                    title = ?,
                    plot = ?,
                    tagline = ?,
                    rating = ?,
                    uniqueids = ?,
                    actors = ?,
                    credits = ?,
                    directors = ?
                WHERE id = ?"#,
            self.collection_id,
            self.directory,
            self.dateadded,
            self.thumb,
            self.fanart,
            self.nfofile,
            self.title,
            self.plot,
            self.tagline,
            self.rating,
            self.uniqueids,
            self.actors,
            self.credits,
            self.directors,
            self.id
        )
        .execute(dbh)
        .await?;

        sqlx::query!(
            r#"
                UPDATE episodes SET
                    aired = ?,
                    runtime = ?,
                    season = ?,
                    episode = ?,
                    displayseason = ?,
                    displayepisode = ?
                WHERE mediaitem_id = ?"#,
            self.aired,
            self.runtime,
            self.season,
            self.episode,
            self.displayseason,
            self.displayepisode,
            self.id
        )
        .execute(dbh)
        .await?;

        Ok(())
    }
}

