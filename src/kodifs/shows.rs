use std::collections::HashMap;
use std::time::Duration;
use std::path::PathBuf;
use tokio::fs;

use crate::collections::*;
use super::*;

struct EpMap {
    season_idx: usize,
    episode_idx: usize,
}

pub async fn build_shows(coll: &mut Collection, pace: u32) {

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
            items.push(m);
        }
        if pace > 0 {
            tokio::time::sleep(Duration::from_secs(pace as u64)).await;
        }
    }
    // XXX FIXME: merge strategy:
    // - use unique_id
    // - if show is gone, mark as deleted, don't really delete
    //   (that will allow it to be restored later)
    coll.items = items;
}

pub async fn build_show(coll: &Collection, name: &str) -> Option<Item> {
    Item::build_show(coll, name).await
}

impl Item {

    // Look up season by seasonno. If not present, create,
    fn get_season(&mut self, seasonno: u32) -> usize {
        // find the season.
        for idx in 0 .. self.seasons.len() {
            if seasonno == self.seasons[idx].seasonno {
                return idx;
            }
        }

        // not present yet. figure out where to insert a new season with this seasonno.
        let mut idx = 0;
        while idx < self.seasons.len() {
            if seasonno < self.seasons[idx].seasonno {
                break;
            }
            idx += 1;
        }

        // insert the fresh season at 'idx'.
        let sn = Season { seasonno, ..Season::default() };
        self.seasons.insert(idx, sn);

        idx
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
                if name == "tvshow.nfo" {
                    let mut nfo_path = PathBuf::from(&dir);
                    nfo_path.push(name);
                    self.nfo_path = Some(nfo_path);
                    continue;
                }

                // other images.
                if let Some(s) = IS_IMAGE.captures(name) {
                    let p = escape_path(name);
                    match &s[1] {
                        "season-all-banner" => self.season_all_banner = Some(p),
                        "season-all-poster" => self.season_all_poster = Some(p),
                        "banner" => self.banner = Some(p),
                        "fanart" => self.fanart = Some(p),
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
                    let p = join_and_escape_path(subdir, name);
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
                let p = join_and_escape_path(subdir, name);
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
            if let Some(s) = IS_VIDEO.captures(name) {
                let mut ep = Episode {
                    video: join_and_escape_path(subdir, name),
                    basename: s[1].to_string(),
                    ..Episode::default()
                };
                ep.video_ts = match entry.metadata().await {
                    Ok(m) => systemtime_to_ms(m.modified().unwrap()),
                    Err(_) => 0,
                };

                if let Some(ep_info) = EpisodeNameInfo::parse(&s[1], season_hint) {

                    // Add info from the filename.
                    ep.name = ep_info.name;
                    ep.seasonno = ep_info.seasonno;
                    ep.episodeno = ep_info.episodeno;
                    ep.double = ep_info.double;

                    // Add this episode to the season.
                    let season_idx = self.get_season(ep.seasonno);
                    self.seasons[season_idx].episodes.push(ep);

                    // And remember basename -> season_idx, episode_idx mapping.
                    let episode_idx = self.seasons[season_idx].episodes.len() - 1;
                    ep_map.insert(s[1].to_string(), EpMap { season_idx, episode_idx });
                }
            }
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

            let p = join_and_escape_path(subdir, name);
            let ep = &mut self.seasons[season_idx].episodes[episode_idx];

            if IS_IMAGE.is_match(&ext) {
                if ext == "tbn" || aux == "thumb" {
                    ep.thumb = Some(p);
                }
                continue
            }

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

            if ext == "nfo" {
                let mut nfo_path = PathBuf::from(basedir);
                if let Some(s) = subdir {
                    nfo_path.push(s);
                }
                nfo_path.push(name);
                ep.nfo_path = Some(nfo_path);
                continue;
            }
        }
    }

    async fn build_show(coll: &Collection, dir: &str) -> Option<Item> {

        let mut item = Item {
            name: dir.split('/').last().unwrap().to_string(),
            baseurl: coll.baseurl.clone(),
            path: escape_path(dir),
            type_: "show",
            ..Item::default()
        };
        let mut ep_map = HashMap::new();
        let showdir = format!("{}/{}", coll.directory, dir);
        item.show_scan_dir(&showdir, None, &mut ep_map, None).await;

        for season_idx in 0 .. item.seasons.len() {
            // remove episodes without video
            item.seasons[season_idx].episodes.retain(|e| e.video != "");

            // Then sort episodes.
            item.seasons[season_idx].episodes.sort_by_key(|e| e.episodeno);
        }

        // remove seasons without episodes
        item.seasons.retain(|s| s.episodes.len() > 0);

        // and sort.
        item.seasons.sort_by_key(|s| s.seasonno);

        // Timestamp of first and last video.
        if item.seasons.len() > 0 {
            let fs = &item.seasons[0];
            let ls = &item.seasons[item.seasons.len() - 1];
            item.firstvideo = fs.episodes[0].video_ts;
            item.lastvideo = ls.episodes[ls.episodes.len() - 1].video_ts;
        }

        // If we have an NFO and at least one image, accept it.
        let mut ok = false;
        if item.nfo_path.is_some() &&
           (item.fanart.is_some() || item.poster.is_some() || item.thumb.is_some()) {
            ok = true;
        }

        // Or if there is at least one video, accept it as well.
        if item.seasons.iter().any(|s| s.episodes.len() > 0) {
            ok = true;
        }

        if !ok {
            return None;
        }

        // XXX
        // db_load_item(coll, item);

        Some(item)
    }
}

#[derive(Default)]
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
        const PAT2: &'static str = r#"^.*[. _[sS]([0-9]+)[eE]([0-9]+)-?[eE]([0-9]+)[. _].*$"#;
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

