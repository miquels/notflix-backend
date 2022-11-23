use chrono::TimeZone;

use crate::collections::Collection;
use crate::models::{self, FileInfo, Thumb};
use crate::util::{Id, SystemTimeToUnixTime};
use super::*;

#[derive(Debug)]
pub struct Episode<'c> {
    pub showdir: String,
    pub basepath: String,
    pub files:   Vec<String>,
    pub episode: models::Episode,
    pub coll: &'c Collection,
}

impl<'c> Episode<'c> {

    pub async fn new<'a>(coll: &'a Collection, showdir: String, name: &str, basepath: &str, season_hint: Option<u32>, db_episode: Option<&mut models::Episode>) -> Option<Episode<'a>> {

        // Parse the episode filename for season and episode number etc.
        let ep_info = match EpisodeNameInfo::parse(basepath, season_hint) {
            Some(ep_info) => ep_info,
            None => return None,
        };

        // Must be able to open it.
        let video = match FileInfo::from_path(&showdir, name).await {
            Ok(v) => v,
            Err(_) => return None,
        };

        let mut episode = models::Episode {
            id: Id::new(),
            ..models::Episode::default()
        };
        if let Some(db_episode) = db_episode {
            std::mem::swap(&mut episode, db_episode);
            db_episode.deleted = true;
            episode.deleted = false;
        }
        let modified = video.modified.unixtime_ms();
        if let chrono::LocalResult::Single(c) = chrono::Local.timestamp_millis_opt(modified) {
            episode.dateadded = Some(c.format("%Y-%m-%d").to_string());
        }
        episode.lastmodified = modified;
        episode.video = video;

        // Add info from the filename.
        episode.nfo_base.title = Some(ep_info.name);
        episode.season = ep_info.season;
        episode.episode = ep_info.episode;
        // XXX TODO episode.double = ep_info.double;

        Some(Episode {
            showdir,
            basepath: basepath.to_string(),
            files: Vec::new(),
            episode,
            coll,
        })
    }

    pub async fn finalize(mut self) -> Option<models::Episode> {
        let mut files = std::mem::replace(&mut self.files, Vec::new());
        for name in files.drain(..) {
            self.add_related_file(name).await;
        }
        if !self.episode.nfo_base.nfo_type.is_episode() {
            return None;
        }
        Some(self.episode)
    }

    // See if this is a file that is related to an episode, by
    // indexing on the basepath.
    async fn add_related_file(&mut self, name: String) {

        let caps = IS_RELATED.captures(&name);
        let (aux, ext) = match caps.as_ref() {
            Some(c) => (c.get(2), &c[3]),
            None => return,
        };
        let aux = aux.map_or("", |m| m.as_str());

        // Thumb: <base>.tbn or <base>-thumb.ext
        if IS_IMAGE.is_match(&name) {
            if ext == "tbn" || aux == "thumb" {
                if let Err(e) = Thumb::add(&mut self.episode.thumbs, &self.showdir, &name, self.coll, self.episode.id, "thumb", None).await {
                    log::debug!("Episode::add_related_file: {}/{}: {}", self.showdir, name, e);
                }
            }
            return;
        }

        /* // XXX FIXME subtitles.
        if ext == "srt" {
            if aux == "" || aux == "und" {
                aux = "zz".to_string();
            }
            ep.srt_subs.push(Subs{
                lang: aux,
                path: p,
            });
            return;
        }

        if ext == "vtt" {
            if aux == "" || aux == "und" {
                aux = "zz".to_string();
            }
            ep.vtt_subs.push(Subs{
                lang: aux,
                path: p,
            });
            return;
        }
        */

        if ext == "nfo" {
            match FileInfo::open(&self.showdir, &name).await {
                Ok((mut file, nfofile)) => {

                    let mut nfofile = Some(nfofile);
                    if self.episode.nfofile == nfofile {
                        // No change.
                        nfofile = None;
                    }

                    if nfofile.is_some() {
                        if let Ok(nfo) = Nfo::read(&mut file).await {
                            nfo.update_episode(&mut self.episode);
                            self.episode.nfofile = nfofile;
                        }
                    }
                },
                Err(_) => {},
            }
        }
    }
}

#[derive(Default, Debug)]
struct EpisodeNameInfo {
    name: String,
    season: u32,
    episode: u32,
    double: bool,
}

// Straight from the documentation of once_cell.
macro_rules! regex {
    ($re:expr $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

impl EpisodeNameInfo {

    pub fn parse(name: &str, season_hint: Option<u32>) -> Option<EpisodeNameInfo> {
        let mut ep = EpisodeNameInfo::default();
        let name = name.split('/').last().unwrap();

        // pattern: ___.s03e04.___ or ___.s03.e04.___
        const PAT1: &'static str = r#"^.*[ ._][sS]([0-9]+)\.?[eE]([0-9]+)[ ._].*$"#;
        if let Some(caps) = regex!(PAT1).captures(name) {
            ep.name = format!("{}x{}", &caps[1], &caps[2]);
            ep.season = caps[1].parse::<u32>().unwrap_or(0);
            ep.episode = caps[2].parse::<u32>().unwrap_or(0);
            return Some(ep);
        }

        // pattern: ___.s03e04e05.___ or ___.s03e04-e05.___
        const PAT2: &'static str = r#"^.*[. _][sS]([0-9]+)[eE]([0-9]+)-?[eE]([0-9]+)[. _].*$"#;
        if let Some(caps) = regex!(PAT2).captures(name) {
            ep.name = format!("{}x{}-{}", &caps[1], &caps[2], &caps[3]);
            ep.season = caps[1].parse::<u32>().unwrap_or(0);
            ep.episode = caps[2].parse::<u32>().unwrap_or(0);
            ep.double = true;
            return Some(ep);
        }

        // pattern: ___.2015.03.08.___
        const PAT3: &'static str = r#"^.*[ .]([0-9]{4})[.-]([0-9]{2})[.-]([0-9]{2})[ .].*$"#;
        if let Some(caps) = regex!(PAT3).captures(name) {
            ep.name = format!("{}.{}.{}", &caps[1], &caps[2], &caps[3]);
            ep.season = season_hint.unwrap_or(0);
            let joined = format!("{}{}{}", &caps[1], &caps[2], &caps[3]);
            ep.episode = joined.parse::<u32>().unwrap_or(0);
            return Some(ep);
        }

        // pattern: ___.308.___  (or 3x08) where first number is season.
        const PAT4: &'static str = r#"^.*[ .]([0-9]{1,2})x?([0-9]{2})[ .].*$"#;
        if let Some(caps) = regex!(PAT4).captures(name) {
            if let Ok(sn) = caps[1].parse::<u32>() {
                // Don't parse ___.2022.___
                let no_hint_ok = season_hint.is_none() && sn >= 19 && sn <= 21;
                if no_hint_ok || season_hint == Some(sn) {
                    ep.name = format!("{:02}x{}", sn, &caps[2]);
                    ep.season = sn;
                    ep.episode = caps[2].parse::<u32>().unwrap_or(0);
                    return Some(ep);
                }
            }
        }

        None
    }
}
