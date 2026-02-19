use std::collections::HashSet;

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::errors::ServerError;
use crate::users::User;
use crate::users::permissions::UserPermission;
use crate::users::roles::UserRole;

#[derive(FromRow, Debug)]
pub struct SqlUser {
    pub id: i64,
    pub anonymous: bool,
    pub username: String,
    pub password: String,
    pub created_at: Option<DateTime<Utc>>,
}

impl SqlUser {
    fn into_user(self, user_perms: Option<Vec<UserPermission>>) -> User {
        let permissions = if let Some(user_perms) = user_perms {
            user_perms.into_iter().map(|perm| perm.token).collect()
        } else {
            HashSet::new()
        };
        User {
            id: self.id,
            anonymous: self.anonymous,
            username: self.username,
            created_at: self.created_at,
            permissions,
        }
    }
}

pub async fn get_users(pool: &PgPool) -> Result<Vec<User>, ServerError> {
    let sql_users = sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE id <> 1")
        .fetch_all(pool)
        .await?;
    Ok(sql_users
        .into_iter()
        .map(|sql_user| sql_user.into_user(None))
        .collect())
}

pub async fn get_sql_user(pool: &PgPool, id: i64) -> Result<SqlUser, ServerError> {
    Ok(
        sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn get_sql_user_from_username(
    pool: &PgPool,
    username: String,
) -> Result<SqlUser, ServerError> {
    Ok(
        sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn get_user_perms(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<UserPermission>, ServerError> {
    let user_perms = sqlx::query_as!(
        UserPermission,
        "SELECT token FROM user_permissions WHERE user_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await?;
    Ok(user_perms)
}

pub async fn add_user_perms(
    pool: &PgPool,
    user_id: i64,
    user_perms: Vec<UserPermission>,
) -> Result<(), ServerError> {
    for perm in user_perms {
        sqlx::query!(
            "INSERT INTO user_permissions (user_id, token) VALUES ($1, $2)",
            user_id,
            perm.token
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_user(pool: &PgPool, id: i64) -> Result<User, ServerError> {
    let sql_user = sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE id = $1", id)
        .fetch_one(pool)
        .await?;
    let user_perms = get_user_perms(pool, sql_user.id).await?;
    Ok(sql_user.into_user(Some(user_perms)))
}

pub async fn get_user_from_username(pool: &PgPool, username: String) -> Result<User, ServerError> {
    let sql_user = sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE username = $1", username)
        .fetch_one(pool)
        .await?;
    let user_perms = get_user_perms(pool, sql_user.id).await?;
    Ok(sql_user.into_user(Some(user_perms)))
}

pub async fn create_user(
    pool: &PgPool,
    username: String,
    password: String,
) -> Result<User, ServerError> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hashed = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    let id = sqlx::query_scalar!(
        "INSERT INTO users (username, password, anonymous) VALUES ($1, $2, $3) RETURNING id",
        username.clone(),
        password_hashed,
        false
    )
    .fetch_one(pool)
    .await?;

    add_user_perms(pool, id, UserRole::Normal.permissions()).await?;

    // To check if the creation of the user was successfull
    let user = get_user(pool, id).await?;

    Ok(user)
}

pub async fn update_user_username(
    pool: &PgPool,
    id: i64,
    username: String,
) -> Result<User, ServerError> {
    let id = sqlx::query_scalar!(
        "UPDATE users SET username = $2 WHERE id = $1 RETURNING id",
        id,
        username.clone(),
    )
    .fetch_one(pool)
    .await?;

    let user = get_user(pool, id).await?;

    Ok(user)
}

pub async fn delete_user(pool: &PgPool, id: i64) -> Result<i64, ServerError> {
    Ok(
        sqlx::query_scalar!("DELETE FROM users WHERE id = $1 RETURNING id", id)
            .fetch_one(pool)
            .await?,
    )
}
