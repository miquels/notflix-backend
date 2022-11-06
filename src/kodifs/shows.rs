use std::collections::HashMap;
use std::time::Duration;

use tokio::fs;

use crate::collections::Collection;
use crate::models::{TVShow, Episode, FileInfo};
use super::*;

#[derive(Debug)]
struct EpMap {
    season_idx: usize,
    episode_idx: usize,
}

pub async fn build_shows(coll: &Collection, pace: u32) {
    todo!()
}

/*
pub async fn build_shows(coll: &Collection, pace: u32) {

    let mut d = match fs::read_dir(&coll.directory).await {
        Ok(d) => d,
        Err(_) => return,
    };
    let mut items = Vec::new();

    while let Ok(Some(entry)) = d.next_entry().await {
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(name) => name,
            None => continue,
        };
        if name.starts_with(".") || name.starts_with("+ ") {
            continue;
        }

        if let Some(m) = Item::build_show(coll, name).await {
            items.push(ArcSwap::new(Arc::new(m)));
        }
        if pace > 0 {
            tokio::time::sleep(Duration::from_secs(pace as u64)).await;
        }
    }
    // XXX FIXME: merge strategy:
    // - use unique_id
    // - if show is gone, mark as deleted, don't really delete
    //   (that will allow it to be restored later)
    *coll.items.lock().unwrap() = items;
}
*/

pub async fn build_show(coll: &Collection, name: &str) -> Option<TVShow> {
    Show::build_show(coll, name).await
}

#[derive(serde::Serialize, Debug, Default)]
struct Season {
    pub seasonno:   u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fanart: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub episodes: Vec<Episode>,
}

#[derive(serde::Serialize, Default, Debug)]
struct Show {
    tvshow:  TVShow,
    seasons: Vec<Season>,
    // db:      DataBase,
    banner:  Option<String>,
    fanart:  Option<String>,
    folder:  Option<String>,
    poster:  Option<String>,
    season_all_banner:  Option<String>,
    season_all_poster:  Option<String>,
    season_specials_banner:  Option<String>,
    season_specials_poster:  Option<String>,
}

impl Show {

    // Look up season by seasonno. If not present, create,
    fn get_season(&mut self, seasonno: u32) -> usize {
        // find the season.
        for idx in 0 .. self.seasons.len() {
            if seasonno == self.seasons[idx].seasonno {
                return idx;
            }
        }

        // not present yet, so add.
        let sn = Season { seasonno, ..Season::default() };
        self.seasons.push(sn);

        self.seasons.len() - 1
    }

    fn get_episode(&self, season_idx: usize, episode_idx: usize) -> &Episode {
        &self.seasons[season_idx].episodes[episode_idx]
    }

    fn get_episode_mut(&mut self, season_idx: usize, episode_idx: usize) -> &mut Episode {
        &mut self.seasons[season_idx].episodes[episode_idx]
    }

    // This function scans a directory for tv show information.
    // It can be the base directory, in which case 'season_hint' is None,
    // or it can be a season subdirectory (like S01/) and then season_hint is Some(1).
    #[async_recursion::async_recursion]
    async fn show_scan_dir(&mut self, basedir: &str, subdir: Option<&'async_recursion str>, ep_map: &mut HashMap<String, EpMap>, season_hint: Option<u32>) {

        // Read the entire directory in one go.
        let dir = match subdir {
            Some(subdir) => format!("{}/{}", basedir, subdir),
            None => basedir.to_string(),
        };
        let mut d = match fs::read_dir(&dir).await {
            Ok(d) => d,
            Err(_) => return,
        };
        let mut entries = Vec::new();
        while let Ok(Some(entry)) = d.next_entry().await {
            entries.push(entry);
        }
        entries.sort_by_key(|e| e.file_name());

        for entry in &entries {

            // Decode name.
            let file_name = entry.file_name();
            let name = match file_name.to_str() {
                Some(name) => name,
                None => continue,
            };
            if name.starts_with(".") || name.starts_with("+ ") {
                continue;
            }

            // first things that can only be found in the
            // shows basedir, not in subdirs.
            if season_hint.is_none() {

                // S* subdir.
                if let Some(s) = IS_SHOW_SUBDIR.captures(name) {
                    let sn = s[1].parse::<u32>().unwrap();
                    self.show_scan_dir(basedir, Some(name), ep_map, Some(sn)).await;
                    continue;
                }

                // nfo file.
                /* XXX TODO
                if name == "tvshow.nfo" {
                    let mut nfo_path = PathBuf::from(&dir);
                    nfo_path.push(name);
                    self.nfo_path = Some(nfo_path);
                    continue;
                }*/

                // other images.
                if let Some(s) = IS_IMAGE.captures(name) {
                    let p = escape_path(name);
                    match &s[1] {
                        "season-all-banner" => self.season_all_banner = Some(p),
                        "season-all-poster" => self.season_all_poster = Some(p),
                        "season-specials-banner" => self.season_specials_banner = Some(p),
                        "season-specials-poster" => self.season_specials_poster = Some(p),
                        "banner" => self.banner = Some(p),
                        "fanart" => self.fanart = Some(p),
                        // Note, folder is probably an alias for poster
                        "folder" => self.folder = Some(p),
                        "poster" => self.poster = Some(p),
                        _ => {},
                    }
                }
            }

            // now things that can only be found in a subdir
            // because they need context.
            if let Some(season_hint) = season_hint {
                if let Some(s) = IS_IMAGE.captures(name) {
                    let p = join_paths(subdir, name);
                    match &s[1] {
                        "banner" =>{
                            let idx = self.get_season(season_hint);
                            self.seasons[idx].banner = Some(p);
                            continue;
                        },
                        "poster" => {
                            let idx = self.get_season(season_hint);
                            self.seasons[idx].poster = Some(p);
                            continue;
                        }
                        _ => {},
                    }
                }
            }

            // season image can be in main dir or subdir.
            if let Some(s) = IS_SEASON_IMG.captures(name) {
                let sn = s[1].parse::<u32>().unwrap_or(0);
                let idx = self.get_season(sn);
                let p = join_paths(subdir, name);
                match &s[2] {
                    "poster" => self.seasons[idx].poster = Some(p),
                    "banner" => self.seasons[idx].banner = Some(p),
                    _ => {
                        // probably a poster.
                        self.seasons[idx].poster = Some(p);
                    },
                }
                continue;
            }

            // episodes can be in main dir or subdir.
            let s = match IS_VIDEO.captures(name) {
                Some(s) => s,
                None => continue,
            };

            // Parse the episode filename for season and episode number etc.
            let ep_info = match EpisodeNameInfo::parse(&s[1], season_hint) {
                Some(ep_info) => ep_info,
                None => continue,
            };

            let p = join_paths(subdir, name);
            let video = match FileInfo::from_path(&p) {
                Ok(v) => sqlx::types::Json(v),
                Err(_) => continue,
            };
            let mut ep = Episode {
                video,
                ..Episode::default()
            };

            // Is it a double entry? (dup mp4s in different dirs)
            if let Some(epm) = ep_map.get(&s[1]) {

                // If it already has related files, dont overwrite.
                let ep = self.get_episode(epm.season_idx, epm.episode_idx);
                if ep.nfofile.is_some() || ep.thumb.0.len() > 0 {
                    continue;
                }

                // Or, if we are in the wrong dir, ignore.
                if Some(ep_info.seasonno) != season_hint {
                    continue;
                }

                // OK, replace.
                self.seasons[epm.season_idx].episodes.remove(epm.episode_idx);
                ep_map.remove(&s[1]);
            }

            // Add info from the filename.
            // XXX TODO we might want to read the NFO first then set title as backup
            ep.nfo_base.title = Some(ep_info.name);
            ep.season = ep_info.seasonno;
            ep.episode = ep_info.episodeno;
            // XXX TODO ep.double = ep_info.double;

            // Add this episode to the season.
            let season_idx = self.get_season(ep.season as u32);
            self.seasons[season_idx].episodes.push(ep);

            // And remember basename -> season_idx, episode_idx mapping.
            let episode_idx = self.seasons[season_idx].episodes.len() - 1;
            ep_map.insert(s[1].to_string(), EpMap { season_idx, episode_idx });
        }

        // Now scan the directory again for episode-related files.
        for entry in &entries {
            let file_name = entry.file_name();
            let name = match file_name.to_str() {
                Some(name) => name,
                None => continue,
            };

            // First try the (basename).(jpg|tbn|...) variant.
            let mut b = match IS_EXT1.captures(name) {
                Some(s) => {
                    let ep = ep_map.get(&s[1]);
                    ep.map(|ep| (ep.season_idx, ep.episode_idx, s[2].to_string(), s[3].to_string()))
                },
                None => None,
            };
            // Then try the (basename).(-poster).(jpg|tbn|...) variant.
            if b.is_none() {
                b = match IS_EXT2.captures(name) {
                    Some(s) => {
                        let ep = ep_map.get(&s[1]);
                        ep.map(|ep| (ep.season_idx, ep.episode_idx, s[2].to_string(), s[3].to_string()))
                    },
                    None => None,
                };
            }
            let (season_idx, episode_idx, mut aux, ext) = match b {
                Some(b) => b,
                None => continue,
            };

            let ep = self.get_episode_mut(season_idx, episode_idx);
            let p = join_paths(subdir, name);

            /* XXX FIXME Thumb
            if IS_IMAGE.is_match(&name) {
                if ext == "tbn" || aux == "thumb" {
                    ep.thumb = Some(p);
                }
                continue
            }*/

            /* // XXX FIXME subtitles.
            if ext == "srt" {
                if aux == "" || aux == "und" {
                    aux = "zz".to_string();
                }
                ep.srt_subs.push(Subs{
                    lang: aux,
                    path: p,
                });
                continue;
            }

            if ext == "vtt" {
                if aux == "" || aux == "und" {
                    aux = "zz".to_string();
                }
                ep.vtt_subs.push(Subs{
                    lang: aux,
                    path: p,
                });
                continue;
            }
            */

            /* XXX FIXME TODO
            if ext == "nfo" {
                let mut nfo_path = PathBuf::from(basedir);
                if let Some(s) = subdir {
                    nfo_path.push(s);
                }
                nfo_path.push(name);
                ep.nfo_path = Some(nfo_path);
                continue;
            }*/
        }
    }

    async fn build_show(coll: &Collection, dir: &str) -> Option<TVShow> {

        let tvshow = TVShow {
            collection_id: coll.collection_id as i64,
            directory: sqlx::types::Json(FileInfo::from_path(dir).ok()?),
            ..TVShow::default()
        };
        let mut item = Show {
            tvshow,
            ..Show::default()
        };

        let mut ep_map = HashMap::new();
        let showdir = format!("{}/{}", coll.directory, dir);
        item.show_scan_dir(&showdir, None, &mut ep_map, None).await;

        for season_idx in 0 .. item.seasons.len() {
            // remove episodes without video
            item.seasons[season_idx].episodes.retain(|e| e.video.0.path != "");

            // Then sort episodes.
            item.seasons[season_idx].episodes.sort_by_key(|e| e.episode);
        }

        // remove seasons without episodes
        item.seasons.retain(|s| s.episodes.len() > 0);

        // and sort.
        item.seasons.sort_by_key(|s| s.seasonno);

        // Timestamp of first and last video.
        if item.seasons.len() > 0 {
            let fs = &item.seasons[0];
            let ls = &item.seasons[item.seasons.len() - 1];
            // XXX FIXME firstvideo lastvideo
            // item.firstvideo = fs.episodes[0].video_ts;
            // item.lastvideo = ls.episodes[ls.episodes.len() - 1].video_ts;
        }

        // If we have an NFO and at least one image, accept it.
        let mut ok = false;
        if item.tvshow.nfofile.is_some() &&
           (item.fanart.is_some() || item.poster.is_some()) {
            ok = true;
        }

        // Or if there is at least one video, accept it as well.
        if item.seasons.iter().any(|s| s.episodes.len() > 0) {
            ok = true;
        }

        if !ok {
            return None;
        }

        // XXX FIXME TODO update show --> tvshow before returning.
        Some(item.tvshow)
    }
}

#[derive(Default, Debug)]
struct EpisodeNameInfo {
    name: String,
    seasonno: u32,
    episodeno: u32,
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

        // pattern: ___.s03e04.___
        const PAT1: &'static str = r#"^.*[ ._][sS]([0-9]+)[eE]([0-9]+)[ ._].*$"#;
        if let Some(caps) = regex!(PAT1).captures(name) {
            ep.name = format!("{}x{}", &caps[1], &caps[2]);
            ep.seasonno = caps[1].parse::<u32>().unwrap_or(0);
            ep.episodeno = caps[2].parse::<u32>().unwrap_or(0);
            return Some(ep);
        }

        // pattern: ___.s03e04e05.___ or ___.s03e04-e05.___
        const PAT2: &'static str = r#"^.*[. _][sS]([0-9]+)[eE]([0-9]+)-?[eE]([0-9]+)[. _].*$"#;
        if let Some(caps) = regex!(PAT2).captures(name) {
            ep.name = format!("{}x{}-{}", &caps[1], &caps[2], &caps[3]);
            ep.seasonno = caps[1].parse::<u32>().unwrap_or(0);
            ep.episodeno = caps[2].parse::<u32>().unwrap_or(0);
            ep.double = true;
            return Some(ep);
        }

        // pattern: ___.2015.03.08.___
        const PAT3: &'static str = r#"^.*[ .]([0-9]{4})[.-]([0-9]{2})[.-]([0-9]{2})[ .].*$"#;
        if let Some(caps) = regex!(PAT3).captures(name) {
            ep.name = format!("{}.{}.{}", &caps[1], &caps[2], &caps[3]);
            ep.seasonno = season_hint.unwrap_or(0);
            let joined = format!("{}{}{}", &caps[1], &caps[2], &caps[3]);
            ep.episodeno = joined.parse::<u32>().unwrap_or(0);
            return Some(ep);
        }

        // pattern: ___.308.___  (or 3x08) where first number is season.
        const PAT4: &'static str = r#"^.*[ .]([0-9]{1,2})x?([0-9]{2})[ .].*$"#;
        if let Some(caps) = regex!(PAT4).captures(name) {
            if let Ok(sn) = caps[1].parse::<u32>() {
                if season_hint.is_none() || season_hint == Some(sn) {
                    ep.name = format!("{:02}x{}", sn, &caps[2]);
                    ep.seasonno = sn;
                    ep.episodeno = caps[2].parse::<u32>().unwrap_or(0);
                    return Some(ep);
                }
            }
        }

        None
    }
}

fn join_paths(dir: Option<&str>, file: &str) -> String {
    match dir {
        Some(dir) if dir != "" && dir != "." && dir != "./" => {
            format!("{}/{}", dir, file)
        },
        Some(_) | None => file.to_string(),
    }
}
