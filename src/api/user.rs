use anyhow::Result;
use poem_openapi::{
    payload::{Json, Response},
    types::{Email, Password},
    ApiResponse, Object,
};
use poem::{
    web::cookie::{Cookie, SameSite},
    Request,
};

use super::{Api, Authenticate};
use crate::models::{self, Session};
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
pub enum LoginResponse {
    /// User successfully authenticated.
    #[oai(status = 200)]
    Ok(Json<String>),

    /// Wrong password or user not found.
    #[oai(status = 404)]
    NotFound,

    /// Origin: header not valid.
    #[oai(status = 403)]
    BadOrigin,
}

#[derive(ApiResponse)]
pub enum LogoutResponse {
    /// User successfully logged out
    #[oai(status = 200)]
    Ok(Json<String>),
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

    pub async fn login(&self, req: &Request, auth: Json<Authenticate>) -> Result<Response<LoginResponse>> {
        let mut txn = self.state.db.handle.begin().await?;

        // Check Origin:
        let origin = req.header("origin");
        let origin_host = match origin {
            Some(mut val) => {
                // Strip prefix
                val = val.strip_prefix("http://").unwrap_or(val);
                val = val.strip_prefix("https://").unwrap_or(val);
                // Strip port.
                val = val.rsplit_once(":").map(|r| r.0).unwrap_or(val);
                let mut names = self.state.config.server.hostname.iter();
                (val == "localhost" || names.any(|h| h.eq_ignore_ascii_case(val))).then(||val)
            },
            None => None,
        };
        if origin_host.is_none() {
            let origin = origin.unwrap_or("[none]");
            log::info!("invalid origin \"{}\", rejecting login request", origin);
            return Ok(Response::new(LoginResponse::BadOrigin))
        }

        // Find user.
        let user = some_or_return!(models::User::lookup(&mut txn, &auth.username).await?, {
            log::info!("login: user {} not found", auth.username);
            Ok(Response::new(LoginResponse::NotFound))
        });

        // Verify password.
        let sha = user.password.starts_with("$6$");
        if (sha && !user.verify(&auth.password)) || (!sha && user.password != auth.password) {
            log::info!("login: user {} auth failed", auth.username);
            return Ok(Response::new(LoginResponse::NotFound));
        }

        // Re-use session if it exists.
        let mut session = None;
        let d = self.state.config.session.timeout;

        let jar = req.cookie();
        if let Some(cookie) = jar.get("x-session-id") {
            // In a cookie?
            session =  Session::find(&mut txn, cookie.value_str(), d).await?;
        }

        if session.is_none() {
            // In the x-session-id header?
            if let Some(sessionid) = req.header("x-session-id") {
                session = Session::find(&mut txn, sessionid, d).await?;
            }
        }

        // Must login as the same user.
        session = session.filter(|s| s.user_id == user.id);

        if session.is_none() {
            // Create new session.
            session = Some(Session::create(&mut txn, user.id, &user.username).await?);
        }

        txn.commit().await?;
        let session = session.unwrap();

        // Create cookie.
        let mut cookie = Cookie::new_with_str("x-session-id", &session.sessionid);
        cookie.set_http_only(true);
        cookie.set_secure(true);
        cookie.set_path("/");
        cookie.set_same_site(SameSite::Lax);
        cookie.make_permanent();

        let resp = Response::new(LoginResponse::Ok(Json(session.sessionid)))
            .header("set-cookie", cookie.to_string());

        Ok(resp)
    }

    pub async fn logout(&self, session: Session, req: &Request) -> Result<Response<LogoutResponse>> {
        let mut txn = self.state.db.handle.begin().await?;
        Session::delete(&mut txn, &session.sessionid).await?;
        txn.commit().await?;

        // We need to build a response manually, we have to add the set-cookie header.
        let mut resp = Response::new(LogoutResponse::Ok(Json("logged out".to_string())));
        let jar = req.cookie();
        if let Some(mut cookie) = jar.get("x-session-id") {
            cookie.make_removal();
            resp = resp.header("set-cookie", cookie.to_string());
        }

        Ok(resp)
    }
}
