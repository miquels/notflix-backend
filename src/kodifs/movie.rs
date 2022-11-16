use std::time::SystemTime;
use chrono::TimeZone;

use crate::collections::*;
use crate::models::{FileInfo, Movie};
use crate::util::SystemTimeToUnixTime;
use super::*;

pub async fn scan_movie_dir(coll: &Collection, mut dirname: &str, dbent: Option<Box<Movie>>, only_nfo: bool) -> Option<Box<Movie>> {

    // First get all directory entries.
    dirname = dirname.trim_end_matches('/');
    let dirinfo = FileInfo::from_path(&coll.directory, dirname).await.ok()?;
    let dirpath = dirinfo.fullpath.clone();
    let mut entries = Vec::new();
    let (oldest, newest) = scandirs::read_dir(&dirpath, false, &mut entries, true).await.ok()?;

    // Loop over all directory entries.
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

    // If we didn't find a video file, return None.
    let video = video?;

    // Initial Movie.
    let mut movie = dbent.unwrap_or_else(|| Box::new(Movie {
        collection_id: coll.collection_id as i64,
        video: sqlx::types::Json(video),
        ..Movie::default()
    }));
    movie.lastmodified = newest;
    if movie.dateadded.is_none() {
        if let chrono::LocalResult::Single(c) = chrono::Local.timestamp_millis_opt(oldest) {
            movie.dateadded = Some(c.format("%Y-%m-%d").to_string());
        }
    }

    // If the directory name changed, we need to update the db.
    if movie.directory.path != dirname {
        if movie.directory.path != "" {
            log::debug!("Movie::scan_movie_dir: directory rename {} -> {}", movie.directory.path, dirname);
        }
        movie.lastmodified = SystemTime::now().unixtime_ms();
    }
    movie.directory = sqlx::types::Json(dirinfo);

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
                    movie.nfofile = nfofile;
                },
                Err(e) => {
                    // We failed. Will automatically try again as soon
                    // as ctime or mtime of the file is updated.
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
