use anyhow::Result;
use poem_openapi::{
    payload::Json,
    ApiResponse,
    Object,
};
use super::Api;
use crate::models;

pub use crate::collections::Collection;

#[derive(ApiResponse)]
pub enum GetCollectionsResponse<'a> {
    /// Returns when the collections are listed.
    #[oai(status = 200)]
    Ok(Json<&'a Vec<Collection>>),

    /// Return when there are no collections.
    #[oai(status = 404)]
    NotFound,
}

/// One tvshow or movie.
#[derive(Object)]
pub struct MediaItem {
    /// Unique ID
    pub id:     i64,
    /// Title
    pub title:  String,
    /// URL to the thumbnail
    pub poster: Option<String>,
}

#[derive(ApiResponse)]
pub enum GetThumbsResponse {
    /// Return when the collections are listed.
    #[oai(status = 200)]
    Ok(Json<Vec<MediaItem>>),

    /// Return when there are no collections.
    #[oai(status = 404)]
    NotFound,
}

impl Api {
    pub async fn get_collections(&self) -> Result<GetCollectionsResponse> {
        if self.state.config.collections.is_empty() {
            Ok(GetCollectionsResponse::NotFound)
        } else {
            Ok(GetCollectionsResponse::Ok(Json(&self.state.config.collections)))
        }
    }

    pub async fn get_thumbs(&self, collection_id: i64) -> Result<GetThumbsResponse> {
        let collections = &self.state.config.collections;
        let coll = match collections.iter().find(|c| c.collection_id as i64 == collection_id) {
            Some(coll) => coll,
            None => return Ok(GetThumbsResponse::NotFound),
        };
        let mut items = models::MediaInfo::get_all(&self.state.db.handle, coll.collection_id as i64).await?;
        let m = items.drain(..).map(|i| {
                let poster = i.poster.map(|_| format!("/api/images/{}/{}/poster.jpg", collection_id, i.id));
                MediaItem {
                    id: i.id,
                    title: i.title,
                    poster,
                }
            })
            .collect::<Vec<_>>();
        Ok(GetThumbsResponse::Ok(Json(m)))
    }
}
