// use serde::Serialize;

use super::schema::{images, rsimages};

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "images"]
pub struct Image {
    pub id: i64,
    pub ino: i64,
    pub dev: i64,
    pub size: i64,
    pub mtime: i64,
    pub width: i32,
    pub height: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "images"]
pub struct NewImage {
    pub ino: i64,
    pub dev: i64,
    pub size: i64,
    pub mtime: i64,
    pub width: i32,
    pub height: i32,
}

#[derive(Identifiable, Queryable, Associations, Debug)]
#[belongs_to(Image)]
#[table_name = "rsimages"]
pub struct RsImage {
    pub id: i64,
    pub image_id: i64,
    pub width: i32,
    pub height: i32,
    pub quality: i32,
    pub path: String,
}

#[derive(Insertable, Debug)]
#[table_name = "rsimages"]
pub struct NewRsImage<'a> {
    pub image_id: i64,
    pub width: i32,
    pub height: i32,
    pub quality: i32,
    pub path: &'a str,
}
