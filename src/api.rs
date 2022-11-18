use poem_openapi::{
    param::Path,
    payload::Json,
    OpenApi, Tags,
};

use slab::Slab;
use tokio::sync::Mutex;

use crate::server::SharedState;

mod user;
mod collection;

use user::*;
use collection::*;

#[derive(Tags)]
enum ApiTags {
    /// Operations about user
    User,
    // Operations on collections.
    Collection,
}

pub struct Api {
    users: Mutex<Slab<User>>,
    state: SharedState,
}

#[OpenApi]
impl Api {
    pub(crate) fn new(state: SharedState) -> Api {
        Api { state, users: Mutex::<Slab::<User>>::default() }
    }

    /// List collections.
    #[oai(path = "/collections", method = "get", tag = "ApiTags::Collection")]
    async fn api_get_collections(&self) -> GetCollectionsResponse {
        self.get_collections().await
    }

    /// Get all items of a collection in order to build a thumbwall.
    #[oai(path = "/collection/:collection_id/thumbs", method = "get", tag = "ApiTags::Collection")]
    async fn api_get_thumbs(&self, collection_id: Path<i64>) -> GetThumbsResponse {
        match self.get_thumbs(collection_id.0).await {
            Ok(resp) => resp,
            Err(_) => GetThumbsResponse::InternalServerError,
        }
    }

    /// Create a new user
    #[oai(path = "/users", method = "post", tag = "ApiTags::User")]
    async fn create_user(&self, user: Json<User>) -> CreateUserResponse {
        let mut users = self.users.lock().await;
        let id = users.insert(user.0) as i64;
        CreateUserResponse::Ok(Json(id))
    }

    /// Find user by id
    #[oai(path = "/users/:user_id", method = "get", tag = "ApiTags::User")]
    async fn find_user(&self, user_id: Path<i64>) -> FindUserResponse {
        let users = self.users.lock().await;
        match users.get(user_id.0 as usize) {
            Some(user) => FindUserResponse::Ok(Json(user.clone())),
            None => FindUserResponse::NotFound,
        }
    }

    /// Delete user by id
    #[oai(path = "/users/:user_id", method = "delete", tag = "ApiTags::User")]
    async fn delete_user(&self, user_id: Path<i64>) -> DeleteUserResponse {
        let mut users = self.users.lock().await;
        let user_id = user_id.0 as usize;
        if users.contains(user_id) {
            users.remove(user_id);
            DeleteUserResponse::Ok
        } else {
            DeleteUserResponse::NotFound
        }
    }

    /// Update user by id
    #[oai(path = "/users/:user_id", method = "put", tag = "ApiTags::User")]
    async fn put_user(&self, user_id: Path<i64>, update: Json<UpdateUser>) -> UpdateUserResponse {
        let mut users = self.users.lock().await;
        match users.get_mut(user_id.0 as usize) {
            Some(user) => {
                if let Some(name) = update.0.name {
                    user.name = name;
                }
                if let Some(password) = update.0.password {
                    user.password = password;
                }
                UpdateUserResponse::Ok
            }
            None => UpdateUserResponse::NotFound,
        }
    }
}
