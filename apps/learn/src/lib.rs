use kreqo_core::users::User;
use kreqo_server::api::login;
use kreqo_ui::auth_forms::UserLoginForm;
use kreqo_ui::component::list::ListRequest;
use kreqo_ui::component::{AsyncList, Form};
use kreqo_ui::theme::BACKGROUND_COLOR;
use kreqo_ui::user_list::UserStorage;
use xilem::core::one_of::OneOf3;
use xilem::core::{fork, lens, map_action, map_state};
use xilem::masonry::layout::{AsUnit, Dim};
use xilem::palette::css::GRAY;
use xilem::style::Style;
use xilem::tokio::sync::mpsc::UnboundedSender;
use xilem::view::{
    FlexExt, MainAxisAlignment, flex_col, flex_row, label, portal, sized_box, text_button, worker,
};
use xilem::{WindowId, WindowView, window};

#[derive(Default)]
pub enum Page {
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
    login_sender: Option<UnboundedSender<(String, String)>>,
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
            login_sender: None,
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
                            .handle_submit(submit, state.login_sender.as_ref());
                    },
                );
                let separator = label("OR").color(GRAY);
                let goto_signup = text_button("Create account", |state: &mut Self| {
                    state.page = Page::Signup
                })
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
                            while let Some((username, password)) = rx.recv().await {
                                let result = login(username, password).await;
                                drop(proxy.message(result));
                            }
                        },
                        |state: &mut Self, sender| {
                            state.login_sender = Some(sender);
                        },
                        |state: &mut Self, result| match result {
                            Ok(user) => {
                                state.current_user = Some(user);
                                state.page = Page::UserList;
                            }
                            Err(error) => eprintln!("{}", error),
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
                let goto_login = text_button("Log into existing account", |state: &mut Self| {
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
                // TODO: add a way to log out
                OneOf3::C(map_action(
                    map_state(AsyncList::worker(content), move |state: &mut Self| {
                        &mut state.user_list
                    }),
                    |_, _| (),
                ))
            }
        };
        std::iter::once(
            window(self.main_window_id, "Kreqo Learn", page)
                .with_options(|options| {
                    options.on_close(|state: &mut AppState| state.running = false)
                })
                .with_base_color(BACKGROUND_COLOR),
        )
    }
}
