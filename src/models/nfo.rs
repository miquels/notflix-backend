use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use super::is_default;
use super::{Actor, Rating, UniqueId};
use crate::jvec::JVec;
use crate::sqlx::impl_sqlx_traits_for;

pub use crate::kodifs::nfo::NfoType;

#[derive(Object, Serialize, Deserialize, Clone, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct NfoBase {
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
}
impl_sqlx_traits_for!(NfoBase);

#[derive(Object, Serialize, Deserialize, Clone, Default, Debug, sqlx::FromRow)]
pub struct NfoMovie {
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
}
impl_sqlx_traits_for!(NfoMovie);

// #[sqlx(flatten)] doesn't work with the query_as! macro.
// So we use query! instead, and then use this macro to copy the
// result from query! into Movie / TVShow / Episode.
macro_rules! build_struct {
    (@E $e:expr) => {
        $e
    };
    (@E $($tt:tt)*) => {
        compile_error!(stringify!("build_struct: @E:" $($tt)*));
    };
    (@V $src:tt, $left:tt.$right:tt) => {
        build_struct!(@E $src.$right)
    };
    (@V $src:tt, $left:tt) => {
        build_struct!(@E $src.$left)
    };
    (@V $($tt:tt)*) => {
        compile_error!(stringify!("build_struct: @V:" $($tt)*));
    };
    ($struct:ident, $src:tt, $($field:tt $(.$field2:tt)*),+) => {
        {
            let mut v = $struct::default();
            $(
                build_struct!(@E v.$field $(.$field2)*) = build_struct!(@V $src, $field $(.$field2)*);
            )+
            v
        }
    };
}
pub(crate) use build_struct;
