use std::time::SystemTime;

use diesel::prelude::*;

pub mod models;
pub mod schema;
use models::*;

type ConnectionManager = bb8_diesel::DieselConnectionManager::<SqliteConnection>;
type ConnectionError = <ConnectionManager as bb8::ManageConnection>::Error;

pub type DbHandle = bb8::Pool<ConnectionManager>;

pub async fn connect_db(db: &str) -> Result<DbHandle, ConnectionError> {
    let manager = bb8_diesel::DieselConnectionManager::<SqliteConnection>::new(db);
    bb8::Pool::builder().build(manager).await
}

fn systemtime_to_ms(tm: SystemTime) -> u64 {
    tm.duration_since(SystemTime::UNIX_EPOCH).map(|t| t.as_millis()).unwrap_or(0) as u64
}

pub async fn get_items(handle: &DbHandle) -> Vec<Item> {
    use schema::items::dsl;

    let conn = handle.get().await.unwrap();
    dsl::items
        .load::<Item>(&*conn)
        .expect("Error loading items")
}

