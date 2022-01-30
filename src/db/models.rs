use serde::Serialize;

use super::schema::items;

#[derive(Serialize, Queryable)]
pub struct Item {
    pub name: String,
    pub votes: Option<i64>,
    pub year: Option<i64>,
    pub genre: String,
    pub rating: Option<f32>,
    pub nfotime: i64,
    pub firstvideo: i64,
    pub lastvideo: i64,
}

#[derive(Insertable)]
#[table_name = "items"]
pub struct NewItem<'a> {
    pub name: &'a str,
    pub votes: Option<i64>,
    pub year: Option<i64>,
    pub genre: &'a str,
    pub rating: Option<f32>,
    pub nfotime: i64,
    pub firstvideo: i64,
    pub lastvideo: i64,
}
