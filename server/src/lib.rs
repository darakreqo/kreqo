use std::env;
use std::sync::LazyLock;
use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub mod database;

pub static DB: LazyLock<PgPool> = LazyLock::new(|| {
    let db_connection_str = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost/kreqo".to_string());

    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect_lazy(&db_connection_str)
        .expect("can't connect to database")
});
