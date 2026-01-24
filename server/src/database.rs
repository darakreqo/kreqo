#[cfg(feature = "ssr")]
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use kreqo_core::User;
use kreqo_core::errors::ServerError;
use server_fn_macro_default::server;

#[cfg(feature = "ssr")]
use crate::DB;

#[server]
pub async fn get_users() -> Result<Vec<User>, ServerError> {
    let pool = &*DB;

    Ok(sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(pool)
        .await?)
}

#[server]
pub async fn get_user(id: i64) -> Result<User, ServerError> {
    let pool = &*DB;

    Ok(
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await?,
    )
}

#[server]
pub async fn get_user_from_username(username: String) -> Result<User, ServerError> {
    let pool = &*DB;

    Ok(
        sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await?,
    )
}

#[server]
pub async fn create_user(username: String, password: String) -> Result<User, ServerError> {
    let pool = &*DB;

    let salt = SaltString::generate(&mut OsRng);
    let password_hashed = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    // return Err(ServerError::Database("Database error!!!".to_string()));

    let id = sqlx::query_scalar!(
        "INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id",
        username.clone(),
        password_hashed
    )
    .fetch_one(pool)
    .await?;

    // To check if the creation of the user was successfull
    let user = get_user(id).await?;

    Ok(user)
}

#[server]
pub async fn update_user_username(id: i64, username: String) -> Result<User, ServerError> {
    let pool = &*DB;

    let id = sqlx::query_scalar!(
        "UPDATE users SET username = $2 WHERE id = $1 RETURNING id",
        id,
        username.clone(),
    )
    .fetch_one(pool)
    .await?;

    let user = get_user(id).await?;

    Ok(user)
}

#[server]
pub async fn delete_user(id: i64) -> Result<i64, ServerError> {
    let pool = &*DB;

    Ok(
        sqlx::query_scalar!("DELETE FROM users WHERE id = $1 RETURNING id", id)
            .fetch_one(pool)
            .await?,
    )
}
