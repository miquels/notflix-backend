use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;

use anyhow::{Context, Result};
use diesel::prelude::*;

pub mod models;
pub mod schema;
use models::*;

type ConnectionManager = bb8_diesel::DieselConnectionManager::<SqliteConnection>;
// type ConnectionError = <ConnectionManager as bb8::ManageConnection>::Error;

// Pool handle.
pub type DbHandle = bb8::Pool<ConnectionManager>;

// Define last_insert_rowid helper.
no_arg_sql_function!(
    last_insert_rowid,
    diesel::sql_types::BigInt,
    "Represents the SQL last_insert_row() function"
);

// Embed migrations.
embed_migrations!();

//
// Connect to the database and return a pool handle.
// Every time we need to make a query, we request a connection via the handle.
//
pub async fn connect_db(db: &str) -> Result<DbHandle> {
    let manager = bb8_diesel::DieselConnectionManager::<SqliteConnection>::new(db);
    let handle = bb8::Pool::builder().build(manager).await?;
    {
        let conn = handle.get().await.unwrap();
        embedded_migrations::run(&*conn).with_context(|| format!("migrating database {}", db))?;
    }
    Ok(handle)
}

//
// Images.
//
use schema::images::dsl::{self as img, images};
use schema::rsimages::dsl::rsimages;

// Wrapping conversion because sqlite doesn't have u64, and
// in some cases 'dev' or 'ino' might have the high bit set.
fn to_i64(n: u64) -> i64 {
    if n > i64::MAX as u64 {
        -((n ^ u64::MAX) as i64) - 1
    } else {
        n as i64
    }
}

// placeholder.
pub async fn get_items(_handle: &DbHandle) -> Vec<()> {
    Vec::new()
}

pub async fn get_image(handle: &DbHandle, m: &Metadata) -> Result<Option<(Image, Vec<RsImage>)>> {
    let (ino, dev, size, mtime) = (to_i64(m.ino()), to_i64(m.dev()), to_i64(m.size()), m.mtime());

    let conn = handle.get().await?;
    let image = images
        .filter(img::ino.eq(ino).and(img::dev.eq(dev)).and(img::size.eq(size)).and(img::mtime.eq(mtime)))
        .first::<Image>(&*conn)
        .optional()?;
    let image = match image {
        Some(img) => img,
        None => return Ok(None),
    };

    let resized = RsImage::belonging_to(&image).get_results(&*conn)?;
    /*
    let resized = rsimages
        .filter(rsimg::image_id.eq(image.id))
        .load(&*conn)?;
    */

    Ok(Some((image, resized)))
}

pub async fn put_image(handle: &DbHandle, image: NewImage) -> Result<i64> {
    let conn = handle.get().await?;
    let id = conn.transaction(|| {
        diesel::insert_into(images)
            .values(image)
            .execute(&*conn)?;
        diesel::select(last_insert_rowid).get_result::<i64>(&*conn)
    })?;

    Ok(id)
}

pub async fn put_rs_image(handle: &DbHandle, image: NewRsImage<'_>) -> Result<()> {
    let conn = handle.get().await?;
    conn.transaction(|| {
        diesel::insert_into(rsimages)
            .values(image)
            .execute(&*conn)
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio::fs;
    // use diesel::prelude::*;
    use super::*;
    // use super::models::*;

    async fn get_test_dbhandle()-> DbHandle {
        let handle = connect_db(":memory:").await.unwrap();
        {
            let conn = handle.get().await.unwrap();
            embedded_migrations::run(&*conn).unwrap();
        }
        handle
    }

    #[tokio::test(flavor = "multi_thread")]
    pub async fn migrate() {
        let handle = get_test_dbhandle().await;
        let file = fs::File::open("/etc/hosts").await.unwrap();
        let meta = file.metadata().await.unwrap();
        let res = get_image(&handle, &meta).await.unwrap();
        assert!(res.is_none());
        let image = NewImage {
            ino:    to_i64(meta.ino()),
            dev:    to_i64(meta.dev()),
            size:   to_i64(meta.size()),
            mtime:  meta.mtime(),
            width:  1920,
            height: 1080,
        };
        let id = put_image(&handle, image).await.unwrap();
        assert!(id > 0);
        let res = get_image(&handle, &meta).await.unwrap();
        if res.as_ref().unwrap().0.id != id {
            panic!("assertion failed, res.unwrap().0.id != Some(id), {:?}, {:?}", res, id);
        }
    }
}

