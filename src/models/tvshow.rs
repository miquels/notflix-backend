use anyhow::Result;
use serde::Serialize;
use crate::db::DbHandle;
use super::nfo::build_struct;
use super::misc::{Actor, Rating, Thumb, UniqueId};
use super::{Episode, NfoBase, NfoMovie, FileInfo, J, JV, is_default};

#[derive(Serialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct TVShow {
    // Common.
    pub id: i64,
    pub collection_id: i64,
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

    // Common NFO
    #[serde(flatten)]
    pub nfo_base: NfoBase,

    // Movie + TVShow NFO
    #[serde(flatten)]
    pub nfo_movie: NfoMovie,

    // TVShow
    #[serde(skip_serializing_if = "is_default")]
    pub total_seasons: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub total_episodes: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub status: Option<String>,

    #[sqlx(default)]
    pub seasons: Vec<Season>,
}

impl TVShow {
    pub async fn select_one(dbh: &DbHandle, id: i64) -> Option<TVShow> {
        let r = sqlx::query!(
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
                       m.originaltitle,
                       m.sorttitle,
                       m.countries AS "countries!: JV<String>",
                       m.genres AS "genres!: JV<String>",
                       m.studios AS "studios!: JV<String>",
                       m.premiered,
                       m.mpaa,
                       m.seasons AS "total_seasons: u32",
                       m.episodes AS "total_episodes: u32",
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
            id, collection_id, directory, lastmodified, dateadded, nfofile, thumbs,
            nfo_base.title, nfo_base.plot, nfo_base.tagline, nfo_base.ratings,
            nfo_base.uniqueids, nfo_base.actors, nfo_base.credits, nfo_base.directors,
            nfo_movie.originaltitle, nfo_movie.sorttitle, nfo_movie.countries,
            nfo_movie.genres, nfo_movie.studios, nfo_movie.premiered, nfo_movie.mpaa,
            total_seasons, total_episodes, status)
    }

    pub async fn insert(&mut self, dbh: &DbHandle) -> Result<()> {
        self.id = sqlx::query!(
            r#"
                INSERT INTO mediaitems(
                    type,
                    collection_id,
                    directory,
                    lastmodified,
                    dateadded,
                    nfofile,
                    thumbs,
                    title,
                    plot,
                    tagline,
                    ratings,
                    uniqueids,
                    actors,
                    credits,
                    directors
                ) VALUES("tvshow", ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.collection_id,
            self.directory,
            self.lastmodified,
            self.dateadded,
            self.nfofile,
            self.thumbs,
            self.nfo_base.title,
            self.nfo_base.plot,
            self.nfo_base.tagline,
            self.nfo_base.ratings,
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
                    countries,
                    genres,
                    studios,
                    premiered,
                    mpaa,
                    seasons,
                    episodes,
                    status
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.nfo_movie.originaltitle,
            self.nfo_movie.sorttitle,
            self.nfo_movie.countries,
            self.nfo_movie.genres,
            self.nfo_movie.studios,
            self.nfo_movie.premiered,
            self.nfo_movie.mpaa,
            self.total_seasons,
            self.total_episodes,
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
        .execute(dbh)
        .await?;

        sqlx::query!(
            r#"
                UPDATE tvshows SET
                    originaltitle = ?,
                    sorttitle = ?,
                    countries = ?,
                    genres = ?,
                    studios = ?,
                    premiered = ?,
                    mpaa = ?,
                    seasons = ?,
                    episodes = ?,
                    status = ?
                WHERE mediaitem_id = ?"#,
            self.nfo_movie.originaltitle,
            self.nfo_movie.sorttitle,
            self.nfo_movie.countries,
            self.nfo_movie.genres,
            self.nfo_movie.studios,
            self.nfo_movie.premiered,
            self.nfo_movie.mpaa,
            self.total_seasons,
            self.total_episodes,
            self.status,
            self.id
        )
        .execute(dbh)
        .await?;

        Ok(())
    }
}

#[derive(Serialize, Default, Debug, sqlx::FromRow)]
pub struct Season {
    pub season:   u32,
    pub episodes: Vec<Episode>,
}
