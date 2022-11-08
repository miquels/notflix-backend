use std::collections::HashMap;
use std::path::PathBuf;
use std::os::unix::fs::MetadataExt;

use tokio::fs;

use crate::collections::*;
use crate::models::{FileInfo, Movie, Thumb};
use super::*;

pub async fn build_movies(_coll: &Collection, _pace: u32) {
    todo!()
}

/*
pub async fn update_movies(db: &DataBase, coll: &Collection, pace: u32) {

    let mut d = match fs::read_dir(&coll.directory).await {
        Ok(d) => d,
        Err(_) => return,
    };
    let mut items = Vec::new();

    while let Ok(Some(entry)) = d.next_entry().await {

        // We might want to pace ourselves.
        if pace > 0 {
            tokio::time::sleep(Duration::from_secs(pace as u64)).await;
        }

        // Get a valid filename.
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(name) => name,
            None => continue,
        };
        if name.starts_with(".") || name.starts_with("+ ") {
            continue;
        }

        // Do we have this in the database? Check by filename.
        if let Some(dbitem) = db.find_basic_info_by_path(coll, name).await {
            update_movie(coll, Some(dbitem), None);
            continue;
        }

        // We don't. So read the directory.
        if let Some(movie) = build_movie(coll, name, true).await {
            update_movie(coll, None, Some(movie));
        }
    }
}
*/

// Get a list of all directories and their last-modified time (in unix ms)
pub async fn list_movies(coll: &Collection, name: &str) -> HashMap<String, i64> {
    let mut hm = HashMap::new();

    let mut dirname = PathBuf::from(&coll.directory);
    dirname.push(name);

    let mut d = match fs::read_dir(&dirname).await {
        Ok(d) => d,
        Err(_) => return hm,
    };

    while let Ok(Some(entry)) = d.next_entry().await {
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(name) if !name.starts_with(".") && !name.starts_with("+ ") => name,
            _ => continue,
        };
        if let Ok(modified) = entry.metadata().await.and_then(|m| m.modified()) {
            hm.insert(name.to_string(), super::systemtime_to_ms(modified) as i64);
        }
    }
    hm
}

pub async fn build_movie(coll: &Collection, name: &str, parse_nfo: bool) -> Option<Movie> {

    // First get all directory entries.
    let mut dirname = PathBuf::from(&coll.directory);
    dirname.push(name);
    let mut entries = Vec::new();
    let mut d = fs::read_dir(&dirname).await.ok()?;
    while let Ok(Some(entry)) = d.next_entry().await {
        entries.push(entry);
    }

    // Collect timestamps.
    let mut added_ts = Vec::new();
    let meta = tokio::fs::metadata(&dirname).await.ok()?;
    let modified = meta.modified().ok()?;
    added_ts.push(modified);
    if let Ok(created) = meta.created() {
        added_ts.push(created);
    }

    // Loop over all directory entries. We need to find a video file.
    // If we don't, skip the entire directory.
    let mut video = String::new();
    let mut base = String::new();
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
        match entry.metadata().await {
            Ok(meta) => {
                if let Ok(created) = meta.created() {
                    added_ts.push(created);
                }
            },
            Err(_) => continue,
        }
        video = caps[0].to_string();
        base = caps[1].to_string();
        break;
    }
    if video.is_empty() {
        return None;
    }

    // Get the year from the directory name. This will be used if we cannot
    // find it in the NFO file.
    // TODO FIXME also use as fall-back title.
    let year = IS_YEAR.captures(name).map(|caps| caps[1].parse::<u32>().unwrap());

    // `added_ts` contains a list of dates, use the oldest as `added`.
    added_ts.sort();
    // let added = (added_ts.len() > 0).then(|| DateTime<offset::Utc>::from(added_ts[0]));

    // Initial Movie.
    let mut movie = Movie {
        collection_id: coll.collection_id as i64,
        directory: sqlx::types::Json(FileInfo {
            path: name.to_string(),
            inode: meta.ino(),
            size: meta.len(),
        }),
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

        // NFO file found. Parse it.
        if ext == "nfo" {
            let mut inode = 0;
            let mut size = 0;
            let mut nfo_path = PathBuf::from(&coll.directory);
            nfo_path.push(&dirname);
            nfo_path.push(name);
            if parse_nfo {
                if let Ok(mut file) = fs::File::open(&nfo_path).await {
                    match super::Nfo::read(&mut file).await {
                        Ok(nfo) => {
                            nfo.update_movie(&mut movie);
                            if movie.nfo_movie.premiered.is_none() && year.is_some() {
                                movie.nfo_movie.premiered = Some(format!("{}-01-01", year.unwrap()));
                            }
                            let m = file.metadata().await.unwrap();
                            inode = m.ino();
                            size = m.len();
                        },
                        Err(e) => println!("error reading nfo: {}", e),
                    }
                }
            }
            movie.nfofile = Some(sqlx::types::Json(FileInfo {
                path: nfo_path.to_str().unwrap().to_string(),
                inode,
                size,
            }));
        }
    }

    Some(movie)
}
