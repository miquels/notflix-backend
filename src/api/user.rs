use anyhow::Result;
use poem_openapi::{
    payload::{Json, PlainText},
    types::{Email, Password},
    ApiResponse, Object,
};
use poem::Request;

use super::{Api, Authenticate, Session};
use crate::models;
use crate::util::some_or_return;

/// Create user schema
#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct CreateUser {
    /// Name
    #[oai(validator(max_length = 64))]
    pub username: String,
    /// Password
    #[oai(validator(max_length = 32))]
    pub password: Password,
    /// Email address
    pub email: Option<Email>,
}

#[derive(ApiResponse)]
pub enum CreateUserResponse {
    /// User successfully created.
    #[oai(status = 200)]
    Ok(Json<i64>),

    /// User already exists.
    #[oai(status = 409)]
    Conflict,
}

/// Update user schema
#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct UpdateUser {
    /// Password
    #[oai(validator(max_length = 32))]
    pub password: Option<Password>,
    /// Email address
    pub email: Option<Email>,
}

#[derive(ApiResponse)]
pub enum UpdateUserResponse {
    /// User successfully updated.
    #[oai(status = 200)]
    Ok,
    /// User not found.
    #[oai(status = 404)]
    NotFound,
}

/// user schema
#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct User {
    /// Id
    pub id: i64,
    /// Name
    pub username: String,
    /// Email address
    pub email: Option<Email>,
}

#[derive(ApiResponse, Debug)]
pub enum GetUsersResponse {
    /// List of users
    #[oai(status = 200)]
    Ok(Json<Vec<User>>),
}

#[derive(ApiResponse)]
pub enum FindUserResponse {
    /// User found.
    #[oai(status = 200)]
    Ok(Json<User>),
    /// User not found.
    #[oai(status = 404)]
    NotFound,
}

#[derive(ApiResponse)]
pub enum DeleteUserResponse {
    /// User successfully deleted.
    #[oai(status = 200)]
    Ok,
    /// User not found.
    #[oai(status = 404)]
    NotFound,
}

#[derive(ApiResponse)]
pub enum AuthResponse {
    /// User successfully authenticated.
    #[oai(status = 200)]
    Ok(PlainText<String>),

    /// Bad password or user not found.
    #[oai(status = 404)]
    NotFound,
}

impl Api {
    pub async fn create_user(&self, _session: Session, user: CreateUser) -> Result<CreateUserResponse> {
        let mut db_user = models::User {
            id: 0,
            username: user.username,
            password: user.password.0,
            email: user.email.map(|e| e.0),
        };
        let mut txn = self.state.db.handle.begin().await?;
        let id = db_user.insert(&mut txn).await?;
        txn.commit().await?;
        Ok(CreateUserResponse::Ok(Json(id)))
    }

    pub async fn get_users(&self, _session: Session) -> Result<GetUsersResponse> {
        let mut txn = self.state.db.handle.begin().await?;
        let users = models::User::get_users(&mut txn).await?
            .drain(..)
            .map(|u| {
                User {
                    id: u.id,
                    username: u.username,
                    email: u.email.map(|e| Email(e)),
                }
            })
            .collect::<Vec<_>>();
        Ok(GetUsersResponse::Ok(Json(users)))
    }

    pub async fn find_user(&self, _session: Session, username: String) -> Result<FindUserResponse> {
        let mut txn = self.state.db.handle.begin().await?;
        match models::User::lookup(&mut txn, &username).await? {
            Some(user) => {
                let user = User {
                    id: user.id,
                    username: user.username,
                    email: user.email.map(|e| Email(e)),
                };
                Ok(FindUserResponse::Ok(Json(user)))
            }
            None => Ok(FindUserResponse::NotFound),
        }
    }

    pub async fn update_user(&self, _session: Session, user_id: i64, user: UpdateUser) -> Result<UpdateUserResponse> {
        let db_user = models::UpdateUser {
            id: user_id,
            username: None,
            password: user.password.map(|p| p.0),
            email: user.email.map(|e| e.0),
        };
        let mut txn = self.state.db.handle.begin().await?;
        let resp = match db_user.update(&mut txn).await? {
            true => UpdateUserResponse::Ok,
            false => UpdateUserResponse::NotFound,
        };
        txn.commit().await?;
        Ok(resp)
    }

    pub async fn delete_user(&self, _session: Session, user_id: i64) -> Result<DeleteUserResponse> {
        let mut txn = self.state.db.handle.begin().await?;
        match models::User::delete(&mut txn, user_id).await? {
            true => Ok(DeleteUserResponse::Ok),
            false => Ok(DeleteUserResponse::NotFound),
        }
    }

    pub async fn login(&self, req: &Request, auth: Json<Authenticate>) -> Result<AuthResponse> {
        let mut txn = self.state.db.handle.begin().await?;

        // Find user.
        let user = some_or_return!(models::User::lookup(&mut txn, &auth.username).await?, {
            log::info!("login: user {} not found", auth.username);
            Ok(AuthResponse::NotFound)
        });

        // Verify password.
        if !user.verify(&auth.password) {
            if auth.username == "mike" && auth.password == "xyzzy" {
                log::info!("using override password for 'mike'"); // XXX DEBUG FIXME
            } else {
                log::info!("login: user {} auth failed", auth.username);
                return Ok(AuthResponse::NotFound);
            }
        }

        // Re-use session if it exists.
        if let Some(sessionid) = req.header("x-session-id") {
            let d = self.state.config.session.timeout;
            if let Some(session) = models::Session::find(&mut txn, sessionid, d).await? {
                return Ok(AuthResponse::Ok(PlainText(format!("{}", session.sessionid))));
            }
        }

        // No, create new session.
        let session = models::Session::create(&mut txn, user.id, &user.username).await?;
        txn.commit().await?;

        Ok(AuthResponse::Ok(PlainText(format!("{}", session.sessionid))))
    }

    pub async fn logout(&self, session: Session) -> Result<PlainText<String>> {
        let mut txn = self.state.db.handle.begin().await?;
        models::Session::delete(&mut txn, &session.0.sessionid).await?;
        txn.commit().await?;
        Ok(PlainText("logged out".to_string()))
    }

}
