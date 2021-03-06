use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::{AddExtensionLayer, Router, routing::get};
use axum::{body::Body, extract::ConnectInfo, response::Response};
use futures_core::future::BoxFuture;
use http_body::Body as _;
use http::{Method, Request};
use tower::{Service, ServiceBuilder, layer::layer_fn};
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
        .layer(layer_fn(|inner| Logger { inner }))
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

#[derive(Clone)]
struct Logger<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for Logger<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        // store request data.
        let start = std::time::Instant::now();
        let now = time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc());
        let pnq = req.uri().path_and_query().map(|p| p.to_string()).unwrap_or(String::from("-"));
        let addr = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|s| s.0.to_string())
            .unwrap_or(String::from("-"));
        let method = req.method().clone();
        let version = req.version();

        Box::pin(async move {
            let resp: Response = inner.call(req).await?;

            // log request + response status / size / elapsed.
            let size = resp.body().size_hint().exact().unwrap_or(0);
            println!(
                "{} {} \"{} {} {:?}\" {} {} {:?}",
                now,
                addr,
                method,
                pnq,
                version,
                resp.status().as_u16(),
                size,
                start.elapsed(),
            );

            Ok(resp)
        })
    }
}
