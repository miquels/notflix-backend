use anyhow::Result;
use poem_openapi::Object;
use serde::Serialize;

use super::nfo::build_struct;
use super::{is_default, FileInfo, NfoBase, NfoMovie};
use super::{Actor, Rating, Thumb, UniqueId};
use crate::db::{self, FindItemBy};
use crate::jvec::JVec;
use crate::util::Id;

#[derive(Object, Serialize, Clone, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct Movie {
    // Common.
    #[oai(read_only)]
    pub id: Id,
    pub collection_id: i64,
    #[oai(skip)]
    pub directory: FileInfo,
    #[oai(skip)]
    pub deleted: bool,
    #[oai(skip)]
    pub lastmodified: i64,
    #[oai(skip)]
    pub dateadded: Option<String>,
    #[oai(skip)]
    pub nfofile: Option<FileInfo>,

    // Common, from filesystem scan.
    #[oai(skip_serializing_if = "is_default")]
    pub thumbs: JVec<Thumb>,

    // Common NFO
    #[oai(flatten)]
    pub nfo_base: NfoBase,

    // Movie + TVShow NFO
    #[oai(flatten)]
    pub nfo_movie: NfoMovie,

    // Movie NFO
    #[oai(skip_serializing_if = "is_default")]
    pub runtime: Option<u32>,

    // Movie specific data.
    #[oai(skip)]
    pub video: FileInfo,
}

impl Movie {
    pub async fn lookup_by(
        dbh: &mut db::TxnHandle<'_>,
        find: &FindItemBy<'_>,
    ) -> Result<Option<Box<Movie>>> {
        // Find the ID.
        let id = match find.is_only_id() {
            Some(id) => id,
            None => match db::lookup(dbh, &find).await? {
                Some(id) => id,
                None => return Ok(None),
            },
        };

        // Find the item in the database.
        let r = sqlx::query!(
            r#"
                SELECT i.id AS "id: Id",
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
                       m.originaltitle,
                       m.sorttitle,
                       m.countries AS "countries!: JVec<String>",
                       m.genres AS "genres!: JVec<String>",
                       m.studios AS "studios!: JVec<String>",
                       m.premiered,
                       m.mpaa,
                       m.runtime AS "runtime: u32",
                       m.video AS "video: FileInfo"
                FROM mediaitems i
                JOIN movies m ON (m.mediaitem_id = i.id)
                WHERE i.id = ? AND (i.deleted = 0 OR i.deleted = ?)"#,
            id,
            find.deleted_too,
        )
        .fetch_one(dbh)
        .await;

        let r = match r {
            Ok(r) => r,
            Err(e) => {
                log::error!("error getting movie by id {}: {}", id, e);
                return Ok(None);
            },
        };

        let m = build_struct!(
            Movie,
            r,
            id,
            collection_id,
            directory,
            deleted,
            lastmodified,
            dateadded,
            nfofile,
            thumbs,
            nfo_base.title,
            nfo_base.plot,
            nfo_base.tagline,
            nfo_base.ratings,
            nfo_base.uniqueids,
            nfo_base.actors,
            nfo_base.credits,
            nfo_base.directors,
            nfo_movie.originaltitle,
            nfo_movie.sorttitle,
            nfo_movie.countries,
            nfo_movie.genres,
            nfo_movie.studios,
            nfo_movie.premiered,
            nfo_movie.mpaa,
            runtime,
            video
        );
        Ok(Some(Box::new(m)))
    }

    pub async fn insert(&self, txn: &mut db::TxnHandle<'_>) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO mediaitems(
                    type,
                    id,
                    collection_id,
                    directory,
                    deleted,
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
                ) VALUES("movie", ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.collection_id,
            self.directory,
            self.deleted,
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
        .await?;

        let id = Id::new();
        sqlx::query!(
            r#"
                INSERT INTO movies(
                    id,
                    mediaitem_id,
                    originaltitle,
                    sorttitle,
                    countries,
                    genres,
                    studios,
                    premiered,
                    mpaa,
                    runtime,
                    video
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id,
            self.id,
            self.nfo_movie.originaltitle,
            self.nfo_movie.sorttitle,
            self.nfo_movie.countries,
            self.nfo_movie.genres,
            self.nfo_movie.studios,
            self.nfo_movie.premiered,
            self.nfo_movie.mpaa,
            self.runtime,
            self.video,
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
                    directory = ?,
                    deleted = ?,
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
            self.deleted,
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
                UPDATE movies SET
                    originaltitle = ?,
                    sorttitle = ?,
                    countries = ?,
                    genres = ?,
                    studios = ?,
                    premiered = ?,
                    mpaa = ?,
                    runtime = ?,
                    video = ?
                WHERE mediaitem_id = ?"#,
            self.nfo_movie.originaltitle,
            self.nfo_movie.sorttitle,
            self.nfo_movie.countries,
            self.nfo_movie.genres,
            self.nfo_movie.studios,
            self.nfo_movie.premiered,
            self.nfo_movie.mpaa,
            self.runtime,
            self.video,
            self.id
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }
}
