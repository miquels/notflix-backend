use std::collections::HashMap;
// use std::time::Duration;

use crate::collections::Collection;
use crate::models::{self, TVShow, Season, FileInfo};
use super::*;

pub async fn scan_tvshow_dir(coll: &Collection, name: &str, db_tvshow: Option<Box<TVShow>>, nfo_only: bool) -> Option<Box<TVShow>> {
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
            Ok(x) => (x.0, Some(sqlx::types::Json(x.1))),
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

    async fn show_scan_dir(&mut self) {

        // Get all the files up front.
        let mut entries = Vec::new();
        scandirs::read_dir(&self.basedir, true, &mut entries).await;

        let mut episodes = Vec::new();

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
                if self.tvshow_image_file(name).await {
                    continue;
                }
            }

            // Parse the name, see if it's a video, extract information.
            let caps = IS_VIDEO.captures(name);
            let (basepath, hint) = match caps.as_ref() {
                Some(caps) => (&caps[1], &caps[2]),
                None => continue,
            };
            let season_hint = hint.parse::<u32>().ok();
            let showdir = self.basedir.clone();
            let db_episode = self.get_episode_mut(basepath);

            if let Some(ep) = Episode::new(showdir, name, basepath, season_hint, db_episode).await {
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
            let ep = ep.finalize().await;
            let season_idx = self.get_season(ep.season as u32);
            self.seasons[season_idx].episodes.push(ep);
        }
    }

    // Check for:
    // - tvshow images (banner, fanart, poster etc)
    // - season images (season01-poster.jpg, season-all-poster.jpg, etc)
    async fn tvshow_image_file(&mut self, name: &str) -> bool {

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

    async fn build_show(coll: &Collection, dir: &str, db_tvshow: Option<Box<TVShow>>, nfo_only: bool) -> Option<Box<TVShow>> {

        let fileinfo = FileInfo::from_path(&coll.directory, dir).await.ok()?;
        let mut tvshow = db_tvshow.unwrap_or_else(|| Box::new(TVShow {
            collection_id: coll.collection_id as i64,
            ..TVShow::default()
        }));
        tvshow.directory = sqlx::types::Json(fileinfo);

        let mut item = Show {
            basedir: format!("{}/{}", coll.directory, dir),
            tvshow,
            ..Show::default()
        };

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
        item.show_scan_dir().await;

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
