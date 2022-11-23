use std::collections::HashMap;

use anyhow::Result;
use futures_util::TryStreamExt;
use poem_openapi::Object;
use serde::{Serialize, Deserialize};

use crate::collections::Collection;
use crate::db;
use crate::util::Id;
use super::FileInfo;

/// Image
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct Image {
    pub id: i64,
    pub collection_id: i64,
    pub mediaitem_id: Id,
    pub image_id: i64,
    pub extra: Option<sqlx::types::Json<HashMap<String, String>>>,
    pub fileinfo: FileInfo,
    pub aspect: String,
    pub width: u32,
    pub height: u32,
    pub quality: u32,
    #[serde(skip)]
    pub state: ImageState,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ImageState {
    Deleted,
    #[default]
    Unchanged,
    New,
}

#[derive(Clone, Debug, Default, Object)]
pub struct GetImage {
    #[oai(skip)]
    pub id: i64,
    #[oai(skip)]
    pub image_id: i64,
    #[oai(skip)]
    pub fileinfo: FileInfo,
    pub path: String,
    pub aspect: String,
    pub width: u32,
    pub height: u32,
    pub quality: u32,
    pub season: Option<String>,
}

impl GetImage {
    /// Get the thumb and all its currently existing variants.
    ///
    /// Can query either by mediaitem_id (gets al images for one mediaitem),
    /// or specific image_id (gets all variants).
    pub async fn find(dbh: &mut db::TxnHandle<'_>, coll: &Collection, mediaitem_id: Option<Id>, image_id: Option<i64>, cache_dir: &str) -> Result<Vec<GetImage>> {

        let mut rows = sqlx::query!(
            r#"
                SELECT  t.id AS "id!",
                        t.image_id as "image_id!",
                        t.fileinfo AS "fileinfo!: FileInfo",
                        t.aspect AS "aspect!",
                        t.width AS "width!: u32",
                        t.height AS "height!: u32",
                        t.quality AS "quality!: u32",
                        t.extra AS "extra: sqlx::types::Json<HashMap<String, String>>",
                        m.directory AS "directory!: FileInfo",
                        m.id AS "mediaitem_id!: i64"
                FROM images t JOIN mediaitems m ON t.mediaitem_id = m.id
                WHERE t.mediaitem_id = ? OR t.image_id = ?
                ORDER BY t.id != t.image_id"#,
            mediaitem_id,
            image_id,
        )
        .fetch(dbh);

        let mut images = Vec::new();
        while let Some(mut row) = rows.try_next().await? {
            let prefix = if row.id == row.image_id {
                coll.directory.as_str()
            } else {
                // sanity check. original _must_ come first.
                if image_id.is_some() && images.len() == 0 {
                    return Ok(Vec::new());
                }
                cache_dir
            };
            row.fileinfo.fullpath = format!("{}/{}/{}", prefix, coll.directory, row.fileinfo.path);

            let ext = match row.fileinfo.path.rsplit_once(".").map(|t| t.1) {
                Some("tbn") => "jpg",
                Some(ext) => ext,
                None => "jpg",
            };
            let path = format!("/api/image/{}/{}/{}.{}", coll.collection_id, row.mediaitem_id, row.id, ext);
            let season = row.extra.and_then(|m| m.get("season").map(|s| s.to_string()));

            images.push(GetImage {
                id: row.id,
                image_id: row.image_id,
                fileinfo: row.fileinfo,
                path,
                aspect: row.aspect,
                width: row.width,
                height: row.height,
                quality: row.quality,
                season,
            });
        }

        Ok(images)
    }

    pub async fn delete(&self, txn: &mut db::TxnHandle<'_>) -> Result<()> {
        let image = Image {
            id: self.id,
            image_id: self.image_id,
            ..Image::default()
        };
        image.delete(txn).await
    }
}

impl Image {
    pub async fn insert(&mut self, txn: &mut db::TxnHandle<'_>) -> Result<()> {
        self.id = sqlx::query!(
            r#"
                INSERT INTO images(
                    collection_id,
                    mediaitem_id,
                    image_id,
                    extra,
                    fileinfo,
                    aspect,
                    width,
                    height,
                    quality
                ) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            self.collection_id,
            self.mediaitem_id,
            self.image_id,
            self.extra,
            self.fileinfo,
            self.aspect,
            self.width,
            self.height,
            self.quality
        )
        .execute(&mut *txn)
        .await?
        .last_insert_rowid();

        Ok(())
    }

    pub async fn delete(&self, txn: &mut db::TxnHandle<'_>) -> Result<()> {
        if self.id == self.image_id {
            sqlx::query!(
                r#"
                    DELETE FROM images
                    WHERE image_id = ?"#,
                self.id,
            )
        } else {
            sqlx::query!(
                r#"
                    DELETE FROM images
                    WHERE id = ?"#,
                self.id,
            )
        }
        .execute(&mut *txn)
        .await?;

        Ok(())
    }
}
