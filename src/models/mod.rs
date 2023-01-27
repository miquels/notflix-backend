// mod episode;
mod fileinfo;
// mod image;
mod mediainfo;
mod mediaitem;
mod misc;
// mod movie;
mod nfo;
mod session;
mod thumb;
// mod tvshow;
mod uniqueids;
mod user;
mod video;

// pub use episode::Episode;
pub use fileinfo::FileInfo;
// pub use self::image::{Image, GetImage, ImageState};
pub use mediainfo::{MediaInfo, MediaInfoOverview};
pub use mediaitem::MediaItem;
pub use misc::*;
// pub use movie::Movie;
pub use nfo::Nfo;
pub use session::Session;
pub use thumb::{Thumb, ThumbState};
// pub use tvshow::{Season, TVShow};
pub use uniqueids::UniqueIds;
pub use user::{UpdateUser, User};
pub use video::*;

// helper function.
fn is_default<'a, T>(t: &'a T) -> bool
where
    T: Default,
    T: PartialEq,
{
    *t == T::default()
}
