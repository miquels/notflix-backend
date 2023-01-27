use std::io::BufReader;

use anyhow::Result;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::collections::Collection;
use crate::models::{is_default, FileInfo};
use crate::sqlx::impl_sqlx_traits_for;
use crate::util::Id;

/// Image
#[derive(Object, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Thumb {
    #[oai(skip)]
    pub image_id: i64,
    #[oai(skip)]
    pub fileinfo: FileInfo,
    pub path: String,
    pub aspect: String,
    pub width: u32,
    pub height: u32,
    pub quality: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub season: Option<String>,
    #[serde(skip)]
    #[oai(skip)]
    pub state: ThumbState,
}
impl_sqlx_traits_for!(Thumb);

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ThumbState {
    Deleted,
    #[default]
    Unchanged,
    New,
}

impl Thumb {
    pub async fn new(
        basedir: &str,
        path: &str,
        mediaitem_id: Id,
        image_id: i64,
        aspect: &str,
        season: Option<String>,
    ) -> Result<Thumb> {
        let (fileinfo, width, height) = task::block_in_place(move || {
            let (file, fileinfo) = FileInfo::open_std(basedir, path)?;
            let ir = ::image::io::Reader::new(BufReader::with_capacity(32000, file));
            let (width, height) = ir.with_guessed_format()?.into_dimensions()?;
            Ok::<_, anyhow::Error>((fileinfo, width, height))
        })?;

        let ext = match fileinfo.path.rsplit_once(".").map(|t| t.1) {
            Some("tbn") => "jpg",
            Some(ext) => ext,
            None => "jpg",
        };
        let path = format!("/api/image/{}/{}.{}", mediaitem_id, image_id, ext);

        let mut season = season;
        if let Some(season) = season.as_mut() {
            while season.len() > 1 && season.starts_with("0") {
                season.remove(0);
            }
        }

        Ok(Thumb {
            image_id,
            fileinfo,
            path,
            aspect: aspect.to_string(),
            width,
            height,
            quality: None,
            season,
            state: ThumbState::New,
        })
    }

    pub async fn add(
        thumbs: &mut Vec<Thumb>,
        basedir: &str,
        path: &str,
        mediaitem_id: Id,
        aspect: &str,
        season: Option<String>,
    ) -> Result<()> {
        let fileinfo = FileInfo::from_path(basedir, path).await?;
        if let Some(thumb) = thumbs.iter_mut().find(|t| t.fileinfo == fileinfo) {
            thumb.state = ThumbState::Unchanged;
            return Ok(());
        }
        let id = thumbs.iter().fold(0, |acc, a| std::cmp::max(acc, a.image_id)) + 1;
        let thumb = Thumb::new(basedir, path, mediaitem_id, id, aspect, season).await?;
        thumbs.push(thumb);
        Ok(())
    }

    pub fn update_mediaitem_id(&mut self, mediaitem_id: Id) {
        let mut elems = self.path.rsplit("/").collect::<Vec<_>>();
        if elems.len() >= 1 {
            let m = mediaitem_id.to_string();
            elems[1] = m.as_str();
            elems.reverse();
            self.path = elems.join("/");
        }
    }
}
