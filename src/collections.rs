use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::nfo::Nfo;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Collection {
    pub name: String,
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
    pub type_: &'static str,
    pub firstvideo: u64,
    pub lastvideo: u64,
    pub sortname: Option<String>,
    pub nfo:    Option<Nfo>,
    pub banner: Option<String>,
    pub fanart: Option<String>,
    pub folder: Option<String>,
    pub poster: Option<String>,
    pub rating: Option<f32>,
    pub votes: Option<u32>,
    pub genre: Vec<String>,
    pub year: Option<u32>,

    // movie
    #[serde(skip_serializing_if = "String::is_empty")]
    pub video:  String,
    pub thumb:  Option<String>,

    // show
    pub season_all_banner:  Option<String>,
    pub season_all_fanart:  Option<String>,
    pub season_all_poster:  Option<String>,
    pub seasons: Vec<Season>,

    #[serde(skip)]
    pub nfo_path: PathBuf,
    #[serde(skip)]
    pub nfo_time: u64,
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Season {
    pub seasonno:   u32,
    pub banner: Option<String>,
    pub fanart: Option<String>,
    pub poster: Option<String>,
    pub episodes: Vec<Episode>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Episode {
    pub name:   String,
    pub video:  String,
    pub seasonno: u32,
    pub episodeno: u32,
    pub double: bool,
    pub sortname:   Option<String>,
    pub nfo:    Option<Nfo>,
    pub thumb:  Option<String>,

    #[serde(skip)]
    pub basename: String,
    #[serde(skip)]
    pub nfo_path: String,
    #[serde(skip)]
    pub nfo_time: u64,
    #[serde(skip)]
    pub video_ts: u64,
}

