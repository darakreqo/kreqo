use std::env;
use std::sync::LazyLock;
use std::time::Duration;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use kreqo_core::errors::ServerError;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::task_local;

use crate::KreqoAuth;

static POOL_CONTEXT: LazyLock<PgPool> = LazyLock::new(|| {
    let db_connection_str = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost/kreqo".to_string());

    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect_lazy(&db_connection_str)
        .expect("can't connect to database")
});

task_local! {
    static AUTH_CONTEXT: KreqoAuth;
}

pub async fn auth_context_middleware(auth: KreqoAuth, request: Request, next: Next) -> Response {
    AUTH_CONTEXT.scope(auth, next.run(request)).await
}

#[inline]
pub fn pool() -> &'static PgPool {
    &POOL_CONTEXT
}

#[inline]
pub fn auth() -> Result<KreqoAuth, ServerError> {
    let auth = AUTH_CONTEXT
        .try_with(|auth| auth.clone())
        .map_err(|error| ServerError::Session(error.to_string()))?;
    Ok(auth)
}

#[inline]
pub fn context() -> (&'static PgPool, Result<KreqoAuth, ServerError>) {
    (pool(), auth())
}
