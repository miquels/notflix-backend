use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};

use crate::kodifs::{self, Nfo};

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
    #[serde(rename(deserialize = "__label__"))]
    pub name: String,

    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename="collection-id")]
    pub collection_id: u32,

    #[serde(skip_serializing)]
    pub directory: String,

    #[serde(default, skip)]
    pub items: Mutex<Vec<ArcSwap<Item>>>,

    #[serde(default, skip)]
    pub baseurl: String,
}

impl Collection {

    /// Scan directory of this collection and update items.
    pub async fn scan(&self) {
        // FIXME: use enum
        match self.type_.as_str() {
            "movies" => {
                println!("scanning collection {}: movies", self.collection_id);
                kodifs::build_movies(self, 0).await;
            },
            "shows" | "series" => {
                println!("scanning collection {}: series", self.collection_id);
                kodifs::build_shows(self, 0).await;
            },
            other => {
                println!("skipping collection {}: unknown type {}", self.collection_id, other);
            },
        }
    }

    /// Get a shallow list of all items (no nfo info, no seasons / episode info).
    pub async fn get_items(&self) -> Vec<Item> {
        self.items.lock().unwrap().iter().map(|i| i.load().shallow_clone()).collect()
    }

    /// Get item details.
    pub async fn get_item(&self, name: &str) -> Option<Arc<Item>> {
        todo!();
    }
    /*
    pub async fn get_item(&self, name: &str) -> Option<Arc<Item>> {
        // Find item.
        let item = self.items.lock().unwrap().iter().map(|i| i.load()).find(|i| i.name == name)?;

        println!("XXX 1");

        // Needs a full rescan if there are still nfo files that we haven't
        // parsed yet. FIXME: use timestamps or something.
        let need_update = match self.type_.as_str() {
            "movies" => item.nfo_path.is_some() && item.nfo.is_none(),
            "shows" | "series" => {
                item
                .seasons
                .iter()
                .any(|s| s.episodes.iter().any(|e| e.nfo_path.is_some() && e.nfo.is_none()))
            }
            _ => return None,
        };

        println!("XXX 2 need update {:?}", need_update);

        if !need_update {
            return Some(item.clone());
        }

        // Try to update.
        println!("XXX 3 {} {} {}", self.directory, name, self.type_);

        let new_item = Arc::new(match self.type_.as_str() {
            "movies" => kodifs::build_movie(self, &name).await?,
            "shows" | "series" => kodifs::build_show(self, &name).await?,
            _ => return None,
        });

        println!("XXX 4 {:?}", new_item);

        // Store updated item.
        let items = self.items.lock().unwrap();
        for item in items.iter() {
            if item.load().name == new_item.name {
                item.store(new_item.clone());
                return Some(new_item);
            }
        }
        None
    }
    */
}

// An 'item' can be a movie, a tv-show, a folder, etc.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Item {
    // generic
    pub name:   String,
    pub path:   String,
    pub baseurl: String,
    #[serde(rename = "type")]
    pub type_: String,
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

impl Item {
    pub fn shallow_clone(&self) -> Item {
        Item {
            name: self.name.clone(),
            path: self.path.clone(),
            baseurl: self.baseurl.clone(),
            type_: self.type_.clone(),
            firstvideo: self.firstvideo,
            lastvideo: self.lastvideo,
            sortname: self.sortname.clone(),
            banner: self.banner.clone(),
            folder: self.folder.clone(),
            poster: self.poster.clone(),
            rating: self.rating.clone(),
            votes: self.votes.clone(),
            genre: self.genre.clone(),
            year: self.year.clone(),
            thumb: self.thumb.clone(),
            season_all_banner: self.season_all_banner.clone(),
            season_all_poster: self.season_all_poster.clone(),
            season_all_fanart: self.season_all_fanart.clone(),
            ..Item::default()
        }
    }
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
