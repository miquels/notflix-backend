use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;
use crate::models::{Thumb, ThumbState};

mod movies;
mod nfo;
mod shows;
mod episode;
pub mod scandirs;

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

// Add a thumb to a Vec of Thumbs.
// If the thumb was already present, we change nothing and return `false` (no update).
// If the thumb was _not_ already present, we return `true` (updated).
fn add_thumb(thumbs: &mut sqlx::types::Json<Vec<Thumb>>, _dir: &str, name: impl Into<String>, aspect: impl Into<String>, season: Option<&str>) -> bool {
    let name = name.into();
    let aspect = aspect.into();

    let season = season.map(|mut s| {
        while s.len() > 1 && s.starts_with("0") {
            s = &s[1..];
        }
        s.to_string()
    });

    let t = Thumb {
        path: name,
        aspect,
        season,
        state: ThumbState::New,
    };

    // See if this thumb was already present.
    let e = thumbs.0
        .iter_mut()
        .find(|x| x.path == t.path && x.aspect == t.aspect && x.season == x.season);
    if let Some(x) = e {
        // Yep, so it's unchanged.
        x.state = ThumbState::Unchanged;
        return false;
    }

    // No, so push it as new.
    thumbs.0.push(t);
    true
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
