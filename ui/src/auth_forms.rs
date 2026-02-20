use thiserror::Error;
use xilem::WidgetView;
use xilem::masonry::layout::Dim;
use xilem::style::Style;
use xilem::tokio::sync::mpsc::UnboundedSender;
use xilem::view::{flex_col, text_button, text_input};

use crate::component::Form;
use crate::component::form::Submit;
use crate::theme::{ACCENT_COLOR, SURFACE_BORDER_COLOR, SURFACE_COLOR};

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
pub struct UserLoginForm {
    username: String,
    password: String,
    last_error: Option<UserError>,
}

// TODO: improve form structure & aesthetic
impl Form for UserLoginForm {
    type Output = (String, String);
    type Error = UserError;

    fn last_error(&mut self) -> &mut Option<UserError> {
        &mut self.last_error
    }

    fn view(&mut self) -> impl WidgetView<Self, Submit> + use<> {
        let username = text_input(self.username.clone(), |state: &mut Self, input| {
            state.username = input;
            Submit::No
        })
        .placeholder("Username");
        let password = text_input(self.password.clone(), |state: &mut Self, input| {
            state.password = input;
            Submit::No
        })
        .on_enter(|_, _| Submit::Yes)
        .placeholder("Password");
        let login_button = text_button("Log In", |_| Submit::Yes)
            .width(Dim::Stretch)
            .background_color(ACCENT_COLOR);
        let error = self.error_view();
        flex_col((username, password, login_button, error))
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
        Ok((
            std::mem::take(&mut self.username),
            std::mem::take(&mut self.password),
        ))
    }
}

#[derive(Debug, Default)]
pub struct UserSignupForm {
    username: String,
    password: String,
    password_confirmation: String,
    last_error: Option<UserError>,
}

// TODO: improve form structure & aesthetic
impl Form for UserSignupForm {
    type Output = (String, String);
    type Error = UserError;

    fn last_error(&mut self) -> &mut Option<UserError> {
        &mut self.last_error
    }

    fn view(&mut self) -> impl WidgetView<Self, Submit> + use<> {
        let username = text_input(self.username.clone(), |state: &mut Self, input| {
            state.username = input;
            Submit::No
        })
        .placeholder("Username");
        let password = text_input(self.password.clone(), |state: &mut Self, input| {
            state.password = input;
            Submit::No
        })
        .placeholder("Password");
        let password_confirmation = text_input(
            self.password_confirmation.clone(),
            |state: &mut Self, input| {
                state.password_confirmation = input;
                Submit::No
            },
        )
        .on_enter(|_, _| Submit::Yes)
        .placeholder("Password confirmation");
        let signup_button = text_button("Sign Up", |_| Submit::Yes)
            .width(Dim::Stretch)
            .background_color(ACCENT_COLOR);
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

impl UserLoginForm {
    pub fn handle_submit(
        &mut self,
        submit: Submit,
        sender: Option<&UnboundedSender<(String, String)>>,
    ) {
        match submit {
            Submit::No => (),
            Submit::Cancel => {
                self.reset();
            }
            Submit::Yes => {
                let output = self.submit();
                if let (Some(output), Some(sender)) = (output, sender) {
                    let _ = sender.send(output);
                }
            }
        }
    }
}
