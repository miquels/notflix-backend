use std::os::unix::fs::MetadataExt;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FileInfo {
    pub path:   String,
    pub inode:  u64,
    pub size:   u64,
    pub modified: SystemTime,
    #[serde(skip)]
    pub fullpath: String,
}

impl std::default::Default for FileInfo {
    fn default() -> FileInfo {
        FileInfo {
            path: String::new(),
            inode: 0,
            size: 0,
            modified: SystemTime::UNIX_EPOCH,
            fullpath: String::new(),
        }
    }
}

use std::io;
use tokio::fs::File;
impl FileInfo {
    pub fn join(basedir: &str, path: &str) -> String {
        format!("{}/{}", basedir, path)
    }

    pub async fn from_path(basedir: &str, path: &str) -> io::Result<FileInfo> {
        let fullpath = FileInfo::join(basedir, path);
        let m = tokio::fs::metadata(&fullpath).await?;
        Ok(FileInfo {
            path: path.to_string(),
            fullpath,
            inode: m.ino(),
            size: m.len(),
            modified: m.modified()?,
        })
    }

    pub async fn open(basedir: &str, path: &str) -> io::Result<(File, FileInfo)> {
        let fullpath = FileInfo::join(basedir, path);
        let f = File::open(&fullpath).await?;
        let m = f.metadata().await?;
        Ok((f, FileInfo {
            path: path.to_string(),
            fullpath,
            inode: m.ino(),
            size: m.len(),
            modified: m.modified()?,
        }))
    }
}
