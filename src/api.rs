use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
    Object, OpenApi, Tags,
};
use poem::{Result, Request};

use crate::server::{SharedState, SessionIdAuthorization};

mod collection;
mod user;
mod tvshow;

use collection::*;
use user::*;
use tvshow::*;

pub type Session = SessionIdAuthorization;

#[derive(Tags)]
enum ApiTags {
    /// Authorization
    Authorization,
    /// Operations on collections.
    Collection,
    /// Operations on tvshows.
    TVShow,
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
    async fn api_login(&self, req: &Request, auth: Json<Authenticate>) -> Result<AuthResponse> {
        let resp = self.login(req, auth).await?;
        Ok(resp)
    }

    /// Invalidate session key.
    #[oai(path = "/auth/logout", method = "post", tag = "ApiTags::Authorization")]
    async fn api_logout(&self, session: Session) -> Result<PlainText<String>> {
        let resp = self.logout(session).await?;
        Ok(resp)
    }

    /// List collections.
    #[oai(path = "/collections", method = "get", tag = "ApiTags::Collection")]
    async fn api_get_collections(&self) -> Result<GetCollectionsResponse> {
        let res = self.get_collections().await?;
        Ok(res)
    }

    /// Get thumbnails of a collection.
    #[oai(path = "/collection/:collection_id/thumbs", method = "get", tag = "ApiTags::Collection")]
    async fn api_get_thumbs(&self, collection_id: Path<i64>) -> Result<GetThumbsResponse> {
        let res = self.get_thumbs(collection_id.0).await?;
        Ok(res)
    }

    /// Find tvshow by id.
    #[oai(path = "/tvshow/:collection_id/:tvshow_id", method = "get", tag = "ApiTags::TVShow")]
    async fn api_get_tvshow(&self, collection_id: Path<i64>, tvshow_id: Path<i64>) -> Result<GetTVShowResponse> {
        let res = self.get_tvshow(collection_id.0, tvshow_id.0).await?;
        Ok(res)
    }

    /// Create a new user
    #[oai(path = "/users", method = "post", tag = "ApiTags::User")]
    async fn api_create_user(&self, session: Session, user: Json<CreateUser>) -> Result<CreateUserResponse> {
       let res = self.create_user(session, user.0).await?;
       Ok(res)
    }

    /// Delete user by id
    #[oai(path = "/users/:user_id", method = "delete", tag = "ApiTags::User")]
    async fn api_delete_user(&self, session: Session, user_id: Path<i64>) -> Result<DeleteUserResponse> {
        let res = self.delete_user(session, user_id.0).await?;
        Ok(res)
    }

    /// Update user by id
    #[oai(path = "/users/:user_id", method = "put", tag = "ApiTags::User")]
    async fn api_update_user(&self, session: Session, user_id: Path<i64>, update: Json<UpdateUser>) -> Result<UpdateUserResponse> {
        let res = self.update_user(session, user_id.0, update.0).await?;
        Ok(res)
    }

    /// Find user by name
    #[oai(path = "/users/:user_id", method = "get", tag = "ApiTags::User")]
    async fn api_find_user(&self, session: Session, user_id: Path<String>) -> Result<FindUserResponse> {
        let res = self.find_user(session, user_id.0).await?;
        Ok(res)
    }

    /// Get all users.
    #[oai(path = "/users/all", method = "get", tag = "ApiTags::User")]
    async fn api_get_users(&self, session: Session) -> Result<GetUsersResponse> {
       let res = self.get_users(session).await?;
       Ok(res)
    }
}
