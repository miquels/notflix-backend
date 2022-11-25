use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use super::{is_default, Thumb};
use crate::sqlx::impl_sqlx_traits_for;

#[derive(Object, Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Actor {
    #[serde(skip_serializing_if = "is_default")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub order: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub thumb: Option<Thumb>,
    #[serde(skip_serializing_if = "is_default")]
    pub thumb_url: Option<String>,
}
impl_sqlx_traits_for!(Actor);

#[derive(Object, Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Rating {
    #[serde(skip_serializing_if = "is_default")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub default: Option<bool>,
    #[serde(skip_serializing_if = "is_default")]
    pub max: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub value: Option<f32>,
    #[serde(skip_serializing_if = "is_default")]
    pub votes: Option<u32>,
}
impl_sqlx_traits_for!(Rating);

#[derive(Object, Deserialize, Serialize, Clone, Default, Debug, PartialEq, sqlx::FromRow)]
#[serde(default)]
pub struct UniqueId {
    #[serde(rename = "type")]
    pub idtype: String,
    pub default: bool,
    pub id: String,
}
impl_sqlx_traits_for!(UniqueId);
