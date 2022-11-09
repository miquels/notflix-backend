use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};

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
        todo!()
    }

    /// Get a shallow list of all items (no nfo info, no seasons / episode info).
    pub async fn get_items(&self) -> Vec<Item> {
        todo!()
    }

    /// Get item details.
    pub async fn get_item(&self, _name: &str) -> Option<Arc<Item>> {
        todo!();
    }
}

// An 'item' can be a movie, a tv-show, a folder, etc.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Item {
}

impl Item {
    pub fn shallow_clone(&self) -> Item {
        todo!()
    }
}
