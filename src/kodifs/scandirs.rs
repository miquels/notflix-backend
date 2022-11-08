use std::collections::HashMap;
use std::path::PathBuf;

use tokio::fs;

use crate::collections::*;
use super::systemtime_to_ms;

// Get a list of all directories and their last-modified time (in unix ms)
pub async fn list_directories(coll: &Collection, name: &str, subdirs: bool) -> HashMap<String, i64> {
    let mut hm = HashMap::new();
    let mut newest = 0;

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
        let meta = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };
        if let Ok(modified) = meta.modified() {
            let ts = systemtime_to_ms(modified) as i64;
            if ts > newest {
                newest = ts;
            }
        }
        if !subdirs {
            if newest > 0 {
                hm.insert(name.to_string(), newest);
            }
            continue;
        }

        // scan the timestamps on the subdirectories as well.
        let mut subdir = dirname.clone();
        subdir.push(name);

        let mut d = match fs::read_dir(&dirname).await {
            Ok(d) => d,
            Err(_) => continue,
        };

        while let Ok(Some(entry)) = d.next_entry().await {
            let meta = match entry.metadata().await {
                Ok(m) if m.file_type().is_dir() => m,
                _ => continue,
            };
            if let Ok(modified) = meta.modified() {
                let ts = systemtime_to_ms(modified) as i64;
                if ts > newest {
                    newest = ts;
                }
            }
        }

        if newest > 0 {
            hm.insert(name.to_string(), newest);
        }
    }
    hm
}
