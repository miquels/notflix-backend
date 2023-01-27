use anyhow::Result;
use poem_openapi::{Enum, Object};
use serde::{de, de::Error as _, Deserialize};

#[derive(Deserialize, Enum, Debug, Default)]
pub enum CollectionType {
    #[default]
    Movies,
    TVShows,
}

#[derive(Deserialize, Object, Debug, Default)]
pub struct Collection {
    #[serde(rename(deserialize = "__label__"))]
    pub name: String,

    #[serde(rename = "type", deserialize_with = "deserialize_type")]
    #[oai(rename = "type")]
    pub type_: CollectionType,

    #[serde(rename = "collection-id")]
    #[oai(rename = "collection-id")]
    pub collection_id: u32,

    #[oai(skip)]
    pub directory: String,

    #[serde(default, skip)]
    #[oai(skip)]
    pub baseurl: String,
}

impl Collection {
    pub fn check(&self) -> Result<()> {
        if let Err(err) = std::fs::metadata(&self.directory) {
            bail!(format!("collection {}: {}: {}", self.name, self.directory, err));
        }
        Ok(())
    }

    pub fn subtype(&self) -> &'static str {
        match self.type_ {
            CollectionType::Movies => "movie",
            CollectionType::TVShows => "tvshow",
        }
    }
}

fn deserialize_type<'de, D>(deserializer: D) -> Result<CollectionType, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(match s.as_str() {
        "tvshows" | "tvseries" => CollectionType::TVShows,
        "movies" => CollectionType::Movies,
        _ => return Err(D::Error::unknown_variant("unknown type", &["tvshows", "movies"])),
    })
}
