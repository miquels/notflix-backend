use axum::{
    extract::Extension,
    extract::Path,
    http::StatusCode,
    http::header::{HeaderMap, HeaderName, HeaderValue},
    response::Json,
    routing::get,
    Router,
};
use serde_json::{Value, json};

use crate::server::SharedState;

type JsonResponse = (HeaderMap, String);
fn to_json<T>(value: &T) -> JsonResponse
where
    T: ?Sized + serde::Serialize,
{
    let mut hm = HeaderMap::new();
    hm.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );
    (hm, serde_json::to_string_pretty(value).unwrap() + "\n")
}

// /api/collections
async fn get_collections(
    Extension(state): Extension<SharedState>,
) -> JsonResponse {
    to_json(&state.config.collections)
}

// /api/collection/:coll
async fn get_collection(
    Path(coll): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<JsonResponse, StatusCode> {
    state.config.collections
        .iter()
        .find(|c| coll == c.name)
        .map(to_json)
        .ok_or(StatusCode::NOT_FOUND)
}

// /api/collection/:coll/genres
async fn get_genres(
    Path(_coll): Path<String>,
    Extension(_state): Extension<SharedState>,
) -> Json<Value> {
    Json(json!({ "genre": "none" }))
}

// /api/collection/:coll/items
async fn get_items(
    Path(coll): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<JsonResponse, StatusCode> {
    let coll = state
        .config
        .collections
        .iter()
        .find(|c| coll == c.name);
    match coll {
        Some(coll) => Ok(to_json(&coll.get_items().await)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// /api/collection/:coll/item/:item
async fn get_item(
    Path((coll, item)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<JsonResponse, StatusCode> {
    let coll = state
        .config
        .collections
        .iter()
        .find(|c| coll == c.name);
    println!("get_item: coll: {:?}", coll.map(|c| &c.name));

    if let Some(coll) = coll {
        println!("get_item: {}", item);
        if let Some(item) = coll.get_item(&item).await {
            return  Ok(to_json(&*item));
        }
    }
    Err(StatusCode::NOT_FOUND)
}

pub fn routes() -> Router {
    Router::new()
        .route("/collections", get(get_collections))
        .route("/collection/:coll", get(get_collection))
        .route("/collection/:coll/genres", get(get_genres))
        .route("/collection/:coll/items", get(get_items))
        .route("/collection/:coll/item/:item", get(get_item))
}

