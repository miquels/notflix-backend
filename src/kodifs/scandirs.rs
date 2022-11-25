use std::collections::HashMap;
use std::io;
use std::os::unix::fs::MetadataExt;

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
    let dir = format!("{}/{}", coll.directory, name);
    let mut new = Some(0i64);
    do_read_dir(&dir, subdirs, None, &mut None, &mut None, &mut new).await?;
    Ok(new.unwrap())
}

// Scan a directory recursively (max 1 subdir deep).
// The files in 'names' will be sorted alphabetically.
pub async fn read_dir(
    basedir: &str,
    subdirs: bool,
    names: &mut Vec<String>,
    timestamps: bool,
) -> io::Result<(i64, i64)> {
    let mut old = timestamps.then(|| 0i64);
    let mut new = timestamps.then(|| 0i64);
    do_read_dir(basedir, subdirs, None, &mut Some(names), &mut old, &mut new).await?;
    names.sort();
    Ok((old.unwrap_or(0), new.unwrap_or(0)))
}

#[async_recursion::async_recursion]
async fn do_read_dir<'a: 'async_recursion>(
    basedir: &str,
    subdirs: bool,
    subdir: Option<&'a str>,
    names: &mut Option<&mut Vec<String>>,
    oldest: &mut Option<i64>,
    newest: &mut Option<i64>,
) -> io::Result<()> {
    // Only call metadata() on the entry if we need it.
    let do_meta = oldest.is_some() || newest.is_some();

    // Read the entire directory in one go.
    let dir = match subdir {
        Some(subdir) => format!("{}/{}", basedir, subdir),
        None => basedir.to_string(),
    };
    let subdir = subdir.map(|s| format!("{}/", s));

    if do_meta {
        // we ignore failed stats in subdirs.
        if let Err(e) = oldest_newest(tokio::fs::metadata(&dir).await, oldest, newest, true) {
            return if subdir.is_some() { Ok(()) } else { Err(e) };
        }
    }

    // we ignore failed read_dirs in subdirs.
    let mut d = match fs::read_dir(&dir).await {
        Ok(d) => d,
        Err(e) if subdir.is_none() => return Err(e),
        _ => return Ok(()),
    };

    while let Ok(Some(entry)) = d.next_entry().await {
        let mut name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        if name.starts_with(".") {
            continue;
        }

        let (is_dir, do_meta, do_oldest) = match entry.file_type().await {
            Ok(t) if t.is_dir() => (subdirs, do_meta, true),
            Ok(_) => {
                let m = name.ends_with(".mp4") || name.ends_with(".nfo");
                (false, m && do_meta, false)
            },
            Err(_) => continue,
        };

        if do_meta {
            let _ = oldest_newest(entry.metadata().await, oldest, newest, do_oldest);
        }

        if is_dir {
            if subdirs {
                do_read_dir(basedir, false, Some(&name), names, oldest, newest).await?;
            }
            continue;
        }

        if let Some(names) = names.as_mut() {
            if let Some(s) = subdir.as_ref() {
                name.insert_str(0, s);
            }
            names.push(name);
        }
    }
    Ok(())
}

fn oldest_newest(
    metadata: io::Result<std::fs::Metadata>,
    oldest: &mut Option<i64>,
    newest: &mut Option<i64>,
    do_oldest: bool,
) -> io::Result<()> {
    let meta = metadata?;
    if do_oldest {
        if let Some(o) = oldest.as_mut() {
            if let Ok(t) = meta.created().or_else(|_| meta.modified()) {
                let t = t.unixtime_ms();
                if t < *o || *o == 0 {
                    *o = t;
                }
            }
        }
    }
    if let Some(n) = newest.as_mut() {
        if let Ok(t) = meta.modified() {
            let t = t.unixtime_ms();
            if t > *n {
                *n = t;
            }
        }

        // ctime is "inode change time", so change in permissions or ownership.
        // need to trace that as well if e.g. permissions on a NFO file changed
        // so that we might now be able to read it where we could not before.
        if meta.ctime() > *n / 1000 {
            *n = meta.ctime() * 1000;
        }
    }
    Ok(())
}
