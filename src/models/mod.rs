mod movie;
mod tvshow;
mod episode;
mod misc;
mod nfo;

pub use movie::Movie;
pub use tvshow::TVShow;
pub use episode::Episode;
pub use misc::*;
pub use nfo::*;

type J<T> = sqlx::types::Json<T>;
type JV<T> = sqlx::types::Json<Vec<T>>;

// helper function.
fn is_default<'a, T>(t: &'a T) -> bool
where
    T: Default,
    T: PartialEq,
{
    *t == T::default()
}

