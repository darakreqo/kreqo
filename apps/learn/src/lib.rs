use kreqo_core::User;
use kreqo_ui::component::AsyncList;
use kreqo_ui::theme::BACKGROUND_COLOR;
use kreqo_ui::user_list::UserStorage;
use xilem::core::map_state;
use xilem::masonry::layout::{AsUnit, Dim};
use xilem::style::Style;
use xilem::view::{FlexExt, MainAxisAlignment, flex_col, flex_row, portal, sized_box};
use xilem::{WindowId, WindowView, window};

pub struct AppState {
    running: bool,
    main_window_id: WindowId,
    user_list: AsyncList<User, UserStorage>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            running: true,
            main_window_id: WindowId::next(),
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
        let user_list = flex_row(sized_box(self.user_list.view()).width(600.px()))
            .main_axis_alignment(MainAxisAlignment::Center)
            .width(Dim::Stretch)
            .padding(15.);
        let error = self.user_list.error_view().map(|error_view| {
            flex_row(error_view)
                .main_axis_alignment(MainAxisAlignment::Center)
                .padding(15.)
        });
        let portal = portal(user_list).flex(1.);
        let content = map_state(
            flex_col((portal, error)).gap(0.px()),
            |state: &mut AppState, ()| &mut state.user_list,
        );
        std::iter::once(
            window(self.main_window_id, "Kreqo Learn", content)
                .with_options(|options| {
                    options.on_close(|state: &mut AppState| state.running = false)
                })
                .with_base_color(BACKGROUND_COLOR),
        )
    }
}
