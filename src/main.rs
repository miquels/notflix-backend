#[macro_use]
extern crate diesel;

mod db;
mod genres;
mod nfo;
mod parsefilename;

type Result<T, E = Box<dyn std::error::Error + 'static>> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
    let handle = db::connect_db("test.db").await?;

    let items = db::get_items(&handle).await;
    println!("{}", serde_json::to_string_pretty(&items)?);

    Ok(())
}
