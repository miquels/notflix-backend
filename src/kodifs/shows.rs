use std::collections::HashMap;
// use std::time::Duration;

use tokio::fs;

use crate::collections::Collection;
use crate::models::{TVShow, Season, Thumb, Episode, FileInfo};
use super::*;

#[derive(Debug)]
struct EpMap {
    season_idx: usize,
    episode_idx: usize,
}

pub async fn build_shows(_coll: &Collection, _pace: u32) {
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

#[derive(serde::Serialize, Default, Debug)]
pub struct Show {
    tvshow:  TVShow,
    seasons: Vec<Season>,
}

impl Show {

    // Look up season by season. If not present, create,
    fn get_season(&mut self, season: u32) -> usize {
        // find the season.
        for idx in 0 .. self.seasons.len() {
            if season == self.seasons[idx].season {
                return idx;
            }
        }

        // not present yet, so add.
        let sn = Season { season, ..Season::default() };
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

            // Get filename of this entry. Skip non-utf8 and dotfiles.
            let file_name = entry.file_name();
            let name = match file_name.to_str() {
                Some(name) => name,
                None => continue,
            };
            if name.starts_with(".") || name.starts_with("+ ") {
                continue;
            }

            // first things that can only be found in the shows basedir, not in subdirs.
            if season_hint.is_none() {

                // S* subdir.
                if let Some(s) = IS_SHOW_SUBDIR.captures(name) {
                    let sn = s[1].parse::<u32>().unwrap();
                    self.show_scan_dir(basedir, Some(name), ep_map, Some(sn)).await;
                    continue;
                }

                // nfo file.
                if name == "tvshow.nfo" {
                    match FileInfo::open(basedir, subdir, name).await {
                        Ok((mut file, nfofile)) => {
                            if let Ok(nfo) = Nfo::read(&mut file).await {
                                let mut nfo_base = nfo.to_nfo_base();
                                if nfo_base.title.is_none() {
                                    std::mem::swap(&mut self.tvshow.nfo_base.title, &mut nfo_base.title);
                                }
                                self.tvshow.nfo_base = nfo_base;
                                self.tvshow.nfofile = Some(sqlx::types::Json(nfofile));
                            }
                        },
                        Err(_) => {},
                    }
                    continue;
                }

                // other images.
                if let Some(s) = IS_IMAGE.captures(name) {
                    let base = &s[1];
                    let (season, aspect) = match base {
                        "season-all-banner" => (Some("all"), "banner"),
                        "season-all-poster" => (Some("all"), "poster"),
                        "season-specials-banner" => (Some("specials"), "banner"),
                        "season-specials-poster" => (Some("specials"), "poster"),
                        "banner" | "fanart" | "folder" | "poster" => (None, base),
                        _ => (None, ""),
                    };
                    if aspect != "" {
                        add_thumb(&mut self.tvshow.thumbs, "", subdir, name, aspect, season);
                        continue;
                    }
                }
            }

            // now things that can only be found in a subdir because they need context.
            // For example "poster.jpg" is a generic poster in the main directory,
            // and a season poster in the S01/ subdirectory.
            if let Some(season) = season_hint {
                if let Some(s) = IS_IMAGE.captures(name) {
                    let a = &s[1];
                    if a == "banner" || a == "fanart" || a == "folder" || a == "poster" {
                        let season = format!("{}", season);
                        let season = Some(season.as_str());
                        add_thumb(&mut self.tvshow.thumbs, "", subdir, name, a, season);
                        continue;
                    }
                }
            }

            // season image (season01-poster.jpg, etc) can be in main dir or subdir.
            if let Some(s) = IS_SEASON_IMG.captures(name) {
                let aspect = match &s[2] {
                    "banner" | "poster" => &s[2],
                    _ => "poster",
                };
                add_thumb(&mut self.tvshow.thumbs, "", subdir, name, aspect, Some(&s[1]));
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

            let video = match FileInfo::from_path(basedir, subdir, name).await {
                Ok(v) => sqlx::types::Json(v),
                Err(_) => continue,
            };
            let mut ep = Episode {
                video,
                ..Episode::default()
            };

            // Is it a duplicate entry? (mp4s for same season/episode in different dirs)
            if let Some(epm) = ep_map.get(&s[1]) {

                // If it already has related files, dont overwrite.
                let ep = self.get_episode(epm.season_idx, epm.episode_idx);
                if ep.nfofile.is_some() || ep.thumbs.0.len() > 0 {
                    continue;
                }

                // Or, if we are in the wrong dir, ignore.
                if Some(ep_info.season) != season_hint {
                    continue;
                }

                // OK, replace.
                self.seasons[epm.season_idx].episodes.remove(epm.episode_idx);
                ep_map.remove(&s[1]);
            }

            // Add info from the filename.
            ep.nfo_base.title = Some(ep_info.name);
            ep.season = ep_info.season;
            ep.episode = ep_info.episode;
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
            let (season_idx, episode_idx, aux, ext) = match b {
                Some(b) => b,
                None => continue,
            };
            let ep = self.get_episode_mut(season_idx, episode_idx);

            // thumb: <base>.tbn or <base>-thumb.ext
            if IS_IMAGE.is_match(&name) {
                if ext == "tbn" || aux == "thumb" {
                    add_thumb(& mut ep.thumbs, "", subdir, name, "thumb", None);
                }
                continue;
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

            if ext == "nfo" {
                match FileInfo::open(basedir, subdir, name).await {
                    Ok((mut file, nfofile)) => {
                        if let Ok(nfo) = Nfo::read(&mut file).await {
                            let mut nfo_base = nfo.to_nfo_base();
                            if nfo_base.title.is_none() {
                                std::mem::swap(&mut ep.nfo_base.title, &mut nfo_base.title);
                            }
                            ep.nfo_base = nfo_base;
                        }
                        ep.nfofile = Some(sqlx::types::Json(nfofile));
                    },
                    Err(_) => {},
                }
                continue;
            }
        }
    }

    async fn build_show(coll: &Collection, dir: &str) -> Option<TVShow> {

        let fileinfo = FileInfo::from_path(&coll.directory, "", dir).await.ok()?;
        let tvshow = TVShow {
            directory: sqlx::types::Json(fileinfo),
            ..TVShow::default()
        };
        let mut item = Show {
            tvshow,
            ..Show::default()
        };

        let showdir = join_paths(Some(&coll.directory), dir);
        let mut ep_map = HashMap::new();
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
        item.seasons.sort_by_key(|s| s.season);

        // Timestamp of first and last video.
        /*
        if item.seasons.len() > 0 {
            // XXX FIXME firstvideo lastvideo
            let fs = &item.seasons[0];
            let ls = &item.seasons[item.seasons.len() - 1];
            item.firstvideo = fs.episodes[0].video_ts;
            item.lastvideo = ls.episodes[ls.episodes.len() - 1].video_ts;
        }*/

        // println!("{:#?}", item);

        // If we have an NFO and at least one image, accept it.
        let mut ok = false;
        if item.tvshow.nfofile.is_some() && item.tvshow.thumbs.0.len() > 0 {
            ok = true;
        }

        // Or if there is at least one video, accept it as well.
        if item.seasons.iter().any(|s| s.episodes.len() > 0) {
            ok = true;
        }

        if !ok {
            return None;
        }

        let Show { mut tvshow, seasons } = item;
        tvshow.seasons = seasons;
        Some(tvshow)
    }
}

fn add_thumb(thumbs: &mut sqlx::types::Json<Vec<Thumb>>, _dir: &str, subdir: Option<&str>, name: &str, aspect: &str, season: Option<&str>) {
    let season = season.map(|mut s| {
        while s.len() > 1 && s.starts_with("0") {
            s = &s[1..];
        }
        s.to_string()
    });

    thumbs.0.push(Thumb {
        path: join_paths(subdir, name),
        aspect: aspect.to_string(),
        season: season.map(|s| s.to_string()),
    });
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

        // pattern: ___.s03e04.___
        const PAT1: &'static str = r#"^.*[ ._][sS]([0-9]+)[eE]([0-9]+)[ ._].*$"#;
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

fn join_paths(dir: Option<&str>, file: &str) -> String {
    match dir {
        Some(dir) if dir != "" && dir != "." && dir != "./" => {
            format!("{}/{}", dir, file)
        },
        Some(_) | None => file.to_string(),
    }
}
