use std::sync::Arc;
use axum::{
    routing::get,
    AddExtensionLayer,
    Router
};
use tower::ServiceBuilder;

use crate::api;
use crate::db::DbHandle;
use crate::config::Config;

#[derive(Clone)]
pub struct SharedState {
    pub db: DbHandle,
    pub config: Arc<Config>,
}

pub async fn serve(cfg: Config, db: DbHandle) -> anyhow::Result<()> {

    let state = SharedState { db, config: Arc::new(cfg) };
    let addr = state.config.server.addrs[0];

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!\n" }))
        .nest("/api", api::routes())
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(state))
        );

    println!("listening on {}", addr);

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
