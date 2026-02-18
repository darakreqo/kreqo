use axum_session_auth::AuthSession;
use axum_session_sqlx::SessionPgPool;
use kreqo_core::users::User;
use sqlx::PgPool;

pub mod api;
#[cfg(feature = "ssr")]
pub mod context;

pub const SERVER_ADDRESS: &str = "localhost:8080";

pub type KreqoAuth = AuthSession<User, i64, SessionPgPool, PgPool>;
