use xilem::WidgetView;
use xilem::style::Style;
use xilem::view::{MainAxisAlignment, flex_row, prose};

use crate::theme::DANGER_COLOR;

pub trait ErrorView
where
    Self: Sized + 'static,
{
    fn view(&self) -> impl WidgetView<Self> + use<Self>;
}

impl<T> ErrorView for T
where
    T: ToString + 'static,
{
    fn view(&self) -> impl WidgetView<Self> + use<T> {
        flex_row(prose(self.to_string()).text_color(DANGER_COLOR))
            .main_axis_alignment(MainAxisAlignment::Center)
            .padding(5.)
    }
}
