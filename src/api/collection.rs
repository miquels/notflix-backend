use anyhow::Result;
use poem_openapi::{
    payload::Json,
    ApiResponse,
    Object,
};
use super::Api;
use crate::models;
use crate::util::Id;

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
    #[oai(read_only)]
    pub id:     Id,
    /// Title
    pub title:  String,
    /// Thumbnail
    pub poster: Option<models::Thumb>,
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
        let mut items = models::MediaInfoOverview::get(&self.state.db.handle, coll.collection_id as i64, coll.subtype()).await?;
        let m = items.drain(..).map(|i| {
                MediaItem {
                    id: i.id,
                    title: i.title,
                    poster: i.poster,
                }
            })
            .collect::<Vec<_>>();
        Ok(GetThumbsResponse::Ok(Json(m)))
    }
}
