use std::time::SystemTime;

use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

mod movies;
mod nfo;
mod scandirs;
mod shows;

pub use movies::{build_movie, build_movies};
pub use scandirs::{scan_directory, scan_directories};
pub use shows::{build_show, build_shows};
pub use nfo::Nfo;

// Helper macro.
macro_rules! def_regex {
    ($name:ident => $re:expr) => {
        pub static $name: Lazy<Regex> = Lazy::new(|| Regex::new($re).unwrap());
    };
}

def_regex!(IS_VIDEO => r#"^(.*)\.(divx|mov|mp4|MP4|m4u|m4v)$"#);
def_regex!(IS_IMAGE => r#"^(.+)\.(jpg|jpeg|png|tbn)$"#);
def_regex!(IS_SEASON_IMG => r#"^season([0-9]+)-?([a-z]+|)\.(jpg|jpeg|png|tbn)$"#);
def_regex!(IS_SHOW_SUBDIR => r#"^S([0-9]+)|Specials([0-9]*)$"#);
def_regex!(IS_EXT1 => r#"^(.*)\.(png|jpg|jpeg|tbn|nfo|srt)$"#);
def_regex!(IS_EXT2 => r#"^(.*)[.-]([a-z]+)\.(png|jpg|jpeg|tbn|nfo|srt)$"#);
def_regex!(IS_YEAR => r#" \(([0-9]+)\)$"#);

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
