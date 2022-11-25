use anyhow::Result;
use poem_openapi::Object;
use serde::Serialize;
use std::collections::BTreeMap;

use super::nfo::build_struct;
use super::{is_default, Episode, FileInfo, NfoBase, NfoMovie};
use super::{Actor, Rating, Thumb, UniqueId};
use crate::db::{self, FindItemBy};
use crate::jvec::JVec;
use crate::util::{some_or_return, Id};

#[derive(Object, Serialize, Clone, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct TVShow {
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
    #[oai(skip_serializing_if = "is_default")]
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

    // TVShow
    #[oai(skip_serializing_if = "is_default")]
    pub total_seasons: Option<u32>,
    #[oai(skip_serializing_if = "is_default")]
    pub total_episodes: Option<u32>,
    #[oai(skip_serializing_if = "is_default")]
    pub status: Option<String>,

    #[sqlx(default)]
    pub seasons: Vec<Season>,
}

impl TVShow {
    pub fn copy_nfo_from(&mut self, other: &TVShow) {
        self.nfofile = other.nfofile.clone();
        self.nfo_base = other.nfo_base.clone();
        self.nfo_movie = other.nfo_movie.clone();
        self.total_seasons = other.total_seasons;
        self.total_episodes = other.total_episodes;
        self.status = other.status.clone();
    }

    pub async fn lookup_by(
        dbh: &mut db::TxnHandle<'_>,
        find: &FindItemBy<'_>,
        include_episodes: bool,
    ) -> Result<Option<Box<TVShow>>> {
        // Find the ID.
        let id = match find.is_only_id() {
            Some(id) => id,
            None => match db::lookup(dbh, &find).await? {
                Some(id) => id,
                None => return Ok(None),
            },
        };
        log::trace!("lookup_by: querying id {}", id);

        // Find the item in the database.
        let row = sqlx::query!(
            r#"
                SELECT i.id AS "id!: Id",
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
                       m.seasons AS "total_seasons: u32",
                       m.episodes AS "total_episodes: u32",
                       m.status
                FROM mediaitems i
                JOIN tvshows m ON m.mediaitem_id = i.id
                WHERE i.id = ? AND (i.deleted = 0 OR i.deleted = ?)"#,
            id,
            find.deleted_too,
        )
        .fetch_optional(&mut *dbh)
        .await?;
        let row = some_or_return!(row, { Ok(None) });

        let mut seasons = Vec::new();
        if include_episodes {
            let episodes = Episode::select(dbh, Some(id), None, None).await?;
            let mut sns = BTreeMap::new();
            for ep in episodes.into_iter() {
                let e = sns.entry(ep.season).or_insert(Vec::new());
                e.push(ep);
            }
            seasons = sns.into_iter().map(|(s, e)| Season { season: s, episodes: e }).collect();
        }

        let mut m = build_struct!(
            TVShow,
            row,
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
            total_seasons,
            total_episodes,
            status
        );
        m.seasons = seasons;
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
                ) VALUES("tvshow", ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.id,
            self.collection_id,
            self.directory,
            self.deleted,
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
        .execute(&mut *txn)
        .await?;

        let id = Id::new();
        sqlx::query!(
            r#"
                INSERT INTO tvshows(
                    id,
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
                ) VALUES(?,?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id,
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
        .execute(&mut *txn)
        .await?;

        // Now the episodes.
        for season in &self.seasons {
            for episode in &season.episodes {
                episode.insert(&mut *txn).await?;
            }
        }

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
        .execute(&mut *txn)
        .await?;

        Ok(())
    }
}

#[derive(Object, Serialize, Clone, Default, Debug, sqlx::FromRow)]
pub struct Season {
    pub season: u32,
    pub episodes: Vec<Episode>,
}
