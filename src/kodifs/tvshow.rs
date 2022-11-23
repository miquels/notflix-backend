use std::collections::HashMap;
use std::time::SystemTime;
use std::io;

use chrono::TimeZone;

use crate::collections::Collection;
use crate::models::{self, Thumb, TVShow, Season, FileInfo};
use crate::util::{Id, SystemTimeToUnixTime};
use super::episode::Episode;
use super::*;

pub async fn scan_tvshow_dir(coll: &Collection, name: &str, db_tvshow: Option<Box<TVShow>>, nfo_only: bool) -> Option<Box<TVShow>> {
    log::trace!("scan_tvshow_dir {}", name);
    Show::build_show(coll, name, db_tvshow, nfo_only).await
}

#[derive(Debug)]
struct EpMap {
    season_idx: usize,
    episode_idx: usize,
}

#[derive(Default, Debug)]
pub struct Show {
    ep_map:  HashMap<String, EpMap>,
    pub basedir: String,
    pub tvshow:  Box<TVShow>,
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

    pub fn get_episode_mut(&mut self, basepath: &str) -> Option<&'_ mut models::Episode> {
        self.ep_map.get(basepath).map(|em| {
            &mut self.tvshow.seasons[em.season_idx].episodes[em.episode_idx]
        })
    }

    async fn show_read_nfo(&mut self) -> bool {
        let (mut file, nfofile) = match FileInfo::open(&self.basedir, "tvshow.nfo").await {
            Ok(x) => (x.0, Some(x.1)),
            Err(_) => return false,
        };
        if self.tvshow.nfofile == nfofile {
            // No change.
            return true;
        }
        if let Ok(nfo) = Nfo::read(&mut file).await {
            nfo.update_tvshow(&mut self.tvshow);
            self.tvshow.nfofile = nfofile;
            return true;
        }
        false
    }

    async fn show_scan_dir(&mut self, coll: &Collection) -> io::Result<()> {

        // Get all the files up front.
        let mut episodes = Vec::new();
        let mut entries = Vec::new();
        let (oldest, newest) = scandirs::read_dir(&self.basedir, true, &mut entries, true).await?;

        self.set_lastmodified(newest);
        if self.tvshow.dateadded.is_none() {
            if let chrono::LocalResult::Single(c) = chrono::Local.timestamp_millis_opt(oldest) {
                self.tvshow.dateadded = Some(c.format("%Y-%m-%d").to_string());
            }
        }

        // First loop: find tvshow and season images, tvshow.nfo file, episode video files.
        for name in &entries {

            // first things that can only be found in the shows basedir, not in subdirs.
            if !name.contains("/") {

                // nfo file.
                if name == "tvshow.nfo" {
                    self.show_read_nfo().await;
                    continue;
                }

                // Show / season images.
                if self.tvshow_image_file(coll, name).await {
                    continue;
                }
            }

            // Parse the name, see if it's a video, extract information.
            let caps = IS_VIDEO.captures(name);
            let (basepath, hint) = match caps.as_ref() {
                Some(caps) => (&caps[1], caps.get(2)),
                None => continue,
            };
            let season_hint = hint.and_then(|season| season.as_str().parse::<u32>().ok());
            let showdir = self.basedir.clone();
            let db_episode = self.get_episode_mut(basepath);

            if let Some(mut ep) = Episode::new(coll, showdir, name, basepath, season_hint, db_episode).await {
                ep.episode.tvshow_id = self.tvshow.id;
                ep.episode.collection_id = coll.collection_id as i64;
                episodes.push(ep);
            }
        }
        episodes.sort_by(|a, b| a.basepath.partial_cmp(&b.basepath).unwrap());

        let mut idx = 0;
        for ep in &mut episodes {
            let files = extract_prefixed(&mut entries, &mut idx, &ep.basepath);
            ep.files = files;
        }

        // We have all episodes now, process the files in them.
        for ep in episodes.drain(..) {
            if let Some(ep) = ep.finalize().await {
                let season_idx = self.get_season(ep.season as u32);
                self.seasons[season_idx].episodes.push(ep);
            }
        }

        Ok(())
    }

    // Check for:
    // - tvshow images (banner, fanart, poster etc)
    // - season images (season01-poster.jpg, season-all-poster.jpg, etc)
    async fn tvshow_image_file(&mut self, coll: &Collection, name: &str) -> bool {

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
                let season = season.map(|s| s.to_string());
                if aspect != "" {
                    let _ = Thumb::add(&mut self.tvshow.thumbs, &self.basedir, name, coll, self.tvshow.id, aspect, season).await;
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
            let _ = Thumb::add(&mut self.tvshow.thumbs, &self.basedir, name, coll, self.tvshow.id, aspect, Some(caps[1].to_string()));
            return true;
        }

        false
    }

    async fn build_show(coll: &Collection, dirname: &str, db_tvshow: Option<Box<TVShow>>, nfo_only: bool) -> Option<Box<TVShow>> {

        let fileinfo = FileInfo::from_path(&coll.directory, dirname).await.ok()?;
        let basedir = fileinfo.fullpath.clone();

        let tvshow = db_tvshow.unwrap_or_else(|| Box::new(TVShow {
            id: Id::new(),
            collection_id: coll.collection_id as i64,
            lastmodified: SystemTime::now().unixtime_ms(),
            ..TVShow::default()
        }));

        let mut item = Show {
            basedir,
            tvshow,
            ..Show::default()
        };

        // If the directory name changed, we need to update the db.
        if item.tvshow.directory.path != dirname {
            if item.tvshow.directory.path != "" {
                log::debug!("tvshow::scan_tvshow_dir: directory rename {} -> {}", item.tvshow.directory.path, dirname);
            }
            item.set_lastmodified(0);
        }
        item.tvshow.directory = fileinfo;

        if nfo_only {
            return item.show_read_nfo().await.then(|| item.tvshow);
        }

        // Index the episodes from the database's TVShow.
        // Mark all episodes as deleted initially. During scanning
        // we'll unmark or move the stuff we retain.
        // Whatever is left must be deleted from the db by the caller.
        for season_idx in 0 .. item.tvshow.seasons.len() {
            let season = &mut item.tvshow.seasons[season_idx];
            for episode_idx in 0 .. season.episodes.len() {
                let episode = &mut season.episodes[episode_idx];
                episode.deleted = true;
                if let Some(caps) = IS_VIDEO.captures(&episode.video.path) {
                    item.ep_map.insert(caps[1].to_string(), EpMap { season_idx, episode_idx });
                }
            }
        }

        // Scan the show's directory.
        item.show_scan_dir(coll).await.ok()?;

        // remove episodes without video, then sort.
        for season_idx in 0 .. item.seasons.len() {
            item.seasons[season_idx].episodes.retain(|e| e.video.path != "" && !e.deleted);
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
        if item.tvshow.nfofile.is_some() && item.tvshow.thumbs.len() > 0 {
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

    pub fn set_lastmodified(&mut self, ts: i64) {
        if ts == 0 {
            self.tvshow.lastmodified = SystemTime::now().unixtime_ms();
            return;
        }
        if self.tvshow.lastmodified < ts || self.tvshow.lastmodified == 0 {
            self.tvshow.lastmodified = ts;
        }
    }
}
