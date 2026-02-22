use thiserror::Error;
use xilem::masonry::layout::AsUnit;
use xilem::palette::css::GRAY;
use xilem::style::{Padding, Style};
use xilem::tokio::sync::mpsc::UnboundedSender;
use xilem::view::{CrossAxisAlignment, flex_col, flex_row, inline_prose, text_input, zstack};
use xilem::{Color, WidgetView};
use zxcvbn::feedback::Feedback;
use zxcvbn::{Score, zxcvbn};

use crate::component::form::Submit;
use crate::component::{Form, action_button, form_input_label, header};
use crate::theme::{
    ACCENT_COLOR, ApplyClass, CONTAINER, DANGER_COLOR, FORM_INPUT, SUCCESS_COLOR, WARNING_COLOR,
    form_border_color,
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

pub enum AuthRequest {
    Login(String, String),
    Logout,
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

    // TODO: refactor into form fields
    fn view(&mut self) -> impl WidgetView<Self, Submit> + use<> {
        let header = header("User Login");
        let username = zstack((
            text_input(self.username.clone(), |state: &mut Self, input| {
                state.username = input;
                state.last_error = state.check().err();
                Submit::No
            })
            .placeholder("username")
            .text_color(ACCENT_COLOR)
            .class(FORM_INPUT)
            .apply(
                form_border_color,
                self.last_error.as_ref().and_then(UserError::username_color),
            ),
            form_input_label("Username"),
        ));
        let password = zstack((
            text_input(self.password.clone(), |state: &mut Self, input| {
                state.password = input;
                state.last_error = state.check().err();
                Submit::No
            })
            .on_enter(|_, _| Submit::Yes)
            .placeholder("password")
            .text_color(ACCENT_COLOR)
            .class(FORM_INPUT)
            .apply(
                form_border_color,
                self.last_error.as_ref().and_then(UserError::password_color),
            ),
            form_input_label("Password"),
        ));
        let login_button = action_button("Log In", |_| Submit::Yes);
        let error = self.error_view();
        flex_col((header, username, password, login_button, error))
            .class(CONTAINER)
            .gap(30.px())
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

impl UserLoginForm {
    pub fn handle_submit(&mut self, submit: Submit, sender: Option<&UnboundedSender<AuthRequest>>) {
        match submit {
            Submit::No => (),
            Submit::Cancel => {
                self.reset();
            }
            Submit::Yes => {
                let output = self.submit();
                if let (Some((username, password)), Some(sender)) = (output, sender) {
                    let _ = sender.send(AuthRequest::Login(username, password));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct UserSignupForm {
    username: String,
    password: String,
    password_confirmation: String,
    score: Score,
    feedback: Option<Feedback>,
    last_error: Option<UserError>,
}

impl Default for UserSignupForm {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            password_confirmation: String::new(),
            score: Score::Zero,
            feedback: None,
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
        let username = zstack((
            text_input(self.username.clone(), |state: &mut Self, input| {
                state.username = input;
                state.last_error = state.check().err();
                Submit::No
            })
            .placeholder("username")
            .text_color(ACCENT_COLOR)
            .class(FORM_INPUT)
            .apply(
                form_border_color,
                self.last_error.as_ref().and_then(UserError::username_color),
            ),
            form_input_label("Username"),
        ));
        let password = flex_col((
            zstack((
                text_input(self.password.clone(), |state: &mut Self, input| {
                    state.password = input;
                    state.last_error = state.check().err();
                    Submit::No
                })
                .placeholder("password")
                .text_color(ACCENT_COLOR)
                .class(FORM_INPUT)
                .apply(
                    form_border_color,
                    self.last_error.as_ref().and_then(UserError::password_color),
                ),
                form_input_label("Password"),
            )),
            (!self.password.is_empty()).then(|| {
                let (color1, color2) = match self.score {
                    Score::Zero | Score::One | Score::Two => (GRAY, DANGER_COLOR),
                    Score::Three => (GRAY, WARNING_COLOR),
                    _ => (SUCCESS_COLOR, SUCCESS_COLOR),
                };
                let (text1, text2) = match self.score {
                    Score::Zero | Score::One => ("️Password strength:", "Very weak"),
                    Score::Two => ("️Password strength:", "Weak"),
                    Score::Three => ("️Password strength:", "Medium"),
                    _ => ("️✓ Password strength:", "Strong"),
                };
                flex_col((
                    flex_row((
                        inline_prose(text1).text_color(color1),
                        inline_prose(text2).text_color(color2),
                    ))
                    .padding(3.),
                    self.feedback.as_ref().map(|feedback| {
                        (
                            feedback.warning().map(|warning| {
                                inline_prose(format!(" ❌ {}", warning))
                                    .text_size(13.)
                                    .text_color(GRAY)
                                    .padding(3.)
                            }),
                            feedback
                                .suggestions()
                                .iter()
                                .map(|suggestion| {
                                    inline_prose(format!(" ✓ {}", suggestion))
                                        .text_size(13.)
                                        .text_color(GRAY)
                                        .padding(3.)
                                })
                                .collect::<Vec<_>>(),
                        )
                    }),
                ))
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .gap(0.px())
                .padding(Padding::horizontal(19.))
            }),
        ));
        let password_confirmation = zstack((
            text_input(
                self.password_confirmation.clone(),
                |state: &mut Self, input| {
                    state.password_confirmation = input;
                    state.last_error = state.check().err();
                    Submit::No
                },
            )
            .on_enter(|_, _| Submit::Yes)
            .placeholder("confirm password")
            .text_color(ACCENT_COLOR)
            .class(FORM_INPUT)
            .apply(
                form_border_color,
                self.last_error
                    .as_ref()
                    .and_then(UserError::confirmation_color),
            ),
            form_input_label("Password Confirmation"),
        ));
        let signup_button = action_button("Sign Up", |_| Submit::Yes);
        let error = self.error_view();
        flex_col((
            header,
            username,
            password,
            password_confirmation,
            signup_button,
            error,
        ))
        .class(CONTAINER)
        .gap(30.px())
    }

    fn check(&mut self) -> Result<(), UserError> {
        let entropy = zxcvbn(self.password.as_str(), &[self.username.as_str()]);
        self.score = entropy.score();
        self.feedback = entropy.feedback().cloned();

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
