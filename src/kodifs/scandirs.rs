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
