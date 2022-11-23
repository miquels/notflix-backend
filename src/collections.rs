use anyhow::Result;
use serde::{Deserialize};
use poem_openapi::Object;

#[derive(Deserialize, Object, Debug, Default)]
pub struct Collection {
    #[serde(rename(deserialize = "__label__"))]
    pub name: String,

    #[serde(rename = "type")]
    #[oai(rename = "type")]
    pub type_: String,

    #[serde(rename="collection-id")]
    #[oai(rename="collection-id")]
    pub collection_id: u32,

    #[oai(skip)]
    pub directory: String,

    #[serde(default, skip)]
    #[oai(skip)]
    pub baseurl: String,
}

impl Collection {
    pub fn check(&self) -> Result<()> {
        if self.type_ != "movies" && self.type_ != "tvshows" {
            bail!(format!("collection {}: unknown type {}", self.name, self.type_));
        }
        if let Err(err) = std::fs::metadata(&self.directory) {
            bail!(format!("collection {}: {}: {}", self.name, self.directory, err));
        }
        Ok(())
    }

    pub fn subtype(&self) -> &'static str {
        match self.type_.as_str() {
            "movies" => "movie",
            "tvshows" => "tvshow",
            other => panic!("Collection: unknown type {}", other),
        }
    }
}
