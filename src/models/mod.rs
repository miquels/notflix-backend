mod movie;
mod tvshow;
mod season;
mod episode;
mod misc;

pub use movie::Movie;
pub use tvshow::TVShow;
pub use season::Season;
pub use episode::Episode;
pub use misc::*;

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

