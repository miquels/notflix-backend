use std::os::unix::fs::MetadataExt;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

use crate::sqlx::impl_sqlx_traits_for;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileInfo {
    pub path:   String,
    pub inode:  u64,
    pub size:   u64,
    pub modified: SystemTime,
    #[serde(skip)]
    pub fullpath: String,
}
impl_sqlx_traits_for!(FileInfo);

impl std::cmp::PartialEq for FileInfo {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.inode == other.inode &&
            self.size == other.size && self.modified == other.modified
    }
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

    pub fn open_std(basedir: &str, path: &str) -> io::Result<(std::fs::File, FileInfo)> {
        let fullpath = FileInfo::join(basedir, path);
        let f = std::fs::File::open(&fullpath)?;
        let m = f.metadata()?;
        Ok((f, FileInfo {
            path: path.to_string(),
            fullpath,
            inode: m.ino(),
            size: m.len(),
            modified: m.modified()?,
        }))
    }
}
