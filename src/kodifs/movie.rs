use chrono::TimeZone;
use std::time::SystemTime;

use super::*;
use crate::collections::*;
use crate::models::{FileInfo, MediaItem, Thumb};
use crate::util::{Id, SystemTimeToUnixTime};
use super::resource::{ItemType, MediaData};

pub async fn scan_movie_dir(
    coll: &Collection,
    mut dirname: &str,
    dbent: Option<Box<MediaItem>>,
    only_nfo: bool,
) -> Option<Box<MediaItem>> {

    // First get all directory entries.
    dirname = dirname.trim_end_matches('/');
    let dirinfo = FileInfo::from_path(&coll.directory, dirname).await.ok()?;
    let dirpath = dirinfo.fullpath.clone();
    let mut entries = Vec::new();
    let (oldest, newest) = scandirs::read_dir(&dirpath, false, &mut entries, true).await.ok()?;

    // Skip tvshow directories.
    if entries.iter().any(|s| s == "tvshow.nfo") {
        return None;
    }

    // Must have an mp4 file - get the basename
    let basename = match entries.iter().find(|v| v.ends_with(".mp4")) {
        Some(file) => file.strip_suffix(".mp4").unwrap(),
        None => return None,
    };

    // Initial Movie.
    let mut movie = dbent.unwrap_or_else(|| {
        Box::new(MediaItem {
            id: Id::new(),
            collection_id: coll.collection_id,
            ..MediaItem::default()
        })
    });
    movie.lastmodified = newest;
    if movie.dateadded == "" {
        if let chrono::LocalResult::Single(c) = chrono::Local.timestamp_millis_opt(oldest) {
            movie.dateadded = c.format("%Y-%m-%d").to_string();
        }
    }

    // If the directory name changed, we need to update the db.
    if let Some(fileinfo) = movie.directory.as_ref() {
        if fileinfo.path != dirname {
            log::debug!(
                "kodifs::scan_movie_dir: directory rename {} -> {}",
                fileinfo.path,
                dirname
            );
            movie.lastmodified = SystemTime::now().unixtime_ms();
        }
    }
    movie.directory = Some(dirinfo);

    // If the directory name ends with <space>(YYYY) then it's a year.
    // Remember that year as backup in case there's no NFO file.
    let (title, year) = match IS_YEAR.captures(dirname) {
        Some(caps) => (caps[1].trim().to_string(), caps[2].parse::<u32>().ok()),
        None => (dirname.to_string(), None),
    };
    movie.title = title;
    movie.year = year;

    // Initialize mediadata.
    let mut mediadata = MediaData {
        basedir: dirpath,
        basename: basename.to_string(),
        item_type: ItemType::Movie,
        updated: false,
        item: movie,
    };

    // Then add all files.
    for entry in &entries {
        if only_nfo && !entry.ends_with(".nfo") {
            continue;
        }
        if let Err(_) = mediadata.add_file(entry).await {
            if entry.ends_with(".mp4") {
                return None;
            }
        }
    }
    mediadata.finalize();

    Some(mediadata.item)
}
