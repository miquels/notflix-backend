use serde::{Serialize, Deserialize};
use super::is_default;
use super::misc::{Rating, UniqueId, Actor};

#[derive(Serialize, Deserialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct NfoBase {
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
}

#[derive(Serialize, Deserialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct NfoMovie {
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
}

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
        Some({
            let mut v = $struct::default();
            $(
                build_struct!(@E v.$field $(.$field2)*) = build_struct!(@V $src, $field $(.$field2)*);
            )+
            v
        })
    };
}
pub(crate) use build_struct;

