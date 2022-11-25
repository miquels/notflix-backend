use crate::db;
use crate::jvec::JVec;
use crate::models::{FileInfo, Thumb};
use crate::util::{some_or_return, Id};
use anyhow::Result;
use futures_util::TryStreamExt;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct MediaInfoOverview {
    /// TVShow or Movie id
    pub id: Id,
    /// Title.
    pub title: String,
    /// Thumbnail in poster aspect (if available)
    pub poster: Option<Thumb>,
}

impl MediaInfoOverview {
    pub async fn get(
        dbh: &db::DbHandle,
        collection_id: i64,
        type_: &str,
    ) -> Result<Vec<MediaInfoOverview>> {
        let mut rows = sqlx::query!(
            r#"
                SELECT i.id AS "id!: Id",
                       i.title,
                       i.thumbs AS "thumbs!: JVec<Thumb>"
                FROM mediaitems i
                WHERE i.collection_id = ? AND i.type = ? AND i.deleted = 0
                ORDER BY LOWER( title)"#,
            collection_id,
            type_,
        )
        .fetch(dbh);

        let mut items = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let poster = row.thumbs.0.iter().find(|t| t.aspect == "poster").cloned();
            if let Some(title) = row.title {
                items.push(MediaInfoOverview { id: row.id, title, poster });
            }
        }

        Ok(items)
    }
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct MediaInfo {
    /// TVShow or Movie id
    pub id: Id,
    /// Title.
    pub title: String,
    /// Thumbnail in poster aspect (if available)
    pub thumbs: JVec<Thumb>,
    /// Directory.
    pub directory: FileInfo,
}

impl MediaInfo {
    pub async fn get(dbh: &db::DbHandle, id: Id) -> Result<Option<MediaInfo>> {
        let row = sqlx::query!(
            r#"
                SELECT  id AS "id!: Id",
                        title,
                        thumbs AS "thumbs!: JVec<Thumb>",
                        directory AS "directory!: FileInfo"
                FROM mediaitems
                WHERE id = ?"#,
            id
        )
        .fetch_optional(dbh)
        .await?;

        let m = some_or_return!(row, Ok(None));
        Ok(m.title.map(|title| MediaInfo {
            id: m.id,
            title,
            thumbs: m.thumbs,
            directory: m.directory,
        }))
    }
}
