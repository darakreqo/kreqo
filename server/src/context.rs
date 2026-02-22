use std::env;
use std::sync::LazyLock;
use std::time::Duration;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::{task_local, time};

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
pub fn auth() -> KreqoAuth {
    AUTH_CONTEXT.with(|auth| auth.clone())
}

#[inline]
pub fn context() -> (&'static PgPool, KreqoAuth) {
    (pool(), auth())
}

pub async fn auto_cleanup() {
    loop {
        time::sleep(Duration::from_mins(5)).await;
        let _ = auth().session.get_store().cleanup().await;
    }
}
