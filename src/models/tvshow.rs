use anyhow::Result;
use serde::Serialize;
use crate::db::DbHandle;
use super::misc::{Ratings, Thumb, Fanart, UniqueId};
use super::{SqlU32, SqlU64, is_default};

#[derive(Serialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct TVShow {
    // Common.
    pub id: SqlU64,
    pub collection_id: SqlU64,
    pub path: Option<String>,
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

    // Movie + TV Show
    #[serde(skip_serializing_if = "is_default")]
    pub originaltitle: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub sorttitle: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub country: sqlx::types::Json<Vec<String>>,
    #[serde(skip_serializing_if = "is_default")]
    pub genre: sqlx::types::Json<Vec<String>>,
    #[serde(skip_serializing_if = "is_default")]
    pub studio: sqlx::types::Json<Vec<String>>,
    #[serde(skip_serializing_if = "is_default")]
    pub premiered: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub mpaa: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub actors: sqlx::types::Json<Vec<String>>,

    // TVShow
    #[serde(skip_serializing_if = "is_default")]
    pub seasons: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    pub episodes: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    pub status: Option<String>,
}

impl TVShow {
    pub async fn select_one(dbh: &DbHandle, id: SqlU64) -> Option<TVShow> {
        sqlx::query_as!(
            TVShow,
            r#"
                SELECT i.id, i.collection_id, i.path, i.title, i.plot, i.tagline,
                       i.dateadded,
                       i.rating AS "rating: _",
                       i.thumb AS "thumb: _",
                       i.fanart AS "fanart: _",
                       i.uniqueid AS "uniqueid: _",
                       m.originaltitle, m.sorttitle,
                       m.country AS "country: _",
                       m.genre AS "genre: _",
                       m.studio AS "studio: _",
                       m.premiered, m.mpaa,
                       m.actors AS "actors: _",
                       m.seasons,
                       m.episodes,
                       m.status
                FROM mediaitems i
                JOIN tvshows m ON m.mediaitem_id = i.id
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
                ) VALUES(?, ?, "tvshow", ?, ?, ?, ?, ?, ?, ?, ?)"#,
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
                INSERT INTO tvshows(
                    mediaitem_id,
                    originaltitle,
                    sorttitle,
                    country,
                    genre,
                    studio,
                    premiered,
                    mpaa,
                    actors,
                    seasons,
                    episodes,
                    status
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.originaltitle,
            self.sorttitle,
            self.country,
            self.genre,
            self.studio,
            self.premiered,
            self.mpaa,
            self.actors,
            self.seasons,
            self.episodes,
            self.status
        )
        .execute(dbh)
        .await?;

        Ok(())
    }

    pub async fn update(&self, dbh: &DbHandle) -> Result<()> {
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
                UPDATE tvshows SET
                    originaltitle = ?,
                    sorttitle = ?,
                    country = ?,
                    genre = ?,
                    studio = ?,
                    premiered = ?,
                    mpaa = ?,
                    actors = ?,
                    seasons = ?,
                    episodes = ?,
                    status = ?
                WHERE mediaitem_id = ?"#,
            self.originaltitle,
            self.sorttitle,
            self.country,
            self.genre,
            self.studio,
            self.premiered,
            self.mpaa,
            self.actors,
            self.seasons,
            self.episodes,
            self.status,
            self.id
        )
        .execute(dbh)
        .await?;

        Ok(())
    }
}

