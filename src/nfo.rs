use tokio::fs;
use tokio::io::AsyncReadExt;

use scan_fmt::scan_fmt;
use serde::{de, Deserialize, Serialize};
use serde_xml_rs::from_str;

#[derive(Serialize, Deserialize, Debug)]
pub struct Nfo {
    pub title:  Option<String>,
    pub id: Option<String>,
    #[serde(deserialize_with = "deserialize_runtime")]
    pub runtime: Option<u32>,
    pub mpaa: Option<String>,
    #[serde(deserialize_with = "deserialize_u32")]
    pub year: Option<u32>,
    pub originaltitle: Option<String>,
    pub plot: Option<String>,
    pub tagline: Option<String>,
    pub premiered: Option<String>,
    pub season: Option<String>,
    pub episode: Option<String>,
    pub aired: Option<String>,
    pub studio: Option<String>,
    #[serde(deserialize_with = "deserialize_f32")]
    pub rating: Option<f32>,
    #[serde(deserialize_with = "deserialize_u32")]
    pub votes: Option<u32>,
    pub genre: Vec<String>,
    pub actor: Vec<String>,
    pub director: Option<String>,
    pub credits: Option<String>,
    pub thumb: Option<Thumb>,
    pub fanart: Vec<String>,
    pub banner: Vec<String>,
    pub discart: Option<String>,
    pub logo: Option<String>,
    pub fileinfo: Option<VidFileInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Thumb {
    pub thumb:   Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Actor {
    pub name:   String,
    pub role:   String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VidFileInfo {
    streamdetails:  Option<StreamDetails>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamDetails {
    video:  Option<VideoDetails>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoDetails {
    codec:  Option<String>,
    aspect: Option<f32>,
    width:  Option<u32>,
    height:  Option<u32>,
}

// Sometimes the "rating" field is invalid. Ignore it if so.
fn deserialize_f32<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    Ok(s.parse::<f32>().ok())
}

// Sometimes the "vote" or "year" field is invalid. Ignore it if so.
fn deserialize_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    Ok(s.parse::<u32>().ok())
}

// Decode the "runtime" field. Should be in minutes, but ..
fn deserialize_runtime<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    if let Ok(t) = s.parse::<u32>() {
        if t > 0 {
            return Ok(Some(t));
        }
        return Ok(None);
    }
    if let Ok((h, m, _)) = scan_fmt!(s, "{}:{}:{}", u32, u32, u32) {
        return Ok(Some(h * 60 + m));
    }
    if let Ok((h, m)) = scan_fmt!(s, "{}:{}", u32, u32) {
        return Ok(Some(h * 60 + m));
    }
    if let Ok((h, m, _)) = scan_fmt!(s, "{}h{}m{}", u32, u32, u32) {
        return Ok(Some(h * 60 + m));
    }
    if let Ok((h, m)) = scan_fmt!(s, "{}h{}", u32, u32) {
        return Ok(Some(h * 60 + m));
    }
    Ok(None)
}

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

/*

// Compare the NFO file data with the 'Item'.
pub async fn update_item(item: &mut Item) -> anyhow::Result<bool> {

    let nfo_path = match item.nfo_path {
        Some(ref p) => p,
        None => return Ok(false),
    };

    let fh = fs::File::open(nfo_path)?;
    let modified = fh.metadata().await.map(|m| systemtime_to_ms(m.modified()?))?;
    if item.nfo_time > 0 && item.nfo_time == modified {
        return Ok(false);
    }
    let nfo = Nfo::read(&file)?;

    item.nfo_time = modified;
    item.genre = nfo.genre;
    item.rating = nfo.rating;
    item.votes = nfo.votes;
    item.year = nfo.year;

    Ok(true)
}

*/
