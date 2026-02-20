use std::cmp::Ordering;

use kreqo_core::errors::ServerError;
use kreqo_core::users::User;
use kreqo_server::api::{delete_user, get_users, signup, update_user_username};
use rapidfuzz::distance::jaro;
use server_fn::error::ServerFnErrorErr;
use xilem::core::one_of::Either;
use xilem::masonry::layout::AsUnit;
use xilem::style::Style;
use xilem::view::{
    FlexExt, MainAxisAlignment, button, flex_col, flex_row, label, prose, spinner, text_button,
    text_input, zstack,
};
use xilem::{TextAlign, WidgetView};

use crate::auth_forms::{UserError, UserSignupForm};
use crate::component::Form;
use crate::component::form::Submit;
use crate::component::list::storage::Retryable;
use crate::component::list::{
    ItemAction, ListFilter, ListItem, ListSorter, ListStorage, PendingItemOperation,
};
use crate::theme::{ApplyClass, BORDERED_ROW, DANGER_COLOR, ROW, ROW_OVERLAY, SUCCESS_COLOR};

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

    fn view(&mut self) -> impl WidgetView<Self, Submit> + use<> {
        let username = text_input(
            self.username.clone(),
            |state: &mut UpdateUserForm, input| {
                state.username = input;
                Submit::No
            },
        )
        .on_enter(|_, _| Submit::Yes)
        .placeholder("Username");
        let ok_button = button(label("Ok").color(SUCCESS_COLOR), |_| Submit::Yes);
        let cancel_button = text_button("Cancel", |_| Submit::Cancel);
        let error = self.error_view();
        flex_col((
            flex_row((username.flex(1.), ok_button, cancel_button)),
            error,
        ))
        .apply(BORDERED_ROW)
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

impl Retryable for ServerError {
    fn should_retry(&self) -> bool {
        matches!(self, ServerError::API(ServerFnErrorErr::Request(_)))
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
        signup(username, password).await
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

#[derive(Default)]
pub struct UserFilter {
    by_username: String,
}

impl ListFilter for UserFilter {
    type Item = User;

    fn view(&mut self) -> impl WidgetView<Self> + use<> {
        let username_search = text_input(self.by_username.clone(), |state: &mut Self, input| {
            state.by_username = input;
        })
        .placeholder("Search by username");
        let clear_button = text_button("Clear", |state: &mut Self| {
            state.by_username = String::new();
        });
        flex_row((username_search.flex(1.), clear_button))
    }

    fn filter(&self, item: &User) -> (bool, f32) {
        if self.by_username.is_empty() {
            (true, 0.)
        } else {
            let score = jaro::similarity(self.by_username.chars(), item.username.chars());
            (score > 0.5, score as f32)
        }
    }
}

#[derive(Default)]
pub enum UserSortBy {
    #[default]
    Id,
    Username,
    CreatedAt,
}

impl std::fmt::Display for UserSortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserSortBy::Id => write!(f, "ID"),
            UserSortBy::Username => write!(f, "Username"),
            UserSortBy::CreatedAt => write!(f, "Signup date"),
        }
    }
}

impl UserSortBy {
    fn next(&self) -> Self {
        match self {
            UserSortBy::Id => UserSortBy::Username,
            UserSortBy::Username => UserSortBy::CreatedAt,
            UserSortBy::CreatedAt => UserSortBy::Id,
        }
    }
}

#[derive(Default)]
pub enum UserSortOption {
    #[default]
    Ascending,
    Descending,
}

impl std::fmt::Display for UserSortOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserSortOption::Ascending => write!(f, "Ascending"),
            UserSortOption::Descending => write!(f, "Descending"),
        }
    }
}

impl UserSortOption {
    fn next(&self) -> Self {
        match self {
            UserSortOption::Ascending => UserSortOption::Descending,
            UserSortOption::Descending => UserSortOption::Ascending,
        }
    }
}

#[derive(Default)]
pub struct UserSorter {
    enabled: bool,
    sort_by: UserSortBy,
    option: UserSortOption,
}

impl ListSorter for UserSorter {
    type Item = User;

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn view(&mut self) -> impl WidgetView<Self> + use<> {
        let sorter = if self.enabled {
            let sort_by = text_button(self.sort_by.to_string(), |state: &mut Self| {
                state.sort_by = state.sort_by.next();
            });
            let sort_option = text_button(self.option.to_string(), |state: &mut Self| {
                state.option = state.option.next();
            });
            let disable_button = button(label("✖").color(DANGER_COLOR), |state: &mut Self| {
                state.enabled = false;
            });
            Either::A(flex_row((sort_by, sort_option, disable_button)))
        } else {
            let enable_button = text_button("Disabled", |state: &mut Self| {
                state.enabled = true;
            });
            Either::B(enable_button)
        };
        flex_row((label("Sort by"), sorter)).main_axis_alignment(MainAxisAlignment::End)
    }

    fn sort(&self, a: &User, b: &User, score_a: f32, score_b: f32) -> Ordering {
        if !self.enabled {
            return score_a.total_cmp(&score_b).reverse();
        }
        let mut ordering = match self.sort_by {
            UserSortBy::Id => a.id.cmp(&b.id),
            UserSortBy::Username => a.username.to_lowercase().cmp(&b.username.to_lowercase()),
            UserSortBy::CreatedAt => a.created_at.cmp(&b.created_at),
        };
        if matches!(self.option, UserSortOption::Descending) {
            ordering = ordering.reverse();
        }
        score_a.total_cmp(&score_b).reverse().then(ordering)
    }
}

impl ListItem for User {
    type Id = i64;
    type CreateForm = UserSignupForm;
    type UpdateForm = UpdateUserForm;
    type Filter = UserFilter;
    type Sorter = UserSorter;

    fn id(&self) -> i64 {
        self.id
    }

    fn view(
        &self,
        pending_item_operation: PendingItemOperation,
    ) -> impl WidgetView<Self, ItemAction<Self>> + use<> {
        let id = prose(format!("{}", self.id))
            .text_alignment(TextAlign::Center)
            .width(25.px());
        let username = prose(self.username.to_string());
        let edit_button = if matches!(pending_item_operation, PendingItemOperation::PendingUpdate) {
            Either::A(button(spinner(), |_| ItemAction::None))
        } else {
            Either::B(text_button("Edit", |_| ItemAction::Edit))
        };
        let delete_button = if matches!(pending_item_operation, PendingItemOperation::PendingDelete)
        {
            Either::A(button(spinner().color(DANGER_COLOR), |_| ItemAction::None))
        } else {
            Either::B(button(label("Delete").color(DANGER_COLOR), |_| {
                ItemAction::Delete
            }))
        };
        flex_row((id, username.flex(1.), edit_button, delete_button)).apply(BORDERED_ROW)
    }

    fn pending_view(
        (username, _): &mut (String, String),
    ) -> impl WidgetView<(String, String)> + use<> {
        let id = prose("⏳").text_alignment(TextAlign::Center).width(25.px());
        let username = prose(username.to_string());
        let edit_button = text_button("Edit", |_| {}).disabled(true);
        let delete_button = text_button("Delete", |_| {}).disabled(true);
        let pending_layer =
            flex_row((id, username.flex(1.), edit_button, delete_button)).apply(ROW);
        let spinner_layer = flex_row(spinner())
            .main_axis_alignment(MainAxisAlignment::Center)
            .apply(ROW_OVERLAY);
        zstack((pending_layer, spinner_layer))
    }
}
