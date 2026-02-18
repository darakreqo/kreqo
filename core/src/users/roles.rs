use crate::users::permissions::UserPermission;

#[derive(Default)]
pub enum UserRole {
    #[default]
    Guest,
    Normal,
    Admin,
}

impl UserRole {
    pub fn permissions(&self) -> Vec<UserPermission> {
        match self {
            UserRole::Guest => Vec::new(),
            UserRole::Normal => vec![
                UserPermission::new("Users::View"),
                UserPermission::new("CurrentUser::Manage"),
            ],
            UserRole::Admin => vec![
                UserPermission::new("Users::View"),
                UserPermission::new("Users::Manage"),
            ],
        }
    }
}
