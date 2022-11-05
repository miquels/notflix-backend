use anyhow::Result;
use serde::Serialize;
use crate::db::DbHandle;
use super::misc::{FindItemBy, FileInfo, Rating, Thumb, Fanart, UniqueId, Actor};
use super::{SqlU32, SqlU64, is_default};

#[derive(Serialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct Movie {
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

    // Detail NFO (Movie + TV Show)
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

    // Movie
    #[serde(skip_serializing)]
    pub video: sqlx::types::Json<FileInfo>,
    #[serde(skip_serializing_if = "is_default")]
    pub runtime: Option<SqlU32>,
}

impl Movie {
    pub async fn lookup(dbh: &DbHandle, find: FindItemBy<'_>) -> Option<Movie> {
        let id = match find.is_only_id() {
            Some(id) => id,
            None => find.lookup(dbh).await?,
        };
        sqlx::query_as!(
            Movie,
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
                       m.originaltitle, m.sorttitle,
                       m.country AS "country: _",
                       m.genre AS "genre: _",
                       m.studio AS "studio: _",
                       m.premiered, m.mpaa, m.runtime,
                       m.video AS "video: _"
                FROM mediaitems i
                JOIN movies m ON (m.mediaitem_id = i.id)
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
                    directory,
                    nfofile,
                    type,
                    title,
                    plot,
                    tagline,
                    dateadded,
                    rating,
                    thumb,
                    fanart,
                    uniqueids,
                    actors,
                    credits,
                    directors
                ) VALUES(?, ?, ?, "movie", ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.collection_id,
            self.directory,
            self.nfofile,
            self.title,
            self.plot,
            self.tagline,
            self.dateadded,
            self.rating,
            self.thumb,
            self.fanart,
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
                INSERT INTO movies(
                    mediaitem_id,
                    originaltitle,
                    sorttitle,
                    country,
                    genre,
                    studio,
                    premiered,
                    mpaa,
                    runtime
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.originaltitle,
            self.sorttitle,
            self.country,
            self.genre,
            self.studio,
            self.premiered,
            self.mpaa,
            self.runtime
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
                    nfofile = ?,
                    title = ?,
                    plot = ?,
                    tagline = ?,
                    dateadded = ?,
                    rating = ?,
                    thumb = ?,
                    fanart = ?,
                    uniqueids = ?,
                    actors = ?,
                    credits = ?,
                    directors = ?
                WHERE id = ?"#,
            self.collection_id,
            self.directory,
            self.nfofile,
            self.title,
            self.plot,
            self.tagline,
            self.dateadded,
            self.rating,
            self.thumb,
            self.fanart,
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
                UPDATE movies SET
                    originaltitle = ?,
                    sorttitle = ?,
                    country = ?,
                    genre = ?,
                    studio = ?,
                    premiered = ?,
                    mpaa = ?,
                    runtime = ?
                WHERE mediaitem_id = ?"#,
            self.originaltitle,
            self.sorttitle,
            self.country,
            self.genre,
            self.studio,
            self.premiered,
            self.mpaa,
            self.runtime,
            self.id
        )
        .execute(dbh)
        .await?;

        Ok(())
    }
}
