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
