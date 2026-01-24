use std::env;

use axum::Router;
use axum::routing::post;
use kreqo_server::DB;
use server_fn::axum::handle_server_fn;
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

    sqlx::migrate!()
        .run(&*DB)
        .await
        .expect("database migrations failed");

    let router = Router::new().route("/api/{*wildcard}", post(handle_server_fn));

    let listener = TcpListener::bind("localhost:8080").await?;

    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}
