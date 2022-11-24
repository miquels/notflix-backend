use std::io;

use poem::{
    error::Error,
    http::{Request as HttpRequest, StatusCode},
    web::headers::{HeaderMapExt, UserAgent},
    web::{Data, Path},
    Request, Response, Result, Route,
    handler, get,
};

use mp4lib::streaming::http_handler::{self, FsPath};

use crate::server::SharedState;

#[handler]
async fn handle_request(
    Path((coll_id, path)): Path<(u32, String)>,
    Data(state): Data<&SharedState>,
    req: &Request,
) -> Result<Response> {

    // Find collection.
    let coll = match state.config.get_collection(coll_id) {
        Some(coll) => coll,
        None => return Err(Error::from_status(StatusCode::NOT_FOUND)),
    };

    // Handle request.
    let req = poem_req_to_http_req(req);
    handle_request2(&path, &coll.directory, &req).await.map_err(|e| translate_io_error(e))
}

async fn handle_request2(
    path: &str,
    dir: &str,
    req: &HttpRequest<()>
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
        return Ok(response.into());
    }

    if let Some(response) = http_handler::handle_pseudo(&req, path).await? {
        return Ok(response.into());
    }

    let response = http_handler::handle_file(&req, path, None).await?;
    Ok(response.into())
}

fn translate_io_error(err: io::Error) -> Error {
    use StatusCode as SC;
    let status = match err.kind() {
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
    };
    Error::from_status(status)
}

pub fn routes() -> Route {
    Route::new().at("/:coll/*path", get(handle_request))
}

fn poem_req_to_http_req(req: &poem::Request) -> poem::http::Request<()> {
    let mut http_req = poem::http::Request::builder()
        .method(req.method())
        .uri(req.uri())
        .version(req.version());

    for (name, val) in req.headers().iter() {
        http_req = http_req.header(name.clone(), val.clone())
    }

    http_req.body(()).unwrap()
}
