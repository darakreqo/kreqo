use kreqo_core::users::User;
use kreqo_server::api::{cleanup_expired_sessions, current_user, login, logout};
use kreqo_server::custom_client::request::save_cookies;
use kreqo_ui::auth_forms::{AuthMessage, AuthRequest, UserLoginForm};
use kreqo_ui::component::list::ListRequest;
use kreqo_ui::component::{AsyncList, Form, action_button, logo, user_profile_overview};
use kreqo_ui::theme::BACKGROUND_COLOR;
use kreqo_ui::user_list::UserStorage;
use xilem::core::one_of::OneOf3;
use xilem::core::{fork, lens, map_action, map_state};
use xilem::masonry::layout::{AsUnit, Dim};
use xilem::palette::css::GRAY;
use xilem::style::Style;
use xilem::tokio::sync::mpsc::UnboundedSender;
use xilem::view::{
    FlexExt, MainAxisAlignment, flex_col, flex_row, label, portal, sized_box, split, text_button,
    worker,
};
use xilem::{WindowId, WindowView, window};

#[derive(Default)]
enum Page {
    #[default]
    Login,
    Signup,
    UserList,
}

pub struct AppState {
    running: bool,
    main_window_id: WindowId,
    page: Page,
    current_user: Option<User>,
    login_form: UserLoginForm,
    auth_sender: Option<UnboundedSender<AuthRequest>>,
    user_list: AsyncList<User, UserStorage>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            running: true,
            main_window_id: WindowId::next(),
            page: Page::default(),
            current_user: None,
            login_form: UserLoginForm::default(),
            auth_sender: None,
            user_list: AsyncList::new(true, true),
        }
    }
}

impl xilem::AppState for AppState {
    fn keep_running(&self) -> bool {
        self.running
    }
}

impl AppState {
    pub fn logic(&mut self) -> impl Iterator<Item = WindowView<AppState>> + use<> {
        let page = match self.page {
            Page::Login => {
                let form = map_action(
                    lens(Form::view, move |state: &mut Self| &mut state.login_form),
                    |state: &mut Self, submit| {
                        state
                            .login_form
                            .handle_submit(submit, state.auth_sender.as_ref());
                    },
                );
                let separator = label("OR").color(GRAY);
                let goto_signup =
                    text_button("Sign Up", |state: &mut Self| state.page = Page::Signup)
                        .corner_radius(100.);
                let content = flex_col((
                    sized_box(form).dims((600.px(), Dim::MinContent)),
                    separator,
                    goto_signup,
                ))
                .main_axis_alignment(MainAxisAlignment::Center);
                let worker = fork(
                    content,
                    worker(
                        |proxy, mut rx| async move {
                            while let Some(request) = rx.recv().await {
                                match request {
                                    AuthRequest::Login(username, password) => {
                                        if login(username, password).await.is_err() {
                                            return;
                                        }
                                        let user = current_user().await.ok();
                                        drop(proxy.message(AuthMessage::UserRefreshed(user)));
                                    }
                                    AuthRequest::RefreshUser => {
                                        let user = current_user().await.ok();
                                        drop(proxy.message(AuthMessage::UserRefreshed(user)));
                                    }
                                    _ => (),
                                }
                            }
                        },
                        |state: &mut Self, sender| {
                            state.auth_sender = Some(sender);
                            state.auth_sender.as_ref().inspect(|sender| {
                                let _ = sender.send(AuthRequest::RefreshUser);
                            });
                        },
                        |state: &mut Self, message| {
                            if let AuthMessage::UserRefreshed(user) = message {
                                state.current_user = user.clone();
                                if let Some(user) = user
                                    && user.id != 1
                                {
                                    state.page = Page::UserList;
                                }
                            }
                        },
                    ),
                );
                OneOf3::A(worker)
            }
            Page::Signup => {
                let form = map_action(
                    map_state(
                        AsyncList::worker(self.user_list.create_view()),
                        move |state: &mut Self| &mut state.user_list,
                    ),
                    |state: &mut Self, resolved| {
                        if matches!(resolved, Some(ListRequest::Create(_))) {
                            state.page = Page::Login;
                        }
                    },
                );
                let separator = label("OR").color(GRAY);
                let goto_login = text_button("Log In", |state: &mut Self| {
                    state.page = Page::Login;
                })
                .corner_radius(100.);
                let content = flex_col((
                    sized_box(form).dims((600.px(), Dim::MinContent)),
                    separator,
                    goto_login,
                ))
                .main_axis_alignment(MainAxisAlignment::Center);
                OneOf3::B(content)
            }
            Page::UserList => {
                let user_profile = self.current_user.as_ref().map(|_| {
                    lens(user_profile_overview, move |state: &mut Self| {
                        &mut state.current_user.as_mut().unwrap().username
                    })
                });
                let cleanup_sessions_button = self.current_user.as_ref().and_then(|user| {
                    user.permissions
                        .contains("Server::Manage")
                        .then_some(action_button("Cleanup sessions", |state: &mut Self| {
                            state.auth_sender.as_ref().inspect(|sender| {
                                let _ = sender.send(AuthRequest::CleanupSessions);
                            });
                        }))
                });
                let logout_button = action_button("Log Out", |state: &mut Self| {
                    state.auth_sender.as_ref().inspect(|sender| {
                        let _ = sender.send(AuthRequest::Logout);
                    });
                });
                let sidebar =
                    flex_col((logo(), user_profile, cleanup_sessions_button, logout_button))
                        .gap(20.px())
                        .padding(15.);
                let sidebar_worker = fork(
                    sidebar,
                    worker(
                        |proxy, mut rx| async move {
                            while let Some(request) = rx.recv().await {
                                match request {
                                    AuthRequest::CleanupSessions => {
                                        if let Ok(removed_session_ids) =
                                            cleanup_expired_sessions().await
                                        {
                                            println!(
                                                "Successfully removed {} sessions",
                                                removed_session_ids.len()
                                            );
                                            drop(proxy.message(AuthMessage::SessionsCleanedUp));
                                        }
                                    }
                                    AuthRequest::Logout => {
                                        if logout().await.is_ok() {
                                            println!("Successfully logged out");
                                            drop(proxy.message(AuthMessage::UserRefreshed(None)));
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        },
                        |state: &mut Self, sender| {
                            state.auth_sender = Some(sender);
                        },
                        |state: &mut Self, message| {
                            if let AuthMessage::UserRefreshed(user) = message {
                                state.current_user = user.clone();
                                if user.is_none() {
                                    state.page = Page::default();
                                }
                            }
                        },
                    ),
                );

                let user_list = flex_row(sized_box(self.user_list.view()).width(600.px()))
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .width(Dim::Stretch)
                    .padding(15.);
                let user_list_error = self.user_list.error_view().map(|error_view| {
                    flex_row(error_view)
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .padding(15.)
                });
                let portal = portal(user_list).flex(1.);
                let content = flex_col((portal, user_list_error)).gap(0.px());
                let worker = map_action(
                    map_state(AsyncList::worker(content), move |state: &mut Self| {
                        &mut state.user_list
                    }),
                    |_, _| (),
                );

                OneOf3::C(
                    split(sidebar_worker, worker)
                        .split_point_from_start(200.px())
                        .draggable(false)
                        .solid_bar(true)
                        .bar_thickness(2.px()),
                )
            }
        };
        std::iter::once(
            window(self.main_window_id, "Kreqo Learn", page)
                .with_options(|options| {
                    options.on_close(|state: &mut AppState| {
                        state.running = false;
                        let _ = save_cookies();
                    })
                })
                .with_base_color(BACKGROUND_COLOR),
        )
    }
}
