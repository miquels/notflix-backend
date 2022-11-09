use anyhow::Result;
use serde::Serialize;

use crate::db::Db;

#[derive(Serialize, Clone, Default, Debug, PartialEq)]
#[serde(default)]
pub struct UniqueIds{
    #[serde(skip)]
    pub mediaitem_id: i64,
    pub ids: Vec<UniqueId>
}

#[derive(Serialize, Clone, Default, Debug, PartialEq)]
#[serde(default)]
pub struct UniqueId {
    #[serde(skip)]
    pub id: i64,
    #[serde(skip)]
    pub mediaitem_id: i64,
    pub idtype: String,
    pub default: bool,
    pub uniqueid: String,
}

impl UniqueIds {
    #[allow(dead_code)]
    pub async fn select(db: &Db, mediaitem_id: i64) -> Option<UniqueIds> {
        let ids = sqlx::query_as!(
            UniqueId,
            r#"SELECT id, mediaitem_id, idtype, is_default AS "default: bool", uniqueid
               FROM uniqueids
               WHERE mediaitem_id = ?"#,
            mediaitem_id
        )
        .fetch_all(&db.handle)
        .await
        .ok()?;
        if ids.len() == 0 {
            return None;
        }
        Some(UniqueIds {
            mediaitem_id,
            ids,
        })
    }

    #[allow(dead_code)]
    pub async fn insert(&mut self, db: &Db) -> Result<()> {
        self.update(db).await
    }

    #[allow(dead_code)]
    pub async fn update(&mut self, db: &Db) -> Result<()> {

        // First delete
        sqlx::query!(
            "DELETE FROM uniqueids WHERE mediaitem_id = ?",
            self.mediaitem_id
        )
        .execute(&db.handle)
        .await?;

        // Then re-insert all.
        for uid in &mut self.ids {
            uid.mediaitem_id = self.mediaitem_id;
            uid.id = sqlx::query!(
                r#"
                    INSERT INTO uniqueids(
                        mediaitem_id,
                        idtype,
                        is_default,
                        uniqueid
                    ) VALUES(?, ?, ?, ?)"#,
                uid.mediaitem_id,
                uid.idtype,
                uid.default,
                uid.uniqueid,
            )
            .execute(&db.handle)
            .await?
            .last_insert_rowid();
        }

        Ok(())
    }
}
