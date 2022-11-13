use std::collections::HashMap;
// use std::time::Duration;

use tokio::fs;

use crate::collections::Collection;
use crate::models::{self, TVShow, Season, FileInfo};
use super::*;

pub async fn build_show(coll: &Collection, name: &str) -> Option<TVShow> {
    Show::build_show(coll, name, &mut TVShow::default(), false).await
}

#[derive(Debug)]
struct EpMap {
    season_idx: usize,
    episode_idx: usize,
}

#[derive(Default, Debug)]
pub struct Show {
    ep_map:  HashMap<String, EpMap>,
    db_ep_map:  HashMap<String, EpMap>,
    pub basedir: String,
    pub tvshow:  TVShow,
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

    fn get_db_episode<'a>(&self, db_tvshow: &'a TVShow, basepath: &str) -> Option<&'a models::Episode> {
        self.db_ep_map.get(basepath).map(|em| {
            &db_tvshow.seasons[em.season_idx].episodes[em.episode_idx]
        })
    }

    pub fn get_db_episode_mut<'a>(&self, db_tvshow: &'a mut TVShow, basepath: &str) -> Option<&'a mut models::Episode> {
        self.db_ep_map.get(basepath).map(|em| {
            &mut db_tvshow.seasons[em.season_idx].episodes[em.episode_idx]
        })
    }

    fn get_episode_mut(&mut self, season_idx: usize, episode_idx: usize) -> &mut models::Episode {
        &mut self.seasons[season_idx].episodes[episode_idx]
    }

    // Scan a directory recursively (max 1 subdir deep).
    #[async_recursion::async_recursion]
    async fn scan_dir<'a: 'async_recursion>(&self, subdir: Option<&'a str>, names: &mut Vec<String>) {

        // Read the entire directory in one go.
        let dir = match subdir {
            Some(subdir) => format!("{}/{}", &self.basedir, subdir),
            None => self.basedir.clone(),
        };
        let subdir = subdir.map(|s| format!("{}/", s));

        let mut d = match fs::read_dir(&dir).await {
            Ok(d) => d,
            Err(_) => return,
        };

        while let Ok(Some(entry)) = d.next_entry().await {
            if let Ok(mut name) = entry.file_name().into_string() {
                if name.starts_with(".") || name.starts_with("+ ") {
                    continue;
                }
                if let Ok(t) = entry.file_type().await {
                    if t.is_dir() {
                        if !subdir.is_some() {
                            self.scan_dir(Some(&name), names).await;
                        }
                        continue;
                    }
                }
                if let Some(s) = subdir.as_ref() {
                    name.insert_str(0, s);
                }
                names.push(name);
            }
        }
    }

    async fn show_read_nfo(&mut self, db_tvshow: &TVShow) -> bool {
        let (mut file, nfofile) = match FileInfo::open(&self.basedir, None, "tvshow.nfo").await {
            Ok(x) => (x.0, Some(sqlx::types::Json(x.1))),
            Err(_) => return false,
        };
        if db_tvshow.nfofile == nfofile {
            // No change, so we can copy the data.
            self.tvshow.copy_nfo_from(db_tvshow);
            return true;
        }
        if let Ok(nfo) = Nfo::read(&mut file).await {
            nfo.update_tvshow(&mut self.tvshow);
            self.tvshow.nfofile = nfofile;
            return true;
        }
        false
    }

    async fn show_scan_dir(&mut self, db_tvshow: &mut TVShow) {

        // Get all the files up front.
        let mut entries = Vec::new();
        self.scan_dir(None, &mut entries).await;

        // First loop: find tvshow and season images, tvshow.nfo file, episode video files.
        for name in &entries {

            // first things that can only be found in the shows basedir, not in subdirs.
            if !name.contains("/") {

                // nfo file.
                if name == "tvshow.nfo" {
                    self.show_read_nfo(db_tvshow).await;
                    continue;
                }

                // Show / season images.
                if self.tvshow_image_file(name, db_tvshow).await {
                    continue;
                }
            }

            // episodes can be in main dir or subdir.
            self.episode_video_file(name, db_tvshow).await;
        }

        // Now scan the directory again for episode-related files.
        for name in entries.drain(..) {
            self.add_related_file(name, db_tvshow).await;
        }
    }

    // Check for:
    // - tvshow images (banner, fanart, poster etc)
    // - season images (season01-poster.jpg, season-all-poster.jpg, etc)
    async fn tvshow_image_file(&mut self, name: &str, _db_tvshow: &mut TVShow) -> bool {

        // Images that can only be found in the base directory.
        if !name.contains("/") {
            if let Some(caps) = IS_IMAGE.captures(name) {
                let (season, aspect) = match &caps[1] {
                    "season-all-banner" => (Some("all"), "banner"),
                    "season-all-poster" => (Some("all"), "poster"),
                    "season-specials-banner" => (Some("specials"), "banner"),
                    "season-specials-poster" => (Some("specials"), "poster"),
                    "banner" | "fanart" | "folder" | "poster" => (None, &caps[1]),
                    _ => (None, ""),
                };
                if aspect != "" {
                    add_thumb(&mut self.tvshow.thumbs, "", name, aspect, season);
                    return true;
                }
            }
        }

        // season image (season01-poster.jpg, etc) can be in main dir or subdir.
        if let Some(caps) = IS_SEASON_IMG.captures(name) {
            let aspect = match &caps[2] {
                "banner" | "poster" => &caps[2],
                _ => "poster",
            };
            add_thumb(&mut self.tvshow.thumbs, "", name, aspect, Some(&caps[1]));
            return true;
        }

        false
    }

    async fn episode_video_file(&mut self, name: &str, db_tvshow: &mut TVShow) {

        // Parse the name, see if it's a video, extract information.
        let caps = IS_VIDEO.captures(name);
        let (basepath, hint, basename, _ext) = match caps.as_ref() {
            Some(caps) => (&caps[1], &caps[2], &caps[3], &caps[4]),
            None => return,
        };
        let season_hint = hint.parse::<u32>().ok();

        // Parse the episode filename for season and episode number etc.
        let ep_info = match EpisodeNameInfo::parse(basename, season_hint) {
            Some(ep_info) => ep_info,
            None => return,
        };

        // Must be able to open it.
        let video = match FileInfo::from_path(&self.basedir, None, name).await {
            Ok(v) => sqlx::types::Json(v),
            Err(_) => return,
        };

        // If this episode was already in the database, copy its ID.
        // We also _un_mark it as deleted.
        let id = match self.get_db_episode_mut(db_tvshow, basepath) {
            Some(ep) => {
                ep.deleted = false;
                ep.id
            },
            None => 0,
        };

        let mut ep = models::Episode {
            id,
            video,
            ..models::Episode::default()
        };

        // Add info from the filename.
        ep.nfo_base.title = Some(ep_info.name);
        ep.season = ep_info.season;
        ep.episode = ep_info.episode;
        // XXX TODO ep.double = ep_info.double;

        // Add this episode to the season.
        let season_idx = self.get_season(ep.season as u32);
        self.seasons[season_idx].episodes.push(ep);

        // And remember basepath -> season_idx, episode_idx mapping.
        let episode_idx = self.seasons[season_idx].episodes.len() - 1;
        self.ep_map.insert(basepath.to_string(), EpMap { season_idx, episode_idx });
    }

    // See if this is a file that is related to an episode, by
    // indexing on the basepath.
    async fn add_related_file(&mut self, name: String, db_tvshow: &TVShow) {

        // Split the filename into basename, aux, and extension.
        // tvshow.s01e01(-poster).jpg
        // -- base------ --aux--- --ext--
        //
        // First try matching without the -aux part.
        let caps1 = IS_EXT1.captures(&name);
        let mut caps2 = None;
        let b = caps1.as_ref().and_then(|s| {
            let ep = self.ep_map.get(&s[1]);
            ep.map(move |ep| (ep.season_idx, ep.episode_idx, &s[1], &s[2], &s[2]))
        }).or_else(|| {
            // Then try matching with the -aux part.
            caps2 = IS_EXT2.captures(&name);
            caps2.as_ref().and_then(|s| {
                let ep = self.ep_map.get(&s[1]);
                ep.map(move |ep| (ep.season_idx, ep.episode_idx, &s[1], &s[2], &s[3]))
            })
        });

        // Only continue if we did have a match.
        let (season_idx, episode_idx, basepath, aux, ext) = match b {
            Some(b) => b,
            None => return,
        };
        // See if the old TVShow from the database has a similar episode.
        let db_episode = self.get_db_episode(db_tvshow, &basepath);

        let basedir = self.basedir.clone();
        let ep = self.get_episode_mut(season_idx, episode_idx);

        // Thumb: <base>.tbn or <base>-thumb.ext
        if IS_IMAGE.is_match(&name) {
            if ext == "tbn" || aux == "thumb" {
                add_thumb(&mut ep.thumbs, "", name, "thumb", None);
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
            match FileInfo::open(&basedir, None, &name).await {
                Ok((mut file, nfofile)) => {
                    let mut nfofile = Some(sqlx::types::Json(nfofile));

                    if let Some(db_ep) = db_episode {
                        if db_ep.nfofile == nfofile {
                            // No change, so we can copy the data.
                            ep.copy_nfo_from(db_ep);
                            nfofile = None;
                        }
                    }

                    if nfofile.is_some() {
                        if let Ok(nfo) = Nfo::read(&mut file).await {
                            nfo.update_episode(ep);
                            ep.nfofile = nfofile;
                        }
                    }
                },
                Err(_) => {},
            }
        }
    }

    async fn build_show(coll: &Collection, dir: &str, db_tvshow: &mut TVShow, nfo_only: bool) -> Option<TVShow> {

        let fileinfo = FileInfo::from_path(&coll.directory, "", dir).await.ok()?;
        let tvshow = TVShow {
            directory: sqlx::types::Json(fileinfo),
            collection_id: coll.collection_id as i64,
            ..TVShow::default()
        };
        let mut item = Show {
            basedir: join_paths(Some(&coll.directory), dir),
            tvshow,
            ..Show::default()
        };

        if nfo_only {
            return item.show_read_nfo(db_tvshow).await.then(|| item.tvshow);
        }

        // Index the episodes from the database's TVShow.
        // Mark all episodes as deleted initially. During scanning
        // we'll unmark or move the stuff we retain.
        // Whatever is left must be deleted from the db by the caller.
        for season_idx in 0 .. db_tvshow.seasons.len() {
            let season = &mut db_tvshow.seasons[season_idx];
            for episode_idx in 0 .. season.episodes.len() {
                let episode = &mut season.episodes[episode_idx];
                episode.deleted = true;
                if let Some(caps) = IS_VIDEO.captures(&episode.video.path) {
                    item.db_ep_map.insert(caps[1].to_string(), EpMap { season_idx, episode_idx });
                }
            }
        }

        // Scan the show's directory.
        item.show_scan_dir(db_tvshow).await;

        // remove episodes without video, then sort.
        for season_idx in 0 .. item.seasons.len() {
            item.seasons[season_idx].episodes.retain(|e| e.video.0.path != "" && !e.deleted);
            item.seasons[season_idx].episodes.sort_by_key(|e| e.episode);
        }

        // remove seasons without episodes, then sort.
        item.seasons.retain(|s| s.episodes.len() > 0);
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

        let Show { mut tvshow, seasons, .. } = item;
        tvshow.seasons = seasons;
        Some(tvshow)
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

fn join_paths(dir: Option<&str>, file: &str) -> String {
    match dir {
        Some(dir) if dir != "" && dir != "." && dir != "./" => {
            format!("{}/{}", dir, file)
        },
        Some(_) | None => file.to_string(),
    }
}
