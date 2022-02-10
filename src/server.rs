use std::sync::Arc;

use axum::{AddExtensionLayer, Router, routing::get};
use http::Method;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::cors::{self, CorsLayer};

use crate::api;
use crate::data;
use crate::db::DbHandle;
use crate::config::Config;

#[derive(Clone)]
pub struct SharedState {
    pub db: DbHandle,
    pub config: Arc<Config>,
}

pub async fn serve(cfg: Config, db: DbHandle) -> anyhow::Result<()> {
    use http::header::{HeaderName, ORIGIN, RANGE};

    let state = SharedState { db, config: Arc::new(cfg) };
    let addr = state.config.server.addrs[0];

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(state))
        .layer(CorsLayer::new()
            .allow_origin(cors::any())
            .allow_methods(vec![Method::GET, Method::HEAD])
            .allow_headers(vec![HeaderName::from_static("x-application"), ORIGIN, RANGE ])
            .expose_headers(cors::any())
            .max_age(std::time::Duration::from_secs(86400)));

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!\n" }))
        .nest("/api", api::routes())
        .nest("/data", data::routes())
        .layer(middleware_stack);

    println!("listening on {}", addr);

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
