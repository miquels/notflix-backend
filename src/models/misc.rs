use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Actor {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Thumb {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Fanart {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Ratings {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct UniqueId {
}
