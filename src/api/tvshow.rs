use anyhow::Result;
use poem_openapi::{
    payload::Json,
    ApiResponse,
};
use super::Api;
use crate::db::FindItemBy;

pub use crate::models::TVShow;

#[derive(ApiResponse)]
pub enum GetTVShowResponse {
    /// Return when the collections are listed.
    #[oai(status = 200)]
    Ok(Json<Box<TVShow>>),

    /// Return when there are no collections.
    #[oai(status = 404)]
    NotFound,
}

impl Api {
    pub async fn get_tvshow(&self, collection_id: i64, tvshow_id: i64) -> Result<GetTVShowResponse> {
        let collections = &self.state.config.collections;
        let _coll = match collections.iter().find(|c| c.collection_id as i64 == collection_id) {
            Some(coll) => coll,
            None => return Ok(GetTVShowResponse::NotFound),
        };
        let mut txn = self.state.db.handle.begin().await?;
        let by = FindItemBy::id(tvshow_id, false);
        match TVShow::lookup_by(&mut txn, &by, true).await? {
            Some(tvshow) => Ok(GetTVShowResponse::Ok(Json(tvshow))),
            None => Ok(GetTVShowResponse::NotFound),
        }
    }
}
