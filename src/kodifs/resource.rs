// Parse filenames and classify as a specific resource.

use anyhow::Result;

use super::Nfo;
use crate::models::{self, FileInfo, ThumbState};

const SUBTITLES: &'static[&'static str] = &[
    "srt", "vtt",
];

const THUMBS: &'static[&'static str] = &[
    "jpg", "jpeg", "png", "tbn",
];

const ASPECTS: &'static[&'static str] = &[
    "banner", "clearart", "clearlogo", "discart", "fanart", "keyart", "landscape", "poster", "thumb"
];

#[derive(PartialEq)]
pub enum ItemType {
    Movie,
    TvShow,
    Episode,
}

pub struct MediaData {
    pub basedir:  String,
    pub basename: String,
    pub item_type: ItemType,
    pub item:   Box<models::MediaItem>,
}

impl MediaData {
    /// Add a file to the MediaItem embedded in this MediaData struct.
    ///
    /// If this is a movie or an episode, make sure to add the mp4 _first_,
    /// or set self.basename in advance.
    pub async fn add_file(&mut self, filename: &str) -> Result<bool> {
        if let Some((base, ext)) = filename.rsplit_once('.') {
            if ext == "mp4" {
                self.basename = base.to_string();
                return self.add_mp4(filename).await;
            }
            if ext == "nfo" {
                return self.add_nfo(filename).await;
            }
            if THUMBS.contains(&ext) {
                return self.add_thumb(filename, base).await;
            }
            if SUBTITLES.contains(&ext) {
                return self.add_subtitle(filename, base, ext).await;
            }
        }
        Ok(false)
    }

    async fn add_mp4(&mut self, filename: &str) -> Result<bool> {
       // TODO, what if this fails? Mark the entire item as 'deleted' ?
        let fileinfo = FileInfo::from_path(&self.basedir, filename).await?;
        if let Some(video_file) = self.item.video_file.as_ref() {
            if &fileinfo == video_file {
                return Ok(false);
            }
        }
        self.item.video_info = super::video::probe(&fileinfo.fullpath).await.ok();
        self.item.video_file = Some(fileinfo);
        Ok(true)
    }

    async fn add_nfo(&mut self, filename: &str) -> Result<bool> {
        // we ignore movie.nfo.
        if filename == "movie.nfo" {
            return Ok(false);
        }

        // If this fails, we just keep the old data.
        let (mut file, fileinfo) = FileInfo::open(&self.basedir, filename).await?;
        if let Some(nfo_file) = self.item.nfo_file.as_ref() {
            if &fileinfo == nfo_file {
                return Ok(false);
            }
        }
        self.item.nfo_info = Some(Nfo::read(&mut file).await?.to_nfo());
        self.item.nfo_file = Some(fileinfo);
        Ok(true)
    }

    async fn add_thumb(&mut self, filename: &str, base: &str) -> Result<bool> {
        let aspect;
        let mut season_name = None;

        if self.item_type != ItemType::Episode && ASPECTS.contains(&base) {
            // simple short name. nothing much to do.
            aspect = base;
        } else if self.item_type == ItemType::TvShow && base.starts_with("season") {

            // season related image. first, split off the aspect.
            let base = match base.rsplit_once('-') {
                Some((base, asp)) if ASPECTS.contains(&asp) => {
                    aspect = asp;
                    base
                },
                _ => return Ok(false),
            };

            // now it must be season-all, season-specials, season<number>.
            season_name = if base.starts_with("season-") {
                // season-all, season-specials
                let t = &base[7..];
                (t == "all" || t == "specials").then(|| t)
            } else if base.starts_with("season") {
                // season1, season01, etc
                let num = base[6..].trim_start_matches('0');
                num.parse::<u32>().ok().map(|_| num)
            } else {
                None
            };
            if season_name.is_none() {
                return Ok(false);
            }
        } else {

            // must be an image that starts with "basename-".
            if !filename.starts_with(&self.basename) {
                return Ok(false);
            }
            let len = filename.len();
            if !base[len..].starts_with("-") {
                return Ok(false);
            }

            // followed by a valid <aspect>.
            aspect = &base[len+1..];
            if !ASPECTS.contains(&aspect) {
                return Ok(false);
            }
        }

        // now check if we already had this image in the database.
        let fileinfo = FileInfo::from_path(&self.basedir, filename).await?;
        for t in self.item.thumbs.iter_mut() {
            if t == &fileinfo {
                t.state = ThumbState::Unchanged;
                return Ok(true);
            }
        }

        // add as a new image.
        let t = models::Thumb {
            image_id: 1,
            fileinfo,
            path: filename.to_string(),
            aspect: aspect.to_string(),
            width: None,
            height: None,
            quality: None,
            season: season_name.to_string(),
            state: ThumbState::New,
        };
        self.items.thumbs.push(t);

        Ok(true)
    }

    async fn add_subtitle(&mut self, _filename: &str, _base: &str, _ext: &str) -> Result<bool> {
        // XXX TODO
        Ok(false)
    }

    // This method needs to be called after all files have been added.
    //
    // Long names are preferred over short names. So if both are present, remove the short name.
    fn finalize(&mut self) {
        let mut i = 0;
        while i < self.item.thumbs.len() {
            let t = self.item.thumbs[i];
            if t.state == ThumbState::Deleted || t.season.is_some() || t.fileinfo.path.contains('-') {
                continue;
            }
            if self.item.thumbs.iter().any(|l| l.state != ThumbState::Deleted && l.season.is_none() && l.fileinfo.path.contains('-') && l.aspect == t.aspect) {
                self.item.thumbs.remove(i);
            } else {
                i += 1;
            }
        }
    }

}
