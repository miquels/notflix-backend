use serde::{Deserialize, Serialize};

use super::is_default;

#[derive(Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Actor {
    #[serde(skip_serializing_if = "is_default")]
    pub name:   Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub role:   Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub order: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub thumb:  Option<Thumb>,
    #[serde(skip_serializing_if = "is_default")]
    pub thumb_url:  Option<String>,
}

/// Image
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Thumb {
    #[serde(rename(deserialize = "$value"))]
    pub path:     String,
    pub aspect:   String,
    #[serde(skip_serializing_if = "is_default")]
    pub season:  Option<String>,
    #[serde(skip)]
    pub state: ThumbState,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Rating {
    #[serde(skip_serializing_if = "is_default")]
    pub name:   Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub default: Option<bool>,
    #[serde(skip_serializing_if = "is_default")]
    pub max:    Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub value:    Option<f32>,
    #[serde(skip_serializing_if = "is_default")]
    pub votes:    Option<u32>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, PartialEq, sqlx::FromRow)]
#[serde(default)]
pub struct UniqueId {
    #[serde(rename = "type")]
    pub idtype: String,
    pub default: bool,
    pub id: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ThumbState {
    Deleted,
    #[default]
    Unchanged,
    New,
}
