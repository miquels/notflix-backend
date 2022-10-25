
use serde::{Deserialize, Serialize};
use super::db::DbHandle;

// helper function.
fn is_default<'a, T>(t: &'a T) -> bool
where
    T: Default,
    T: PartialEq,
{
    *t == T::default()
}

pub type SqlU32 = i64;
pub type SqlU64 = i64;

#[derive(Serialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct Movie {
    // Common.
    pub id: SqlU64,
    pub path: Option<String>,
    pub title: String,
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

    // Movie
    #[serde(skip_serializing_if = "is_default")]
    pub runtime: Option<SqlU32>,
    #[serde(skip_serializing_if = "is_default")]
    #[sqlx(default)]
    pub actors: sqlx::types::Json<Vec<Actor>>,
    #[serde(skip_serializing_if = "is_default")]
    pub credits: sqlx::types::Json<Vec<String>>,
    #[serde(skip_serializing_if = "is_default")]
    pub director: sqlx::types::Json<Vec<String>>,
}

impl Movie {
    async fn get_by_id(dbh: &DbHandle, id: SqlU64) -> Option<Movie> {
        sqlx::query_as!(
            Movie, r#"
                SELECT i.id, i.path, i.title, i.plot, i.tagline,
                       i.dateadded,
                       i.rating AS "rating: _",
                       i.thumb AS "thumb: _",
                       i.fanart AS "fanart: _",
                       i.uniqueid AS "uniqueid: _",
                       m.originaltitle, m.sorttitle,
                       m.country AS "country: _",
                       m.genre AS "genre: _",
                       m.studio AS "studio: _",
                       m.premiered, m.mpaa, m.runtime,
                       m.actors AS "actors: _",
                       m.credits AS "credits: _",
                       m.director AS "director: _"
                FROM mediaitems i
                JOIN movies m ON (m.mediaitem_id = i.id)
                WHERE i.id = ? AND i.deleted = 0 AND rating IS NOT NULL"#,
            id,
        )
        .fetch_one(dbh)
        .await
        .ok()
    }
}

#[derive(Serialize, Default, Debug)]
#[serde(default)]
pub struct TVShow {
    // Common.
    pub id: SqlU64,
    pub title: String,
    pub plot: Option<String>,
    pub tagline: Option<String>,
    pub dateadded: Option<String>,
    pub rating: sqlx::types::Json<Vec<Ratings>>,
    pub thumb: Vec<Thumb>,
    pub fanart: Vec<Fanart>,
    pub uniqueid: Vec<UniqueId>,
    pub actors: Vec<Actor>,

    // Movie + TV Show
    pub originaltitle: Option<String>,
    pub sorttitle: Option<String>,
    pub country: Vec<String>,
    pub genre: Vec<String>,
    pub premiered: Option<String>,
    pub studio: Vec<String>,
    pub mpaa: Option<String>,

    // TVShow
    pub seasons: SqlU32,
    pub episodes: SqlU32,
    pub status: Option<String>,
}

#[derive(Serialize, Default, Debug)]
#[serde(default)]
pub struct Season {
    pub id: SqlU64,
    pub season: SqlU32,
    pub name: Option<String>,
    pub thumb: Vec<Thumb>,
}

#[derive(Serialize,  Default, Debug)]
#[serde(default)]
pub struct Episode {
    // Common.
    pub id: SqlU64,
    pub title: String,
    pub plot: Option<String>,
    pub tagline: Option<String>,
    pub dateadded: Option<String>,
    pub rating: Option<sqlx::types::Json<Vec<Ratings>>>,
    pub thumb: Vec<Thumb>,
    pub fanart: Vec<Fanart>,
    pub uniqueid: Vec<UniqueId>,
    pub actors: Vec<Actor>,

    // Episode
    pub aired: Option<String>,
    pub runtime: Option<SqlU32>,
    pub season: Option<SqlU32>,
    pub episode: Option<SqlU32>,
    pub displayseason: Option<SqlU32>,
    pub displayepisode: Option<SqlU32>,
    pub actor: Vec<Actor>,
    pub credits: Vec<String>,
    pub director: Vec<String>,
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Actor {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Thumb {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Fanart {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Ratings {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct UniqueId {
}
