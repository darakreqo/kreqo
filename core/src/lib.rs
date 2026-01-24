use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub mod errors;

#[derive(FromRow, Debug, Clone, Default, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub created_at: Option<DateTime<Utc>>,
}
