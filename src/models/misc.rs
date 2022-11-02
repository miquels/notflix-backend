use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::db::DbHandle;
use super::SqlU64;

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Actor {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Thumb {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Fanart {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Ratings {
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct UniqueIds {
    pub id: SqlU64,
    pub uniqueids: Vec<UniqueId>,
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct UniqueId {
    #[serde(rename = "type")]
    pub idtype: String,
    pub default: bool,
    pub id: String,
}

impl UniqueIds {
    pub fn has(&self, idtype: &str, id: &str) -> bool {
        for uid in &self.uniqueids {
            if uid.idtype == idtype && uid.id == id {
                return true;
            }
        }
        false
    }
}

#[derive(Default)]
pub struct FindItemBy<'a> {
    pub id: Option<SqlU64>,
    pub imdb: Option<&'a str>,
    pub tmdb: Option<&'a str>,
    pub tvdb: Option<&'a str>,
    pub title: Option<&'a str>,
    pub path: Option<&'a str>,
}


impl<'a> FindItemBy<'a> {

    pub(crate) fn new() -> FindItemBy<'a> {
        FindItemBy::default()
    }

    pub(crate) fn is_only_id(&self) -> Option<SqlU64> {
        if let Some(id) = self.id {
            if self.imdb.is_none() &&
                self.imdb.is_none() &&
                self.tmdb.is_none() &&
                self.tvdb.is_none() &&
                self.title.is_none() &&
                self.path.is_none() {
                return Some(id);
            }
        }
        None
    }

    // TODO: if multiple matches, return the one we trust most (a match on 'id'
    //       has 100% trust, ofcourse).
    //       Later, return a Vec of (id, matched_on, trust) instead of just one value.
    pub(crate) async fn lookup(&self, dbh: &DbHandle) -> Option<SqlU64> {

        // If we match on id, return right away.
        // It's basically just a test 'is this entry in the db'.
        if let Some(id) = self.id {
            let row = sqlx::query!(
                r#"
                    SELECT i.id
                    FROM mediaitems i
                    WHERE i.id == ?"#,
                id
            )
            .fetch_one(dbh)
            .await
            .ok();
            if row.is_some() {
                return Some(id);
            }
        }

        #[derive(sqlx::FromRow)]
        struct Result {
            id: SqlU64,
            path: String,
            title: Option<String>,
            pub uniqueids: sqlx::types::Json<UniqueIds>,
        }
        let mut rows = sqlx::query_as!(
            Result,
            r#"
                SELECT  i.id,
                        i.path, i.title, 
                        i.uniqueids AS "uniqueids: _"
                FROM mediaitems i"#,
        )
        .fetch(dbh);

        // Inspect each row. Could do this in SQL, but we might want to
        // compare path and/or title in a fuzzy way.
        while let Some(row) = rows.try_next().await.unwrap_or(None) {
            let mut res = false;
            res |= self.id.map(|x| x == row.id).unwrap_or(false);
            res |= self.imdb.map(|x| row.uniqueids.has("imdb", x)).unwrap_or(false);
            res |= self.tmdb.map(|x| row.uniqueids.has("tmdb", x)).unwrap_or(false);
            res |= self.tvdb.map(|x| row.uniqueids.has("tvdb", x)).unwrap_or(false);
            res |= self.path.map(|x| x == row.path).unwrap_or(false);
            let title = row.title.as_ref().map(|p| p.as_str());
            res |= self.title.is_some() && self.title == title;
            if res {
                return Some(row.id);
            }
        }
        None
    }
}
