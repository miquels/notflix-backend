use anyhow::Result;
use serde::Serialize;
use crate::db::DbHandle;
use super::nfo::build_struct;
use super::misc::{Actor, FileInfo, Rating, Thumb, Fanart, UniqueId};
use super::{NfoBase, NfoMovie, J, JV, SqlU32, SqlU64, is_default};

#[derive(Serialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct TVShow {
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

    // Common NFO
    #[serde(flatten)]
    pub nfo_base: NfoBase,

    // Movie + TVShow NFO
    #[serde(flatten)]
    pub nfo_movie: NfoMovie,

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
        let r = sqlx::query!(
            r#"
                SELECT i.id, i.collection_id,
                       i.directory AS "directory!: J<FileInfo>",
                       i.dateadded,
                       i.nfofile AS "nfofile?: J<FileInfo>",
                       i.thumb AS "thumb!: JV<Thumb>",
                       i.fanart AS "fanart!: JV<Fanart>",
                       i.title, i.plot, i.tagline,
                       i.rating AS "rating!: JV<Rating>",
                       i.uniqueids AS "uniqueids!: JV<UniqueId>",
                       i.actors AS "actors!: JV<Actor>",
                       i.credits AS "credits!: JV<String>",
                       i.directors AS "directors!: JV<String>",
                       m.originaltitle,
                       m.sorttitle,
                       m.country AS "country!: JV<String>",
                       m.genre AS "genre!: JV<String>",
                       m.studio AS "studio!: JV<String>",
                       m.premiered,
                       m.mpaa,
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
        .ok()?;
        build_struct!(TVShow, r,
            id, collection_id, directory, dateadded, nfofile, thumb, fanart,
            nfo_base.title, nfo_base.plot, nfo_base.tagline, nfo_base.rating,
            nfo_base.uniqueids, nfo_base.actors, nfo_base.credits, nfo_base.directors,
            nfo_movie.originaltitle, nfo_movie.sorttitle, nfo_movie.country,
            nfo_movie.genre, nfo_movie.studio, nfo_movie.premiered, nfo_movie.mpaa,
            seasons, episodes, status)
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
                ) VALUES(?, ?, ?, "tvshow", ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.collection_id,
            self.directory,
            self.nfofile,
            self.dateadded,
            self.thumb,
            self.fanart,
            self.nfo_base.title,
            self.nfo_base.plot,
            self.nfo_base.tagline,
            self.nfo_base.rating,
            self.nfo_base.uniqueids,
            self.nfo_base.actors,
            self.nfo_base.credits,
            self.nfo_base.directors,
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
                    seasons,
                    episodes,
                    status
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.nfo_movie.originaltitle,
            self.nfo_movie.sorttitle,
            self.nfo_movie.country,
            self.nfo_movie.genre,
            self.nfo_movie.studio,
            self.nfo_movie.premiered,
            self.nfo_movie.mpaa,
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
            self.nfo_base.title,
            self.nfo_base.plot,
            self.nfo_base.tagline,
            self.nfo_base.rating,
            self.nfo_base.uniqueids,
            self.nfo_base.actors,
            self.nfo_base.credits,
            self.nfo_base.directors,
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
                    seasons = ?,
                    episodes = ?,
                    status = ?
                WHERE mediaitem_id = ?"#,
            self.nfo_movie.originaltitle,
            self.nfo_movie.sorttitle,
            self.nfo_movie.country,
            self.nfo_movie.genre,
            self.nfo_movie.studio,
            self.nfo_movie.premiered,
            self.nfo_movie.mpaa,
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

