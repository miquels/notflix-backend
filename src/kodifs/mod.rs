use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

use crate::collections::{Collection, CollectionType};
use crate::models;

// mod episode;
// mod movie;
pub(crate) mod nfo;
//pub mod resource;
pub mod scandirs;
// mod tvshow;
mod video;

// pub use movie::scan_movie_dir;
pub use nfo::Nfo;
// pub use tvshow::scan_tvshow_dir;
pub use video::probe as probe_video;

pub async fn scan_mediaitem_dir(
    coll: &Collection,
    dirname: &str,
    dbent: Option<Box<models::MediaItem>>,
    only_nfo: bool,
) -> Option<Box<models::MediaItem>> {
    /*
    match coll.type_ {
        CollectionType::Movie => scan_movie_dir(coll, dirname, dbent, only_nfo).await,
        CollectionType::TVShow => scan_tvshow_dir(coll, dirname, dbent, only_nfo).await,
    }*/
    todo!()
}

// Helper macro.
macro_rules! def_regex {
    ($name:ident => $re:expr) => {
        pub static $name: Lazy<Regex> = Lazy::new(|| Regex::new($re).unwrap());
    };
}

def_regex!(IS_VIDEO => r#"^((?:.+?([0-9]+)/|)(.*))\.(divx|mov|mp4|MP4|m4u|m4v)$"#);
def_regex!(IS_IMAGE => r#"^(.+)\.(jpg|jpeg|png|tbn)$"#);
def_regex!(IS_SEASON_IMG => r#"^season([0-9]+)-?([a-z]+|)\.(jpg|jpeg|png|tbn)$"#);
def_regex!(IS_RELATED => r#"^(.*?)(?:[.-](?:(poster|thumb|fanart|)))?\.([a-z]+)$"#);
def_regex!(IS_YEAR => r#"^(.*) \(([0-9]{4})\)$"#);

/// URL escape a path - relative or absolute.
pub fn escape_path(p: &str) -> String {
    if p.starts_with("/") {
        let u = Url::from_file_path(p).unwrap();
        u.path().to_string()
    } else {
        let u = Url::from_file_path(&(String::from("/") + p)).unwrap();
        u.path()[1..].to_string()
    }
}

/// Like the name says :)
pub fn join_and_escape_path(subdir: Option<&str>, name: &str) -> String {
    match subdir {
        Some(d) => escape_path(&format!("{}/{}", d, name)),
        None => escape_path(name),
    }
}

// The list must be sorted. If this function is called for multiple
// prefixes, those must be in sorted order as well.
fn extract_prefixed(list: &mut Vec<String>, idx: &mut usize, prefix: &str) -> Vec<String> {
    let mut v = Vec::new();
    while *idx < list.len() {
        if list[*idx].as_str() < prefix {
            *idx += 1;
            continue;
        }
        if !list[*idx].starts_with(prefix) {
            break;
        }
        v.push(list.remove(*idx));
    }
    v
}
