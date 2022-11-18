use poem_openapi::{
    payload::Json,
    types::{Email, Password},
    ApiResponse, Object,
};

/// Create user schema
#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct User {
    /// Id
    #[oai(read_only)]
    pub id: i64,
    /// Name
    #[oai(validator(max_length = 64))]
    pub name: String,
    /// Password
    #[oai(validator(max_length = 32))]
    pub password: Password,
    pub email: Email,
}

/// Update user schema
#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct UpdateUser {
    /// Name
    pub name: Option<String>,
    /// Password
    pub password: Option<Password>,
}

#[derive(ApiResponse)]
pub enum CreateUserResponse {
    /// Returns when the user is successfully created.
    #[oai(status = 200)]
    Ok(Json<i64>),
}

#[derive(ApiResponse)]
pub enum FindUserResponse {
    /// Return the specified user.
    #[oai(status = 200)]
    Ok(Json<User>),
    /// Return when the specified user is not found.
    #[oai(status = 404)]
    NotFound,
}

#[derive(ApiResponse)]
pub enum DeleteUserResponse {
    /// Returns when the user is successfully deleted.
    #[oai(status = 200)]
    Ok,
    /// Return when the specified user is not found.
    #[oai(status = 404)]
    NotFound,
}

#[derive(ApiResponse)]
pub enum UpdateUserResponse {
    /// Returns when the user is successfully updated.
    #[oai(status = 200)]
    Ok,
    /// Return when the specified user is not found.
    #[oai(status = 404)]
    NotFound,
}
