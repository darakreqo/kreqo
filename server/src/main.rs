use std::env;

use axum::Router;
use axum::middleware::from_fn;
use axum::routing::{get, post};
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionPgPool;
use kreqo_core::users::User;
use kreqo_server::SERVER_ADDRESS;
use kreqo_server::context::{auth_context_middleware, pool};
use server_fn::axum::handle_server_fn;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = pool();

    sqlx::migrate!()
        .run(pool)
        .await
        .expect("database migrations failed");

    let session_config = SessionConfig::default().with_table_name("axum_sessions");
    let auth_config = AuthConfig::<i64>::default().with_anonymous_user_id(Some(1));

    let session_store =
        SessionStore::<SessionPgPool>::new(Some(pool.clone().into()), session_config).await?;

    let router = Router::new()
        .route("/", get(|| async { "kreqo-server is running" }))
        .route("/api/{*wildcard}", post(handle_server_fn))
        .layer(from_fn(auth_context_middleware))
        .layer(
            AuthSessionLayer::<User, i64, SessionPgPool, PgPool>::new(Some(pool.clone()))
                .with_config(auth_config),
        )
        .layer(SessionLayer::new(session_store));

    let listener = TcpListener::bind(SERVER_ADDRESS).await?;

    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}
