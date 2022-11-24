use poem_openapi::{
    param::{Path, Query},
    payload::{Binary, Json, Response},
    Object, OpenApi, Tags,
};
use poem::{Body, Result, Request};

use crate::server::{SharedState, SessionFK, SessionFC};
use crate::util::Id;

mod collection;
mod image;
mod movie;
mod tvshow;
mod user;

use collection::*;
use self::image::*;
use movie::*;
use tvshow::*;
use user::*;

#[derive(Tags)]
enum ApiTags {
    /// Authorization
    Authorization,
    /// Operations on collections.
    Collection,
    /// Operations on tvshows, movies.
    Media,
    /// Operations on users.
    User,
}

#[derive(Object)]
pub struct Authenticate {
    pub username: String,
    pub password: String,
}

pub struct Api {
    state: SharedState,
}

#[OpenApi]
impl Api {
    pub(crate) fn new(state: SharedState) -> Api {
        Api { state }
    }

    /// Authenticate to get a session key
    #[oai(path = "/auth/login", method = "post", tag = "ApiTags::Authorization")]
    async fn api_login(&self, auth: Json<Authenticate>, req: &Request) -> Result<Response<LoginResponse>> {
        let resp = self.login(auth, req).await?;
        Ok(resp)
    }

    /// Invalidate session key.
    #[oai(path = "/auth/logout", method = "post", tag = "ApiTags::Authorization")]
    async fn api_logout(&self, session: SessionFK, req: &Request) -> Result<Response<LogoutResponse>> {
        let resp = self.logout(session.0, req).await?;
        Ok(resp)
    }

    /// List collections.
    #[oai(path = "/collections", method = "get", tag = "ApiTags::Collection")]
    async fn api_get_collections(&self, _session: SessionFK) -> Result<GetCollectionsResponse> {
        let res = self.get_collections().await?;
        Ok(res)
    }

    /// Get thumbnails of a collection.
    #[oai(path = "/collection/:collection_id/thumbs", method = "get", tag = "ApiTags::Collection")]
    async fn api_get_thumbs(&self, _session: SessionFK, collection_id: Path<i64>) -> Result<GetThumbsResponse> {
        let res = self.get_thumbs(collection_id.0).await?;
        Ok(res)
    }

    /// Find tvshow by id.
    #[oai(path = "/tvshow/:collection_id/:tvshow_id", method = "get", tag = "ApiTags::Media")]
    async fn api_get_tvshow(&self, _session: SessionFK, collection_id: Path<i64>, tvshow_id: Path<String>) -> Result<GetTVShowResponse> {
        let res = self.get_tvshow(collection_id.0, Id::from_str(&tvshow_id.0)?).await?;
        Ok(res)
    }

    /// Find movie by id.
    #[oai(path = "/movie/:collection_id/:movie_id", method = "get", tag = "ApiTags::Media")]
    async fn api_get_movie(&self, _session: SessionFK, collection_id: Path<i64>, movie_id: Path<String>) -> Result<GetMovieResponse> {
        let res = self.get_movie(collection_id.0, Id::from_str(&movie_id.0)?).await?;
        Ok(res)
    }

    /// Retrieve image.
    #[oai(path = "/image/:collection_id/:mediaitem_id/:image_id", method = "get", tag = "ApiTags::Media")]
    async fn api_get_image(&self, _session: SessionFC, collection_id: Path<u32>, mediaitem_id: Path<String>, image_id: Path<i64>, w: Query<Option<u32>>, h: Query<Option<u32>>, q: Query<Option<u32>>, req: &Request) -> Result<Response<Binary<Body>>> {
        let whq = ImageOpts { width: w.0, height: h.0, quality: q.0 };
        let res = self.get_image(collection_id.0, Id::from_str(&mediaitem_id.0)?, image_id.0, whq, req).await?;
        Ok(res)
    }

    /// Create a new user
    #[oai(path = "/users", method = "post", tag = "ApiTags::User")]
    async fn api_create_user(&self, session: SessionFK, user: Json<CreateUser>) -> Result<CreateUserResponse> {
       let res = self.create_user(session.0, user.0).await?;
       Ok(res)
    }

    /// Delete user by id
    #[oai(path = "/users/:user_id", method = "delete", tag = "ApiTags::User")]
    async fn api_delete_user(&self, session: SessionFK, user_id: Path<i64>) -> Result<DeleteUserResponse> {
        let res = self.delete_user(session.0, user_id.0).await?;
        Ok(res)
    }

    /// Update user by id
    #[oai(path = "/users/:user_id", method = "put", tag = "ApiTags::User")]
    async fn api_update_user(&self, session: SessionFK, user_id: Path<i64>, update: Json<UpdateUser>) -> Result<UpdateUserResponse> {
        let res = self.update_user(session.0, user_id.0, update.0).await?;
        Ok(res)
    }

    /// Find user by name
    #[oai(path = "/users/:user_id", method = "get", tag = "ApiTags::User")]
    async fn api_find_user(&self, session: SessionFK, user_id: Path<String>) -> Result<FindUserResponse> {
        let res = self.find_user(session.0, user_id.0).await?;
        Ok(res)
    }

    /// Get all users.
    #[oai(path = "/users", method = "get", tag = "ApiTags::User")]
    async fn api_get_users(&self, session: SessionFK) -> Result<GetUsersResponse> {
       let res = self.get_users(session.0).await?;
       Ok(res)
    }
}
