use crate::models::{self, TVShow, FileInfo};
use super::shows::Show;
use super::*;

#[derive(Default, Debug)]
pub struct Episode {
    pub basedir: String,
    pub basepath: String,
    pub files:   Vec<String>,
    pub episode: models::Episode,
}

impl Episode {

    pub async fn new(show: &mut Show, name: &str, db_tvshow: &mut TVShow) -> Option<Episode> {

        // Parse the name, see if it's a video, extract information.
        let caps = IS_VIDEO.captures(name);
        let (basepath, hint, basename, _ext) = match caps.as_ref() {
            Some(caps) => (&caps[1], &caps[2], &caps[3], &caps[4]),
            None => return None,
        };
        let season_hint = hint.parse::<u32>().ok();

        // Parse the episode filename for season and episode number etc.
        let ep_info = match EpisodeNameInfo::parse(basename, season_hint) {
            Some(ep_info) => ep_info,
            None => return None,
        };

        // Must be able to open it.
        let video = match FileInfo::from_path(&show.basedir, None, name).await {
            Ok(v) => sqlx::types::Json(v),
            Err(_) => return None,
        };

        // If this episode was already in the database, copy its ID.
        // We also _un_mark it as deleted.
        let id = match show.get_db_episode_mut(db_tvshow, basepath) {
            Some(ep) => {
                ep.deleted = false;
                ep.id
            },
            None => 0,
        };

        let mut episode = models::Episode {
            id,
            video,
            ..models::Episode::default()
        };

        // Add info from the filename.
        episode.nfo_base.title = Some(ep_info.name);
        episode.season = ep_info.season;
        episode.episode = ep_info.episode;
        // XXX TODO episode.double = ep_info.double;

        Some(Episode {
            basedir: show.basedir.clone(),
            basepath: basepath.to_string(),
            files: Vec::new(),
            episode,
        })
    }

    pub async fn finalize(mut self, db_episode: Option<&models::Episode>) -> models::Episode {
        let mut files = std::mem::replace(&mut self.files, Vec::new());
        for name in files.drain(..) {
            self.add_related_file(name, db_episode).await;
        }
        self.episode
    }

    // See if this is a file that is related to an episode, by
    // indexing on the basepath.
    async fn add_related_file(&mut self, name: String, db_episode: Option<&models::Episode>) {

        let caps = IS_RELATED.captures(&name);
        let (aux, ext) = match caps.as_ref() {
            Some(c) => (c.get(2), &c[3]),
            None => return,
        };
        let aux = aux.map_or("", |m| m.as_str());

        // Thumb: <base>.tbn or <base>-thumb.ext
        if IS_IMAGE.is_match(&name) {
            if ext == "tbn" || aux == "thumb" {
                add_thumb(&mut self.episode.thumbs, "", name, "thumb", None);
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
            match FileInfo::open(&self.basedir, None, &name).await {
                Ok((mut file, nfofile)) => {
                    let mut nfofile = Some(sqlx::types::Json(nfofile));

                    if let Some(db_ep) = db_episode {
                        if db_ep.nfofile == nfofile {
                            // No change, so we can copy the data.
                            self.episode.copy_nfo_from(db_ep);
                            nfofile = None;
                        }
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
                if season_hint.is_none() || season_hint == Some(sn) {
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
