use anyhow::Result;

use crate::db::Db;
use crate::models::UniqueId;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct UniqueIds {
    pub mediaitem_id: i64,
}

impl UniqueIds {
    pub fn new(mediaitem_id: i64) -> UniqueIds {
        // println!("UniqueIds::new({mediaitem_id})");
        UniqueIds {
            mediaitem_id,
            ..UniqueIds::default()
        }
    }

    pub async fn get_mediaitem_id(db: &Db, uids: &[UniqueId]) -> Option<i64> {

        if uids.len() == 0 {
            return None;
        }

        // Well, this is ugly, but I don't know a better way.

        // First, build the query.
        let mut query_str = String::from(
            r#"SELECT mediaitem_id
               FROM uniqueids"#
        );
        for idx in 0 .. uids.len() {
            if idx == 0 {
                query_str.push_str(" WHERE ");
            } else {
                query_str.push_str(" OR ");
            }
            query_str.push_str("idtype = ? AND uniqueid = ?");
        }

        // Now build the basic query and bind the args.
        let mut query = sqlx::query_as::<_, (i64,)>(&query_str);
        for uid in uids {
            query = query.bind(&uid.idtype);
            query = query.bind(&uid.id);
        }

        // And execute it.
        let rows = query
            .fetch_all(&db.handle)
            .await
            .ok()?;

        if rows.len() == 0 {
            return None;
        }

        // FIXME: check if there's only one unique mediaitem_id.
        Some(rows[0].0)
    }

    pub async fn update(&self, db: &Db, uids: &[UniqueId]) -> Result<()> {

        // XXX TODO could probably be smarter about this.
        for uid in uids {
            sqlx::query!(
                r#"
                    INSERT INTO uniqueids(
                        mediaitem_id,
                        idtype,
                        uniqueid,
                        is_default
                    ) VALUES(?, ?, ?, ?)
                    ON CONFLICT DO NOTHING"#,
                self.mediaitem_id,
                uid.idtype,
                uid.id,
                uid.default
            )
            .execute(&db.handle)
            .await?;
        }

        Ok(())
    }
}
