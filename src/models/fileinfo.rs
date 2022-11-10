use std::os::unix::fs::MetadataExt;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FileInfo {
    pub path:   String,
    pub inode:  u64,
    pub size:   u64,
    pub modified: SystemTime,
}

impl std::default::Default for FileInfo {
    fn default() -> FileInfo {
        FileInfo {
            path: String::new(),
            inode: 0,
            size: 0,
            modified: SystemTime::UNIX_EPOCH,
        }
    }
}

use std::io;
use tokio::fs::File;
impl FileInfo {
    pub fn join(mut basedir: &str, mut subdir: &str, path: &str) -> String {
        let mut slash1 = "/";
        let mut slash2 = "/";
        if basedir == "" || basedir == "." || basedir == "./" {
            basedir = "";
            slash1 = "";
        }
        if subdir == "" || subdir == "." || subdir == "./" {
            subdir = "";
            slash2 = "";
        }
        [ basedir, slash1, subdir, slash2, path ].join("")
    }

    pub async fn from_path<'a, T>(basedir: &str, subdir: T, path: &str) -> io::Result<FileInfo>
    where
        T: Into<Option<&'a str>>
    {
        let subdir = subdir.into().unwrap_or("");
        let m = tokio::fs::metadata(FileInfo::join(basedir, subdir, path)).await?;
        Ok(FileInfo {
            path: FileInfo::join("", subdir, path),
            inode: m.ino(),
            size: m.len(),
            modified: m.modified()?,
        })
    }

    pub async fn open<'a, T>(basedir: &str, subdir: T, path: &str) -> io::Result<(File, FileInfo)>
    where
        T: Into<Option<&'a str>>
    {
        let subdir = subdir.into().unwrap_or("");
        let f = File::open(FileInfo::join(basedir, subdir, path)).await?;
        let m = f.metadata().await?;
        Ok((f, FileInfo {
            path: FileInfo::join("", subdir, path),
            inode: m.ino(),
            size: m.len(),
            modified: m.modified()?,
        }))
    }
}
