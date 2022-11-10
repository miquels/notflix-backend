use std::path::PathBuf;

use tokio::fs;

use crate::collections::*;
use crate::models::{FileInfo, Movie, Thumb};
use crate::util::SystemTimeToUnixTime;
use super::*;

pub async fn update_movie(coll: &Collection, dirname: &str, dbent: &Movie, only_nfo: bool) -> Option<Movie> {

    // First get all directory entries.
    let mut dirpath = PathBuf::from(&coll.directory);
    dirpath.push(dirname);
    let dirpath = dirpath.to_str().unwrap().to_string();
    let dirinfo = FileInfo::from_path(&dirpath, None, "").await.ok()?;

    let mut entries = Vec::new();
    let mut d = fs::read_dir(&dirpath).await.ok()?;
    while let Ok(Some(entry)) = d.next_entry().await {
        entries.push(entry);
    }

    // Collect timestamps.
    let mut added_ts = Vec::new();
    added_ts.push(dirinfo.modified);
    if let Ok(created) = tokio::fs::metadata(&dirpath).await.and_then(|m| m.created()) {
        added_ts.push(created);
    }

    // Loop over all directory entries. We need to find a video file.
    // If we don't, skip the entire directory.
    let mut base = String::new();
    let mut video = None;
    for entry in &entries {
        let file_name = entry.file_name();
        let file_name = match file_name.to_str() {
            Some(name) => name,
            None => continue,
        };
        let caps = match IS_VIDEO.captures(file_name) {
            Some(caps) => caps,
            None => continue,
        };
        video = match FileInfo::from_path(&dirpath, None, file_name).await {
            Ok(v) => Some(v),
            Err(_) => continue,
        };
        base = caps[1].to_string();
        break;
    }
    let video = video?;
    added_ts.push(video.modified);

    // If the directory name ends with <space>(YYYY) then it's a year.
    // Remember that year as backup in case there's no NFO file.
    let (title, year) = match IS_YEAR.captures(dirname) {
        Some(caps) => (Some(caps[1].to_string()), Some(caps[2].parse::<u32>().unwrap())),
        None => (Some(dirname.to_string()), None),
    };

    // `added_ts` contains a list of dates, use the oldest as `added`.
    added_ts.sort();
    // let added = (added_ts.len() > 0).then(|| DateTime<offset::Utc>::from(added_ts[0]));

    // Initial Movie.
    let mut movie = Movie {
        id: dbent.id,
        lastmodified: video.modified.unixtime_ms(),
        collection_id: coll.collection_id as i64,
        directory: sqlx::types::Json(dirinfo),
        // XXX TODO dateadded: DateTime::offset::Utc::from(created),
        ..Movie::default()
    };

    // Loop over all directory entries.
    for entry in &entries {
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(name) => name,
            None => continue,
        };

        let mut aux = String::new();
        let mut ext = String::new();

        // poster.jpg
        match IS_EXT1.captures(name) {
            Some(caps) => {
                ext = caps[2].to_string();
                if &caps[1] != base {
                    aux = caps[1].to_string();
                    if let Some(caps) = IS_EXT2.captures(name) {
                        if &caps[1] == base {
                            aux = caps[2].to_string();
                            ext = caps[3].to_string();
                        }
                    }
                }
            },
            None => {
                // big.bucks.bunny-poster.jpg
                if let Some(caps) = IS_EXT2.captures(name) {
                    if &caps[1] == base {
                        aux = caps[2].to_string();
                        ext = caps[3].to_string();
                    }
                }
            },
        }

        if ext == "" {
            continue;
        }

        // NFO file found. Parse it.
        if ext == "nfo" {
            let (mut file, fileinfo) = match FileInfo::open(&dirpath, None, name).await {
                Ok(f) => f,
                Err(_) => continue,
            };

            // Same? Just copy it.
            let dbent_nfofile = dbent.nfofile.as_ref().map(|f| &f.0);
            if dbent_nfofile == Some(&fileinfo) {
               movie.nfo_base = dbent.nfo_base.clone();
               movie.nfo_movie = dbent.nfo_movie.clone();
               movie.nfofile = dbent.nfofile.clone();
               continue;
            }

            match super::Nfo::read(&mut file).await {
                Ok(nfo) => {
                    nfo.update_movie(&mut movie);
                    if movie.nfo_base.title.is_none() && title.is_some() {
                        movie.nfo_base.title = title.clone();
                    }
                    if movie.nfo_movie.premiered.is_none() && year.is_some() {
                        movie.nfo_movie.premiered = Some(format!("{}-01-01", year.unwrap()));
                    }
                    let m = fileinfo.modified.unixtime_ms();
                    if m > movie.lastmodified {
                        movie.lastmodified = m;
                    }
                    movie.nfofile = Some(sqlx::types::Json(fileinfo));
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
                    aux = "poster".to_string();
            }
            let aspect = match aux.as_str() {
                "banner" |
                "fanart" |
                "poster" |
                "landscape" |
                "clearart" |
                "clearlogo" => aux,
                _ => continue,
            };
            movie.thumbs.push(Thumb {
                path: name.to_string(),
                aspect: aspect.to_string(),
                season: None,
            });
        }
    }

    Some(movie)
}
