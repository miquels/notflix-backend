use chrono::TimeZone;
use std::time::SystemTime;

use super::*;
use crate::collections::*;
use crate::models::{FileInfo, MediaItem, Thumb};
use crate::util::{Id, SystemTimeToUnixTime};
use super::resource::{ItemType, MediaData};

pub async fn scan_tvshow_dir(
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

    // Initial TVShow.
    let mut tvshow = dbent.unwrap_or_else(|| {
        Box::new(MediaItem {
            id: Id::new(),
            collection_id: coll.collection_id,
            ..MediaItem::default()
        })
    });
    tvshow.lastmodified = newest;
    if tvshow.dateadded == "" {
        if let chrono::LocalResult::Single(c) = chrono::Local.timestamp_millis_opt(oldest) {
            tvshow.dateadded = c.format("%Y-%m-%d").to_string();
        }
    }

    // If the directory name changed, we need to update the db.
    if let Some(fileinfo) = tvshow.directory.as_ref() {
        if fileinfo.path != dirname {
            log::debug!(
                "kodifs::scan_tvshow_dir: directory rename {} -> {}",
                fileinfo.path,
                dirname
            );
            tvshow.lastmodified = SystemTime::now().unixtime_ms();
        }
    }
    tvshow.directory = Some(dirinfo);
    tvshow.title = dirname.to_string();

    // Initialize mediadata.
    let mut mediadata = MediaData {
        basedir: dirpath,
        basename: String::new(),
        item_type: ItemType::TVShow,
        updated: false,
        item: tvshow,
    };

    // Then add all files.
    for entry in &entries {
        if only_nfo && !entry.ends_with(".nfo") {
            continue;
        }
        let _ = mediadata.add_file(entry).await;
    }
    mediadata.finalize();

    Some(mediadata.item)
}
