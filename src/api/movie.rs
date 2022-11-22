use anyhow::Result;
use poem_openapi::{
    payload::Json,
    ApiResponse,
};
use super::Api;
use crate::db::FindItemBy;

pub use crate::models::Movie;

#[derive(ApiResponse)]
pub enum GetMovieResponse {
    /// Return when the collections are listed.
    #[oai(status = 200)]
    Ok(Json<Box<Movie>>),

    /// Return when there are no collections.
    #[oai(status = 404)]
    NotFound,
}

impl Api {
    pub async fn get_movie(&self, collection_id: i64, movie_id: i64) -> Result<GetMovieResponse> {
        let collections = &self.state.config.collections;
        let _coll = match collections.iter().find(|c| c.collection_id as i64 == collection_id) {
            Some(coll) => coll,
            None => return Ok(GetMovieResponse::NotFound),
        };
        let mut txn = self.state.db.handle.begin().await?;
        let by = FindItemBy::id(movie_id, false);
        match Movie::lookup_by(&mut txn, &by).await? {
            Some(movie) => Ok(GetMovieResponse::Ok(Json(movie))),
            None => Ok(GetMovieResponse::NotFound),
        }
    }
}
