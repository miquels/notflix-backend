use std::io;

use anyhow::Result;
use axum::{body, response::Response, Router, routing::get};
use axum::extract::{Extension, Path};
use headers::{HeaderMapExt, UserAgent};
use http::{Request, StatusCode};

use mp4lib::streaming::http_handler::{self, FsPath};

use crate::server::SharedState;

async fn handle_request(
    Path((coll, path)): Path<(u32, String)>,
    Extension(state): Extension<SharedState>,
    req: Request<body::Body>,
) -> Result<Response, StatusCode> {

    // Lose the request body.
    let (parts, _) = req.into_parts();
    let req = Request::from_parts(parts, ());

    // Find collection.
    let coll = match state.config.collections.iter().find(|c| c.collection_id == coll) {
        Some(coll) => coll,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Handle request.
    handle_request2(&path, &coll.directory, &req).await.map_err(|e| translate_io_error(e))
}

async fn handle_request2(
    path: &str,
    dir: &str,
    req: &Request<()>
) -> io::Result<Response> {

    let path = FsPath::Combine((dir, path));

    // See if this is the Notflix custom receiver running on Chromecast.
    let is_notflix = match req.headers().get("x-application").map(|v| v.to_str()) {
        Some(Ok(v)) => v.contains("Notflix"),
        _ => false,
    };
    // Is it a chromecast?
    let is_cast = match req.headers().typed_get::<UserAgent>() {
        Some(ua) => ua.as_str().contains("CrKey/"),
        None => false,
    };
    // Chromecast and not Notflix, filter subs.
    let filter_subs = is_cast && !is_notflix;

    if let Some(response) = http_handler::handle_hls(&req, path, filter_subs).await? {
        return Ok(response);
    }

    if let Some(response) = http_handler::handle_pseudo(&req, path).await? {
        return Ok(response);
    }

    http_handler::handle_file(&req, path, None).await
}

fn translate_io_error(err: io::Error) -> StatusCode {
    use http::StatusCode as SC;
    match err.kind() {
        io::ErrorKind::NotFound => SC::NOT_FOUND,
        io::ErrorKind::PermissionDenied => SC::FORBIDDEN,
        io::ErrorKind::TimedOut => SC::REQUEST_TIMEOUT,
        _ => {
            let e = err.to_string();
            let field = e.split_whitespace().next().unwrap();
            if let Ok(status) = field.parse::<u16>() {
                SC::from_u16(status).unwrap_or(SC::INTERNAL_SERVER_ERROR)
            } else {
                SC::INTERNAL_SERVER_ERROR
            }
        },
    }
}

pub fn routes() -> Router {
    Router::new()
        .route("/:coll/*path", get(handle_request))
}
