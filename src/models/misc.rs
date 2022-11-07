use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::db::DbHandle;
use super::is_default;
use super::fileinfo::FileInfo;

#[derive(Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Actor {
    #[serde(skip_serializing_if = "is_default")]
    pub name:   Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub role:   Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub order: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub thumb:  Option<Thumb>,
    #[serde(skip_serializing_if = "is_default")]
    pub thumb_url:  Option<String>,
}

/// Image
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Thumb {
    #[serde(rename(deserialize = "$value"))]
    pub path:     String,
    pub aspect:   String,
    #[serde(skip_serializing_if = "is_default")]
    pub season:  Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Rating {
    #[serde(skip_serializing_if = "is_default")]
    pub name:   Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub default: Option<bool>,
    #[serde(skip_serializing_if = "is_default")]
    pub max:    Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    pub value:    Option<f32>,
    #[serde(skip_serializing_if = "is_default")]
    pub votes:    Option<u32>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, PartialEq)]
#[serde(default)]
pub struct UniqueId {
    #[serde(rename = "type")]
    pub idtype: Option<String>,
    pub default: bool,
    pub id: String,
}

fn has_uid(uids: &Vec<UniqueId>, idtype: &str, id: &str) -> bool {
    for uid in uids {
        let uid_idtype = uid.idtype.as_ref().map(|s| s.as_str()).unwrap_or("");
        if uid_idtype == idtype && uid.id == id {
            return true;
        }
    }
    false
}

#[derive(Default)]
pub struct FindItemBy<'a> {
    pub id: Option<i64>,
    pub imdb: Option<&'a str>,
    pub tmdb: Option<&'a str>,
    pub tvdb: Option<&'a str>,
    pub title: Option<&'a str>,
    pub directory: Option<&'a str>,
}


impl<'a> FindItemBy<'a> {

    pub fn new() -> FindItemBy<'a> {
        FindItemBy::default()
    }

    pub(crate) fn is_only_id(&self) -> Option<i64> {
        if let Some(id) = self.id {
            if self.imdb.is_none() &&
                self.imdb.is_none() &&
                self.tmdb.is_none() &&
                self.tvdb.is_none() &&
                self.title.is_none() &&
                self.directory.is_none() {
                return Some(id);
            }
        }
        None
    }

    // TODO: if multiple matches, return the one we trust most (a match on 'id'
    //       has 100% trust, ofcourse).
    //       Later, return a Vec of (id, matched_on, trust) instead of just one value.
    pub(crate) async fn lookup(&self, dbh: &DbHandle) -> Option<i64> {

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
            id: i64,
            directory: sqlx::types::Json<FileInfo>,
            title: Option<String>,
            pub uniqueids: sqlx::types::Json<Vec<UniqueId>>,
        }
        let mut rows = sqlx::query_as!(
            Result,
            r#"
                SELECT  i.id,
                        i.directory AS "directory: _",
                        i.title, 
                        i.uniqueids AS "uniqueids: _"
                FROM mediaitems i"#,
        )
        .fetch(dbh);

        // Inspect each row. Could do this in SQL, but we might want to
        // compare directory and/or title in a fuzzy way.
        while let Some(row) = rows.try_next().await.unwrap_or(None) {
            let mut res = false;
            res |= self.id.map(|x| x == row.id).unwrap_or(false);
            res |= self.imdb.map(|x| has_uid(&row.uniqueids, "imdb", x)).unwrap_or(false);
            res |= self.tmdb.map(|x| has_uid(&row.uniqueids, "tmdb", x)).unwrap_or(false);
            res |= self.tvdb.map(|x| has_uid(&row.uniqueids, "tvdb", x)).unwrap_or(false);
            res |= self.directory.map(|x| x == row.directory.path).unwrap_or(false);
            let title = row.title.as_ref().map(|p| p.as_str());
            res |= self.title.is_some() && self.title == title;
            if res {
                return Some(row.id);
            }
        }
        None
    }
}
