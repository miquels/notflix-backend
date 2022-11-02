use tokio::fs;
use tokio::io::AsyncReadExt;

use scan_fmt::scan_fmt;
use serde::{de, Deserialize, Serialize};
use serde_xml_rs::from_str;
// use quick_xml::de::from_str;

use crate::collections::Item;
use crate::kodifs::systemtime_to_ms;

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct Nfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title:  Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub originaltitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorttitle: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ratings: Vec<Rating>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plot: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_runtime")]
    pub runtime: Option<u32>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub thumb: Vec<Thumb>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fanart: Vec<Fanart>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mpaa: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub uniqueid: Vec<UniqueId>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub genre: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub country: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub credits: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub director: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub premiered: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub year: Option<u32>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub studio: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fileinfo: Option<VidFileInfo>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actor: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub season: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aired: Option<String>,

    // The below fields are unofficial and should not be used.
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_f32")]
    pub rating: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub votes: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub banner: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discart: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Thumb {
    #[serde(skip_serializing_if = "Option::is_none", rename(deserialize = "$value"))]
    pub image:   Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect:  Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview:  Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Fanart {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub thumb: Vec<Thumb>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename(deserialize = "$value"))]
    pub image: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct UniqueId {
    #[serde(skip_serializing_if = "Option::is_none", rename="type")]
    pub type_:  Option<String>,
    #[serde(skip_serializing_if = "not_true")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename(deserialize = "$value"))]
    pub id:   Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Actor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name:   Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role:   Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub order: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb:  Option<Thumb>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct VidFileInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streamdetails:  Option<StreamDetails>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct StreamDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video:  Option<VideoDetails>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct VideoDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec:  Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_f32")]
    pub aspect: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub width:  Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub height:  Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Rating {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name:   Option<String>,
    #[serde(skip_serializing_if = "not_true")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub max:    Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_f32")]
    pub value:    Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub votes:    Option<u32>,
}

// Sometimes the "rating" field is invalid. Ignore it if so.
fn deserialize_f32<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<f32>().ok())
}

// Sometimes the "vote" or "year" field is invalid. Ignore it if so.
fn deserialize_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<u32>().ok())
}

// Decode the "runtime" field. Should be in minutes, but ..
fn deserialize_runtime<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if let Ok(t) = s.parse::<u32>() {
        if t > 0 {
            return Ok(Some(t));
        }
        return Ok(None);
    }
    if let Ok((h, m, _)) = scan_fmt!(&s, "{}:{}:{}", u32, u32, u32) {
        return Ok(Some(h * 60 + m));
    }
    if let Ok((h, m)) = scan_fmt!(&s, "{}:{}", u32, u32) {
        return Ok(Some(h * 60 + m));
    }
    if let Ok((h, m, _)) = scan_fmt!(&s, "{}h{}m{}", u32, u32, u32) {
        return Ok(Some(h * 60 + m));
    }
    if let Ok((h, m)) = scan_fmt!(&s, "{}h{}", u32, u32) {
        return Ok(Some(h * 60 + m));
    }
    Ok(None)
}

fn not_true(s: &Option<String>) -> bool {
    s.as_ref().map(|v| v != "true").unwrap_or(true)
}

impl Nfo {
    // Read NFO from a tokio::fs::File handle.
    pub async fn read(file: &mut fs::File) -> anyhow::Result<Nfo> {
        let mut xml = String::new();
        file.read_to_string(&mut xml).await?;
        let mut nfo: Nfo = from_str(&xml)?;

        // Fix up genre.
        if nfo.genre.iter().any(|g| g.contains(",") || g.contains("/")) {
            let g = nfo.genre
                .iter()
                .map(|g| g.split(|c| c == ',' || c == '/'))
                .flatten()
                .map(|s| s.trim())
                .filter_map(|s| (s != "").then(|| s.to_string()))
                .collect::<Vec<_>>();
            nfo.genre = g;
        }
        nfo.genre = crate::genres::normalize_genres(&nfo.genre);

        Ok(nfo)
    }

    // Compare the NFO file data with the 'Item'.
    pub async fn update_item(item: &mut Item) -> anyhow::Result<bool> {

        let nfo_path = match item.nfo_path {
            Some(ref p) => p,
            None => return Ok(false),
        };

        let mut file = fs::File::open(nfo_path).await?;
        let modified = systemtime_to_ms(file.metadata().await.map(|m| m.modified().unwrap())?);
        if item.nfo_time > 0 && item.nfo_time == modified {
            return Ok(false);
        }
        let nfo = Nfo::read(&mut file).await?;

        item.nfo_time = modified;
        item.genre = nfo.genre;
        item.rating = nfo.rating;
        item.votes = nfo.votes;
        item.year = nfo.year;

        Ok(true)
    }
}
