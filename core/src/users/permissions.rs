use sqlx::FromRow;

#[derive(FromRow, Default, Debug, Clone)]
pub struct UserPermission {
    pub token: String,
}

impl UserPermission {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_owned(),
        }
    }
}
