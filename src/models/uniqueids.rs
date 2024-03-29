use anyhow::Result;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::models::UniqueId;
use crate::sqlx::impl_sqlx_traits_for;
use crate::util::Id;

#[derive(Object, Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct UniqueIds {
    #[oai(read_only)]
    pub mediaitem_id: Id,
}
impl_sqlx_traits_for!(UniqueIds);

impl UniqueIds {
    pub fn new(mediaitem_id: Id) -> UniqueIds {
        // println!("UniqueIds::new({mediaitem_id})");
        UniqueIds { mediaitem_id, ..UniqueIds::default() }
    }

    pub async fn get_mediaitem_id(
        dbh: &mut db::TxnHandle<'_>,
        uids: &[UniqueId],
    ) -> Result<Option<Id>> {
        if uids.len() == 0 {
            return Ok(None);
        }

        // Well, this is ugly, but I don't know a better way.

        // First, build the query.
        let mut query_str = String::from(
            r#"SELECT mediaitem_id AS "mediaitem_id!: Id"
               FROM uniqueids"#,
        );
        for idx in 0..uids.len() {
            if idx == 0 {
                query_str.push_str(" WHERE ");
            } else {
                query_str.push_str(" OR ");
            }
            query_str.push_str("idtype = ? AND uniqueid = ?");
        }

        // Now build the basic query and bind the args.
        let mut query = sqlx::query_as::<_, (Id,)>(&query_str);
        for uid in uids {
            query = query.bind(&uid.idtype);
            query = query.bind(&uid.id);
        }

        // And execute it.
        let rows = query.fetch_all(dbh).await?;

        if rows.len() == 0 {
            return Ok(None);
        }

        // FIXME: check if there's only one unique mediaitem_id.
        Ok(Some(rows[0].0.clone()))
    }

    pub async fn update(&self, txn: &mut db::TxnHandle<'_>, uids: &[UniqueId]) -> Result<()> {
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
            .execute(&mut *txn)
            .await?;
        }

        Ok(())
    }
}
