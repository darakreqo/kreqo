use argon2::password_hash::Error as PasswordHashError;
use axum_session::SessionError;
use serde::{Deserialize, Serialize};
use server_fn::codec::JsonEncoding;
use server_fn::error::{FromServerFnError, ServerFnErrorErr};
use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerError {
    #[error("API error: {0}")]
    API(ServerFnErrorErr),
    #[error("database error: {0}")]
    Database(String),
    #[error("session error: {0}")]
    Session(String),
    #[error("failed to hash password: {0}")]
    PasswordHash(String),
    #[error("wrong username or password")]
    WrongLogin,
    #[error("authentication required or missing permissions")]
    Unauthorized,
}

impl FromServerFnError for ServerError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        Self::API(value)
    }
}

impl From<SqlxError> for ServerError {
    fn from(value: SqlxError) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<SessionError> for ServerError {
    fn from(value: SessionError) -> Self {
        Self::Session(value.to_string())
    }
}

impl From<PasswordHashError> for ServerError {
    fn from(value: PasswordHashError) -> Self {
        Self::PasswordHash(value.to_string())
    }
}
