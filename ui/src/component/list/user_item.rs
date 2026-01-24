use kreqo_core::User;
use kreqo_core::errors::ServerError;
use kreqo_server::database::{create_user, delete_user, get_users, update_user_username};
use thiserror::Error;
use xilem::core::{Edit, Read};
use xilem::masonry::layout::{AsUnit, Dim};
use xilem::style::Style;
use xilem::view::{FlexExt, button, flex_col, flex_row, label, prose, text_button, text_input};
use xilem::{TextAlign, WidgetView};

use crate::component::Form;
use crate::component::form::Submit;
use crate::component::list::{ItemAction, ListItem, ListStorage};
use crate::theme::{DANGER_COLOR, SUCCESS_COLOR, SURFACE_BORDER_COLOR, SURFACE_COLOR};

#[derive(Debug, Error)]
pub enum UserError {
    #[error("username is required")]
    EmptyUsername,
    #[error("password is required")]
    EmptyPassword,
    #[error("password confirmation doesn't match")]
    PasswordConfirmationMismatch,
}

#[derive(Debug, Default)]
pub struct CreateUserForm {
    username: String,
    password: String,
    password_confirmation: String,
    last_error: Option<UserError>,
}

impl Form for CreateUserForm {
    type Output = (String, String);
    type Error = UserError;

    fn last_error(&mut self) -> &mut Option<UserError> {
        &mut self.last_error
    }

    fn view(&mut self) -> impl WidgetView<Edit<Self>, Submit> + use<> {
        let username = text_input(
            self.username.clone(),
            |state: &mut CreateUserForm, input| {
                state.username = input;
                Submit::No
            },
        )
        .placeholder("Username");
        let password = text_input(
            self.password.clone(),
            |state: &mut CreateUserForm, input| {
                state.password = input;
                Submit::No
            },
        )
        .placeholder("Password");
        let password_confirmation = text_input(
            self.password_confirmation.clone(),
            |state: &mut CreateUserForm, input| {
                state.password_confirmation = input;
                Submit::No
            },
        )
        .placeholder("Password confirmation");
        let signup_button = text_button("Signup", |_| Submit::Yes).width(Dim::Stretch);
        let error = self.error_view();
        flex_col((
            username,
            password,
            password_confirmation,
            signup_button,
            error,
        ))
        .padding(25.)
        .corner_radius(15.)
        .background_color(SURFACE_COLOR)
        .border(SURFACE_BORDER_COLOR, 1.)
    }

    fn validate(&mut self) -> Result<(String, String), UserError> {
        if self.username.is_empty() {
            return Err(UserError::EmptyUsername);
        }
        if self.password.is_empty() {
            return Err(UserError::EmptyPassword);
        }
        if self.password != self.password_confirmation {
            return Err(UserError::PasswordConfirmationMismatch);
        }
        self.password_confirmation = String::default();
        Ok((
            std::mem::take(&mut self.username),
            std::mem::take(&mut self.password),
        ))
    }
}

#[derive(Debug, Default)]
pub struct UpdateUserForm {
    username: String,
    last_error: Option<UserError>,
}

impl Form for UpdateUserForm {
    type Output = String;
    type Error = UserError;

    fn last_error(&mut self) -> &mut Option<UserError> {
        &mut self.last_error
    }

    fn view(&mut self) -> impl WidgetView<Edit<Self>, Submit> + use<> {
        let username = text_input(
            self.username.clone(),
            |state: &mut UpdateUserForm, input| {
                state.username = input;
                Submit::No
            },
        )
        .on_enter(|_, _| Submit::Yes)
        .placeholder("Username")
        .flex(1.);
        let ok_button = button(label("Ok").color(SUCCESS_COLOR), |_| Submit::Yes);
        let cancel_button = text_button("Cancel", |_| Submit::Cancel);
        let error = self.error_view();
        flex_row((username, ok_button, cancel_button, error))
            .padding(5.)
            .corner_radius(10.)
            .background_color(SURFACE_COLOR)
            .border(SURFACE_BORDER_COLOR, 1.)
    }

    fn validate(&mut self) -> Result<String, UserError> {
        if self.username.is_empty() {
            return Err(UserError::EmptyUsername);
        }
        Ok(std::mem::take(&mut self.username))
    }
}

impl From<User> for UpdateUserForm {
    fn from(value: User) -> Self {
        Self {
            username: value.username.clone(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct UserStorage {
    last_error: Option<ServerError>,
}

impl ListStorage for UserStorage {
    type Item = User;
    type Error = ServerError;

    fn last_error(&mut self) -> &mut Option<ServerError> {
        &mut self.last_error
    }

    #[inline(always)]
    async fn fetch_all() -> Result<Vec<User>, ServerError> {
        get_users().await
    }

    #[inline(always)]
    async fn create((username, password): (String, String)) -> Result<User, ServerError> {
        create_user(username, password).await
    }

    #[inline(always)]
    async fn update(id: i64, username: String) -> Result<User, ServerError> {
        update_user_username(id, username).await
    }

    #[inline(always)]
    async fn delete(id: i64) -> Result<i64, ServerError> {
        delete_user(id).await
    }
}

impl ListItem for User {
    type Id = i64;
    type CreateForm = CreateUserForm;
    type UpdateForm = UpdateUserForm;

    fn id(&self) -> i64 {
        self.id
    }

    fn view(&self) -> impl WidgetView<Read<Self>, ItemAction> + use<> {
        let id = prose(format!("{}", self.id))
            .text_alignment(TextAlign::Center)
            .width(25.px());
        let username = prose(self.username.to_string()).flex(1.);
        let edit_button = text_button("Edit", |_| ItemAction::Edit);
        let delete_button = button(label("Delete").color(DANGER_COLOR), |_| ItemAction::Delete);
        flex_row((id, username, edit_button, delete_button))
            .padding(5.)
            .corner_radius(10.)
            .background_color(SURFACE_COLOR)
            .border(SURFACE_BORDER_COLOR, 1.)
    }
}
