// Parse filenames and classify as a specific resource.

use anyhow::Result;

use super::Nfo;
use crate::models::{self, FileInfo, ThumbState};

const SUBTITLES: &'static [&'static str] = &["srt", "vtt"];

const THUMBS: &'static [&'static str] = &["jpg", "jpeg", "png", "tbn"];

const ASPECTS: &'static [&'static str] = &[
    "banner",
    "clearart",
    "clearlogo",
    "discart",
    "fanart",
    "keyart",
    "landscape",
    "poster",
    "thumb",
];

#[derive(PartialEq)]
pub enum ItemType {
    Movie,
    TVShow,
    Episode,
}

pub struct MediaData {
    pub basedir: String,
    pub basename: String,
    pub item_type: ItemType,
    pub updated: bool,
    pub item: Box<models::MediaItem>,
}

impl MediaData {
    /// Add a file to the MediaItem embedded in this MediaData struct.
    ///
    /// If this is a movie or an episode, make sure to add the mp4 _first_,
    /// or set self.basename in advance.
    pub async fn add_file(&mut self, filename: &str) -> Result<()> {
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
        Ok(())
    }

    async fn add_mp4(&mut self, filename: &str) -> Result<()> {
        // only movies and episodes.
        if self.item_type == ItemType::TVShow {
            return Ok(());
        }

        // TODO, what if this fails? Mark the entire item as 'deleted' ?
        let fileinfo = FileInfo::from_path(&self.basedir, filename).await?;
        if let Some(video_file) = self.item.video_file.as_ref() {
            if &fileinfo == video_file {
                return Ok(());
            }
        }
        self.item.video_info = super::video::probe(&fileinfo.fullpath).await.ok();
        self.item.video_file = Some(fileinfo);
        self.updated = true;
        Ok(())
    }

    async fn add_nfo(&mut self, filename: &str) -> Result<()> {
        // we ignore movie.nfo, it's not in the "standard".
        if filename == "movie.nfo" {
            return Ok(());
        }

        // for itemtype tvshow, make sure to skip the episode nfos.
        if self.item_type == ItemType::TVShow && filename != "tvshow.nfo" {
            return Ok(());
        }

        // If this fails, we just keep the old data.
        let (mut file, fileinfo) = FileInfo::open(&self.basedir, filename).await?;
        if let Some(nfo_file) = self.item.nfo_file.as_ref() {
            if &fileinfo == nfo_file {
                return Ok(());
            }
        }
        self.item.nfo_info = Some(Nfo::read(&mut file).await?.to_nfo());
        self.item.nfo_file = Some(fileinfo);
        self.updated = true;
        Ok(())
    }

    async fn add_thumb(&mut self, filename: &str, name_noext: &str) -> Result<()> {
        let aspect;
        let mut season_name = None;

        if self.item_type != ItemType::Episode && ASPECTS.contains(&name_noext) {
            // simple short name. nothing much to do.
            aspect = name_noext;
        } else if self.item_type == ItemType::TVShow {
            // it's either a simple short name (handled above) or a season
            // image. in all other cases, return.
            if !name_noext.starts_with("season") {
                return Ok(());
            }

            // season related image. first, split off the aspect.
            let base = match name_noext.rsplit_once('-') {
                Some((base, asp)) if ASPECTS.contains(&asp) => {
                    aspect = asp;
                    base
                },
                _ => return Ok(()),
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
                return Ok(());
            }
        } else {
            // must be an image that starts with "basename-".
            if !filename.starts_with(&self.basename) {
                return Ok(());
            }
            let len = self.basename.len();
            if !name_noext[len..].starts_with("-") {
                return Ok(());
            }

            // followed by a valid <aspect>.
            aspect = &name_noext[len + 1..];
            if !ASPECTS.contains(&aspect) {
                return Ok(());
            }
        }

        models::Thumb::add(
            &mut self.item.thumbs,
            &self.basedir,
            filename,
            self.item.id,
            aspect,
            season_name.map(str::to_string),
        )
        .await?;

        Ok(())
    }

    async fn add_subtitle(&mut self, _filename: &str, _base: &str, _ext: &str) -> Result<()> {
        if self.item_type == ItemType::TVShow {
            return Ok(());
        }
        // XXX TODO
        Ok(())
    }

    // This method needs to be called after all files have been added.
    //
    // Long names are preferred over short names. So if both are present, remove the short name.
    //
    // Returns `true` if we updated the MediaData struct (and so it has to be
    // updated in the database as well), `false` otherwise.
    pub fn finalize(&mut self) -> bool {
        let mut i = 0;
        while i < self.item.thumbs.len() {
            let t = &self.item.thumbs[i];
            if t.state == ThumbState::Deleted || t.season.is_some() || t.fileinfo.path.contains('-')
            {
                i += 1;
                continue;
            }
            if self.item.thumbs.iter().any(|l| {
                l.state != ThumbState::Deleted
                    && l.season.is_none()
                    && l.fileinfo.path.contains('-')
                    && l.aspect == t.aspect
            }) {
                self.item.thumbs.remove(i);
            } else {
                i += 1;
            }
        }

        // Thumbs in state 'new' or 'deleted': need to update the db.
        if self.item.thumbs.iter().any(|t| t.state != ThumbState::Unchanged) {
            self.updated = true;
        }
        // remove deleted thumbs from the list.
        self.item.thumbs.retain(|t| t.state != ThumbState::Deleted);

        // update the type.
        self.item.type_ = match self.item_type {
            ItemType::Movie => "movies",
            ItemType::TVShow => "tvshow",
            ItemType::Episode => "episode",
        }.to_string();

        self.updated
    }
}
