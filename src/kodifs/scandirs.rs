use std::collections::HashMap;
use std::path::PathBuf;
use std::io;

use tokio::fs;

use crate::collections::*;
use crate::util::SystemTimeToUnixTime;

// Get a list of all directories and their last-modified time (in unix ms)
pub async fn scan_directories(coll: &Collection, subdirs: bool) -> HashMap<String, i64> {
    let mut hm = HashMap::new();

    let mut d = match fs::read_dir(&coll.directory).await {
        Ok(d) => d,
        Err(_) => return hm,
    };

    while let Ok(Some(entry)) = d.next_entry().await {
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(name) if !name.starts_with(".") && !name.starts_with("+ ") => name,
            _ => continue,
        };
        if let Ok(newest) = scan_directory(coll, name, subdirs).await {
            hm.insert(name.to_string(), newest);
        }
    }

    hm
}

pub async fn scan_directory(coll: &Collection, name: &str, subdirs: bool) -> io::Result<i64> {
    let mut dirname = PathBuf::from(&coll.directory);
    dirname.push(name);

    let meta = fs::metadata(&dirname).await?;
    let mut newest = meta.modified()?.unixtime_ms();
    if !subdirs {
        return Ok(newest);
    }

    let mut d = fs::read_dir(&dirname).await?;
    while let Ok(Some(entry)) = d.next_entry().await {
        let meta = match entry.metadata().await {
            Ok(m) if m.file_type().is_dir() => m,
            _ => continue,
        };
        if let Ok(modified) = meta.modified() {
            let ts = modified.unixtime_ms();
            if ts > newest {
                newest = ts;
            }
        }
    }
    Ok(newest)
}

// Scan a directory recursively (max 1 subdir deep).
// The files in 'names' will be sorted alphabetically.
pub async fn read_dir(basedir: &str, subdirs: bool, names: &mut Vec<String>) {
    do_read_dir(basedir, subdirs, None, names).await;
    names.sort();
}

#[async_recursion::async_recursion]
async fn do_read_dir<'a: 'async_recursion>(basedir: &str, subdirs: bool, subdir: Option<&'a str>, names: &mut Vec<String>) {

    // Read the entire directory in one go.
    let dir = match subdir {
        Some(subdir) => format!("{}/{}", basedir, subdir),
        None => basedir.to_string(),
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
                    if subdirs {
                        do_read_dir(basedir, false, Some(&name), names).await;
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
