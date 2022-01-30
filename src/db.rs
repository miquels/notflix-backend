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

pub async fn get_items(handle: &DbHandle) -> Vec<Item> {
    use schema::items::dsl;

    let conn = handle.get().await.unwrap();
    dsl::items
        .load::<Item>(&*conn)
        .expect("Error loading items")
}

