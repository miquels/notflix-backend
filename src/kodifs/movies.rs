use std::time::SystemTime;

use crate::collections::*;
use crate::models::{FileInfo, Movie};
use crate::util::SystemTimeToUnixTime;
use super::*;

pub async fn scan_movie_dir(coll: &Collection, mut dirname: &str, dbent: Option<Movie>, only_nfo: bool) -> Option<Movie> {

    // First get all directory entries.
    dirname = dirname.trim_end_matches('/');
    let dirinfo = FileInfo::from_path(&coll.directory, dirname).await.ok()?;
    let dirpath = dirinfo.fullpath.clone();
    let mut entries = Vec::new();
    scandirs::read_dir(&dirpath, false, &mut entries).await;

    // Collect timestamps.
    let mut added_ts = Vec::new();
    added_ts.push(dirinfo.modified);
    if let Ok(created) = tokio::fs::metadata(&dirpath).await.and_then(|m| m.created()) {
        added_ts.push(created);
    }

    // Loop over all directory entries. We need to find a video file.
    // If we don't, skip the entire directory.
    let mut basepath = String::new();
    let mut video = None;
    for name in &entries {
        let caps = match IS_VIDEO.captures(name) {
            Some(caps) => caps,
            None => continue,
        };
        video = match FileInfo::from_path(&dirpath, name).await {
            Ok(v) => Some(v),
            Err(_) => continue,
        };
        basepath = caps[1].to_string();
        break;
    }
    let video = video?;
    added_ts.push(video.modified);

    // `added_ts` contains a list of dates, use the oldest as `added`.
    added_ts.sort();
    // let added = (added_ts.len() > 0).then(|| DateTime<offset::Utc>::from(added_ts[0]));

    // Initial Movie.
    let mut movie = dbent.unwrap_or_else(|| Movie {
        lastmodified: video.modified.unixtime_ms(),
        collection_id: coll.collection_id as i64,
        ..Movie::default()
    });

    // If the directory name changed, we need to update the db.
    if movie.directory.path != dirname {
        if movie.directory.path != "" {
            log::debug!("Movie::scan_movie_dir: directory rename {} -> {}", movie.directory.path, dirname);
        }
        movie.lastmodified = SystemTime::now().unixtime_ms();
        movie.directory = sqlx::types::Json(dirinfo);
    }

    // If the directory name ends with <space>(YYYY) then it's a year.
    // Remember that year as backup in case there's no NFO file.
    let (title, year) = match IS_YEAR.captures(dirname) {
        Some(caps) => (Some(caps[1].trim().to_string()), caps[2].parse::<u32>().ok()),
        None => (Some(dirname.to_string()), None),
    };
    movie.nfo_base.title = title;
    if let Some(year) = year {
        movie.nfo_movie.premiered = Some(format!("{}-01-01", year));
    }

    // Loop over all directory entries.
    for name in &entries {

        let mut i = name.split('/').last().unwrap().rsplitn(2, '.');
        let (base, ext) = match (i.next(), i.next()) {
            (Some(ext), Some(base)) => (base, ext),
            _ => continue,
        };

        let mut aux = "";
        if let Some(suffix) = name.strip_prefix(&basepath) {
            if suffix.len() > ext.len() + 2 {
                if suffix.starts_with(".") || suffix.starts_with("-") {
                    aux = suffix[1..].strip_suffix(ext).unwrap();
                    aux = &aux[..aux.len() - 1];
                }
            }
        }

        // NFO file found. Parse it.
        if ext == "nfo" {
            let (mut file, nfofile) = match FileInfo::open(&dirpath, name).await {
                Ok((file, fileinfo)) => (file, Some(sqlx::types::Json(fileinfo))),
                Err(_) => continue,
            };

            if movie.nfofile == nfofile {
                // No change, nothing to do!
                continue;
            }

            match super::Nfo::read(&mut file).await {
                Ok(nfo) => {
                    nfo.update_movie(&mut movie);
                    let m = nfofile.as_ref().unwrap().modified.unixtime_ms();
                    if m > movie.lastmodified {
                        movie.lastmodified = m;
                    }
                    movie.nfofile = nfofile;
                },
                Err(e) => {
                    println!("error reading nfo: {}", e);
                },
            }
            continue;
        }

        if only_nfo {
            continue;
        }

        // Image: banner, fanart, folder, poster etc
        if IS_IMAGE.is_match(name) {
            if ext == "tbn" && aux == "" {
                    aux = "poster";
            }
            if aux == "" {
                aux = base;
            }
            let aspect = match aux {
                "banner" |
                "fanart" |
                "poster" |
                "landscape" |
                "clearart" |
                "clearlogo" => aux,
                _ => continue,
            };
            add_thumb(&mut movie.thumbs, "", name, aspect, None);
        }

        // XXX TODO: subtitles srt/vtt
    }

    Some(movie)
}
