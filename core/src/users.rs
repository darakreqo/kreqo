pub mod permissions;
pub mod roles;

use std::collections::HashSet;

use anyhow::anyhow;
use async_trait::async_trait;
use axum_session_auth::{Authentication, HasPermission};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::database::get_user;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub anonymous: bool,
    pub username: String,
    pub created_at: Option<DateTime<Utc>>,
    pub permissions: HashSet<String>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: 1,
            anonymous: true,
            username: "Guest".into(),
            created_at: None,
            permissions: HashSet::new(),
        }
    }
}

#[async_trait]
impl Authentication<User, i64, PgPool> for User {
    async fn load_user(userid: i64, pool: Option<&PgPool>) -> Result<User, anyhow::Error> {
        let pool = pool.ok_or_else(|| anyhow!("expected a PgPool"))?;
        Ok(get_user(pool, userid).await?)
    }

    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
    }
}

#[async_trait]
impl HasPermission<PgPool> for User {
    async fn has(&self, perm: &str, _pool: &Option<&PgPool>) -> bool {
        self.permissions.contains(perm)
    }
}
