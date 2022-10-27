use anyhow::Result;
use serde::Serialize;
use crate::db::DbHandle;
use super::misc::{Ratings, Thumb, Fanart, UniqueId, Actor};
use super::{SqlU32, SqlU64, is_default};

#[derive(Serialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct Episode {
    // Common.
    pub id: SqlU64,
    pub collection_id: SqlU64,
    #[serde(skip_serializing_if = "is_default")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub plot: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub tagline: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub dateadded: Option<String>,

    #[serde(skip_serializing_if = "is_default")]
    #[sqlx(default)]
    pub rating: sqlx::types::Json<Vec<Ratings>>,
    #[serde(skip_serializing_if = "is_default")]
    #[sqlx(default)]
    pub thumb: sqlx::types::Json<Vec<Thumb>>,
    #[serde(skip_serializing_if = "is_default")]
    #[sqlx(default)]
    pub fanart: sqlx::types::Json<Vec<Fanart>>,
    #[serde(skip_serializing_if = "is_default")]
    #[sqlx(default)]
    pub uniqueid: sqlx::types::Json<Vec<UniqueId>>,

    // Episode
    #[serde(skip_serializing_if = "is_default")]
    pub aired: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub runtime: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    pub season: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    pub episode: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    pub displayseason: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    pub displayepisode: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    pub actors: sqlx::types::Json<Vec<Actor>>,
    #[serde(skip_serializing_if = "is_default")]
    pub credits: sqlx::types::Json<Vec<String>>,
    #[serde(skip_serializing_if = "is_default")]
    pub director: sqlx::types::Json<Vec<String>>,
}

impl Episode {
    pub async fn select_one(dbh: &DbHandle, id: SqlU64) -> Option<Episode> {
        sqlx::query_as!(
            Episode,
            r#"
                SELECT i.id, i.collection_id, i.path, i.title, i.plot, i.tagline,
                       i.dateadded,
                       i.rating AS "rating: _",
                       i.thumb AS "thumb: _",
                       i.fanart AS "fanart: _",
                       i.uniqueid AS "uniqueid: _",
                       m.aired, m.runtime, m.season, m.episode,
                       m.displayseason, m.displayepisode,
                       m.actors AS "actors: _",
                       m.credits AS "credits: _",
                       m.director AS "director: _"
                FROM mediaitems i
                JOIN episodes m ON (m.mediaitem_id = i.id)
                WHERE i.id = ? AND i.deleted = 0"#,
            id,
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
                    path,
                    type,
                    title,
                    plot,
                    tagline,
                    dateadded,
                    rating,
                    thumb,
                    fanart,
                    uniqueid
                ) VALUES(?, ?, "episode", ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.collection_id,
            self.path,
            self.title,
            self.plot,
            self.tagline,
            self.dateadded,
            self.rating,
            self.thumb,
            self.fanart,
            self.uniqueid
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
                    displayepisode,
                    actors,
                    credits,
                    director
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.aired,
            self.runtime,
            self.season,
            self.episode,
            self.displayseason,
            self.displayepisode,
            self.actors,
            self.credits,
            self.director
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
                    path = ?,
                    title = ?,
                    plot = ?,
                    tagline = ?,
                    dateadded = ?,
                    rating = ?,
                    thumb = ?,
                    fanart = ?,
                    uniqueid = ?
                WHERE id = ?"#,
            self.collection_id,
            self.path,
            self.title,
            self.plot,
            self.tagline,
            self.dateadded,
            self.rating,
            self.thumb,
            self.fanart,
            self.uniqueid,
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
                    displayepisode = ?,
                    actors = ?,
                    credits = ?,
                    director = ?
                WHERE mediaitem_id = ?"#,
            self.aired,
            self.runtime,
            self.season,
            self.episode,
            self.displayseason,
            self.displayepisode,
            self.actors,
            self.credits,
            self.director,
            self.id
        )
        .execute(dbh)
        .await?;

        Ok(())
    }
}

