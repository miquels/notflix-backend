use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;

use crate::collections::*;
use super::*;

pub async fn build_movies(coll: &mut Collection, pace: u32) {

    let mut d = match fs::read_dir(&coll.directory).await {
        Ok(d) => d,
        Err(_) => return,
    };

    while let Ok(Some(entry)) = d.next_entry().await {
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(name) => name,
            None => continue,
        };
        if name.starts_with(".") || name.starts_with("+ ") {
            continue;
        }

        if let Some(m) = Item::build_movie(coll, name).await {
            coll.items.push(m);
        }
        if pace > 0 {
            tokio::time::sleep(Duration::from_secs(pace as u64)).await;
        }
    }
}

pub async fn build_movie(coll: &Collection, name: &str) -> Option<Item> {
    Item::build_movie(coll, name).await
}

impl Item {
    async fn build_movie(coll: &Collection, name: &str) -> Option<Item> {
        let mut dirname = PathBuf::from(&coll.directory);
        dirname.push(name);

        let mut d = match fs::read_dir(&dirname).await {
            Ok(d) => d,
            Err(_) => return None,
        };
        let mut entries = Vec::new();
        while let Ok(Some(entry)) = d.next_entry().await {
            entries.push(entry);
        }

        // created: timestamp.
        // video: filename of the video (the .mp4) without path
        // base: the filename without .mp4 extension.
        let (created, video, base) = async {
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
                let created = match entry.metadata().await {
                    Ok(m) => m.modified().unwrap(),
                    Err(_) => continue,
                };
                return Some((created, caps[0].to_string(), caps[1].to_string()));
            }
            None
        }.await?;

        let year = match IS_YEAR.captures(name) {
            Some(caps) => caps[1].parse::<u32>().unwrap(),
            None => time::OffsetDateTime::from(created).year() as u32,
        };

        let mut movie = Item {
            name: name.to_string(),
            year: Some(year),
            baseurl: coll.baseurl.clone(),
            path: escape_path(name),
            video: escape_path(&video),
            firstvideo: systemtime_to_ms(created),
            lastvideo: systemtime_to_ms(created),
            type_: "movie",
            ..Item::default()
        };

        for entry in &entries {
            let file_name = entry.file_name();
            let name = match file_name.to_str() {
                Some(name) => name,
                None => continue,
            };

            let mut aux = String::new();
            let mut ext = String::new();

            match IS_EXT1.captures(name) {
                Some(caps) => {
                    ext = caps[3].to_string();
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
            let p = escape_path(name);

            if IS_IMAGE.is_match(name) {
                if ext == "tbn" && aux == "" {
                        aux = "poster".to_string();
                }
                match aux.as_str() {
                    "banner" => movie.banner = Some(p),
                    "fanart" => movie.fanart = Some(p),
                    "folder" => movie.folder = Some(p),
                    "poster" => movie.poster = Some(p),
                    _ => continue,
                }
            }

            if ext == "nfo" {
                movie.nfo_path = PathBuf::from(&coll.directory);
                movie.nfo_path.push(&dirname);
                movie.nfo_path.push(name);
            }
        }

        // XXX TODO
        // db_load_item(coll, movie).await;

        Some(movie)
    }
}
