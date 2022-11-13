use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;
use crate::models::Thumb;

mod movies;
mod nfo;
mod scandirs;
mod shows;
mod episode;

pub use movies::scan_movie_dir;
pub use scandirs::{scan_directory, scan_directories};
pub use shows::build_show;
pub use nfo::Nfo;
pub use episode::Episode;

// Helper macro.
macro_rules! def_regex {
    ($name:ident => $re:expr) => {
        pub static $name: Lazy<Regex> = Lazy::new(|| Regex::new($re).unwrap());
    };
}

def_regex!(IS_VIDEO => r#"^((?:.+?([0-9]+)/|)(.*))\.(divx|mov|mp4|MP4|m4u|m4v)$"#);
def_regex!(IS_IMAGE => r#"^(.+)\.(jpg|jpeg|png|tbn)$"#);
def_regex!(IS_SEASON_IMG => r#"^season([0-9]+)-?([a-z]+|)\.(jpg|jpeg|png|tbn)$"#);
def_regex!(IS_EXT1 => r#"^(.*)()\.(png|jpg|jpeg|tbn|nfo|srt)$"#);
def_regex!(IS_EXT2 => r#"^(.*)[.-]([a-z]+)\.(png|jpg|jpeg|tbn|nfo|srt)$"#);
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

fn add_thumb(thumbs: &mut sqlx::types::Json<Vec<Thumb>>, _dir: &str, name: impl Into<String>, aspect: &str, season: Option<&str>) {
    let name = name.into();

    let season = season.map(|mut s| {
        while s.len() > 1 && s.starts_with("0") {
            s = &s[1..];
        }
        s
    });

    thumbs.0.push(Thumb {
        path: name,
        aspect: aspect.to_string(),
        season: season.map(|s| s.to_string()),
    });
}
