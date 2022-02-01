use axum::{routing::get, Router};
use std::net::SocketAddr;

pub async fn serve(port: u16) -> anyhow::Result<()> {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("listening on {}", addr);
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
