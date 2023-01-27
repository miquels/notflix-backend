use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use super::is_default;
use super::{Actor, Rating, UniqueId};
use crate::jvec::JVec;
use crate::sqlx::impl_sqlx_traits_for;

pub use crate::kodifs::nfo::NfoType;

#[derive(Object, Serialize, Deserialize, Clone, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct Nfo {
    // Basic NFO
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip)]
    pub nfo_type: NfoType,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub plot: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub tagline: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(flatten, skip_serializing_if = "is_default")]
    pub ratings: JVec<Rating>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(flatten, skip_serializing_if = "is_default")]
    pub uniqueids: JVec<UniqueId>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(flatten, skip_serializing_if = "is_default")]
    pub actors: JVec<Actor>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(flatten, skip_serializing_if = "is_default")]
    pub credits: JVec<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(flatten, skip_serializing_if = "is_default")]
    pub directors: JVec<String>,
    // Detail NFO (Movie + TV Show)
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub originaltitle: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub sorttitle: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(flatten, skip_serializing_if = "is_default")]
    pub countries: JVec<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(flatten, skip_serializing_if = "is_default")]
    pub genres: JVec<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(flatten, skip_serializing_if = "is_default")]
    pub studios: JVec<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub premiered: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub mpaa: Option<String>,

    // Detail NFO (movie + episode)
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub runtime: Option<u32>,

    // Detail NFO (tvshow)
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub status: Option<String>,

    // Detail NFO (tvshow + episode)
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub season: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub episode: Option<u32>,

    // Detail NFO (episode)
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub aired: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub displayseason: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    #[oai(skip_serializing_if = "is_default")]
    pub displayepisode: Option<u32>,
}
impl_sqlx_traits_for!(Nfo);
