//! Kodi style NFO file support.
//!
//! `NFO::read()` is the main entry point.
//!
//! For information about the Kodi NFO file stucture, see:
//!
//! - [Movies](https://kodi.wiki/view/NFO_files/Movies)
//! - [TV Shows](https://kodi.wiki/view/NFO_files/TV_shows)
//! - [Episodes](https://kodi.wiki/view/NFO_files/Episodes)
//!
use tokio::fs;
use tokio::io::AsyncReadExt;

use scan_fmt::scan_fmt;
use serde::{de, Deserialize, Serialize};
use serde_xml_rs::from_str;

use crate::jvec::JVec;
use crate::models;

/// Thumbnail
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Thumb {
    #[serde(skip_serializing_if = "Option::is_none", rename(deserialize = "$value"))]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

/// Fanart (16:9 1080x1920 image, usually).
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Fanart {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub thumb: Vec<Thumb>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename(deserialize = "$value"))]
    pub image: Vec<String>,
}

/// Ids from imdb, themoviedb, thetvdb, etc
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct UniqueId {
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub idtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_bool")]
    pub default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename(deserialize = "$value"))]
    pub id: Option<String>,
}

/// Actor information.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Actor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub order: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<Thumb>,
}

/// video/audio info.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct VidFileInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streamdetails: Option<StreamDetails>,
}

/// video/audio info
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct StreamDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoDetails>,
}

/// video/audio info
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct VideoDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_f32")]
    pub aspect: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_f32")]
    pub duration: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub durationinseconds: Option<u32>,
}

/// Ratings.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Ratings {
    rating: Vec<Rating>,
}

impl Ratings {
    fn is_empty(&self) -> bool {
        self.rating.is_empty()
    }
}

/// Rating from a certain source ('name' can be imdb, tmdb, etc)
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Rating {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_bool")]
    pub default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_f32")]
    pub value: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub votes: Option<u32>,
}

/// NFO file type.
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub enum NfoType {
    Movie,
    TVShow,
    Episode,
    #[default]
    Unknown,
}

impl NfoType {
    /// Movie or Unknown.
    pub fn is_movie(&self) -> bool {
        *self == NfoType::Movie || *self == NfoType::Unknown
    }
    /// TVShow or Unknown.
    pub fn is_tvshow(&self) -> bool {
        *self == NfoType::TVShow || *self == NfoType::Unknown
    }
    /// Episode or Unknown.
    pub fn is_episode(&self) -> bool {
        *self == NfoType::Episode || *self == NfoType::Unknown
    }
}

/// NFO file contents.
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct Nfo {
    #[serde(skip)]
    pub nfo_type: NfoType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub originaltitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub sorttitle: Option<String>,

    #[serde(skip_serializing_if = "Ratings::is_empty")]
    pub ratings: Ratings,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub outline: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plot: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub tagline: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_runtime")]
    pub runtime: Option<u32>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub thumb: Vec<Thumb>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fanart: Vec<Fanart>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
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

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub premiered: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub year: Option<u32>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub studio: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub trailer: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fileinfo: Option<VidFileInfo>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actor: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub season: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub episode: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub displayseason: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub displayepisode: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub aired: Option<String>,

    /// The following fields are unofficial and should not be used.
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_f32")]
    pub rating: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_u32")]
    pub votes: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub banner: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub discart: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub logo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub imdbid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub tmdbid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string")]
    pub tvdbid: Option<String>,
}

impl Nfo {
    // Read NFO from a tokio::fs::File handle.
    pub async fn read(file: &mut fs::File) -> anyhow::Result<Nfo> {
        let mut xml = String::new();
        file.read_to_string(&mut xml).await?;
        let mut nfo: Nfo = from_str(&xml)?;

        let xml = xml.trim_start();
        if xml.starts_with("<movie>") {
            nfo.nfo_type = NfoType::Movie;
        }
        if xml.starts_with("<tvshow>") {
            nfo.nfo_type = NfoType::TVShow;
        }
        if xml.starts_with("<episode") {
            nfo.nfo_type = NfoType::Episode;
        }

        // Fix up genre.
        if nfo.genre.iter().any(|g| g.contains(",") || g.contains("/")) {
            let g = nfo
                .genre
                .iter()
                .map(|g| g.split(|c| c == ',' || c == '/'))
                .flatten()
                .map(|s| s.trim())
                .filter_map(|s| (s != "").then(|| s.to_string()))
                .collect::<Vec<_>>();
            nfo.genre = g;
        }
        nfo.genre = crate::genres::normalize_genres(&nfo.genre);

        // Fix up empty vecs.
        nfo.credits.retain(|s| s.len() > 0);
        nfo.director.retain(|s| s.len() > 0);
        nfo.country.retain(|s| s.len() > 0);
        nfo.studio.retain(|s| s.len() > 0);

        //println!("{:#?}", nfo);
        Ok(nfo)
    }

    /// Fill `models::Nfo` with data from the nfo file.
    pub fn to_nfo(&self) -> models::Nfo {
        let mut ratings = self
            .ratings
            .rating
            .iter()
            .map(|r| models::Rating {
                name: r.name.clone(),
                default: r.default.clone(),
                max: r.max,
                value: r.value,
                votes: r.votes,
            })
            .collect::<Vec<_>>();
        let mut uniqueids = self
            .uniqueid
            .iter()
            .filter(|i| i.id.is_some() && i.idtype.is_some())
            .map(|i| models::UniqueId {
                idtype: i.idtype.clone().unwrap(),
                default: i.default.unwrap_or(false),
                id: i.id.clone().unwrap(),
            })
            .collect::<Vec<_>>();
        let actors = self
            .actor
            .iter()
            .map(|a| models::Actor {
                name: a.name.clone(),
                role: a.role.clone(),
                order: a.order,
                // FIXME thumb_url: a.thumb.clone(),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        // If we have no `ratings` but we do have `rating` use that.
        if ratings.len() == 0 && self.rating.is_some() {
            let mut idtype = Some(String::from(""));
            if self.id.is_some() {
                for uid in &self.uniqueid {
                    if uid.id == self.id && uid.idtype.is_some() {
                        idtype = uid.idtype.clone();
                    }
                }
            }
            ratings.push(models::Rating {
                name: idtype,
                default: None,
                max: None,
                value: self.rating.clone(),
                votes: self.votes.clone(),
            });
        }

        if uniqueids.len() == 0 {
            let mut default = true;

            if let Some(id) = self.imdbid.as_ref().or_else(|| self.id.as_ref()) {
                if id.starts_with("tt") {
                    uniqueids.push(models::UniqueId {
                        idtype: "imdb".to_string(),
                        default,
                        id: id.to_string(),
                    });
                    default = false;
                }
            }
            if let Some(id) = self.tmdbid.as_ref() {
                if id.len() > 0 && id != "0" {
                    uniqueids.push(models::UniqueId {
                        idtype: "tmdb".to_string(),
                        default,
                        id: id.to_string(),
                    });
                    default = false;
                }
            }
            if let Some(id) = self.tvdbid.as_ref() {
                if id.len() > 0 && id != "0" {
                    uniqueids.push(models::UniqueId {
                        idtype: "tvdb".to_string(),
                        default,
                        id: id.to_string(),
                    });
                }
            }

            // Last resort.
            if uniqueids.len() == 0 && self.id.is_some() {
                uniqueids.push(models::UniqueId {
                    idtype: "nfoid".to_string(),
                    default: false,
                    id: self.id.clone().unwrap(),
                });
            }
        }

        models::Nfo {
            nfo_type: self.nfo_type.clone(),
            title: self.title.clone(),
            plot: self.plot.clone(),
            tagline: self.tagline.clone(),
            ratings: JVec(ratings),
            uniqueids: JVec(uniqueids),
            actors: JVec(actors),
            credits: JVec(self.credits.clone()),
            directors: JVec(self.director.clone()),
            originaltitle: self.originaltitle.clone(),
            sorttitle: self.sorttitle.clone(),
            countries: JVec(self.country.clone()),
            genres: JVec(self.genre.clone()),
            studios: JVec(self.studio.clone()),
            premiered: self.premiered.clone(),
            mpaa: self.mpaa.clone(),
            runtime: self.get_runtime_in_mins(),
            season: self.season.clone(),
            episode: self.episode.clone(),
            status: self.status.clone(),
            aired: self.status.clone(),
            displayseason: self.displayseason.clone(),
            displayepisode: self.displayepisode.clone(),
        }
    }

    fn get_runtime_in_mins(&self) -> Option<u32> {
        if self.runtime.is_some() {
            return self.runtime;
        }
        let video = self
            .fileinfo
            .as_ref()
            .and_then(|f| f.streamdetails.as_ref())
            .and_then(|s| s.video.as_ref())?;
        if let Some(d) = video.duration {
            return Some(d as u32);
        }
        if let Some(d) = video.durationinseconds {
            return Some(d / 60);
        }
        None
    }
}

// Sometimes a f32 field (like "rating") is invalid. Ignore it if so.
fn deserialize_f32<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<f32>().ok().filter(|v| *v >= 0f32 && *v < u32::MAX as f32))
}

// Sometimes a u32 field (like "vote" or "year") is invalid. Ignore it if so.
fn deserialize_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<u32>().ok())
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(if s == "true" || s == "True" { Some(true) } else { None })
}

fn deserialize_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(if s.len() > 0 { Some(s) } else { None })
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
