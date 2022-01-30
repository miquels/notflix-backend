use notflix_backend::db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let handle = db::connect_db("test.db").await?;

    let items = db::get_items(&handle).await;
    println!("{}", serde_json::to_string_pretty(&items)?);

    Ok(())
}
