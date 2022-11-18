use poem_openapi::{
    payload::Json,
    ApiResponse,
};

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
