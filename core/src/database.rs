use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use chrono::{DateTime, Utc};
use hashbrown::{HashMap, HashSet};
use sqlx::{FromRow, PgPool};

use crate::errors::ServerError;
use crate::users::User;
use crate::users::permissions::UserPermission;
use crate::users::roles::UserRole;

#[derive(FromRow, Clone, Debug)]
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

    pub async fn to_user(self, pool: &PgPool) -> Result<User, ServerError> {
        let user_perms = get_user_perms(pool, self.id).await?;
        Ok(self.into_user(Some(user_perms)))
    }
}

#[derive(FromRow)]
pub struct SqlUserPermission {
    pub user_id: i64,
    pub token: String,
}

impl From<SqlUserPermission> for UserPermission {
    fn from(val: SqlUserPermission) -> Self {
        UserPermission { token: val.token }
    }
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

pub async fn get_sql_users(pool: &PgPool) -> Result<Vec<SqlUser>, ServerError> {
    Ok(
        sqlx::query_as!(SqlUser, "SELECT * FROM users WHERE id <> 1")
            .fetch_all(pool)
            .await?,
    )
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

pub async fn get_users(pool: &PgPool) -> Result<Vec<User>, ServerError> {
    let sql_users = get_sql_users(pool).await?;
    let mut perms_map = HashMap::with_capacity(sql_users.len());
    for sql_user_perm in sqlx::query_as!(SqlUserPermission, "SELECT * FROM user_permissions")
        .fetch_all(pool)
        .await?
    {
        let entry = perms_map.entry(sql_user_perm.user_id).or_insert(Vec::new());
        entry.push(sql_user_perm.into());
    }
    Ok(sql_users
        .iter()
        .map(|sql_user| {
            sql_user
                .clone()
                .into_user(perms_map.get(&sql_user.id).cloned())
        })
        .collect())
}

pub async fn get_user(pool: &PgPool, id: i64) -> Result<User, ServerError> {
    get_sql_user(pool, id).await?.to_user(pool).await
}

pub async fn get_user_from_username(pool: &PgPool, username: String) -> Result<User, ServerError> {
    get_sql_user_from_username(pool, username)
        .await?
        .to_user(pool)
        .await
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
