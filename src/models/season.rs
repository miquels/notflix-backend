use anyhow::Result;
use serde::Serialize;
use crate::db::DbHandle;
use super::misc::Thumb;
use super::{SqlU32, SqlU64, is_default};

#[derive(Serialize, Default, Debug, sqlx::FromRow)]
#[serde(default)]
pub struct Season {
    #[serde(skip_serializing)]
    pub id: SqlU64,
    #[serde(skip_serializing)]
    pub collection_id: SqlU64,
    #[serde(skip_serializing_if = "is_default")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub thumb: sqlx::types::Json<Vec<Thumb>>,
    pub tvshow_id: SqlU64,
    pub season: SqlU32,
}

impl Season {
    pub async fn select_one(dbh: &DbHandle, id: SqlU64) -> Option<Season> {
        sqlx::query_as!(
            Season,
            r#"
                SELECT  i.id,
                        i.collection_id,
                        i.path,
                        i.title,
                        i.thumb AS "thumb: _",
                        m.tvshow_id,
                        m.season
                FROM mediaitems i
                JOIN seasons m ON m.tvshow_id = i.id
                WHERE i.id = ? AND i.deleted = 0"#,
            id,
        )
        .fetch_one(dbh)
        .await
        .ok()
    }

    pub async fn insert(&mut self, dbh: &DbHandle) -> Result<()> {
        self.id = sqlx::query!(
            r#"
                INSERT INTO mediaitems(
                    collection_id,
                    path,
                    title,
                    type,
                    thumb
                ) VALUES(?, ?, ?, "season", ?)"#,
            self.collection_id,
            self.path,
            self.title,
            self.thumb
        )
        .execute(dbh)
        .await?
        .last_insert_rowid();

        sqlx::query!(
            r#"
                INSERT INTO seasons(
                    tvshow_id,
                    season
                ) VALUES(?, ?)"#,
            self.tvshow_id,
            self.season,
        )
        .execute(dbh)
        .await?;

        Ok(())
    }

    pub async fn update(&mut self, dbh: &DbHandle) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE mediaitems SET
                    collection_id = ?,
                    path = ?,
                    title = ?,
                    thumb = ?
                WHERE id = ?"#,
            self.collection_id,
            self.path,
            self.title,
            self.thumb,
            self.id
        )
        .execute(dbh)
        .await?;

        sqlx::query!(
            r#"
                UPDATE seasons SET
                    season = ?
                WHERE tvshow_id = ?"#,
            self.season,
            self.tvshow_id,
        )
        .execute(dbh)
        .await?;

        Ok(())
    }
}
