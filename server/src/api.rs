use kreqo_core::errors::ServerError;
use kreqo_core::users::User;
use server_fn_macro_default::server;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        use axum_session_auth::{Auth, Rights};
        use kreqo_core::database;
        use kreqo_core::database::get_sql_user_from_username;
        use server_fn::request::reqwest::Method;
        use sqlx::PgPool;

        use crate::context::{auth, pool, context};

        async fn require_perms(user: User, rights: Rights) -> Result<(), ServerError> {
            if !Auth::<User, i64, PgPool>::build([Method::POST], true)
                .requires(rights)
                .validate(&user, &Method::POST, None)
                .await
            {
                return Err(ServerError::Unauthorized);
            }
            Ok(())
        }
    }
}

#[server]
pub async fn login(username: String, password: String) -> Result<bool, ServerError> {
    let pool = pool();

    let sql_user = get_sql_user_from_username(pool, username).await?;
    let parsed_hash = PasswordHash::new(&sql_user.password)?;
    let matches = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    if matches {
        let auth = auth();
        auth.login_user(sql_user.id);
    }

    Ok(matches)
}

#[server]
pub async fn logout() -> Result<(), ServerError> {
    let auth = auth();
    auth.logout_user();
    Ok(())
}

#[server]
pub async fn signup(username: String, password: String) -> Result<User, ServerError> {
    let pool = pool();

    #[cfg(debug_assertions)]
    std::thread::sleep(std::time::Duration::from_millis(500));
    database::create_user(pool, username, password).await
}

#[server]
pub async fn get_users() -> Result<Vec<User>, ServerError> {
    let (pool, auth) = context();
    let current_user = auth.current_user.unwrap_or_default();
    require_perms(current_user, Rights::permission("Users::View")).await?;

    #[cfg(debug_assertions)]
    std::thread::sleep(std::time::Duration::from_millis(500));
    database::get_users(pool).await
}

#[server]
pub async fn get_user(id: i64) -> Result<User, ServerError> {
    let (pool, auth) = context();
    let current_user = auth.current_user.unwrap_or_default();
    require_perms(current_user, Rights::permission("Users::View")).await?;

    #[cfg(debug_assertions)]
    std::thread::sleep(std::time::Duration::from_millis(500));
    database::get_user(pool, id).await
}

#[server]
pub async fn get_user_from_username(username: String) -> Result<User, ServerError> {
    let (pool, auth) = context();
    let current_user = auth.current_user.unwrap_or_default();
    require_perms(current_user, Rights::permission("Users::View")).await?;

    #[cfg(debug_assertions)]
    std::thread::sleep(std::time::Duration::from_millis(500));
    database::get_user_from_username(pool, username).await
}

#[server]
pub async fn update_user_username(id: i64, username: String) -> Result<User, ServerError> {
    let (pool, auth) = context();
    let current_user = auth.current_user.unwrap_or_default();

    if id == 1 {
        return Err(ServerError::Unauthorized);
    }

    if id == current_user.id {
        require_perms(
            current_user,
            Rights::any([
                Rights::permission("Users::Manage"),
                Rights::permission("CurrentUser::Manage"),
            ]),
        )
        .await?;
    } else {
        require_perms(current_user, Rights::permission("Users::Manage")).await?;
    }

    #[cfg(debug_assertions)]
    std::thread::sleep(std::time::Duration::from_millis(500));
    database::update_user_username(pool, id, username).await
}

#[server]
pub async fn delete_user(id: i64) -> Result<i64, ServerError> {
    let (pool, auth) = context();
    let current_user = auth.current_user.unwrap_or_default();

    if id == 1 {
        return Err(ServerError::Unauthorized);
    }

    if id == current_user.id {
        require_perms(
            current_user,
            Rights::any([
                Rights::permission("Users::Manage"),
                Rights::permission("CurrentUser::Manage"),
            ]),
        )
        .await?;
    } else {
        require_perms(current_user, Rights::permission("Users::Manage")).await?;
    }

    #[cfg(debug_assertions)]
    std::thread::sleep(std::time::Duration::from_millis(500));
    database::delete_user(pool, id).await
}
