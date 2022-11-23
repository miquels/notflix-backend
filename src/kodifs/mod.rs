use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

use crate::models;
use crate::collections::Collection;

mod movie;
mod tvshow;
mod episode;
pub(crate) mod nfo;
pub mod scandirs;

pub use movie::scan_movie_dir;
pub use tvshow::scan_tvshow_dir;
pub use nfo::Nfo;

#[async_trait]
pub trait KodiFS {
    async fn scan_directory(coll: &Collection, name: &str, db_item: Option<Box<Self>>, only_nfo: bool) -> Option<Box<Self>>;
}

#[async_trait]
impl KodiFS for models::TVShow {
    async fn scan_directory(coll: &Collection, name: &str, db_item: Option<Box<Self>>, only_nfo: bool) -> Option<Box<Self>> {
        let item = scan_tvshow_dir(coll, name, db_item, only_nfo).await?;
        Some(item)
    }
}

#[async_trait]
impl KodiFS for models::Movie {
    async fn scan_directory(coll: &Collection, name: &str, db_item: Option<Box<Self>>, only_nfo: bool) -> Option<Box<Self>> {
        let item = scan_movie_dir(coll, name, db_item, only_nfo).await?;
        Some(item)
    }
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
