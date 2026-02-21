use thiserror::Error;
use xilem::masonry::layout::AsUnit;
use xilem::palette::css::GRAY;
use xilem::style::Style;
use xilem::tokio::sync::mpsc::UnboundedSender;
use xilem::view::{FlexSpacer, flex_col, flex_row, inline_prose, prose, text_input};
use xilem::{Color, WidgetView};
use zxcvbn::{Score, zxcvbn};

use crate::component::Form;
use crate::component::form::Submit;
use crate::theme::{
    ACTION_BTN, ApplyClass, CONTAINER, DANGER_COLOR, SUCCESS_COLOR, WARNING_COLOR, action_button,
    constant_border_color, header,
};

#[derive(Debug, Error)]
pub enum UserError {
    #[error("username is required")]
    EmptyUsername,
    #[error("password is required")]
    EmptyPassword,
    #[error("password confirmation doesn't match")]
    PasswordConfirmationMismatch,
    #[error("password is too weak")]
    WeakPassword,
}

impl UserError {
    pub fn username_color(&self) -> Option<Color> {
        matches!(self, UserError::EmptyUsername).then_some(DANGER_COLOR)
    }

    pub fn password_color(&self) -> Option<Color> {
        matches!(self, UserError::EmptyPassword).then_some(DANGER_COLOR)
    }

    pub fn confirmation_color(&self) -> Option<Color> {
        matches!(self, UserError::PasswordConfirmationMismatch).then_some(DANGER_COLOR)
    }
}

#[derive(Debug, Default)]
pub struct UserLoginForm {
    username: String,
    password: String,
    last_error: Option<UserError>,
}

impl Form for UserLoginForm {
    type Output = (String, String);
    type Error = UserError;

    fn last_error(&mut self) -> &mut Option<UserError> {
        &mut self.last_error
    }

    fn view(&mut self) -> impl WidgetView<Self, Submit> + use<> {
        let header = header("User Login");
        let username = flex_col((
            prose("Username:"),
            text_input(self.username.clone(), |state: &mut Self, input| {
                state.username = input;
                state.last_error = state.check().err();
                Submit::No
            })
            .placeholder("username")
            .apply_fn(
                constant_border_color,
                self.last_error.as_ref().and_then(UserError::username_color),
            ),
        ));
        let password = flex_col((
            prose("Password:"),
            text_input(self.password.clone(), |state: &mut Self, input| {
                state.password = input;
                state.last_error = state.check().err();
                Submit::No
            })
            .on_enter(|_, _| Submit::Yes)
            .placeholder("password")
            .apply_fn(
                constant_border_color,
                self.last_error.as_ref().and_then(UserError::password_color),
            ),
        ));
        let login_button = action_button("Log In", |_| Submit::Yes).apply(ACTION_BTN);
        let error = self.error_view();
        flex_col((
            header,
            username,
            password,
            FlexSpacer::Fixed(0.px()),
            login_button,
            error,
        ))
        .apply(CONTAINER)
        .gap(20.px())
    }

    fn check(&mut self) -> Result<(), UserError> {
        if self.username.is_empty() {
            return Err(UserError::EmptyUsername);
        }
        if self.password.is_empty() {
            return Err(UserError::EmptyPassword);
        }
        Ok(())
    }

    fn validate(&mut self) -> Result<(String, String), UserError> {
        self.check()?;
        Ok((
            std::mem::take(&mut self.username),
            std::mem::take(&mut self.password),
        ))
    }
}

#[derive(Debug)]
pub struct UserSignupForm {
    username: String,
    password: String,
    password_confirmation: String,
    score: Score,
    last_error: Option<UserError>,
}

impl Default for UserSignupForm {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            password_confirmation: String::new(),
            score: Score::Zero,
            last_error: None,
        }
    }
}

impl Form for UserSignupForm {
    type Output = (String, String);
    type Error = UserError;

    fn last_error(&mut self) -> &mut Option<UserError> {
        &mut self.last_error
    }

    fn view(&mut self) -> impl WidgetView<Self, Submit> + use<> {
        let header = header("Account Creation");
        let username = flex_col((
            prose("Username:"),
            text_input(self.username.clone(), |state: &mut Self, input| {
                state.username = input;
                state.last_error = state.check().err();
                Submit::No
            })
            .placeholder("username")
            .apply_fn(
                constant_border_color,
                self.last_error.as_ref().and_then(UserError::username_color),
            ),
        ));
        let password = flex_col((
            prose("Password:"),
            text_input(self.password.clone(), |state: &mut Self, input| {
                state.password = input;
                state.last_error = state.check().err();
                Submit::No
            })
            .placeholder("password")
            .apply_fn(
                constant_border_color,
                self.last_error.as_ref().and_then(UserError::password_color),
            ),
            (!self.password.is_empty()).then(|| {
                let (color1, color2) = match self.score {
                    Score::Zero | Score::One | Score::Two => (GRAY, DANGER_COLOR),
                    Score::Three => (GRAY, WARNING_COLOR),
                    _ => (SUCCESS_COLOR, SUCCESS_COLOR),
                };
                let (text1, text2) = match self.score {
                    Score::Zero | Score::One => ("️ ❌ Password strength:", "Very weak"),
                    Score::Two => ("️ ❌ Password strength:", "Weak"),
                    Score::Three => ("️ ❌ Password strength:", "Medium"),
                    _ => ("️ ✓ Password strength:", "Strong"),
                };
                flex_row((
                    inline_prose(text1).text_color(color1),
                    inline_prose(text2).text_color(color2),
                ))
            }),
        ));
        let password_confirmation = flex_col((
            prose("Confirm password:"),
            text_input(
                self.password_confirmation.clone(),
                |state: &mut Self, input| {
                    state.password_confirmation = input;
                    state.last_error = state.check().err();
                    Submit::No
                },
            )
            .on_enter(|_, _| Submit::Yes)
            .placeholder("password confirmation")
            .apply_fn(
                constant_border_color,
                self.last_error
                    .as_ref()
                    .and_then(UserError::confirmation_color),
            ),
        ));
        let signup_button = action_button("Sign Up", |_| Submit::Yes).apply(ACTION_BTN);
        let error = self.error_view();
        flex_col((
            header,
            username,
            password,
            password_confirmation,
            FlexSpacer::Fixed(1.px()),
            signup_button,
            error,
        ))
        .apply(CONTAINER)
        .gap(20.px())
    }

    fn check(&mut self) -> Result<(), UserError> {
        self.score = zxcvbn(self.password.as_str(), &[self.username.as_str()]).score();
        if self.username.is_empty() {
            return Err(UserError::EmptyUsername);
        }
        if self.password.is_empty() {
            return Err(UserError::EmptyPassword);
        }
        if self.score < Score::Four {
            return Err(UserError::WeakPassword);
        }
        if self.password != self.password_confirmation {
            return Err(UserError::PasswordConfirmationMismatch);
        }
        Ok(())
    }

    fn validate(&mut self) -> Result<(String, String), UserError> {
        self.check()?;
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
