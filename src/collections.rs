use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::nfo::Nfo;

trait IsEmpty {
    fn empty(&self) -> bool;
}

impl IsEmpty for Option<String> {
    fn empty(&self) -> bool {
        self.as_ref().map(|s| s == "").unwrap_or(true)
    }
}

impl IsEmpty for Vec<String> {
    fn empty(&self) -> bool {
        self.is_empty() || self.len() == 1 && self[0] == ""
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Collection {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub items: Vec<Item>,
    pub directory: String,
    pub baseurl: String,

    #[serde(skip)]
    pub source_id: u32,
}

// An 'item' can be a movie, a tv-show, a folder, etc.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Item {
    // generic
    pub name:   String,
    pub path:   String,
    pub baseurl: String,
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub firstvideo: u64,
    pub lastvideo: u64,
    #[serde(skip_serializing_if = "Option::empty")]
    pub sortname: Option<String>,
    #[serde(skip_serializing_if = "Option::empty")]
    pub banner: Option<String>,
    #[serde(skip_serializing_if = "Option::empty")]
    pub fanart: Option<String>,
    #[serde(skip_serializing_if = "Option::empty")]
    pub folder: Option<String>,
    #[serde(skip_serializing_if = "Option::empty")]
    pub poster: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub votes: Option<u32>,
    #[serde(skip_serializing_if = "Vec::empty")]
    pub genre: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,

    // movie
    #[serde(skip_serializing_if = "String::is_empty")]
    pub video:  String,
    #[serde(skip_serializing_if = "Option::empty")]
    pub thumb:  Option<String>,

    // show
    #[serde(skip_serializing_if = "Option::empty")]
    pub season_all_banner:  Option<String>,
    #[serde(skip_serializing_if = "Option::empty")]
    pub season_all_fanart:  Option<String>,
    #[serde(skip_serializing_if = "Option::empty")]
    pub season_all_poster:  Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub seasons: Vec<Season>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nfo:    Option<Nfo>,

    #[serde(skip)]
    pub nfo_path: Option<PathBuf>,
    #[serde(skip)]
    pub nfo_time: u64,
}

fn is_false(val: &bool) -> bool {
    !*val
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Season {
    pub seasonno:   u32,
    #[serde(skip_serializing_if = "Option::empty")]
    pub banner: Option<String>,
    #[serde(skip_serializing_if = "Option::empty")]
    pub fanart: Option<String>,
    #[serde(skip_serializing_if = "Option::empty")]
    pub poster: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub episodes: Vec<Episode>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Episode {
    pub name:   String,
    pub video:  String,
    pub seasonno: u32,
    pub episodeno: u32,
    #[serde(skip_serializing_if = "is_false")]
    pub double: bool,
    #[serde(skip_serializing_if = "Option::empty")]
    pub sortname:   Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nfo:    Option<Nfo>,
    #[serde(skip_serializing_if = "Option::empty")]
    pub thumb:  Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub srt_subs: Vec<Subs>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vtt_subs: Vec<Subs>,

    #[serde(skip)]
    pub basename: String,
    #[serde(skip)]
    pub nfo_path: Option<PathBuf>,
    #[serde(skip)]
    pub nfo_time: u64,
    #[serde(skip)]
    pub video_ts: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Subs {
    pub lang: String,
    pub path: String,
}
