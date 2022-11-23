use anyhow::Result;
use futures_util::TryStreamExt;
use crate::db;
use crate::models::Thumb;
use crate::jvec::JVec;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct MediaInfo {
    /// TVShow or Movie id
    pub id: i64,
    /// Title.
    pub title: String,
    /// Thumbnail in poster aspect (if available)
    pub poster: Option<Thumb>,
}

impl MediaInfo {
    pub async fn get_all(dbh: &db::DbHandle, collection_id: i64, type_: &str) -> Result<Vec<MediaInfo>> {
        let mut rows = sqlx::query!(
            r#"
                SELECT i.id AS "id!: i64",
                       i.title,
                       i.thumbs AS "thumbs!: JVec<Thumb>"
                FROM mediaitems i
                WHERE i.collection_id = ? AND i.type = ?
                ORDER BY LOWER( title)"#,
            collection_id,
            type_,
        )
        .fetch(dbh);

        let mut items = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let poster = row.thumbs.0.iter().find(|t| t.aspect == "poster").cloned();
            if let Some(title) = row.title {
                items.push(MediaInfo {
                    id: row.id,
                    title,
                    poster,
                });
            }
        }

        Ok(items)
    }
}
